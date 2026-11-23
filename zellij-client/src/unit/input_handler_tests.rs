use crate::input_handler::from_termwiz;
use zellij_utils::input::mouse::MouseEvent;
use zellij_utils::position::Position;
use zellij_utils::vendored::termwiz::input::{InputEvent, InputParser};

fn mouse_event_from_host_terminal(bytes: &[u8]) -> MouseEvent {
    let mut events = InputParser::new().parse_as_vec(bytes, false);
    assert_eq!(events.len(), 1, "expected a single event, got {:?}", events);
    match events.remove(0) {
        InputEvent::Mouse(event) => from_termwiz(&mut MouseEvent::new(), event),
        other => panic!("expected a mouse event, got {:?}", other),
    }
}

#[test]
fn horizontal_scroll_left_from_host_terminal_is_read_as_left() {
    let event = mouse_event_from_host_terminal(b"\x1b[<66;42;12M");
    assert!(event.wheel_left);
    assert!(!event.wheel_right);
    assert!(!event.wheel_up && !event.wheel_down);
    assert_eq!(event.position, Position::new(11, 41));
}

#[test]
fn horizontal_scroll_right_from_host_terminal_is_read_as_right() {
    let event = mouse_event_from_host_terminal(b"\x1b[<67;42;12M");
    assert!(event.wheel_right);
    assert!(!event.wheel_left);
    assert!(!event.wheel_up && !event.wheel_down);
    assert_eq!(event.position, Position::new(11, 41));
}

#[derive(Clone, Debug)]
struct NoServerCommands;

impl crate::os_input_output::ClientOsApi for NoServerCommands {
    fn get_terminal_size(&self) -> zellij_utils::pane_size::Size {
        zellij_utils::pane_size::Size { rows: 24, cols: 80 }
    }
    fn set_raw_mode(&mut self) {}
    fn unset_raw_mode(&self) -> std::io::Result<()> {
        Ok(())
    }
    fn get_stdout_writer(&self) -> Box<dyn std::io::Write> {
        Box::new(std::io::sink())
    }
    fn get_stdin_reader(&self) -> Box<dyn std::io::BufRead> {
        Box::new(std::io::empty())
    }
    fn update_session_name(&mut self, _: String) {}
    fn read_from_stdin(&mut self) -> Result<Vec<u8>, &'static str> {
        Ok(vec![])
    }
    fn box_clone(&self) -> Box<dyn crate::os_input_output::ClientOsApi> {
        Box::new(self.clone())
    }
    fn send_to_server(&self, message: zellij_utils::ipc::ClientToServerMsg) {
        panic!("rejection must exit the client, not send a session command: {message:?}");
    }
    fn recv_from_server(
        &self,
    ) -> Option<(
        zellij_utils::ipc::ServerToClientMsg,
        zellij_utils::errors::ErrorContext,
    )> {
        None
    }
    fn handle_signals(
        &self,
        _: Box<dyn Fn()>,
        _: Box<dyn Fn()>,
        _: Option<std::sync::mpsc::Receiver<()>>,
    ) {
    }
    fn connect_to_server(&self, _: &std::path::Path) -> std::io::Result<()> {
        Ok(())
    }
    fn load_palette(&self) -> zellij_utils::data::Palette {
        zellij_utils::data::Palette::default()
    }
    fn enable_mouse(&self) -> anyhow::Result<()> {
        Ok(())
    }
    fn disable_mouse(&self) -> anyhow::Result<()> {
        Ok(())
    }
}

#[test]
fn rejected_nested_attachment_exits_the_viewer_without_a_session_command() {
    use crate::command_is_executing::CommandIsExecuting;
    use crate::nested_reannounce::NestedReannounce;
    use crate::{ClientInstruction, InputInstruction};
    use zellij_utils::channels::{bounded, SenderWithContext};
    use zellij_utils::data::InputMode;
    use zellij_utils::errors::ErrorContext;
    use zellij_utils::input::{config::Config, options::Options};
    use zellij_utils::ipc::ExitReason;
    use zellij_utils::nested_session::{encode_payload, NestedSessionMessage};

    let os_input = Box::new(NoServerCommands);
    let reannounce = NestedReannounce::spawn(os_input.clone(), "recursive-viewer".into());
    let (input_tx, input_rx) = bounded(2);
    let (client_tx, client_rx) = bounded(2);
    let reason = "Cannot attach a containing session inside itself.";
    input_tx
        .send((
            InputInstruction::NestedSessionFrameFromHost(encode_payload(
                &NestedSessionMessage::AnnounceRejected {
                    reason: reason.into(),
                },
            )),
            ErrorContext::default(),
        ))
        .unwrap();
    input_tx
        .send((InputInstruction::Exit, ErrorContext::default()))
        .unwrap();

    crate::input_handler::input_loop(
        os_input,
        Config::default(),
        Options::default(),
        CommandIsExecuting::new(),
        SenderWithContext::new(client_tx),
        InputMode::Normal,
        input_rx,
        reannounce.clone(),
    );
    reannounce.stop();

    let (instruction, _) = client_rx.try_recv().expect("client must be told to exit");
    assert!(
        matches!(instruction, ClientInstruction::Exit(ExitReason::Error(message)) if message == reason)
    );
    assert!(client_rx.try_recv().is_err());
}

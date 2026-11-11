#![cfg(unix)]

use base64::{engine::general_purpose::STANDARD, Engine as _};
use zellij_integration_tests::{keys, FakePtyHandle, TestRunner, TestSession, TERMINAL_SIZE};

const CONFIG: &str = r#"
mouse_mode false
pane_frame_style "full"
keybinds clear-defaults=true {
    normal {
        bind "Ctrl s" { SwitchToMode "Copy"; }
        bind "Ctrl q" { Quit; }
    }
    copy {
        bind "Esc" { CopyModeInput "ESC"; }
    }
}
"#;

fn start(content: &[u8], marker: &str) -> (TestSession, FakePtyHandle) {
    let zellij = TestRunner::new(TERMINAL_SIZE).with_config(CONFIG).start();
    let terminal = zellij.expect_pty_spawn();
    terminal.output(content);
    zellij.wait_until("terminal output rendered", |grid| grid.contains(marker));
    zellij.send_stdin(&keys::ctrl('s'));
    zellij.wait_until("native copy cursor active", |grid| grid.contains("COPY"));
    (zellij, terminal)
}

fn expect_copy(zellij: &TestSession, text: &str) {
    let expected = format!("\x1b]52;c;{}\x1b\\", STANDARD.encode(text));
    zellij.wait_until_raw_output("selection emitted to clipboard", |bytes| {
        bytes
            .windows(expected.len())
            .any(|window| window == expected.as_bytes())
    });
}

#[test]
fn keyboard_copy_word_selection_emits_clipboard_without_an_editor() {
    let (mut zellij, _terminal) = start(b"alpha beta gamma\r\nREADY", "READY");
    zellij.send_stdin(b"ggwvey");
    expect_copy(&zellij, "beta");
    zellij.wait_until("copy mode exited after yank", |grid| {
        !grid.contains("COPY") && !grid.contains("VISUAL")
    });
    zellij.quit();
}

#[test]
fn keyboard_copy_line_jump_and_line_selection_cross_scrollback() {
    let content = (1..=60)
        .map(|n| format!("line{n:02}\r\n"))
        .collect::<String>();
    let (mut zellij, _terminal) = start(content.as_bytes(), "line60");
    zellij.send_stdin(b"42GVy");
    expect_copy(&zellij, "line42\n");
    zellij.quit();
}

#[test]
fn keyboard_copy_holds_new_output_until_selection_is_copied() {
    let (mut zellij, terminal) = start(b"alpha beta gamma\r\nREADY", "READY");
    zellij.send_stdin(b"ggwve");
    zellij.wait_until("visual selection active", |grid| grid.contains("VISUAL"));
    terminal.output(b"\r\nNEW OUTPUT");
    zellij.send_stdin(b"y");
    expect_copy(&zellij, "beta");
    zellij.wait_until("buffered output resumed", |grid| {
        grid.contains("NEW OUTPUT") && !grid.contains("VISUAL")
    });
    zellij.quit();
}

#[test]
fn keyboard_copy_handles_explicit_shift_events_from_kitty() {
    let content = (1..=60)
        .map(|n| format!("line{n:02}\r\n"))
        .collect::<String>();
    let (mut zellij, _terminal) = start(content.as_bytes(), "line60");
    zellij.send_stdin(b"42\x1b[103;2u\x1b[118;2uy");
    expect_copy(&zellij, "line42\n");
    zellij.quit();
}

#[test]
fn keyboard_copy_peer_scroll_cannot_change_the_selected_text() {
    use zellij_utils::input::actions::Action;
    let content = (1..=60)
        .map(|n| format!("alpha beta gamma {n:02}\r\n"))
        .collect::<String>();
    let (mut zellij, _terminal) = start(content.as_bytes(), "gamma 60");
    let second = zellij.attach_client(TERMINAL_SIZE);
    second.wait_until("peer attached", |grid| grid.contains("gamma 60"));
    zellij.send_stdin(b"ggwve");
    let selected = zellij.wait_until("owner selection active", |grid| grid.contains("VISUAL"));
    let cursor = selected.cursor.expect("copy cursor visible");
    second.send_action(Action::ScrollDown);
    zellij.send_stdin(b"l");
    let after = zellij.wait_until("owner motion rendered after peer scroll attempt", |grid| {
        grid.contains("VISUAL")
            && grid
                .cursor
                .is_some_and(|p| p.x == cursor.x + 1 && p.y == cursor.y)
    });
    assert!(
        after.contains("gamma 01"),
        "the shared viewport stayed on the selected first row"
    );
    zellij.send_stdin(b"h");
    zellij.send_stdin(b"y");
    expect_copy(&zellij, "beta");
    zellij.quit();
}

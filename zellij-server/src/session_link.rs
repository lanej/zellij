pub use zellij_utils::session_link::*;

pub fn local_session_attention(
    senders: &crate::thread_bus::ThreadSenders,
    include_command_text: bool,
) -> zellij_utils::attention::SessionAttention {
    let (response_sender, response_receiver) = crossbeam::channel::bounded(1);
    let sent = senders.send_to_screen(crate::screen::ScreenInstruction::GetSessionAttention {
        include_command_text,
        response_channel: response_sender,
    });
    if sent.is_err() {
        return zellij_utils::attention::SessionAttention::default();
    }
    response_receiver
        .recv_timeout(zellij_utils::session_link::LOCAL_ANSWER_TIMEOUT)
        .unwrap_or_default()
}

pub(crate) fn local_session_attention_with_identity(
    senders: &crate::thread_bus::ThreadSenders,
    instance_id: String,
    include_command_text: bool,
) -> zellij_utils::attention::SessionAttention {
    let mut attention = local_session_attention(senders, include_command_text);
    // Identity belongs to the live server even when its screen cannot answer promptly.
    attention.server_instance_id = Some(instance_id);
    attention
}

#[cfg(test)]
mod tests {
    #[test]
    fn unavailable_screen_does_not_erase_the_live_identity() {
        let mut senders = crate::thread_bus::ThreadSenders::default();
        senders.should_silently_fail = true;
        let attention =
            super::local_session_attention_with_identity(&senders, "server-id".into(), false);
        assert_eq!(attention.server_instance_id.as_deref(), Some("server-id"));
    }
}

use super::Screen;
use crate::nested_navigation::NavigationTarget;
use crate::plugins::{PluginInstruction, PromptCaller, SERVER_NOTICE_CALLER_ID};
use crate::route::NotificationEnd;
use crate::{ClientId, ServerInstruction};
use std::time::{Duration, Instant};
use zellij_utils::data::ConnectToSession;
use zellij_utils::nested_session::{self, NestedSessionCapability, NestedSessionMessage};

impl Screen {
    pub(super) fn announce_server_identity(&self, client_id: ClientId) {
        let message = NestedSessionMessage::Announce {
            session_name: self.session_name.clone(),
            server_instance_id: Some(self.navigation_guard.instance_id.clone()),
            capabilities: vec![
                NestedSessionCapability::NestedControl,
                NestedSessionCapability::HintReporting,
            ],
        };
        let _ = self
            .bus
            .senders
            .send_to_server(ServerInstruction::EmitNestedSessionFrameToClient(
                client_id,
                nested_session::encode_payload(&message),
            ));
    }

    pub(super) fn guest_identity_ancestry(&self) -> Option<Vec<String>> {
        self.navigation_guard.descendant_ancestry()
    }

    pub(super) fn configure_nesting_client(
        &mut self,
        client_id: ClientId,
        env: &std::collections::BTreeMap<String, String>,
    ) {
        let expected_nested = nested_session::expects_nested_attachment(env);
        self.navigation_guard
            .configure_client(client_id, expected_nested, Instant::now());
        self.broadcast_ancestry_update_to_guests();
        if !expected_nested {
            let _ = self.bus.senders.send_to_background_jobs(
                crate::background_jobs::BackgroundJob::SettleClientNesting(client_id),
            );
        }
    }

    pub(super) fn validate_session_switch(
        &mut self,
        mut connect: ConnectToSession,
        client_id: ClientId,
        completion_tx: Option<NotificationEnd>,
        close_current: bool,
    ) {
        let target = connect.name.as_deref();
        if !self
            .client_host_terminal_env
            .get(&client_id)
            .is_some_and(|env| {
                env.get(nested_session::VERIFIED_SWITCH_CAPABILITY)
                    .map(String::as_str)
                    == Some("1")
            })
        {
            self.reject_session_switch(
                client_id,
                crate::nested_navigation::NavigationError::UnsupportedClient.notice(),
                completion_tx,
            );
            return;
        }
        if target == Some(self.session_name.as_str()) {
            self.reject_session_switch(
                client_id,
                "This attachment is already in that session.",
                completion_tx,
            );
            return;
        }
        let target_reply = target.map(|name| {
            zellij_utils::session_link::request_session_attention_with(
                name,
                false,
                zellij_utils::session_link::PEER_REPLY_TIMEOUT,
            )
        });
        let target_identity = target_reply
            .as_ref()
            .and_then(|reply| reply.as_ref().ok())
            .and_then(|a| a.server_instance_id.clone());
        let existing_target = target_reply.as_ref().is_some_and(|reply| reply.is_ok())
            || target.is_some_and(|name| zellij_utils::consts::session_socket_path(name).exists());
        if existing_target && target_identity.is_none() {
            self.reject_session_switch(
                client_id,
                crate::nested_navigation::NavigationError::MissingTarget.notice(),
                completion_tx,
            );
            return;
        }
        let target = match target_identity.as_deref() {
            Some(id) => NavigationTarget::Existing(id),
            None => NavigationTarget::New,
        };
        if let Err(error) = self.navigation_guard.check(client_id, target) {
            self.reject_session_switch(client_id, error.notice(), completion_tx);
            return;
        }
        connect
            .switch_context
            .get_or_insert_with(Default::default)
            .expected_server_instance_id = target_identity;
        let _ = self
            .bus
            .senders
            .send_to_server(ServerInstruction::ValidatedSessionSwitch(
                connect,
                client_id,
                completion_tx,
                close_current,
            ));
    }

    fn reject_session_switch(
        &self,
        client_id: ClientId,
        message: &str,
        mut completion_tx: Option<NotificationEnd>,
    ) {
        if let Some(completion) = completion_tx.as_mut() {
            completion.set_exit_status(1);
            completion.set_error_message(message.to_owned());
        }
        log::warn!(
            "Rejected session navigation for client {}: {}",
            client_id,
            message
        );
        let request = zellij_utils::prompt::PromptRequest::notify(message.to_owned())
            .timeout(Duration::from_secs(8));
        let _ = self
            .bus
            .senders
            .send_to_plugin(PluginInstruction::PromptRequest {
                caller: PromptCaller {
                    plugin_id: SERVER_NOTICE_CALLER_ID,
                    client_id,
                    request_id: 0,
                },
                owner_client_id: client_id,
                caller_pane_id: None,
                request,
            });
        let _ = self.bus.senders.send_to_server(ServerInstruction::LogError(
            vec![message.to_owned()],
            client_id,
            completion_tx,
        ));
    }
}

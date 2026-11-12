#![cfg(unix)]

use zellij_integration_tests::{
    claim_first_terminal_and_wait_for_prompt, TestRunner, TERMINAL_SIZE,
};
use zellij_utils::{data::LayoutInfo, input::actions::Action};

#[test]
fn nested_policy_allows_top_level_start_switch_and_reconnect() {
    for metadata_disabled in [false, true] {
        let config = format!("allow_nested_sessions false\nallow_cyclic_sessions false\ndisable_session_metadata {metadata_disabled}");
        let target = TestRunner::new(TERMINAL_SIZE)
            .with_config(&config)
            .with_layout(LayoutInfo::Stringified("layout { pane }".to_owned()))
            .skip_concurrency_slot()
            .start_in_background();
        let mut session = TestRunner::new(TERMINAL_SIZE)
            .with_config(&config)
            .with_layout(LayoutInfo::Stringified("layout { pane }".to_owned()))
            .start();
        claim_first_terminal_and_wait_for_prompt(&session);
        let identity_path =
            zellij_utils::consts::session_info_folder_for_session(&session.session_id())
                .join("nesting-identity");
        assert_eq!(identity_path.exists(), !metadata_disabled);
        let name = target.session_name().to_owned();
        session.main_client().send_action(Action::SwitchSession {
            name: name.clone(),
            tab_position: None,
            pane_id: None,
            layout: None,
            cwd: None,
            close_current: false,
        });
        let (_switch, _previous) = session.follow_switch_into(target);
        assert_eq!(session.session_name(), name);
        claim_first_terminal_and_wait_for_prompt(&session);
        session.quit();
    }
}

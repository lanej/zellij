#![cfg(unix)]
use base64::{engine::general_purpose::STANDARD, Engine as _};
use zellij_integration_tests::{keys, FakePtyHandle, TestRunner, TestSession, TERMINAL_SIZE};

fn start(content: &[u8], column: u64) -> (TestSession, FakePtyHandle, tempfile::TempDir) {
    let source = std::env::var_os("ZELLIJ_COPY_MODE_FIXTURE_WASM")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            std::path::PathBuf::from(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../zellij-utils/assets/plugins/fixture-plugin-for-tests.wasm"
            ))
        });
    let plugin_dir = tempfile::tempdir().unwrap();
    let plugin = plugin_dir.path().join("copy-api-fixture.wasm");
    std::fs::hard_link(&source, &plugin)
        .or_else(|_| std::fs::copy(&source, &plugin).map(|_| ()))
        .unwrap();
    let config = format!(
        r#"
        mouse_mode false
        pane_frame_style "full"
        plugins {{
            control location="file:{}" {{
                copy_mode_fixture true
                copy_column "{column}"
            }}
        }}
        load_plugins {{
            "control"
        }}
        keybinds clear-defaults=true {{
            normal {{
                bind "Ctrl s" {{ SwitchToMode "Copy"; }}
                bind "Ctrl q" {{ Quit; }}
            }}
            copy {{
                bind "s" {{
                    MessagePlugin "control" {{
                        name "copy_mode_control"
                    }}
                }}
                bind "Esc" {{ CopyModeInput "ESC"; }}
            }}
        }}
    "#,
        plugin.display()
    );
    let zellij = TestRunner::new(TERMINAL_SIZE).with_config(&config).start();
    let terminal = zellij.expect_pty_spawn();
    terminal.output(content);
    zellij.wait_until("fixture requests permissions", |grid| {
        grid.contains("Allow? (y/n)")
    });
    zellij.send_stdin(b"y");
    zellij.wait_until("fixture ready", |grid| {
        grid.contains("READY") && !grid.contains("Allow? (y/n)")
    });
    zellij.send_stdin(&keys::ctrl('s'));
    zellij.wait_until("native copy active", |grid| grid.contains("COPY"));
    (zellij, terminal, plugin_dir)
}

fn capture(zellij: &TestSession, visible: &str) {
    zellij.send_stdin(b"s");
    zellij.wait_until("native API label rendered", |grid| {
        grid.contains("JUMP") && grid.contains(visible)
    });
}

fn copied(zellij: &TestSession, text: &str) {
    let expected = format!("\x1b]52;c;{}\x1b\\", STANDARD.encode(text));
    zellij.wait_until_raw_output("native clipboard emits original text", |bytes| {
        bytes
            .windows(expected.len())
            .any(|window| window == expected.as_bytes())
    });
}

#[test]
fn plugin_api_keeps_captured_keys_in_native_order_and_resumes_output() {
    let (mut zellij, terminal, _plugin) = start(b"alpha beta gamma\r\nREADY", 9);
    zellij.send_stdin(b"ggwv");
    zellij.wait_until("native selection", |grid| grid.contains("VISUAL"));
    capture(&zellij, "etx gamma");
    terminal.output(b"\r\nBUFFERED");
    // The fixture acknowledges a, moves on x, and leaves h/y queued for native copy.
    zellij.send_stdin(b"axhy");
    copied(&zellij, "bet");
    zellij.wait_until("pending terminal output released", |grid| {
        grid.contains("BUFFERED") && !grid.contains("VISUAL")
    });
    zellij.quit();
}

#[test]
fn plugin_api_uses_native_unicode_boundaries_and_cancels_overlays() {
    let (mut zellij, _, _plugin) = start("界e\u{301} beta\r\nREADY".as_bytes(), 2);
    zellij.send_stdin(b"ggv");
    capture(&zellij, "x beta");
    zellij.send_stdin(b"axy");
    copied(&zellij, "界e\u{301}");
    zellij.send_stdin(&keys::ctrl('s'));
    zellij.wait_until("new native session", |grid| grid.contains("COPY"));
    capture(&zellij, "x beta");
    zellij.send_stdin(&keys::ESC);
    zellij.wait_until("overlay canceled", |grid| {
        !grid.contains("JUMP") && grid.contains("COPY")
    });
    zellij.send_stdin(b"ggwvey");
    copied(&zellij, "beta");
    zellij.quit();
}

#[test]
fn plugin_api_resize_and_reload_release_capture() {
    use zellij_utils::input::{actions::Action, layout::RunPluginOrAlias};
    let (mut zellij, terminal, plugin) = start(b"alpha beta gamma\r\nREADY", 9);
    zellij.send_stdin(b"gg");
    capture(&zellij, "betx gamma");
    let mut size = TERMINAL_SIZE;
    size.cols -= 5;
    zellij.resize(size);
    zellij.wait_until("resize invalidates session", |grid| {
        grid.contains("COPY CANCELED")
    });
    zellij.send_stdin(b"x");
    zellij.wait_until("copy exited", |grid| !grid.contains("COPY CANCELED"));
    terminal.output(b"\r\nAFTER RESIZE");
    zellij.wait_until("terminal recovered", |grid| grid.contains("AFTER RESIZE"));
    zellij.send_stdin(&keys::ctrl('s'));
    zellij.wait_until("copy reentered", |grid| grid.contains("COPY"));
    zellij.send_stdin(b"gg");
    capture(&zellij, "betx gamma");
    let configuration = std::collections::BTreeMap::from([
        ("copy_mode_fixture".to_owned(), "true".to_owned()),
        ("copy_column".to_owned(), "9".to_owned()),
    ]);
    zellij
        .main_client()
        .send_action(Action::StartOrReloadPlugin {
            plugin: RunPluginOrAlias::from_url(
                &format!(
                    "file:{}",
                    plugin.path().join("copy-api-fixture.wasm").display()
                ),
                &Some(configuration),
                None,
                None,
            )
            .unwrap(),
        });
    zellij.wait_until("reload clears native ownership", |grid| {
        !grid.contains("JUMP") && grid.contains("lpha beta gamma")
    });
    zellij.send_stdin(b"ggwvey");
    copied(&zellij, "beta");
    zellij.quit();
}

#[test]
fn plugin_api_capture_preserves_focused_popup_priority() {
    use zellij_integration_tests::CliCaller;
    let (mut zellij, terminal, _plugin) = start(b"alpha beta gamma\r\nREADY", 9);
    capture(&zellij, "betx gamma");
    let prompt = zellij.run_prompt(
        &["confirm", "Keep these keys?"],
        CliCaller::from_pane(terminal.terminal_id()),
    );
    zellij.wait_until("focused popup", |grid| grid.contains("Keep these keys?"));
    zellij.send_stdin(&keys::ENTER);
    assert_eq!(prompt.wait_for_exit().exit_code, 0);
    zellij.wait_until("popup consumed Enter", |grid| {
        !grid.contains("Keep these keys?")
    });
    zellij.quit();
}

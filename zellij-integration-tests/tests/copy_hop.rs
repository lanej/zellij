#![cfg(unix)]
use base64::{engine::general_purpose::STANDARD, Engine as _};
use zellij_integration_tests::{keys, FakePtyHandle, TestRunner, TestSession, TERMINAL_SIZE};

const PLUGIN: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../zellij-utils/assets/plugins/copy-hop.wasm"
);

fn start(content: &[u8], marker: &str) -> (TestSession, FakePtyHandle, tempfile::TempDir) {
    assert!(
        std::path::Path::new(PLUGIN).is_file(),
        "Build copy-hop for wasm32-wasip1 and copy it to zellij-utils/assets/plugins first"
    );
    let plugin_dir = tempfile::tempdir().unwrap();
    let plugin = plugin_dir.path().join("copy-hop.wasm");
    std::fs::hard_link(PLUGIN, &plugin)
        .or_else(|_| std::fs::copy(PLUGIN, &plugin).map(|_| ()))
        .unwrap();
    let plugin_path = plugin.display();
    let config = format!(
        r#"
        mouse_mode false
        pane_frame_style "full"
        plugins {{
            copy-hop location="file:{plugin_path}"
        }}
        load_plugins {{
            "copy-hop"
        }}
        keybinds clear-defaults=true {{
            normal {{
                bind "Ctrl s" {{ SwitchToMode "Copy"; }}
                bind "Ctrl q" {{ Quit; }}
            }}
            copy {{
                bind "s" {{
                    MessagePlugin "copy-hop" {{
                        name "hop"
                    }}
                }}
                bind "Esc" {{ CopyModeInput "ESC"; }}
            }}
        }}
    "#
    );
    let zellij = TestRunner::new(TERMINAL_SIZE).with_config(&config).start();
    let terminal = zellij.expect_pty_spawn();
    terminal.output(content);
    zellij.wait_until("Hop plugin requests permission", |grid| {
        grid.contains("Allow? (y/n)")
    });
    zellij.send_stdin(b"y");
    zellij.wait_until("permission granted and terminal restored", |grid| {
        grid.contains(marker) && !grid.contains("Allow? (y/n)")
    });
    zellij.send_stdin(&keys::ctrl('s'));
    zellij.wait_until("copy mode active", |grid| grid.contains("COPY"));
    (zellij, terminal, plugin_dir)
}

fn hop(zellij: &TestSession, character: &[u8]) {
    zellij.send_stdin(b"s");
    zellij.wait_until("plugin owns jump input", |grid| grid.contains("JUMP"));
    zellij.send_stdin(character);
}

fn copied(zellij: &TestSession, text: &str) {
    let expected = format!("\x1b]52;c;{}\x1b\\", STANDARD.encode(text));
    zellij.wait_until_raw_output("native selection emitted as OSC52", |bytes| {
        bytes
            .windows(expected.len())
            .any(|window| window == expected.as_bytes())
    });
}

#[test]
fn hop_labels_extend_native_selection_and_copy_original_text() {
    let (mut zellij, terminal, _plugin) = start(b"alpha beta gamma\r\nREADY", "READY");
    zellij.send_stdin(b"ggwv");
    zellij.wait_until("selection active", |grid| grid.contains("VISUAL"));
    hop(&zellij, b"a");
    zellij.wait_until("Hop labels overlay matches", |grid| {
        grid.contains("etd gfmmg")
    });
    terminal.output(b"\r\nBUFFERED");
    // a at columns 0, 4, 9, 12, 15 gets a,s,d,f,g respectively.
    zellij.send_stdin(b"dy");
    copied(&zellij, "beta");
    zellij.wait_until("output released after copy", |grid| {
        grid.contains("BUFFERED") && !grid.contains("VISUAL")
    });
    zellij.quit();
}

#[test]
fn hop_unicode_coordinates_and_escape_preserve_native_copy() {
    let (mut zellij, _, _plugin) = start("界e\u{301} beta\r\nREADY".as_bytes(), "READY");
    zellij.send_stdin(b"ggv");
    hop(&zellij, b"e");
    zellij.wait_until("labels use native cells after wide character", |grid| {
        grid.contains("a bsta")
    });
    zellij.send_stdin(b"a");
    zellij.wait_until("unicode jump finished", |grid| !grid.contains("JUMP"));
    zellij.send_stdin(b"y");
    copied(&zellij, "界e\u{301}");
    zellij.send_stdin(&keys::ctrl('s'));
    zellij.wait_until("new copy session", |grid| grid.contains("COPY"));
    hop(&zellij, b"z");
    zellij.wait_until("no matches cancels jump", |grid| !grid.contains("JUMP"));
    hop(&zellij, b"e");
    zellij.wait_until("labels shown again", |grid| grid.contains("a bsta"));
    zellij.send_stdin(&keys::ESC);
    zellij.wait_until("escape cancels only jump", |grid| {
        !grid.contains("JUMP") && grid.contains("COPY") && grid.contains("beta")
    });
    zellij.send_stdin(b"ggwvey");
    copied(&zellij, "beta");
    zellij.quit();
}

#[test]
fn hop_pages_dense_matches_and_accepts_activation_key_as_a_label() {
    let content = format!("{}\r\nREADY", "x ".repeat(30));
    let (mut zellij, _, _plugin) = start(content.as_bytes(), "READY");
    zellij.send_stdin(b"ggv");
    hop(&zellij, b"x");
    zellij.wait_until("first page labeled", |grid| grid.contains("s d f g h j k"));
    zellij.send_stdin(&keys::TAB);
    zellij.wait_until("second page labeled", |grid| {
        grid.contains("x x x x a s d f")
    });
    zellij.send_stdin(b"ay");
    copied(&zellij, &format!("{}x", "x ".repeat(26)));
    zellij.send_stdin(&keys::ctrl('s'));
    zellij.wait_until("second native session", |grid| grid.contains("COPY"));
    zellij.send_stdin(b"ggv");
    hop(&zellij, b"x");
    zellij.wait_until("labels restored", |grid| grid.contains("s d f g h j k"));
    zellij.send_stdin(b"sy");
    copied(&zellij, "x x");
    zellij.quit();
}

#[test]
fn hop_resize_and_plugin_reload_release_keyboard_ownership() {
    use zellij_utils::input::{actions::Action, layout::RunPluginOrAlias};
    let (mut zellij, terminal, _plugin) = start(b"alpha beta gamma\r\nREADY", "READY");
    zellij.send_stdin(b"gg");
    hop(&zellij, b"a");
    zellij.wait_until("labels before resize", |grid| grid.contains("etd gfmmg"));
    let mut size = TERMINAL_SIZE;
    size.cols -= 5;
    zellij.resize(size);
    zellij.wait_until("resize invalidates jump", |grid| {
        grid.contains("COPY CANCELED")
    });
    zellij.send_stdin(b"d");
    zellij.wait_until("resize exits copy", |grid| !grid.contains("COPY CANCELED"));
    terminal.output(b"\r\nAFTER RESIZE");
    zellij.wait_until("native terminal remains usable", |grid| {
        grid.contains("AFTER RESIZE")
    });
    zellij.send_stdin(&keys::ctrl('s'));
    zellij.wait_until("copy mode after resize", |grid| grid.contains("COPY"));
    zellij.send_stdin(b"gg");
    hop(&zellij, b"a");
    zellij.wait_until("labels before reload", |grid| grid.contains("etd gfmmg"));
    zellij
        .main_client()
        .send_action(Action::StartOrReloadPlugin {
            plugin: RunPluginOrAlias::from_url(
                &format!("file:{}", _plugin.path().join("copy-hop.wasm").display()),
                &None,
                None,
                None,
            )
            .unwrap(),
        });
    zellij.wait_until("reloading plugin removes labels and capture", |grid| {
        !grid.contains("JUMP") && grid.contains("lpha beta gamma")
    });
    zellij.send_stdin(b"ggwvey");
    copied(&zellij, "beta");
    zellij.quit();
}

#[test]
fn hop_keeps_jump_motion_and_yank_in_keyboard_order() {
    let (mut zellij, _, _plugin) = start(b"alpha beta gamma\r\nREADY", "READY");
    zellij.send_stdin(b"ggwv");
    zellij.wait_until("visual selection", |grid| grid.contains("VISUAL"));
    zellij.send_stdin(b"s");
    zellij.wait_until("jump input captured", |grid| grid.contains("JUMP"));
    zellij.send_stdin(b"adhy");
    copied(&zellij, "bet");
    zellij.quit();
}

#[test]
fn hop_preserves_focused_prompt_keyboard_priority() {
    use zellij_integration_tests::CliCaller;
    let (mut zellij, terminal, _plugin) = start(b"alpha beta gamma\r\nREADY", "READY");
    hop(&zellij, b"a");
    zellij.wait_until("labels before prompt", |grid| grid.contains("etd gfmmg"));
    let prompt = zellij.run_prompt(
        &["confirm", "Keep these keys?"],
        CliCaller::from_pane(terminal.terminal_id()),
    );
    zellij.wait_until("focused prompt", |grid| grid.contains("Keep these keys?"));
    zellij.send_stdin(&keys::ENTER);
    assert_eq!(prompt.wait_for_exit().exit_code, 0);
    zellij.wait_until("prompt receives Enter", |grid| {
        !grid.contains("Keep these keys?")
    });
    zellij.quit();
}

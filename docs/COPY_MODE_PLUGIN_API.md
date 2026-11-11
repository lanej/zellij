# Native copy-mode plugin API

External plugins can choose cursor targets and temporary labels while Zellij owns the original pane, frozen text, selection, clipboard, and input ordering. The API exposes the native selection primitives; plugins choose matching, label assignment, and keymap policies. Native copy motions remain available.

`zellij_tile::prelude::copy_mode` exposes generated protobuf request and response types, and `copy_mode(Request)` sends a request through the host. The server operates only on the acting client's active terminal copy session.

| Operation | Request | Outcome |
| --- | --- | --- |
| `Read` | No token | A snapshot with a token, viewport dimensions, and visible cells containing absolute frozen row, display column, display width, and grapheme text |
| `SetLabels` | Token and visible cell positions with single ASCII alphanumeric labels | Updates this plugin's temporary overlays without returning pane text |
| `MoveCursor` | Token and an absolute frozen cell position | Moves the native cursor while retaining its selection anchor; clears labels and releases capture |
| `ClearLabels` | Token | Removes this plugin's labels and releases capture |

Reads require `ReadPaneContents`; mutations require `ChangeApplicationState`. Invalid ownership, expired tokens, malformed positions, and conflicting label ownership return errors. Mutation responses contain no snapshot. A host timeout reports that the outcome is unknown. Positions use terminal cells rather than UTF-8 offsets, and mutations must address grapheme boundaries. A geometry change, copy exit, or owner disconnect invalidates the session. Output buffering and copying use the existing native paths; overlays never alter copied text.

A plugin may call `intercept_key_presses()` with `InterceptInput` permission before setting labels. During native capture, `CustomMessage("copy_mode_key", "<session_id>:<key>")` delivers input before ordinary keybinding dispatch. Zellij retains a bounded FIFO and delivers one key at a time. Each successful label update acknowledges that key. Moving or clearing labels drains queued keys through native copy handling before later input can overtake them. Capture exceeding 1,024 queued keys exits copy mode. Popup, live-view, and passthrough ownership retain precedence. Cleanup is scoped to both client and plugin, including mode changes, disconnect, reload, and unload.

The existing test fixture exercises literal read, label, move, and clear operations in `zellij-integration-tests/tests/copy_mode_plugin_api.rs`. It contains no character matching, jump-label assignment, paging, or user keymap policy. Build the fixture and select the integration-test target explicitly so no other plugin assets need rebuilding:

```sh
cargo build --locked -p fixture-plugin-for-tests --target wasm32-wasip1
ZELLIJ_COPY_MODE_FIXTURE_WASM="$PWD/target/wasm32-wasip1/debug/fixture-plugin-for-tests.wasm" \
  cargo test --locked -p zellij-integration-tests --test copy_mode_plugin_api
```

Narrow CI likewise supplies a freshly built fixture through `ZELLIJ_COPY_MODE_FIXTURE_WASM`. The checks observe native OSC 52 emission, not host clipboard acceptance.

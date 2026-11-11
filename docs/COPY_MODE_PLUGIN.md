# Copy-mode Hop plugin

The `copy-hop` plugin adds character jumps to native copy mode. It reads a frozen viewport and chooses labels; Zellij keeps the original terminal pane, cursor, selection, and clipboard. The built-in Vim motions remain available. This is the first plugin path, before moving the rest of the motion policy out of the fork.

## Build and configure

Build the fork runtime and plugin from the same checkout. Install the runtime through the fork's normal build process, then build and install the plugin:

```sh
cargo build -p copy-hop --target wasm32-wasip1
mkdir -p ~/.local/share/zellij/plugins
cp target/wasm32-wasip1/debug/copy-hop.wasm ~/.local/share/zellij/plugins/copy-hop.wasm
```

Use the absolute path to the installed plugin in your config:

```kdl
plugins {
    copy-hop location="file:/absolute/path/to/copy-hop.wasm"
}
load_plugins {
    "copy-hop"
}
keybinds {
    copy {
        bind "s" {
            MessagePlugin "copy-hop" {
                name "hop"
            }
        }
        bind "Esc" { CopyModeInput "ESC"; }
    }
}
```

Grant the plugin's requested permissions when it first loads. It needs pane-content access, application-state changes, input interception, and mode updates. It does not invoke an editor or execute host commands. The activation binding can be changed; while jumping, the native capture receives keys before ordinary keybinding dispatch.

Enter native copy mode, optionally start a selection with `v` or `V`, and press `s`. The frame shows `JUMP`. Type a character, then the label displayed at the target. The jump extends an active selection, and `y` copies through the native clipboard path. Matching is case-sensitive and uses the first character of each terminal grapheme. Labels replace rendered glyphs temporarily; they never replace the copied text.

There are 26 single-letter labels, assigned in display order. Tab pages through additional matches. Escape cancels the jump and preserves the current selection; the next Escape uses ordinary copy-mode cancellation. No matches, an invalid label, or 15 seconds without completing the jump also cancels it. Resize invalidates the copy session; the next key exits safely. Plugin unload or reload clears its labels and releases its intercepted keys. The first version targets the visible viewport; scroll with the ordinary copy motions before starting another jump.

## Native contract

`zellij_tile::prelude::copy_mode` exposes generated protobuf request and response types, and `copy_mode(Request)` sends them through the plugin host. The server operates only on the acting client's active terminal copy session.

| Operation | Request | Outcome |
| --- | --- | --- |
| `Read` | No session token | A snapshot containing a token, viewport dimensions, and cells with absolute frozen row, display column, display width, and grapheme text |
| `SetLabels` | Token and visible cell positions with single ASCII alphanumeric labels | Updates this plugin's temporary render overlays; returns no pane contents |
| `MoveCursor` | Token and an absolute frozen cell position | Moves the native cursor, retaining the selection anchor; clears labels and releases this plugin's interception atomically |
| `ClearLabels` | Token | Removes this plugin's labels and releases its interception |

`Read` requires `ReadPaneContents`. Mutations require `ChangeApplicationState`. The server validates client ownership, token freshness, row/column bounds, and grapheme boundaries before applying a mutation. A token identifies one frozen session and is never reused during the server lifetime. Resize, copy exit, and owner disconnect make its positions unusable. Mutations return no snapshot, so permission to move a cursor does not grant permission to read terminal text. An unavailable session, expired token, malformed position, or conflicting label owner returns an error. A host timeout reports that the outcome is unknown.

While the plugin intercepts copy input, `CustomMessage("copy_mode_key", "<session_id>:<key>")` delivers each key with the session that captured it. Zellij retains a bounded FIFO and delivers one captured key at a time. A label update acknowledges that key. Moving or clearing labels drains queued keys through native copy handling before later input can overtake them. A stalled plugin exceeding 1,024 queued keys causes copy mode to exit safely. Interception cleanup is scoped to the owning plugin, including reload and unload.

The native fork patch contains the session, validation, cursor/selection effects, overlays, and transport. Character matching, label ordering, paging, and interaction state live in `default-plugins/copy-hop`. Adding different target matching does not require changing native cursor handling.

## Verification

`cargo xtask integration-test --test copy_hop` builds the plugin assets and runs the real WASM plugin through character jumps, selection, rapid label-plus-yank input, Unicode cells, cancellation, paging, resize, and reload. `keyboard_copy` retains coverage for the built-in behavior. These tests observe emitted OSC 52 bytes; they do not establish host clipboard acceptance.

# Keyboard copy mode

Enter `Copy` mode to move a cursor over the current pane's scrollback, select text, and copy it through Zellij's configured clipboard. The pane stays in place; no editor, plugin viewer, or mouse is involved. Ordinary scroll mode and `EditScrollback` remain available.

From the standard keybindings, enter scroll mode and press `v`. Alternatively bind `SwitchToMode "Copy"` directly to the shortcut you use for scrollback. With a custom keymap, give copy mode its own Escape binding so a shared Escape binding does not bypass selection cancellation:

```kdl
keybinds {
    normal {
        bind "Alt y" { SwitchToMode "Copy"; }
    }
    copy {
        bind "Esc" { CopyModeInput "ESC"; }
    }
}
```

| Keys | Behavior |
| --- | --- |
| `h/j/k/l`, arrows | Move by character or display row |
| `w/b/e` | Next word, previous word, end of word; punctuation forms separate words |
| `W/B/E` | The same motions with whitespace-delimited words |
| `0`, `^`, `$` | Start of row, first nonblank character, last character |
| `gg`, `G` | First or last retained display row |
| `42G`, `42gg` | Jump to retained display row 42, counting from 1 |
| `10j`, `3w` | Repeat a motion |
| `Ctrl-u/d`, `Ctrl-b/f`, Page Up/Down | Move by half or full pages |
| `v`, Space | Toggle character selection, including the cursor character |
| `V` | Toggle whole logical-line selection, including soft-wrapped continuations |
| `y` | Copy the selection and return to the configured base mode |
| Escape | Cancel the selection; press again to leave copy mode |
| `q`, Enter, Ctrl-c | Leave copy mode |

The pane frame shows `COPY`, `VISUAL`, or `VISUAL LINE`, the current retained display-row number, and a pending count. Character movement and selection follow whole terminal graphemes, including wide characters. Copied text has no ANSI styling and rejoins soft-wrapped rows. Line selection appends a newline. Clipboard output follows existing Zellij configuration; an OSC 52 request does not establish that the host terminal accepted it.

## Native boundary and lifecycle

Before this change, scroll mode exposed a viewport offset and mouse selection; arbitrary keyboard selection required `EditScrollback`. The new `InputMode::Copy` routes otherwise unbound keys through `Action::CopyModeInput { key: String }`. The string uses Zellij's existing `KeyWithModifier` display representation with Shift-only printable characters normalized, preserving modifier and special-key distinctions across client/server and plugin protobuf messages. New enum values and fields are appended relative to the current upstream base; upstream wire numbers and keybindings keep their meaning. Clients and servers using the new mode must run the same build. Mode-update events encode a legacy Scroll value with an additive `copy_mode_active` flag: existing status plugins keep working, while updated readers recover Copy. The native frame provides exact copy-mode feedback.

The terminal grid owns `Option<CopyMode>`, containing the owner client, an immutable snapshot of retained native rows, a cursor, an optional selection anchor, character/line selection state, viewport origin, and pending motion/count. The snapshot preserves text identity and wrapping for word motions and copying even when the native scrollback packs offscreen rows. Rendering continues through the existing pane viewport and selection; snapshot rows are not exported or displayed by another program. The tab boundary accepts key input, obtains `Continue`, `Yank(text)`, or `Exit`, and sends text through the existing clipboard path. Clipboard requests retain the existing success/failure reporting behavior.

One client owns copy mode per pane, matching the shared native pane viewport. Another client cannot replace the owner, yank its selection, or move the shared viewport underneath it. New PTY output uses the existing bounded scrollback-input queue while copying, including at the live bottom. Leaving the mode or disconnecting the owner releases queued output. A geometry change invalidates the snapshot and selection rather than applying old coordinates to reflowed text. The existing output-queue safety cap can also cancel the snapshot; the next copy-mode key exits without copying stale text.

The new mode does not yet implement search input, rectangular selection, marks, or Vim text objects. Existing native search remains available before entering copy mode.

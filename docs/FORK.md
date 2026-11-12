# Lanej rebased runtime candidate

This branch rebases the selected Lanej runtime changes onto upstream `c8585459c37186fae1321727ad8dd4c96480ef41`. It preserves the fork merge topology; the published pre-rebase runtime and installed dotfiles pin remain unchanged.

The canonical branch contracts, contribution inventory, checkpoints, and promotion process live on [fork-maintenance](https://github.com/lanej/zellij/blob/fork-maintenance/docs/FORK.md). Independent upstream contributions start from upstream main. Branches based on the runtime are fork integration reviews and may include unrelated changes in an upstream PR diff.

This candidate is a new wire revision relative to the pre-rebase fork. Upstream now owns Options field 88 (`stacked_pane_list_orientation`), plugin action 155 (`ToggleStackListOrientation`), and IPC action 194 (the corresponding orientation action). The fork additions use Options field 89 (`hide_scroll_indicator_when_idle`), plugin action 157 (`CopyModeInput`), and IPC action 196 (`copy_mode_input`).

Before installing this generation, regenerate dotfiles client bindings from the selected runtime commit, rebuild affected fork-specific plugins, and validate the combined client/server/plugin set. Do not send pre-rebase fork copy-mode commands to the candidate: their old numbers mean different actions. Restart existing Zellij servers and clients together. Legacy plugin mode-update projection remains available; it does not establish compatibility for fork-specific command IDs.

Publication of this candidate does not promote it to stable. Keep the previous installation pin and immutable checkpoint until Linux/macOS CI and affected real-terminal workflows pass for the exact new revision.

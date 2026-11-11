# Lanej runtime

The `lanej-runtime` branch carries the native fixes used by [lanej/dotfiles](https://github.com/lanej/dotfiles). Its upstream base is [`bf517077`](https://github.com/zellij-org/zellij/commit/bf517077c848d91f75091f66d1996cfeb84f4732), which includes OSC 7501 and the earlier startup repair. Dotfiles installs a pinned commit from this branch through Cargo; it no longer applies patches during installation.

The integration branch currently contains these independent changes on top of upstream:

- Scope command completion to the requesting client, so an unrelated pane closure cannot finish a CLI query before its reply arrives. See [dotfiles #121](https://github.com/lanej/dotfiles/issues/121).
- Flush queued client exit messages before server shutdown, with one shared 500 ms deadline for stalled clients and explicit IPC write/flush errors. See [dotfiles #120](https://github.com/lanej/dotfiles/issues/120).

- Preserve initial client registration while a new session is pending ([#1](https://github.com/lanej/zellij/pull/1)).
- Serialize and atomically persist permission-cache updates ([#3](https://github.com/lanej/zellij/pull/3)).
- Route popup paste to the correct client and preserve Unicode plugin input ([#5](https://github.com/lanej/zellij/pull/5)).
- Make idle scroll indicators configurable ([#6](https://github.com/lanej/zellij/pull/6)).

The manager help shortcut ([#4](https://github.com/lanej/zellij/pull/4)) and recursive nested-session guard ([#7](https://github.com/lanej/zellij/pull/7)) remain separate open changes as of 2026-11-10. Neither is a prerequisite for [keyboard copy mode](KEYBOARD_COPY_MODE.md), which is developed on `feature/keyboard-copy-mode` from `lanej-runtime`.

The Lanej runtime workflow runs the existing native regressions on Linux and macOS. Dotfiles keeps the terminal workflows that exercise its configuration and helpers. The pending first-client registration repair is integrated through PR #1.

Keep `main` aligned with upstream and carry these fixes on `lanej-runtime`. When updating the branch, preserve the revisions already pinned by dotfiles. Remove a local fix once upstream includes an equivalent repair and the same regressions pass. Update the dotfiles pin after both repositories' workflows pass; restart existing sessions to use the new server.

The 2026-11-10 review found `main` at the pinned upstream base, 31 commits behind upstream `main` at `644d78ce`. Handle that upstream update as separate integration work: inspect equivalents of the local fixes, reconcile them, and rerun the runtime regressions before moving the dotfiles pin. Copy mode is based on the existing integration branch so it can be reviewed independently of that update.

Keep independent capabilities on branches based on `lanej-runtime`, with PRs targeting that integration branch. Stack PRs only when one actually depends on another's unmerged behavior. Preserve the integration branch and pinned revisions; reorganizing independent work does not require rewriting published runtime history.

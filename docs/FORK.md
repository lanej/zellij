# Lanej runtime

The `lanej-runtime` branch carries the native fixes used by [lanej/dotfiles](https://github.com/lanej/dotfiles). Its upstream base is [`bf517077`](https://github.com/zellij-org/zellij/commit/bf517077c848d91f75091f66d1996cfeb84f4732), which includes OSC 7501 and the earlier startup repair. Dotfiles installs a pinned commit from this branch through Cargo; it no longer applies patches during installation.

Two fixes remain on top of upstream:

- Scope command completion to the requesting client, so an unrelated pane closure cannot finish a CLI query before its reply arrives. See [dotfiles #121](https://github.com/lanej/dotfiles/issues/121).
- Flush queued client exit messages before server shutdown, with one shared 500 ms deadline for stalled clients and explicit IPC write/flush errors. See [dotfiles #120](https://github.com/lanej/dotfiles/issues/120).

The Lanej runtime workflow runs the existing native regressions on Linux and macOS. Dotfiles keeps the terminal workflows that exercise its configuration and helpers. A separate upstream startup race remains: wait for a new session's first shell prompt before issuing CLI queries to it.

Keep `main` aligned with upstream and carry these fixes on `lanej-runtime`. When updating the branch, preserve the revisions already pinned by dotfiles. Remove a local fix once upstream includes an equivalent repair and the same regressions pass. Update the dotfiles pin after both repositories' workflows pass; restart existing sessions to use the new server.

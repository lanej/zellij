# Maintaining the Lanej Zellij fork

This guide belongs on `fork-maintenance`, separate from both upstream contributions and the installed runtime. The branch inventory is [fork/topics.toml](../fork/topics.toml). Its heads and check results are a dated snapshot; refresh them when advancing a branch. Josh owns contribution scope, runtime promotion, and upstream submission text.

## Branch contracts

| Branch | Purpose | Allowed changes |
| --- | --- | --- |
| `main` | Clean upstream mirror | Fast-forward from `zellij-org/zellij:main`; no fork patches or documentation |
| `fix/upstream-*` | Independent bug-fix candidates | One capability on upstream main; no runtime merge ancestry or fork CI |
| `lanej-runtime` | Combined development and daily-use runtime | Forward integration of selected features and fork-only CI; never rewrite published history |
| `lanej-stable` | Conservative fallback | Advance only to an exact runtime commit with passing native CI and recorded manual acceptance |
| `fork-maintenance` | Fork operating guide and inventory | Fork policy, topic tracking, and evidence links; exclude from contribution branches |
| Existing feature/fix branches | Preserved fork review history | Keep active PR bases and heads intact until their owners coordinate migration |

Upstream contribution branches exchange only their capability's code, tests, and relevant public documentation. Fork CI, this guide, the inventory, integration conflict resolutions, and local agent instructions stay within fork maintenance/integration. A PR based on `lanej-runtime` is a fork integration review, not evidence that its head is clean for upstream.

## Checkpoints as of 2026-11-11

- Upstream mirror: `c8585459c37186fae1321727ad8dd4c96480ef41`.
- Runtime and existing dotfiles installation pin: `5f8dd464fc97a0bb77feaaec617f0ac3f8cd5026`. Preserved by annotated tag `lanej-runtime-checkpoint-2026-11-11`; no installation change was made during organization.
- Last CI-green fallback: `aa8b83d76ebf5163f68188bd3a4630b62b251d03`, referenced by `lanej-stable` and annotated tag `lanej-ci-green-2026-11-11`. This predates keyboard copy mode. Both Linux and macOS native-regressions passed in [run 42543514823](https://github.com/lanej/zellij/actions/runs/42543514823). That check coverage does not establish every terminal workflow.
- Runtime [run 43044691280](https://github.com/lanej/zellij/actions/runs/43044691280) passed on macOS; its initial Linux attempt failed at apt installation with DNS errors, before executing the tests. The failed Linux job was retried during organization. Check the live run before treating the runtime head as CI-green.

Keep published checkpoint tags immutable. Never move the installer pin just because a branch advanced. Replacing the executable does not update existing Zellij session servers.

## Prepared independent candidates

The following branches each start directly at the upstream mirror above. Their extracted diffs passed `git diff --check` and `cargo +1.95.0 fmt --all -- --check`. Independent review checked their scope and visible dependencies. No compilation, behavioral tests, current-upstream bug reproduction, or maintainer acceptance has been established for these rebased heads. They are preparation branches, not submission-ready or stable-runtime claims.

| Branch | Capability | Source |
| --- | --- | --- |
| `fix/upstream-cli-completion` | Scope a CLI query response to its requesting client | `8072318b` |
| `fix/upstream-shutdown-flush` | Flush client exit messages with a bounded shutdown deadline | `18be8a2a` |
| `fix/upstream-atomic-permission-cache` | Serialize and atomically persist permission-cache updates | `315e27e1` |
| `fix/upstream-plugin-unicode` | Preserve Unicode scalars in plugin key conversion | Unicode paths from `876c9e05` |
| `fix/upstream-popup-input` | Route paste to the focused popup | Server paths from `876c9e05`, then `7d738ed5` |
| `fix/upstream-manager-help` | Make manager key help accessible without function keys | `c1d81ae1` |

The original popup/Unicode fork PR combines two independent changes; the contribution branches separate them. Existing fork PRs #4, #7, and #9 and all original topic branches remain intact. There were no upstream PRs by lanej at the organization audit.

The inherited Rust workflow runs on pushes to main, PRs targeting main, or workflow_dispatch. Pushing a topic branch alone does not run it. Run the applicable local tests and checks against that branch; manually dispatch the upstream workflow against the exact branch when useful. Its Build/Test jobs do not substitute for all submission requirements or runtime acceptance.

## Work requiring adaptation

| Preserved branch | Remaining work |
| --- | --- |
| `fix/pending-initial-client` | Resolve screen-routing overlap and reproduce the issue on current upstream |
| `fix/nested-session-cycles` (#7) | Separate inherited test-setup dependencies and adapt tests to upstream |
| `fix/hide-scroll-label` | Reconcile configuration wire fields and regenerate affected assets/snapshots |
| `feature/keyboard-copy-mode` | Reconcile action/protocol identifiers, fork-influenced tests, and configuration snapshots |
| `codex/copy-mode-hop-plugin` (#9) | Depends on copy mode; native plugin controls and Hop are separate subsequent contributions |

The failed extraction attempts were not published as complete candidates. A partial scroll-indicator extraction remains only on local `archive/2026-11-11/scroll-indicator-extraction`; its original full implementation remains on the published `fix/hide-scroll-label`. Do not use the partial extraction for integration or submission. The closed compact-title branch is preserved, not selected for promotion.

For protocol changes, inspect current upstream field numbers and enum variants before adding fields. Regenerate protobuf Rust assets with the documented toolchain and update tests from the actual result. Do not import another fork capability merely to make copied snapshots or test setup compile. Stack contributions only for actual API/behavior dependencies, and name those dependencies explicitly.

## Upstream policy and submission

Read the current [CONTRIBUTING.md](../CONTRIBUTING.md), [upstream copy](https://github.com/zellij-org/zellij/blob/main/CONTRIBUTING.md), and Code of Conduct before contribution work. Zellij asks contributors to establish maintainer willingness and availability for larger roadmap projects; minor fixes may wait. Keep larger features in the fork until Josh establishes upstream interest. Do not contact maintainers automatically.

Zellij requests avoiding LLM-generated issue and PR descriptions. Josh writes upstream submission text. Agent-authored maintenance notes, including this guide, are not submission prose. Do not copy them into issues or PR descriptions or automatically open upstream issues/PRs.

Before submission, refresh upstream, verify the bug still exists where applicable, inspect the complete diff against upstream main, run the guide's cargo fmt and cargo test requirements plus applicable repository checks, and record unresolved failures accurately. For nontrivial changes, consider an issue discussion first. Follow the guide's contextual Result and error-logging conventions when adapting code.

Once a contribution merges upstream, record the upstream commit and any semantic differences. Retire the outstanding patch from the next integration plan only after verifying the upstream equivalent preserves the intended behavior. Keep old runtime commits and tags reachable.

## Runtime integration and promotion

Integrate selected, reviewed topics into a candidate based on lanej-runtime; incorporate upstream updates as a separate reviewable change. Run Linux/macOS native regressions plus affected terminal/plugin workflows, using the exact candidate commit. A successful branch merge or a CI run on another head does not validate the candidate.

Record manual acceptance evidence and exact tested heads in this guide/inventory before promotion. On a dedicated stable checkout, advance lanej-stable by fast-forward only to the validated runtime commit; create a new immutable checkpoint tag. Then update the dotfiles commit pin in its own reviewed change and verify installation and session restart. Keep the previous pin/tag available for rollback.

For a fresh integration experiment, start a disposable candidate from an explicit upstream commit and integrate only the outstanding selected topics. If its ancestry diverges from the published runtime, do not blindly merge it back to remove old patches. Either reconcile through a forward upstream merge on the existing runtime or publish a new versioned runtime branch, validate it, and change the installer pin explicitly. Never force-push lanej-runtime or lanej-stable.

Branch protection prevents force pushes and deletion of main, lanej-runtime, and lanej-stable. Stable updates additionally require the existing `native-regressions (ubuntu-latest)` and `native-regressions (macos-latest)` contexts. These checks cover selected native regressions; manual acceptance and other applicable checks remain separate obligations. The workflow runs against lanej-runtime before stable promotion.

## Local worktrees and remotes

The local upstream remote fetches zellij-org/zellij and has a deliberately unusable push URL. The default push remote is origin (lanej/zellij); main tracks upstream/main, with fast-forward-only pulls. Never configure upstream push access as a side effect of fork maintenance.

Use Treehouse from the primary checkout, /private/tmp/zellij-keyboard-copy-mode. Acquire a topic from local main with `treehouse get --lease --json --lease-holder <task> --branch fix/<topic> --base main`. Acquire fork work from local lanej-runtime explicitly. Preserve the returned lease identity, copy the excluded AGENTS.md into the acquired root if absent, and return only task-owned leases after preserving changes and inspecting valuable ignored files. Do not nest pools or use gh pr checkout to switch worktrees.

AGENTS.md is local and excluded with /AGENTS.md in the shared Git info/exclude. Its instructions must be present in each new checkout. Exclusions propagate across worktrees; file contents do not. Do not force-add it. This checkout lives in temporary storage: published refs and this maintenance branch preserve Git work, but local remotes, excludes, and agent instructions need recreation if the checkout is replaced.

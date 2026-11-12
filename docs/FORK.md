# Maintaining the Lanej Zellij fork

This guide and [fork/topics.toml](../fork/topics.toml) live on fork-maintenance, separate from upstream contribution branches. The inventory records exact heads, original source commits, preserved fork branches, and real dependencies. Heads and validation are dated snapshots; refresh them when advancing a branch. Josh owns contribution scope, runtime promotion, and upstream submission text.

## Branch contracts

| Branch | Purpose | Rule |
| --- | --- | --- |
| main | Clean upstream mirror | Fast-forward from zellij-org/zellij:main; no fork patches/docs |
| fix/upstream-* and feature/upstream-* | Upstream contribution candidates | One capability based on upstream main; stack only for actual dependencies |
| lanej-runtime | Published runtime generation | Preserve its history and pinned revisions; accept validated forward integration |
| lanej-runtime-next | Rebased next-generation runtime candidate | Current upstream plus selected fork changes; not promoted or installed |
| lanej-stable | Conservative fallback | Advance only to an exact validated descendant; no automatic promotion |
| fork-maintenance | Fork guide and contribution inventory | Keep fork policy and tracking out of upstream diffs |
| Existing topic branches | Preserved fork review history | Keep active PR heads/bases intact until owners coordinate migration |

Contribution branches exchange only their capability's code, tests, and relevant public documentation. Fork CI, this guide, the inventory, integration conflict resolutions, and local AGENTS.md remain within fork maintenance/integration. A PR based on lanej-runtime or lanej-runtime-next is a fork integration review; those branches are unsuitable bases for independent upstream PRs.

## Upstream scope

The upstream queue contains core fixes and capabilities required by external plugins. User plugin implementations, matching/label-assignment policy, installation, and keymaps remain outside that queue. Required exhaustive-match compatibility in existing plugins and literal-operation test fixtures are integration requirements, not new shipped plugin features.

Session-manager help, Hop, scroll-indicator customization, and compact agent titles remain preserved fork/external-plugin work. Do not submit those branches as part of the current upstream effort.

## Checkpoints as of 2026-11-11

- Upstream mirror: c8585459c37186fae1321727ad8dd4c96480ef41.
- Published runtime: b960a9e46e70454a703bede0eba4b49d8db246ae, advanced only by the Clippy fix. The dotfiles installation pin remains 5f8dd464fc97a0bb77feaaec617f0ac3f8cd5026, preserved by annotated tag lanej-runtime-checkpoint-2026-11-11.
- Last CI-green fallback: aa8b83d76ebf5163f68188bd3a4630b62b251d03, referenced by lanej-stable and annotated tag lanej-ci-green-2026-11-11. It predates keyboard copy mode. Linux and macOS native-regressions passed in [run 42543514823](https://github.com/lanej/zellij/actions/runs/42543514823); that coverage does not establish every terminal workflow.
- Published runtime [run 43044691280](https://github.com/lanej/zellij/actions/runs/43044691280) passed on macOS but failed on Linux. Its original Linux attempt failed at package installation with DNS errors, before tests; the failed job was retried and again failed at apt DNS resolution, before test execution.
- Next-generation runtime: 3c6d79d01c661320616625833042de323584bd09. Its upstream/fork rebase is resolved, with generated bindings refreshed using official protoc 23.4 and cargo xtask proto. Validation and promotion status are recorded below; publication is not stable acceptance.

Checkpoint tags are immutable. Advancing a branch does not change the installer pin. Replacing an executable does not update existing session servers.

## Upstream candidates

Eight candidates are independent of one another; the plugin API depends on the native selection capability. Keep those two capabilities as separate cohesive changes. Do not add default shortcuts or optional plugins to demonstrate the API.

| Candidate | Scope | Prerequisite |
| --- | --- | --- |
| fix/upstream-cli-completion | Requesting-client CLI reply completion | None |
| fix/upstream-shutdown-flush | Bounded exit-message flush | None |
| fix/upstream-pending-initial-client | Initial registration lifecycle | None |
| fix/upstream-atomic-permission-cache | Concurrent cache persistence | None |
| fix/upstream-plugin-unicode | Unicode scalar conversion | None |
| fix/upstream-popup-input | Interactive paste and explicit CLI destinations | None |
| fix/upstream-nested-session-cycles | Held: separate lifecycle fixes from opt-in policy | None |
| feature/upstream-keyboard-copy-mode | Native selection engine and opt-in mode | None |
| feature/upstream-copy-mode-plugin-api | Guarded cells, labels, cursor movement and ordered capture | Native selection |

The native selection candidate leaves shipped preset files unchanged. Existing shared keyboard/mouse rules naturally expand into the additive mode; regenerated snapshots were checked to contain only those new-mode sections, with every existing mode unchanged. Its full affected libraries pass: 2,805 server tests and 1,209 utility tests; existing ignored tests remain ignored. The two small bars changes only make enum matches exhaustive and compile on wasm32-wasip1.

The plugin API candidate is refreshed from the core-only fork PR. Native copy/protocol regressions and the cell/session-token regression pass. All four whole-app API tests pass with a freshly compiled existing fixture, covering input ordering/output recovery, Unicode boundaries, resize/reload cleanup, and focused-popup precedence. No Hop implementation, plugin installation, or jump/keymap policy is included. The public API documentation describes the capability neutrally and provides a focused fixture build/test command.

The nested-session candidate passes all 76 selected server regressions, with five existing ignored tests. Review found that retaining every disconnected client ID in Screen was unbounded. The repair uses authoritative SessionState registration instead: pending handshakes remain valid, membership/ancestry/enqueue share the registration lock, removed/deferred frames are rejected, and legitimate reused IDs work. The screen tombstones were removed. The lifecycle fix is in fork PR #7. Its original mandatory cycle restriction has been superseded by opt-in policy: allow_nested_sessions and allow_cyclic_sessions default to true; setting both false declines starting or hosting nested sessions. Ordinary top-level switches and detached creation remain available. The revised behavior uses stable UUIDs, preserves unknown ancestry across mixed-version peers, and sends a departure frame before refusal teardown. The shared fork base now removes the duplicated output bounds comparison; default native-library Clippy passes on the refreshed fork PR head. The earlier independent upstream nested-session candidate is retained as historical source and is not submission-ready: separate its lifecycle fixes from the configurable policy before preparing upstream PRs.

The other six independent candidates retain their earlier source/format review status; the inventory does not claim that combined-runtime tests validate each individual head. Every upstream submission still needs its own applicable checks, reproduction where relevant, maintainer policy gates, and owner-authored submission text. Larger core features should be discussed with maintainers before submission.

## Fork PR state

| PR | Scope and state |
| --- | --- |
| [#4](https://github.com/lanej/zellij/pull/4) | Closed: bundled session-manager help is outside the selected scope; source branch retained |
| [#7](https://github.com/lanej/zellij/pull/7) | Draft policy review at 87255197: configurable client/host nesting preferences; independent code review approved; local validation recorded below; [new-head CI in progress](https://github.com/lanej/zellij/actions/runs/43128358237) |
| [#9](https://github.com/lanej/zellij/pull/9) | Ready for fork review at 9bc7fe5c; core-only; native-library Clippy and hosted Linux checks pass; hosted macOS run canceled after one hour |

Both active fork PRs forward-merge the shared runtime Clippy fix, keeping that maintenance change out of their capability diffs. PR #7 now adds the explicitly requested configurable nesting feature and remains unmerged for scope review. Their independent upstream candidates remain separate. Before this refresh, #7 had a failed hosted run and #9 had a canceled hosted run; these are not passing results for the new heads. Existing merged PRs and published runtime/stable history remain preserved. No upstream PR was opened or described by this work.

The prepared API head before this scope correction is preserved by annotated tag fork-checkpoint/copy-api-before-core-scope-2026-11-11. Its prepared branch was restacked with an explicit expected-head lease; no active PR used that prepared branch. The historical upstream-copy-hop branch remains only as preserved source and is ineligible for the current upstream queue.

Inherited Rust CI runs on main pushes, PRs targeting main, or explicit dispatch; publishing a candidate does not automatically validate it. Fork PR #7 now explicitly runs the nested lifecycle detectors in its native workflow. Both open fork PRs still need their current-head hosted results before any merge decision. These are fork integration reviews; upstream contribution heads remain separate and clean.

## Wire migration for the rebased runtime

| Contract | Pre-rebase fork meaning | Current upstream meaning | Rebased fork allocation |
| --- | --- | --- | --- |
| Options field 88 | hide_scroll_indicator_when_idle Boolean | stacked_pane_list_orientation string | Idle-scroll Boolean moves to field 89 |
| Plugin action 155 | CopyModeInput | ToggleStackListOrientation | CopyModeInput moves to 157; preserve upstream 155/156 |
| IPC action 194 | copy_mode_input | toggle_stack_list_orientation | copy_mode_input moves to 196; preserve upstream 194/195 |

This preserves current upstream wire assignments and intentionally breaks compatibility with the old fork-specific commands. Before installation, regenerate dotfiles bindings from the chosen runtime commit, rebuild affected fork-specific plugins, and validate the client/server/plugin set together. Never send old-fork copy commands to the new runtime: the old numbers mean other actions. Restart all existing session servers/clients together. Legacy plugin mode-event projection does not establish compatibility for fork-specific command IDs.

When upstream allocates another field/action, inspect its schema first, choose an unused identifier, regenerate Rust assets, and verify round trips and actual callers. Keep snapshot expectations tied to the real rendered result rather than importing another feature merely to make snapshots compile.

## Upstream policy

Read the current [CONTRIBUTING.md](../CONTRIBUTING.md), [upstream guide](https://github.com/zellij-org/zellij/blob/main/CONTRIBUTING.md), and Code of Conduct. Larger roadmap projects require checking maintainer willingness and availability first; minor fixes may wait. Keep local fork development separate from upstream acceptance, and do not contact maintainers automatically.

Zellij requests avoiding LLM-generated issue/PR descriptions. Josh writes upstream submission text; agent-authored maintenance notes are not submission prose and must not be copied into descriptions. Do not automatically open upstream issues or PRs. For nontrivial work, consider issue discussion first and link the existing discussion when applicable.

Before submission, refresh upstream, verify current bug reproduction where applicable, inspect the complete diff against the intended base, and meet the guide's cargo fmt and cargo test requirements plus applicable repository checks. Report failures/skips accurately. Use contextual Result errors and documented non_fatal logging when adapting code. After upstream merge, record the accepted commit and semantic differences, then retire the outstanding patch only after confirming the upstream equivalent preserves the intended behavior.

## Runtime validation and promotion

On macOS, the final runtime code passes 2,814 server library tests and 1,222 utility library tests with the pinned Rust toolchain. The server run excludes two temporary-file safety tests that also fail on unchanged upstream c8585459 when its checkout resides under /tmp; 185 server tests and two utility tests retain their existing ignored status. Four configuration snapshots were corrected after the rebase retained an obsolete F9 session-card binding in Copy mode.

The full server run exposed a fork popup regression that passes on unchanged upstream: generic WriteCharacter capture stole explicit CLI writes from their pane. The repair preserves interactive versus CLI provenance in the input router and passes the existing CLI-write detector, a Unicode bracketed-paste screen-thread detector, keyboard-popup routing, and rejection of pasted permission answers. The standalone popup candidate includes the same repair. Independent review approved the material fix, and the runtime workflow now runs those four regression tests explicitly.

The duplicated output bounds comparison has been removed without changing overlap behavior. The isolated fix/upstream-clippy-output-overlap candidate bc7865c96a5c2086fe7e06251fff59b07f4a9055 is based on upstream d9df94cc (the defect was still present there), and passes rustfmt, all 126 selected output tests, and default Clippy. The maintained runtime, next runtime, and both refreshed fork PR heads also pass default Clippy for zellij-server and zellij-utils native libraries with Rust 1.95.0, vendored_curl and web_server_capability, targeting aarch64-apple-darwin. These scoped checks retain inherited warnings; warnings-denied Clippy and all-target/workspace lint are not claimed. The previous strict upstream run produced 811 utility diagnostic messages. This maintenance fix is separate from the plugin-support queue.

The preceding runtime head ff852b1b50b034f27ad5766f2349f40c83f76dd2 had a failed [Linux/macOS CI run 43063026187](https://github.com/lanej/zellij/actions/runs/43063026187). Fresh hosted checks for 3c6d79d01c661320616625833042de323584bd09 are [in progress](https://github.com/lanej/zellij/actions/runs/43103290679) at this snapshot; the maintained runtime also has a [fresh run](https://github.com/lanej/zellij/actions/runs/43103289974). Actual dotfiles installation/client-binding refresh and real-terminal acceptance remain promotion gates. The published topic branches have structural/format checks; their individual behavioral readiness remains unclaimed.

The next runtime was rebased with merge topology retained. After replaying/reconciling all selected patches, history bridge 96972bac recorded the preserved runtime as a second parent while keeping the rebased tree exactly. That deliberate supersession merge permits future fast-forward promotion without force-pushing published runtime/stable refs. It is not an ordinary content merge or a general conflict-resolution shortcut. Independent upstream branches exclude this fork ancestry.

The Lanej runtime workflow now covers both lanej-runtime and lanej-runtime-next. Validate the exact final candidate head on Linux/macOS and run affected real-terminal/plugin workflows. Record manual acceptance and exact heads here before promotion. Then fast-forward lanej-runtime and lanej-stable to that validated descendant, create a new immutable checkpoint, and update the dotfiles pin in its own reviewed change with the coordinated wire migration. Retain the old tag/pin for rollback.

Branch protection prevents force pushes and deletion of main, lanej-runtime, and lanej-stable. Stable updates require native-regressions (ubuntu-latest) and native-regressions (macos-latest). Those contexts cover selected native regressions, not every acceptance claim. Do not force-push published runtime branches, automatically install an unvalidated candidate, or merge a freshly reconstructed tree merely because Git reports no conflicts.

## Local checkouts

The upstream remote fetches zellij-org/zellij and has a deliberately unusable push URL. Default pushes go to origin (lanej/zellij); main tracks upstream/main with fast-forward-only pulls. Recreate these local settings when cloning; do not add upstream push access as a maintenance side effect.

The permanent primary checkout is /Users/joshlane/src/zellij, checked out on fork-maintenance so this guide is immediately visible. The original temporary repository and other sessions' checkouts remain intact. Use Treehouse from the permanent primary checkout for new tasks. Acquire contribution work from local main with treehouse get --lease --json --lease-holder <task> --branch fix/<topic> --base main; acquire fork integration work explicitly from lanej-runtime or lanej-runtime-next. Preserve the returned lease identity and return only task-owned leases after preserving work and inspecting valuable ignored files. Do not nest pools or use gh pr checkout to switch worktrees.

AGENTS.md is intentionally untracked and excluded by /AGENTS.md in the shared Git info/exclude. Copy it into each new checkout/worktree if absent; exclusions propagate across worktrees, file contents do not. Do not force-add it. Published refs preserve Git work, while local configuration, excludes, and instructions must be recreated if a checkout is replaced.

## Configurable nesting policy review

Josh clarified that ordinary nesting and cyclic attachment are valid workflows; the desired restriction is a preference, like tmux's explicit nesting opt-in. Fork PR #7 at 87255197b73b9ff39b09925f1dc07a8e2cabd869 implements separate allow_nested_sessions and allow_cyclic_sessions settings, both defaulting to true. Setting both false declines both starting nested clients and hosting guests. A client's explicit override cannot override its host's refusal. See the [configuration and compatibility notes](https://github.com/lanej/zellij/blob/87255197b73b9ff39b09925f1dc07a8e2cabd869/docs/SESSION_NESTING.md).

This feature is a fork review, not an upstream submission. The former unconditional cycle-guard candidate is held out of the upstream queue until lifecycle correctness and preference policy are separated into independently reviewable changes. Do not merge #7 merely because its preceding head had passing CI, and do not change the stable branch or installation pin as part of this policy work.

The feature changes no bundled plugin implementation or shipped keymap. Cycle policy declines unverified identities or incomplete ancestry; older guests can ignore the additive refusal frame. Metadata-disabled sessions do not write UUID sidecars. Local behavioral results and final validation status must refer to the policy head rather than its preceding Clippy-only refresh.

At policy code commit cf837dd1 and the published CI-wiring descendant 87255197, local Rust 1.95.0 validation passes: 5 client, 78 server and 39 utility nesting tests (five existing server ignores); 41 lifecycle/state tests; nine configuration-setting tests; the actual client/server wire-contract round trip; and the whole-app top-level start/switch/reconnect detector with metadata enabled and disabled. Formatting passes. Default Clippy passes for the changed native libraries and CLI, with inherited warnings retained. Independent code review approved the final implementation. Real-terminal/SSH acceptance and new-head hosted results remain unclaimed; the workflow now runs the new detectors on Linux and macOS.

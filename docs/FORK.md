# Maintaining the Lanej Zellij fork

This guide and [fork/topics.toml](../fork/topics.toml) live on fork-maintenance, separate from upstream contribution branches. The inventory records exact heads, original source commits, preserved fork branches, and real dependencies. Heads and validation are dated snapshots; refresh them when advancing a branch. Josh owns contribution scope, runtime promotion, and upstream submission text.

## Branch contracts

| Branch | Purpose | Rule |
| --- | --- | --- |
| main | Clean upstream mirror | Fast-forward from zellij-org/zellij:main; no fork patches/docs |
| fix/upstream-* and feature/upstream-* | Upstream contribution candidates | One capability based on upstream main; stack only for actual dependencies |
| lanej-runtime | Published runtime generation | Preserve its history and pinned revisions; accept validated forward integration |
| lanej-runtime-next | Rebased next-generation runtime candidate | Current upstream plus selected fork changes; currently matches the promoted runtime |
| lanej-stable | Conservative fallback | Advance only to an exact validated descendant; no automatic promotion |
| fork-maintenance | Fork guide and contribution inventory | Keep fork policy and tracking out of upstream diffs |
| Existing topic branches | Preserved fork review history | Keep active PR heads/bases intact until owners coordinate migration |

Contribution branches exchange only their capability's code, tests, and relevant public documentation. Fork CI, this guide, the inventory, integration conflict resolutions, and local AGENTS.md remain within fork maintenance/integration. A PR based on lanej-runtime or lanej-runtime-next is a fork integration review; those branches are unsuitable bases for independent upstream PRs.

## Fork retirement and core requirements

The fork is a temporary bridge. The goal is an official Zellij release that supports Josh's required workflows, with personal behavior implemented in external plugins, helpers and configuration. A merged fork PR is integration history, not an upstream requirement. Each retained patch needs an exit: an upstream equivalent, a verified external alternative, or an explicit decision to relinquish that behavior. Preserve the working runtime while resolving those exits.

The source audit below was checked against upstream main d9df94cca1ebf5819a8c6e1dcdc872cb512fed91. All previously recorded validation applies only to its recorded heads; this audit revalidates no patch behavior against current upstream. Reproduce and test each candidate independently before submission.

| Core correctness candidate | User workflow / contract | Why core owns it | Existing topic |
| --- | --- | --- | --- |
| Request-scoped CLI completion | Helpers must receive completion for their own action | Server broadcasts must exclude transient CLI clients; the caller cannot repair an unrelated acknowledgement after it arrives | fix/upstream-cli-completion |
| Flush exit messages before teardown | Session shutdown must release connected callers cleanly | The server owns queued socket writes and connection destruction | fix/upstream-shutdown-flush |
| Preserve initial client identity | Startup must survive concurrent helper/CLI connections | Screen registration and initial layout attachment must identify the actual first client | fix/upstream-pending-initial-client |
| Atomic permission-cache updates | Concurrent sessions must preserve grants and avoid replaying revoked grants | Zellij owns the shared permission store and its transaction boundary | fix/upstream-atomic-permission-cache |
| Preserve Unicode key scalars | External plugins must receive the typed character intact | The core/plugin protocol decoder currently truncates a scalar through u8 | fix/upstream-plugin-unicode |
| Route popup input with focus and provenance | The command prompt must receive interactive paste/keys while explicit CLI writes retain their target | Core routing chooses whether input reaches a popup, a pane or a permission prompt before the external plugin can act | fix/upstream-popup-input |
| Bind nested host frames to client lifetime | Late/retried transport frames must not mutate a removed client's state | Registration, disconnect and enqueue are server-owned; pending handshakes and legitimate ID reuse must remain valid | Extract from PR #7; independent upstream reproduction and validation still needed |

These repair existing contracts; they do not require shipping a plugin implementation or defining new user policy. They are candidates for the small-fix queue, not seven demonstrated mandatory fork dependencies. The user-facing workflow must still justify retaining each patch if upstream behavior changes or a tested alternative removes the dependency.

### Native keyboard selection is the main capability decision

Upstream already exposes pane/full-scrollback reading (including viewport, selection endpoints and application cursor), key interception, configured clipboard copying, action execution and completion, and regex highlights. Pane navigation already uses released zellij-tile 0.45.1. The external command prompt's UI, fuzzy/path completion and keymaps do not themselves require new core implementations.

The missing safe contract for keyboard selection in the original native pane is narrower than requiring the entire current Vim engine:

- An owned selection session with stable pane/text identity and bounded reads; reflow, disconnect and output recovery must have defined outcomes.
- Authoritative grapheme/display-cell and logical-line/soft-wrap geometry, with cursor and selection anchor.
- Atomic, owner- and token-checked native cursor/selection updates, preserving native selection rendering and text extraction.
- Reliable input ownership and release, preserving popup precedence. Assess existing interception and action completion before adding another capture/ordering protocol.
- Exact-cell text labels only when supporting Hop overlays on the original pane. Regex styling cannot draw arbitrary labels; basic keyboard selection does not require labels.

Vim motions, counts, word rules, selection-mode policy, Hop matching, label assignment and keymaps can conceptually live in an external plugin. The current native engine is a tested implementation, not proof that upstream must accept that exact boundary. Likewise, the current PR #9 API depends on the engine and exposes visible-cell navigation hooks; it is not yet a proven complete replacement interface for moving the engine out of core. Prototype the smaller native selection contract before changing the working implementation or representing it as sufficient.

A plugin-owned scrollback viewer would change the requested native in-pane experience. Synthetic MouseEvent actions inherit mousebindings, application reporting, timing, pane coordinates and live-output races; they are an unproven compatibility shim, not an established substitute for Josh's keyboard-only requirement.

### Nesting preference is separate from lifecycle correctness

The attach helper already rejects starting inside Zellij through its normal path. That does not cover direct invocations or hosting remote nested guests, so it is not equivalent to Josh's requested client-and-host refusal. Full cooperative host enforcement requires a core/client handshake capability. Disabling all nesting also prevents cyclic attachment; independent cycle-only restrictions and their persistent identity machinery are additional scope, not prerequisites for declining all nesting.

Keep that preference separate from the host-frame lifetime fix. Preserve upstream's existing defaults, and do not treat upstream acceptance of the whole PR #7 policy/identity implementation as the only possible exit. The current SSH helper intentionally launches remote Zellij inside a local tab when already in Zellij; adopting full host refusal also needs a nonnested launch path for that workflow. Older guests can ignore the additive refusal frame, as documented in the policy's compatibility notes.

### Outside the exit-critical core queue

Keep the command prompt, pane-navigation behavior, Hop implementation, completion/matching rules, agent-title policy, keymaps and installation choices external. Native frame cosmetics such as idle scroll counters still need native changes if their exact appearance is retained, but are optional customization rather than plugin-enabling capabilities. Session-manager help and compact-title branches remain preserved outside the upstream core queue. The duplicate-comparison Clippy fix is useful maintenance, not a runtime capability blocking a return to official releases.

The next upstream work should prioritize independently reproduced correctness fixes and resolve the native selection contract with maintainers. Keep required exhaustive-match compatibility and minimal fixtures where a core change needs them; do not ship optional/default plugin implementations to demonstrate a capability. Larger core work still requires maintainer willingness and owner-authored upstream submission text under CONTRIBUTING.md.

## Checkpoints as of 2026-11-11

- Upstream mirror: c8585459c37186fae1321727ad8dd4c96480ef41.
- Published runtime: 6647ab291f68c5154dfc4bcceab07b659a2d0efd, promoted after exact-head Linux/macOS checks and isolated installed-release acceptance. Previous head b960a9e4 is preserved by fork-checkpoint/runtime-before-release-promotion-2026-11-11. The installed-runtime marker records 5f8dd464fc97a0bb77feaaec617f0ac3f8cd5026, preserved by annotated tag lanej-runtime-checkpoint-2026-11-11. The dotfiles installer source currently pins 744c99f5a3a51682553897411d1a891dac04d886; that differs from the installation marker and must not be treated as evidence of what existing sessions run. Neither was changed by this branch preparation.
- Current stable: 6647ab291f68c5154dfc4bcceab07b659a2d0efd, matching the published runtime and immutable tag lanej-ci-green-v0.46.0-2026-11-11. Previous fallback aa8b83d76ebf5163f68188bd3a4630b62b251d03 remains preserved by annotated tag lanej-ci-green-2026-11-11; it predates keyboard copy mode. Linux and macOS native-regressions passed in [run 42543514823](https://github.com/lanej/zellij/actions/runs/42543514823); that coverage does not establish every terminal workflow.
- Historical runtime [run 43044691280](https://github.com/lanej/zellij/actions/runs/43044691280) passed on macOS but failed on Linux. Its original Linux attempt failed at package installation with DNS errors, before tests; the failed job was retried and again failed at apt DNS resolution, before test execution.
- Next-generation preparation branch: lanej-runtime-next currently matches the promoted 6647ab29 head; future refreshes start here. The prior next-runtime 3c6d79d0 remains preserved by fork-checkpoint/runtime-next-before-release-refresh-2026-11-11.

Checkpoint tags are immutable. Advancing a branch does not change the installer pin. Replacing an executable does not update existing session servers.

## Upstream candidates

Prepare one distinct correctness fix at a time. Josh selected branch preparation only: publishing a fork branch does not authorize an upstream PR, issue, comment, or generated submission description. Keep the working runtime and stable fallback in place while shrinking the patch queue.

| Order | Candidate | Next action |
| --- | --- | --- |
| 1 | fix/upstream-cli-completion | Published at d595cb99; locally validated and all 12 inherited hosted jobs passed |
| Track existing | fix/upstream-plugin-unicode | Follow upstream [#5484](https://github.com/zellij-org/zellij/pull/5484) and [#5814](https://github.com/zellij-org/zellij/pull/5814); do not open a third Unicode fix |
| Track existing | fix/upstream-clippy-output-overlap | Follow upstream [#5300](https://github.com/zellij-org/zellij/pull/5300), which repairs the missing clipping bound as well as the duplicated comparison; our deletion-only cleanup is not equivalent |
| Next independent fixes | fix/upstream-pending-initial-client; fix/upstream-atomic-permission-cache | Reproduce and validate separately; neither depends on the CLI change |
| Larger correctness work | fix/upstream-shutdown-flush | Review queue/flush acknowledgement and socket lifetime together; check overlap with upstream [#5709](https://github.com/zellij-org/zellij/pull/5709) before preparing a submission |
| Larger routing work | fix/upstream-popup-input | Keep focus, explicit target, and permission/provenance behavior cohesive; do not combine with Unicode conversion |
| Needs split | fix/upstream-nested-session-cycles | Extract the registration/lifetime repair from configurable nesting policy; retain both in the working fork meanwhile |
| Needs contract decision | feature/upstream-keyboard-copy-mode; feature/upstream-copy-mode-plugin-api | Resolve the minimum native selection contract described above before preparing a larger upstream contribution |

The CLI candidate excludes transient command sockets from the interactive completion broadcast. Each command's route thread retains responsibility for its own reply and completion. It changes no IPC schema, plugins, defaults, keymaps, or runtime installation. The related closed upstream [#5756](https://github.com/zellij-org/zellij/pull/5756) concerns blocking new-pane completion; closure alone is not evidence of maintainer rejection of this distinct query fix.

The prepared CLI head is d595cb99f8332412b5c933a1c9869ce580ff3113, based on upstream d9df94cca1ebf5819a8c6e1dcdc872cb512fed91. The previous published head 21f85437 is preserved by annotated tag fork-checkpoint/upstream-cli-completion-before-refresh-2026-11-11. The final diff changes only zellij-server/src/lib.rs and zellij-integration-tests/tests/clients.rs (109 additions, one deletion); the production change is nine additions and one deletion. No PR is open for this branch.

Independent review found that keyboard pane closure bypassed the faulty broadcast; the regression now uses natural process exit. On unchanged upstream, cli_query_ignores_completion_from_another_panes_exit fails specifically with `unrelated pane exit completed the CLI query: UnblockInputThread`. With the fix, all 15 clients integration tests pass on macOS/aarch64, including the query's own reply/completion and continued interactive input; cargo fmt --all -- --check passes. The test waits for an interactive acknowledgement counter increase, which is not uniquely correlated to this broadcast; the baseline failure establishes sensitivity for the exercised schedule.

Reproduce the scoped suite with the repository's Rust 1.95.0 toolchain: `env -u CARGO_TARGET_DIR CARGO_BUILD_JOBS=2 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 cargo test --locked -p zellij-integration-tests --test clients --features zellij-utils/vendored_curl --target aarch64-apple-darwin`. Append the regression name and `-- --exact` to isolate it. To repeat the upstream sensitivity check in an isolated checkout, keep the regression and replace only zellij-server/src/lib.rs with the recorded upstream version; restore the candidate file before testing the fix.

Default server-library Clippy fails with clippy::eq_op at output/mod.rs:3085 on both unchanged upstream and this candidate. The same candidate invocation with only `-- -A clippy::eq_op` passes with existing warnings; this is a diagnostic command-line exception, not a passing default Clippy check or a source suppression. The separate tracked clipping fix owns that repair. The inherited Rust workflow passed all 12 jobs against the exact candidate in [run 43154577533](https://github.com/lanej/zellij/actions/runs/43154577533), including Linux integration, Linux/macOS native tests and builds, Windows tests/build, no-web tests, formatting, bundled assets and desktop checks. This does not change the default-Clippy baseline failure or authorize runtime/stable promotion.

The existing Unicode PRs overlap the valid-scalar requirement; their handling of invalid scalars differs, so verify the eventual accepted semantics rather than expecting our patch verbatim. All three tracked upstream PRs were open when checked on 2026-11-11. Tracking an open proposal does not establish a merged fix or permit retiring its fork patch.

The native selection candidate leaves shipped preset files unchanged. Existing shared keyboard/mouse rules naturally expand into the additive mode; regenerated snapshots were checked to contain only those new-mode sections, with every existing mode unchanged. Its full affected libraries pass: 2,805 server tests and 1,209 utility tests; existing ignored tests remain ignored. The two small bars changes only make enum matches exhaustive and compile on wasm32-wasip1.

The plugin API candidate is refreshed from the core-only fork PR. Native copy/protocol regressions and the cell/session-token regression pass. All four whole-app API tests pass with a freshly compiled existing fixture, covering input ordering/output recovery, Unicode boundaries, resize/reload cleanup, and focused-popup precedence. No Hop implementation, plugin installation, or jump/keymap policy is included. The public API documentation describes the capability neutrally and provides a focused fixture build/test command.

The nested-session candidate passes all 76 selected server regressions, with five existing ignored tests. Review found that retaining every disconnected client ID in Screen was unbounded. The repair uses authoritative SessionState registration instead: pending handshakes remain valid, membership/ancestry/enqueue share the registration lock, removed/deferred frames are rejected, and legitimate reused IDs work. The screen tombstones were removed. The lifecycle fix is in fork PR #7. Its original mandatory cycle restriction has been superseded by opt-in policy: allow_nested_sessions and allow_cyclic_sessions default to true; setting both false declines starting or hosting nested sessions. Ordinary top-level switches and detached creation remain available. The revised behavior uses stable UUIDs, preserves unknown ancestry across mixed-version peers, and sends a departure frame before refusal teardown. The shared fork base now removes the duplicated output bounds comparison; default native-library Clippy passes on the refreshed fork PR head. The earlier independent upstream nested-session candidate is retained as historical source and is not submission-ready: separate its lifecycle fixes from the configurable policy before preparing upstream PRs.

Candidates outside the validated CLI scope retain their earlier source/format review status; the inventory does not claim that combined-runtime tests validate each individual head. Every upstream submission still needs its own applicable checks, reproduction where relevant, maintainer policy gates, and owner-authored submission text. Larger core features should be discussed with maintainers before submission.

## Fork PR state

| PR | Scope and state |
| --- | --- |
| [#4](https://github.com/lanej/zellij/pull/4) | Closed: bundled session-manager help is outside the selected scope; source branch retained |
| [#7](https://github.com/lanej/zellij/pull/7) | Held for generation rebase at e156bda6: configurable client/host nesting preferences; independent code review approved; local validation recorded below; [Linux/macOS CI passed](https://github.com/lanej/zellij/actions/runs/43136738796) |
| [#9](https://github.com/lanej/zellij/pull/9) | Held for generation rebase at 2fc05e5d; core-only; [Linux/macOS CI passed](https://github.com/lanej/zellij/actions/runs/43136742180) after a build-profile repair |

Both active fork PRs retain their existing heads. Reversible merge previews against promoted 6647ab29 found conflicts in #7 (workflow, screen tests, IPC schema) and #9 (workflow). Reconcile their wire allocations and validate their new merge results before merging; earlier green checks apply to the older generation. Both PRs forward-merge the shared runtime Clippy fix, keeping that maintenance change out of their capability diffs. PR #7 now adds the explicitly requested configurable nesting feature and remains unmerged for scope review. Their independent upstream candidates remain separate. Before this refresh, #7 had a failed hosted run and #9 had a canceled hosted run; these are not passing results for the new heads. Existing merged PRs and published runtime/stable history remain preserved. No upstream PR was opened or described by this work.

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

### v0.46.0 runtime promotion

The published runtime, next-runtime preparation branch, and stable now all point to 6647ab291f68c5154dfc4bcceab07b659a2d0efd. Promotion was an atomic fast-forward with remote refs verified; immutable tag lanej-ci-green-v0.46.0-2026-11-11 identifies this validated generation. It forward-merges upstream d9df94cc into the already rebased v0.46.0 generation, preserving the previous next-runtime 3c6d79d0, published runtime b960a9e4, and stable aa8b83d7 ancestry. The two newly incorporated upstream changes are the inbox-empty behavior and dropped-file paste handling. The proven CLI process-exit detector replaces the insensitive keyboard-close test; the fork workflow selects its new name. The already verified native debug-test profile repair from the fork PRs is carried into runtime CI without dropping any detector or changing required status names. Independent review approved both source and workflow changes.

Local macOS/aarch64 validation passes all 2,816 server and 1,222 utility tests, all 15 clients tests, five keyboard-copy integration tests, formatting, and default native-library Clippy with existing warnings. Existing ignored tests remain ignored. Both required exact-head Linux/macOS checks passed in [run 43159156140](https://github.com/lanej/zellij/actions/runs/43159156140), including the selected CLI and keyboard-copy detectors. The exact pinned installer also completed in a disposable HOME with revision-target-v1:6647ab291f68c5154dfc4bcceab07b659a2d0efd. Native copy/Lua action196, two-client command prompt/completion, and full Zsh/Neovim navigation acceptance all passed again using that installed release binary before promotion.

The matched installation/client change is [dotfiles branch fix/zellij-runtime-release-bindings at 6579e9f1](https://github.com/lanej/dotfiles/tree/6579e9f1d0f505226cc33cee5b9129d3b22958f2). Installer, command-prompt SDK, complete Rust IPC contract, and Neovim lazy pin are coordinated. The generated Lua client is [f0d4e835](https://github.com/lanej/zellij-client.nvim/tree/f0d4e83555d5aa2b22c365865e3153ff38ffcdc4), with [Linux/macOS client checks passed](https://github.com/lanej/zellij-client.nvim/actions/runs/43159215301). Its five vendored schemas and the Rust contract were checked byte-for-byte against the runtime. Wire IDs remain the rebased allocations documented above.

Real PTY acceptance passes with the matched candidate binary: rebuilt command-prompt actions, two-client isolation, completion and Unicode filenames/live editing; native copy input sent by the regenerated Lua client through action 196 selects beta, emits the exact OSC 52 payload and resumes input; Zsh/Neovim pane navigation passes, including delayed requests and inactive-tab safeguards. This establishes OSC 52 emission, not acceptance by the host clipboard. The navigation check exposed the helper's outdated fixed session-index filename under ZELLIJ_SOCKET_DIR overrides. The reviewed helper repair uses the runtime's SHA-256 socket-directory namespace and preserves default discovery, read-only indexes and socket identity validation; committed helper/title checks, including warnings-denied helper Clippy, pass.

Matching pins are prepared on their own reviewed branch. This work does not replace the installed executable or restart live sessions. Applying that installation change requires switching client/server/plugin generations together and restarting all old session servers. Prior published runtime and next-generation heads are preserved by fork-checkpoint/runtime-before-release-promotion-2026-11-11 and fork-checkpoint/runtime-next-before-release-refresh-2026-11-11; the old stable tag remains immutable.

On macOS, the final runtime code passes 2,814 server library tests and 1,222 utility library tests with the pinned Rust toolchain. The server run excludes two temporary-file safety tests that also fail on unchanged upstream c8585459 when its checkout resides under /tmp; 185 server tests and two utility tests retain their existing ignored status. Four configuration snapshots were corrected after the rebase retained an obsolete F9 session-card binding in Copy mode.

The full server run exposed a fork popup regression that passes on unchanged upstream: generic WriteCharacter capture stole explicit CLI writes from their pane. The repair preserves interactive versus CLI provenance in the input router and passes the existing CLI-write detector, a Unicode bracketed-paste screen-thread detector, keyboard-popup routing, and rejection of pasted permission answers. The standalone popup candidate includes the same repair. Independent review approved the material fix, and the runtime workflow now runs those four regression tests explicitly.

The duplicated output bounds comparison has been removed without changing overlap behavior. The isolated fix/upstream-clippy-output-overlap candidate bc7865c96a5c2086fe7e06251fff59b07f4a9055 is based on upstream d9df94cc (the defect was still present there), and passes rustfmt, all 126 selected output tests, and default Clippy. The maintained runtime, next runtime, and both refreshed fork PR heads also pass default Clippy for zellij-server and zellij-utils native libraries with Rust 1.95.0, vendored_curl and web_server_capability, targeting aarch64-apple-darwin. These scoped checks retain inherited warnings; warnings-denied Clippy and all-target/workspace lint are not claimed. The previous strict upstream run produced 811 utility diagnostic messages. This maintenance fix is separate from the plugin-support queue.

The preceding runtime head ff852b1b50b034f27ad5766f2349f40c83f76dd2 had a failed [Linux/macOS CI run 43063026187](https://github.com/lanej/zellij/actions/runs/43063026187). The pre-refresh generation 3c6d79d01c661320616625833042de323584bd09 subsequently [passed](https://github.com/lanej/zellij/actions/runs/43103290679); the preceding published runtime b960a9e4 also [passed](https://github.com/lanej/zellij/actions/runs/43103289974). The newer promotion and its coordinated installation/client acceptance are recorded above. The published topic branches have structural/format checks; their individual behavioral readiness remains unclaimed.

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

## Hosted CI repairs

The policy head 87255197 failed on both Linux and macOS because the configuration plugin's exhaustive setting matches did not cover AllowNestedSessions and AllowCyclicSessions. The compatibility fix at e156bda640c3395e5a0d73af5c245bb291abf380 completes its metadata/category entries and config-derived help without enabling either option by default. All 163 configuration-plugin tests, native Clippy, WASI compilation, formatting and actionlint pass locally. CI now runs the complete fast configuration suite.

PR #9's preceding macOS job was canceled at its one-hour limit while release compilation was still running. Its head 2fc05e5d26fba23aa75e74bfa61191fad9f0fbb3 changes native CI checks to debug builds with debug symbols disabled and a separate cache prefix; it preserves every detector and the required release WASM fixture. PR #7 uses the same native test profile. Fresh hosted checks pass on both Linux and macOS for both exact repair heads: [PR #7 at e156bda6](https://github.com/lanej/zellij/actions/runs/43136738796) and [PR #9 at 2fc05e5d](https://github.com/lanej/zellij/actions/runs/43136742180). PR #7 remains a draft policy review; neither PR was merged, and no stable or installation pin was advanced.

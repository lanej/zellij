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

## Checkpoints as of 2026-11-11

- Upstream mirror: c8585459c37186fae1321727ad8dd4c96480ef41.
- Published runtime and unchanged dotfiles installation pin: 5f8dd464fc97a0bb77feaaec617f0ac3f8cd5026, preserved by annotated tag lanej-runtime-checkpoint-2026-11-11.
- Last CI-green fallback: aa8b83d76ebf5163f68188bd3a4630b62b251d03, referenced by lanej-stable and annotated tag lanej-ci-green-2026-11-11. It predates keyboard copy mode. Linux and macOS native-regressions passed in [run 42543514823](https://github.com/lanej/zellij/actions/runs/42543514823); that coverage does not establish every terminal workflow.
- Published runtime [run 43044691280](https://github.com/lanej/zellij/actions/runs/43044691280) passed on macOS but failed on Linux. Its original Linux attempt failed at package installation with DNS errors, before tests; the failed job was retried and again failed at apt DNS resolution, before test execution.
- Next-generation runtime: ff852b1b50b034f27ad5766f2349f40c83f76dd2. Its upstream/fork rebase is resolved, with generated bindings refreshed using official protoc 23.4 and cargo xtask proto. Validation and promotion status are recorded below; publication is not stable acceptance.

Checkpoint tags are immutable. Advancing a branch does not change the installer pin. Replacing an executable does not update existing session servers.

## Upstream candidates

Ten candidates start directly at the upstream mirror; the plugin API and Hop form a dependency stack on keyboard copy mode. All candidate source diffs and Rust formatting were checked, with independent structural review. Snapshot files retain meaningful rendered whitespace and are excluded from the source whitespace check. Compilation/tests of the combined runtime do not validate these independent heads. None is submission-ready until its own required checks, current-upstream reproduction where applicable, and upstream policy gates are satisfied.

| Branch | Capability | Actual prerequisite |
| --- | --- | --- |
| fix/upstream-cli-completion | CLI completion replies | None |
| fix/upstream-shutdown-flush | Bounded shutdown flush | None |
| fix/upstream-pending-initial-client | Pending initial client registration | None |
| fix/upstream-atomic-permission-cache | Atomic permission-cache updates | None |
| fix/upstream-plugin-unicode | Unicode plugin key scalars | None |
| fix/upstream-popup-input | Focused popup paste routing | None |
| fix/upstream-manager-help | Manager help without function keys | None |
| fix/upstream-nested-session-cycles | Nested-session cycle guard | None |
| feature/upstream-scroll-indicators | Configurable idle scroll indicators | None |
| feature/upstream-keyboard-copy-mode | Native keyboard copy mode | None |
| feature/upstream-copy-mode-plugin-api | Native copy-mode plugin controls | feature/upstream-keyboard-copy-mode |
| feature/upstream-copy-hop | Hop labeled character jumps | feature/upstream-copy-mode-plugin-api |

The original popup/Unicode fork PR combines independent changes; its upstream candidates separate them. The nested-session candidate consolidates the original implementation/follow-ups and omits unrelated formatting changes. The pending-client port preserves upstream recent-disconnect bookkeeping alongside the explicit pending identity. Protocol-bearing branches use upstream-compatible unused numbers and exclude inherited fork-only options/tests.

Original fork PRs [#4](https://github.com/lanej/zellij/pull/4), [#7](https://github.com/lanej/zellij/pull/7), and [#9](https://github.com/lanej/zellij/pull/9) and all original topic branches remain intact. No upstream PRs by lanej existed at the audit, and this organization creates none. The closed compact-title branch is preserved but not selected. Partial extraction experiments are not submission candidates; original implementations remain on published branches.

The inherited Rust workflow runs on main pushes, PRs targeting main, or workflow_dispatch. Publishing a topic branch does not automatically run it. Run applicable local checks on that branch and manually dispatch the upstream workflow for the selected ref when useful. Build/Test jobs do not replace the complete contribution requirements or runtime acceptance.

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

Rust formatting passes. Default Clippy remains blocked by clippy::eq_op in zellij-server/src/output/mod.rs:3085; the same default-Clippy error reproduces on unchanged upstream c8585459. Strict Clippy with warnings denied also fails on unchanged upstream with the same 811 utility diagnostic messages as the candidate. These are recorded limits, not a clean lint result.

The final runtime head ff852b1b50b034f27ad5766f2349f40c83f76dd2 has [Linux/macOS CI run 43063026187](https://github.com/lanej/zellij/actions/runs/43063026187). Hosted checks are still pending at this snapshot; the superseded candidate run was canceled. Actual dotfiles installation/client-binding refresh and real-terminal acceptance remain promotion gates. The published topic branches have structural/format checks; their individual behavioral readiness remains unclaimed.

The next runtime was rebased with merge topology retained. After replaying/reconciling all selected patches, history bridge 96972bac recorded the preserved runtime as a second parent while keeping the rebased tree exactly. That deliberate supersession merge permits future fast-forward promotion without force-pushing published runtime/stable refs. It is not an ordinary content merge or a general conflict-resolution shortcut. Independent upstream branches exclude this fork ancestry.

The Lanej runtime workflow now covers both lanej-runtime and lanej-runtime-next. Validate the exact final candidate head on Linux/macOS and run affected real-terminal/plugin workflows. Record manual acceptance and exact heads here before promotion. Then fast-forward lanej-runtime and lanej-stable to that validated descendant, create a new immutable checkpoint, and update the dotfiles pin in its own reviewed change with the coordinated wire migration. Retain the old tag/pin for rollback.

Branch protection prevents force pushes and deletion of main, lanej-runtime, and lanej-stable. Stable updates require native-regressions (ubuntu-latest) and native-regressions (macos-latest). Those contexts cover selected native regressions, not every acceptance claim. Do not force-push published runtime branches, automatically install an unvalidated candidate, or merge a freshly reconstructed tree merely because Git reports no conflicts.

## Local checkouts

The upstream remote fetches zellij-org/zellij and has a deliberately unusable push URL. Default pushes go to origin (lanej/zellij); main tracks upstream/main with fast-forward-only pulls. Recreate these local settings when cloning; do not add upstream push access as a maintenance side effect.

The permanent primary checkout is /Users/joshlane/src/zellij, checked out on fork-maintenance so this guide is immediately visible. The original temporary repository and other sessions' checkouts remain intact. Use Treehouse from the permanent primary checkout for new tasks. Acquire contribution work from local main with treehouse get --lease --json --lease-holder <task> --branch fix/<topic> --base main; acquire fork integration work explicitly from lanej-runtime or lanej-runtime-next. Preserve the returned lease identity and return only task-owned leases after preserving work and inspecting valuable ignored files. Do not nest pools or use gh pr checkout to switch worktrees.

AGENTS.md is intentionally untracked and excluded by /AGENTS.md in the shared Git info/exclude. Copy it into each new checkout/worktree if absent; exclusions propagate across worktrees, file contents do not. Do not force-add it. Published refs preserve Git work, while local configuration, excludes, and instructions must be recreated if a checkout is replaced.

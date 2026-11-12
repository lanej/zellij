# Session nesting policy

Nesting policy is independent of `nested_session_handling`, which controls how an accepted nested session is displayed and focused. Ordinary nesting and cyclic attachment remain available by default, subject to Zellij's existing self-attachment checks.

To decline both starting inside another Zellij session and hosting a nested session:

```kdl
allow_nested_sessions false
allow_cyclic_sessions false
```

To permit ordinary nesting while declining attachment back to a containing session:

```kdl
allow_nested_sessions true
allow_cyclic_sessions false
```

Both options default to `true`. `allow_nested_sessions false` takes precedence over the cycle option. Each participant applies its own configuration: a client can decline being nested, and a host can decline accepting a guest. A client override does not override a host's refusal. Existing sessions are not terminated when configuration changes; the policy is checked on subsequent attachment, session-switch and nested-handshake operations. Detached server creation and ordinary top-level session switching remain available.

The CLI accepts explicit Boolean values, for example `zellij options --allow-nested-sessions true --allow-cyclic-sessions true`. Use the same configuration/CLI option placement supported by the command being run. Enable the host's options as well when the host is configured to refuse guests.

Local nested clients are refused before starting or attaching when the enclosing Zellij environment is present. Across SSH, detection occurs through the existing nested-session handshake; a refused guest detaches and returns an explanation while the host keeps running. A client that declines an otherwise accepted handshake sends the normal departure frame so the host can clear its nested presentation.

Cycle detection uses a stable UUID stored atomically beside each session’s cache, carried separately from display names and socket names. Legacy socket names are never treated as cross-host identities. With `disable_session_metadata true`, no nesting identity is written, so cycle prohibition reports that identity cannot be verified. Two different sessions named `main`, including on different hosts, are not a cycle merely because their names match. The new fields and refusal frame are additive; old peers can decode existing messages but may omit identifiers or ignore refusals. An empty identity entry preserves an unknown ancestor through modern hosts. When cycle prohibition is enabled and any ancestor or the peer’s identity cannot be verified, attachment is declined with an explanation instead of silently allowing it. URL/WebSocket clients do not yet advertise the remote session’s identity, so they require an explicit cycle-policy override to nest. Host refusal still requires a guest that understands the refusal message. These options are cooperative session preferences, not a security boundary against arbitrary terminal programs or older clients.

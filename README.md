# signal-forge

Layered protocol crate atop `signal`.
Carries the **criome ↔ forge wire** — effect-bearing verbs
(Build, Deploy, store-entry operations) — using signal's
Frame envelope, handshake, and auth.

The audience is narrow: criome (sender) and forge
(receiver). Front-end clients (nexus daemon, GUI editor,
mentci-lib) depend only on `signal`, not on this crate —
builder-internal field churn doesn't recompile front-ends.

See `ARCHITECTURE.md`. Project-wide context:
criome/ARCHITECTURE.md.

## Status

**Skeleton-as-design.** Type signatures pinned; bodies are
`todo!()`. Lands when forge-daemon is wired.

## License

[License of Non-Authority](LICENSE.md).

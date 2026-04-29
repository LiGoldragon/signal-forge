# ARCHITECTURE — signal-forge

Layered protocol crate atop [signal](https://github.com/LiGoldragon/signal).
Carries the criome ↔ forge wire — effect-bearing verbs that
criome forwards to forge for execution.

## Role in the sema-ecosystem

```
   front-end clients (nexus, GUI, mentci-lib, agents)
            │
            │  signal (front-end verbs: Assert, Query,
            │            BuildRequest, ...)
            ▼
        criome
            │
            │  signal-forge (effect-bearing verbs:
            │                Build, Deploy, store-entry ops)
            │  — layered atop signal: same Frame, same
            │    handshake, same auth
            ▼
        forge daemon
```

## What's here

Per-verb typed payloads on the criome → forge leg. criome
itself runs nothing (per
[criome/ARCHITECTURE.md §10](https://github.com/LiGoldragon/criome/blob/main/ARCHITECTURE.md#10--project-wide-rules)
"criome communicates; it never runs"); these verbs are the
typed envelope by which it dispatches effect-bearing work to
forge.

- **`Build`** — carries the records criome read from sema
  (Opus + transitive OpusDeps) plus a **criome-signed
  capability token** authorising forge to deposit into a
  target arca store. forge links prism + assembles workdir +
  invokes nix (crane + fenix) + bundles the closure into
  arca's `_staging/` + signal-arca-deposits to arca-daemon —
  all internally. Reply is the `CompiledBinary` outcome
  payload `{ arca_hash, narhash, wall_ms }`. criome asserts
  the `CompiledBinary` record to sema using `arca_hash` as
  canonical identity.
- **`Deploy`** — `nixos-rebuild` against a target host
  (system flake + host identity + activation mode). forge
  spawns the rebuild; replies with `{ generation, wall_ms }`.
- **store-entry operations** — get / put / materialize / delete
  against arca, gated by capability tokens. Bulk bytes never
  cross criome — these are control-plane verbs only.

Reply payloads:

- `BuildOk { arca_hash, narhash, wall_ms }`
- `DeployOk { generation, wall_ms }`
- `Failed { code, message }`

## Boundaries

Owns:

- The verbs criome sends to forge (and the matching replies).
- Capability-token shape used on this leg (criome-signed).

Does not own:

- The Frame envelope, handshake, or auth primitives — those
  live in [signal](https://github.com/LiGoldragon/signal) and
  this crate re-uses them.
- The front-end-visible verbs (`Assert`, `Query`,
  `BuildRequest`, `Subscribe`, ...) — those live in signal.
- The prism emission templates — those live in
  [prism](https://github.com/LiGoldragon/prism), linked by
  forge.

## Why layered atop signal (not parallel to it)

**Audience-scoped compile-time isolation.** The criome ↔ forge
leg has a narrow audience — criome (sender), forge (receiver),
lojix-cli (transitional sender of deploy verbs). Front-end
clients (nexus daemon, GUI editor, mentci-lib, agents) never
need these verbs and must not depend on them. Splitting the
builder protocol into its own crate means builder-internal
field churn (adding `nix_target_platform`, refining outcomes,
evolving capability-token shapes, growing the store-entry
verb family) recompiles only criome + forge, not the wider
workspace.

**Layered, not parallel.** signal-forge re-uses signal's
`Frame` envelope, handshake, auth, and record-kind types — it
contributes only the per-verb typed payloads on this one leg.
A parallel protocol would duplicate envelope/handshake/auth
machinery and force every implementer to ship two stacks.
Layering keeps the wire-protocol invariants (rkyv encoding,
content-addressing of attached records, capability-token
verification) in one place.

## Code map

```
src/
└── lib.rs   — module entry; verbs + payloads will live here
              when the bodies fill in.
```

All types are `todo!()` skeleton-as-design.

## Status

**Skeleton-as-design.** Lands when forge-daemon is wired.

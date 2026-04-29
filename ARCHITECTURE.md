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

Per-verb typed payloads on the criome → forge leg:

- **`Build`** — records → `CompiledBinary` outcome. forge runs
  prism + workdir-assembly + nix + bundle internally; replies
  with `{ store_entry_hash, narhash, wall_ms }`.
- **`Deploy`** — nixos-rebuild on a target host.
- **store-entry operations** — get / put / materialize / delete
  against arca, gated by capability tokens.

Reply payloads:

- `BuildOk { store_entry_hash, narhash, wall_ms }`
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

## Why a separate crate

**Audience-scoped compile-time isolation.** Front-end clients
depend only on `signal`. Builder-internal field churn here
(adding `nix_target_platform`, refining outcomes, evolving
capability-token shapes) recompiles only criome and forge, not
the wider workspace.

## Code map

```
src/
└── lib.rs   — module entry; verbs + payloads will live here
              when the bodies fill in.
```

All types are `todo!()` skeleton-as-design.

## Status

**Skeleton-as-design.** Lands when forge-daemon is wired.

# signal-forge — architecture

*Layered Signal contract for the criome ↔ forge leg. Carries
effect-bearing requests that criome dispatches to forge for
execution — typed payloads only; criome itself runs nothing.*

## 0 · TL;DR

`signal-forge` is a **layered effect crate** atop `signal-frame` and
`signal`. It re-uses `signal-frame`'s `Frame` envelope, handshake,
and exchange-identifier mechanics, and adds the contract-local
operation payloads on the narrow criome ↔ forge wire.

## MUST IMPLEMENT — three-layer migration

This contract is migrating to the three-layer model affirmed
2026-05-20 per
`primary/reports/designer/246-v4-bundled-fix-deep-design-with-examples.md`
and `primary/reports/designer/248-three-layer-changes-for-operators.md`.

**Layer 1 — Contract Operations on the wire (this crate).** Drop the
SignalVerb wrappers entirely. The current shape is two `Mutate`-tagged
variants (`Build`, `Deploy`) plus a deferred `StoreEntry*` family.
`Build` and `Deploy` are already verb-form contract-local roots; they
read correctly as "criome is asking forge to build this" / "criome is
asking forge to deploy this." Payloads stay as the typed nouns
(`BuildRequest` becomes `Build`'s payload, perhaps renamed to a
plain noun like `Target`). The `StoreEntry*` family that's still
TBD likely splits to `signal-arca` per the existing note in §2;
when it does, use contract-local verbs there too (`Get`, `Put`,
`Materialize`, `Delete` — all already verb-form).

**Forge is not a persona component.** The mandatory `Tap`/`Untap`
observable block does not apply; criome ↔ forge is a directed
authority leg, not a persona daemon with introspection peers. If
forge later needs an observation surface, it lives in a separate
contract.

**Layer 2 — Component Commands (forge daemon crate).** Forge's daemon
owns its typed Command enum (e.g. `ForgeCommand::AssembleWorkdir`,
`ForgeCommand::InvokeNixBuild`, `ForgeCommand::DepositArchive`) plus
a `CommandExecutor` that knows forge's tables and the nix/crane/fenix
toolchain. Lowering from contract operation to commands happens in
the daemon.

**Layer 3 — Sema classification (signal-sema).** Each Component
Command projects to a payloadless `SemaOperation` class label via
`ToSemaOperation`, so cross-component observers can filter by class.
Forge does not import payload-bearing Sema variants.

**Frame layer.** The dependency on `signal-core` shifts to
`signal-frame`; `Frame` envelope and handshake stay the same (frame
mechanics only).

References:
- `primary/reports/designer/246-v4-bundled-fix-deep-design-with-examples.md`
- `primary/reports/designer/248-three-layer-changes-for-operators.md`
- `primary/skills/component-triad.md` §"Verbs come in three layers"
- `primary/skills/contract-repo.md` §"Public contracts use contract-local operation verbs"

**Note to remover:** when the refactor lands, remove this section and
add a `## Migration history — three-layer model (2026-05-XX)`
paragraph noting the shape change.

Front-end clients (nexus, GUI editor, agents, mentci-lib) depend on
`signal` for the sema-ecosystem vocabulary; they do **not** depend
on `signal-forge`. Builder-internal churn in this crate recompiles
only criome + forge, not the wider workspace. This is the canonical
example of `~/primary/skills/contract-repo.md` §"The layered
pattern".

```mermaid
flowchart TB
    frame["signal-frame<br/>(wire kernel — Frame, exchange ids, handshake)"]
    base["signal<br/>(sema-ecosystem records vocabulary)"]
    forge["signal-forge<br/>(criome ↔ forge effect verbs)"]

    front["front-end clients<br/>(nexus, GUI, agents)"]
    criome["criome"]
    forged["forge daemon"]

    base --> frame
    forge --> frame
    forge --> base

    front --> base
    criome --> base
    criome --> forge
    forged --> base
    forged --> forge
```

> **Status (2026-05-17): skeleton-as-design.** Bodies land when
> forge-daemon is wired. The current `src/lib.rs` is `todo!()`
> placeholders. The discipline below is the shape implementations
> must satisfy.

> **Scope (today vs eventually).** This contract sits on today's
> stack — `signal-frame` wire kernel, rkyv archives. The
> eventually-self-hosting stack is Sema-on-Sema; this layered crate
> is a realization step. See `~/primary/ARCHITECTURE.md` §"Workspace vision and intent".

## 1 · Channel boundary

| Side | Component |
|---|---|
| Request producer | `criome` (dispatching effect-bearing work it itself does not execute). |
| Request consumer | `forge` daemon. |
| Reply producer | `forge` daemon. |
| Reply consumer | the `criome` request that issued the operation. |

Criome holds typed records (per `criome/ARCHITECTURE.md`); when an
effect needs to be performed (build a binary, deploy a system,
manipulate the arca store), criome dispatches the typed request
over this leg. Criome never invokes `nix`, `nixos-rebuild`, or the
filesystem directly. Forge owns the effect surface.

Transport is `signal-frame` length-prefixed rkyv frames over a Unix
socket. The transport itself belongs to forge, not this contract.

## 2 · Contract operations (Layer 1)

The channel is declared via one `signal_channel!` invocation in
`src/lib.rs` per `signal-frame/ARCHITECTURE.md`. The macro emits
the typed `ForgeRequest` / `ForgeReply` enums, the frame alias
(`ExchangeFrame` — this channel has no streams in the v1 design),
and the NOTA codec impls.

| Operation | Payload | What happens | Authority direction | Expected Sema class |
|---|---|---|---|---|
| `Build` | `BuildRequest` | Criome orders forge to build a target. Payload carries the records criome read from sema (target `Graph` + transitive `DependsOn` + `Contains` nodes + edges) plus a criome-signed capability token authorising forge to deposit into the target arca store. Forge links prism, assembles the workdir, invokes nix (crane + fenix), bundles the closure into arca's staging area, and signals arca-daemon. | top-down (criome → forge) — authority order. Criome holds *possibly-mutated* state until forge confirms. | `Mutate` |
| `Deploy` | `DeployRequest` | Criome orders forge to perform a `nixos-rebuild` against a target host (system flake + host identity + activation mode). | top-down — authority order. | `Mutate` |
| `StoreEntry*` family | (TBD) | Get / put / materialize / delete against arca, gated by capability tokens. **Likely migrates to `signal-arca` when that contract crate lands.** Sema-class assignments deferred to that design pass. | TBD | TBD |

`Build` and `Deploy` are authority-order operations — under the
three-layer model the *wire verb* is the contract-local action
(`Build` / `Deploy`); the *Sema class* `Mutate` is the daemon-side
classification used for observation only. Criome is the authority
root for forge on this leg; forge obeys and confirms.

## 3 · Reply variants and reply discipline

Reply payloads:

- `BuildOk { arca_hash, narhash, wall_ms }` — forge confirms the
  build completed; criome then asserts the `CompiledBinary` record
  to sema using `arca_hash` as canonical identity.
- `DeployOk { generation, wall_ms }` — forge confirms activation.
- `Failed { code, message }` — typed failure for either; closed
  reason vocabulary, never an untyped error string.

Replies are causally tied to the request that issued them, and their
legality is checked against that request's operation. Per
`~/primary/skills/contract-repo.md` §"Reply discipline": if a
*"reply"* becomes a standalone observation that can travel
independently (e.g., a build-progress event observed by a
subscriber other than the issuing criome), it lands as a separate
contract-local operation — a domain-named verb for a new fact, a
`Subscribe`-shaped verb for a streaming observation — not as a
verbless message.

## 4 · Capability tokens

The `BuildRequest` payload includes a criome-signed capability
token authorising forge to deposit into a specific arca store
namespace. Forge does not interpret the token semantically beyond
verifying the signature; arca-daemon checks the token against its
own policy when forge submits the closure.

Capability-token shape on this leg is part of this contract; arca's
own capability validation lives in `signal-arca` when that crate
lands.

## 5 · Constraints

- This is a pure contract crate. No actors, no `tokio`, no
  filesystem I/O, no `nix` calls. Behavior lives in `forge`.
- The channel re-uses `signal-frame`'s `ExchangeFrame` envelope and
  `ProtocolVersion`/handshake. This crate does **not** redefine
  those types.
- Every `ForgeRequest` variant is a contract-local verb in verb form;
  the `signal_channel!` macro emits the NOTA codec keyed on the
  payload's head.
- Closed enums only. **No `Unknown` variant.** Lifecycle uncertainty
  is encoded as a positive closed variant (e.g., `Failed::reason`
  carries a closed `BuildFailureReason` enum, not a string kind).
- Bulk byte payloads never cross criome. Effect-bearing payloads
  reference content by `arca_hash`; the bytes themselves travel
  arca↔forge↔target out-of-band.
- Naming follows `~/primary/skills/naming.md`: full English words;
  no crate-name prefix on types.
- rkyv on the wire; NOTA derives on every typed record so the same
  type IS the binary record AND IS the text record.
- Round-trip tests per record kind in `tests/`: rkyv archive
  round-trip AND NOTA text round-trip, both witnessed.
- Sema classification projections live in the forge daemon
  (Component Commands impl `ToSemaOperation`), not in this contract
  crate.

## 6 · Owned / not owned

**Owned:**

- The criome → forge contract-local operation payloads on this layer.
- Capability-token shape for the criome → forge leg (criome-signed
  authorisation to deposit into a target arca store).
- Round-trip witnesses for every record kind (rkyv + NOTA).

**Not owned (re-used from `signal-frame`, `signal-sema`, and `signal`):**

- `Frame` envelope, handshake, `ProtocolVersion` — `signal-frame`.
- The Sema classification labels (`Assert`, `Mutate`, `Retract`,
  `Match`, `Subscribe`, `Validate`) — `signal-sema`. Used as
  observation-only projection targets, not on the wire.
- The sema-ecosystem record vocabulary (`Node`, `Edge`, `Graph`,
  `Records`, etc.) — `signal`. `BuildRequest` carries instances of
  those types but does not redefine them.

**Not owned (component responsibility):**

- Forge's actor tree, the `nix`/`crane`/`fenix` invocation, the
  workdir-assembly logic, the prism emission templates — forge.
- Arca's store policy, capability-validation rules, replication
  topology — arca / future `signal-arca`.

## 7 · Why layered atop signal-frame (not parallel to it)

**Audience-scoped compile-time isolation.** The criome ↔ forge leg
has a narrow audience — criome (sender), forge (receiver),
lojix-cli (transitional sender of deploy verbs). Front-end clients
(nexus daemon, GUI editor, mentci-lib, agents) never need these
verbs and must not depend on them. Splitting the builder protocol
into its own crate means builder-internal field churn (adding
`nix_target_platform`, refining outcomes, evolving capability-token
shapes, growing the store-entry verb family) recompiles only
criome + forge, not the wider workspace.

**Layered, not parallel.** `signal-forge` re-uses the
`signal-frame` kernel — `Frame`, handshake, exchange identifiers — and
contributes only the contract-local operation payloads on this leg.
A parallel protocol would duplicate envelope/handshake machinery and
force every implementer to ship two stacks. Layering keeps the
wire-protocol invariants (rkyv encoding, content-addressing of
attached records, capability-token verification) in one place.

## 8 · Code map

```text
src/
└── lib.rs   — signal_channel! declaration + typed payloads.
              Currently todo!() skeleton-as-design.
tests/
└── round_trip.rs  — per-variant rkyv + NOTA round-trips
                     (lands with the bodies).
```

## Pending schema-engine upgrade

**Status:** scheduled for migration to schema-language-based contract per `reports/designer/326-v13-spirit-complete-schema-vision.md` + `reports/designer/324-migration-mvp-spirit-handover-re-specification.md`.

**Target:** this contract's hand-written `signal_channel!` invocation converts to a single `forge/forge.schema` file shared with the `forge` daemon's repository. The brilliant macro library (`primary-ezqx.1`) reads the schema + emits this crate's wire types + ShortHeader projection + dispatcher binding + VersionProjection impls.

**Sequence:** per `reports/designer/316` forge family direction. Spirit is the MVP pilot landing first via `primary-ezqx.1`; this contract's schema cutover follows after pilot succeeds and the forge family direction settles.

**Per-component concerns:** Per `/316` forge family direction; schema cutover follows persona triad. The contract surface here is still skeleton-as-design (variant bodies `todo!()`); cutover bundles with first-real-implementation.

**References:**
- `reports/designer/326-v13-spirit-complete-schema-vision.md` — uniform header form + schema-language design
- `reports/designer/324-migration-mvp-spirit-handover-re-specification.md` — migration MVP + handover state
- `reports/designer/322-spirit-mvp-positional-schema-worked-example.md` — Spirit MVP worked example
- `reports/operator/174-schema-import-header-design-critique-2026-05-24.md` — header/body/feature separation + lowering rules
- `reports/designer/316-…` — forge family direction

## See also

- `~/primary/skills/contract-repo.md` §"The layered pattern" — the
  canonical discipline this crate exemplifies.
- `~/primary/skills/contract-repo.md` §"Public contracts use
  contract-local operation verbs" — the three-layer framing.
- `~/primary/skills/component-triad.md` §"Verbs come in three layers".
- `~/primary/ARCHITECTURE.md` §"Workspace vision and intent" — the
  principle this contract repo encodes across processes.
- `/git/github.com/LiGoldragon/signal-frame/ARCHITECTURE.md` — the
  wire kernel this crate layers atop.
- `/git/github.com/LiGoldragon/signal-sema/ARCHITECTURE.md` — the
  payloadless Sema classification vocabulary used at the observation
  layer.
- `/git/github.com/LiGoldragon/signal/ARCHITECTURE.md` — the
  sema-ecosystem records vocabulary `BuildRequest` carries.
- `/git/github.com/LiGoldragon/criome/ARCHITECTURE.md` — the
  sender side of this leg.
- `/git/github.com/LiGoldragon/forge/ARCHITECTURE.md` — the
  receiver side; owns the effect surface.

//! signal-forge — criome ↔ forge wire (layered atop signal).
//!
//! Effect-bearing verbs that criome forwards to forge for
//! execution: build (records → CompiledBinary outcome),
//! deploy (nixos-rebuild), store-entry operations against
//! arca. Replies carry the outcomes back.
//!
//! The Frame envelope, handshake, and auth primitives live in
//! [`signal`](https://github.com/LiGoldragon/signal); this
//! crate layers on top, contributing only the per-verb typed
//! payloads.
//!
//! # Audience
//!
//! - **criome** (sender) — forwards effect-bearing verbs after
//!   validating the front-end request.
//! - **forge** (receiver) — links prism, runs nix, bundles into
//!   arca; replies with outcome payloads.
//! Lojix deploy orchestration uses the signal-lojix and
//! meta-signal-lojix contracts rather than this builder protocol.
//!
//! Front-end clients (nexus daemon, GUI editor, mentci-lib,
//! agents) depend only on [`signal`], not on this crate.
//! Builder-internal field churn here doesn't recompile
//! front-ends.
//!
//! # Skeleton-as-design
//!
//! Types are pinned; bodies are `todo!()`. Real implementation
//! lands alongside forge-daemon scaffolding.

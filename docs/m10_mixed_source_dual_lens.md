# M10.2 mixed-source dual-lens workbench

This branch consumes the typed source-context comparison exported by
`slr` PR #51. It does **not** introduce a second source ontology or an
independent PNF matcher. The SLR dependencies in `Cargo.toml` are pinned
together to a single PR-branch commit; this is a temporary integration pin
until the SLR source-family integration is merged.

## Launch (database already migrated)

```bash
ITIR_MIXED_LEFT_REV='source-revision:...' \
ITIR_MIXED_RIGHT_REV='chat-message-revision:...' \
ITIR_MIXED_CHAT_MESSAGE_REF='message:...' \
ITIR_MIXED_OBSERVER_SCOPE='excluded' \
  cargo run --features desktop
```

The left/right revision IDs must already be persisted by their **native**
producer paths. The chat message is optional and must belong to one of the
selected source revisions for its source joins to be considered. There is no
global string-match query.

`ITIR_MIXED_OBSERVER_SCOPE` is an explicit local selector:
- `available` — load only existing, reviewed StatiBaker-to-source links;
- `excluded` — scope policy excludes operational context, not an assertion of absence;
- `redacted` — operational context is intentionally withheld;
- omitted/other — context is unavailable, never silently fetched.

The UI never requests activity records in the last three cases. Even when
operational context is available, it is observer history, **not semantic
evidence**. If the operational schema does not exist, report unavailable
rather than no activity.

## Two synchronized graph projections

The **PNF lens** connects source nodes to read-model-only shared candidate
fingerprint nodes. Edges are named
`pnf_candidate_overlap_not_subject_identity`. A match does not establish
a common subject, proposition, event, or independent witness.

The **operational lens** displays source nodes and explicitly sourced
operational association nodes. Without typed owner/target coordinates for an
association, the UI does not invent a directed source edge.

Both projections use the existing
`GraphIr`/`VisualObjectId` vocabulary, and
`decode_shell(ShellInput::Select(id))` and
`decode_gpu(GpuPickInput::Hit(id))` resolve to the same
`DomainCommand::SelectObject(id)`. wgpu continues to own rendering and
picking, while Dioxus owns reading, navigation and explanation.

The source cards present bounded excerpts reopened from the SLR canonical
source store or native chat archive, preserving the source-family identity.
Selection exposes source/provenance references and never mutates review state.

## The independence firewall

- Equal raw text is not a subject match.
- Overlapping L2/PNF candidate fingerprints are not an admitted equivalence.
- Chat digests alone do not authenticate an external producer.
- Qualified source backreferences are distinct from reviewed derivation.
- An activity timestamp does not establish copying or semantic relevance.
- Ten copies of one producer-owned utterance do not contribute ten independent
  witnesses.
- Unknown, unavailable, scoped, redacted and not-observed operational contexts
  have different read-model values.
- The UI never pays evidence, applicability, truth or semantic admission.

S29 review write commands remain with the existing review workstation and
are not synthesized by this reader. No relationship-disposition button is
wired to a persistence API until the reviewed relation owner exists.

## Acceptance still outstanding

This is an implementation cut, **not runtime certification**. It needs:
1. exact-head SLR and Dioxus Rust compile;
2. real PG fixtures including one transcript, its chat quotation, one
   unrelated identical sentence and actual StatiBaker observer links;
3. precise native source reopening, scope isolation, PNF/genealogy abstention,
   and shell/GPU pick parity;
4. review-state and privacy-scoped reopening;
5. existing SCALE-1 same-workload economy receipts separately.

Nothing in this branch certifies waveform transcription completeness or
native speaker identity.

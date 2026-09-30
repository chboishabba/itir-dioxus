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

## Physical wgpu receipt (source-written)

The new `m10_mixed_source_gpu_receipt` reuses the repository's existing
wgpu draw/picking code. Against a real PG pair it prepares **both** graphs,
submits offscreen render passes, executes `R32Uint` pick passes and GPU
readback, and checks that a source node selects the exact same domain
object in shell, PNF GPU lens and operational GPU lens. Its operational
scope is hardcoded to `ExcludedByScope`, so running a GPU acceptance
check does not quietly fetch private activity records.

```bash
cargo run --features "production-data gpu" \
  --example m10_mixed_source_gpu_receipt -- \
  'source-revision:...' 'chat-message-revision:...'
```

This does not claim execution until the receipt actually runs on
hardware; a committed executable is not a GPU measurement.

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

## M10.4 review journey (existing S29, not GPU-owned)

Select source revisions and open their comparison. Candidate widgets for
same-subject, same-event, quotation or source-dependency review appear
only when the typed SLR comparator has a corresponding L2 factor or
registered producer/source backreference. The operator must actively
choose one: there is **no automatic review admission** for repeated
literal phrases.

To enable mutating review, explicitly set
`ITIR_MIXED_CONSUMER_SCOPE` to a workspace-specific scope reference.
It is **not an authentication token or scope-grant**. The caller must
also enforce the established S30 MatterContext visibility and
disclosure rules. S30 now requires *both* native source revisions to be
included before projecting a correspondence review item, even if its
review item reference is directly selected.

The chosen proposal is persisted through
`propose_correspondence_review` and reopened as a typed
`SourceCorrespondence` S29 item. Standard S29 actions (accept,
reject, abstain, qualify, supersede and request-evidence) go through
`apply_correspondence_review`, then the existing
`apply_persisted_review_command` transactional reducer. The original
evidence references, two source identities, axis and consumer scope
remain stable. A selected item displays the persisted status and
attributable receipt history, not optimistic UI-only status.

An acceptance records a review decision, not proposition truth,
established quoting, same-source identity, or independent evidence.
Changing operational observer context does not create a review item or
set user priority. Navigation/GPU picking uses the same existing
selection vocabulary and never submits a review command by itself.

### Outstanding acceptance

All new M10.4 code is source-written. No claims of a successful exact
head Rust build, SQL migration/replay, S30 workspace security audit,
Dioxus interaction, Agda type-check, or physical GPU receipt are made.
The S29 ledger presently lacks an authoritative per-receipt timestamp;
the history UI therefore sorts by command reference without
pretending to establish chronological order.

### Actual Matter authorization for mutating review

The M10.4 write gateway now enforces the existing
`SENSIBLAW_MATTER_SCOPE` manifest **before both proposal persistence
and every S29 status mutation**. It uses the repository-native
`load_matter_scope_manifest` +
`project_matter_context` machinery. The configured
`ITIR_MIXED_CONSUMER_SCOPE` must equal the manifest's `matter_ref`;
both native source revisions must appear among that MatterContext's
included refs, after sealed/role/purpose/knowledge-cut/minimum-necessary
filtering. Failure is read-only. This is materially stronger than
trusting an arbitrary scope string. It does not replace authenticated
DB access or a higher-level permission service.

```bash
SENSIBLAW_MATTER_SCOPE=/path/to/sensiblaw.matter-scope.json \
ITIR_MIXED_CONSUMER_SCOPE='matter:your-matter' \
ITIR_MIXED_LEFT_REV='source-revision:...' \
ITIR_MIXED_RIGHT_REV='chat-message-revision:...' \
  cargo run --features desktop
```

After a Matter visibility change the action gateway re-evaluates the
manifest on each write, rather than trusting the review item's earlier
visibility. The canonical S30 Matter workspace also refuses to project
a correspondence item unless both source endpoints are included.

## WIKI-UI-1 — finite Wikidata ontology diagnostics

The same Dioxus shell (no new UI persistence layer) now consumes
the WIKI-1 typed read model from SLR. Its explicit entry selector is
`ITIR_WIKI_DIAGNOSTIC_REF=<persisted-diagnostic-ref>`. Both an
existing `SENSIBLAW_MATTER_SCOPE` and the original source/witness
visibility are mandatory; a diagnostic ID alone does not reveal
private statements. Every S29 review action re-checks current Matter
visibility before writing.

A case displays the **graph projection actually checked** (full
statement, truthy rank or scoped), source revision and snapshot
digest, JMD-attributed Lean owner and commit, observed execution
receipt, concrete problem witnesses, optional full native GUID/rank/
qualifier/reference bundles, unresolved obligations and advisory
repair predictions. The modeled verdict is never a public-edit
permission; no Wikidata write endpoint exists in this view.

The source-path contract and JMD CSV adapter are documented in
`slr/docs/wiki1_ontology_diagnostics.md`. Nat's climate migration is
a downstream use of this generic reader, not the reader's authority
for choosing Wikidata properties or authorizing edits.

No exact-head Rust/Lean/Agda/PG/UI runtime receipt was generated by
this source-writing tranche.

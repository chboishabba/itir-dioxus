# M11 — Comparative Workbench

Status: production boundary repaired; focused compiler/PostgreSQL/GPU receipts still required for this newest tranche.

## Canonical production path

JSON is **not** the production comparative ABI.

The canonical path is:

```text
PostgreSQL normalized follow rows
        ↓
sensiblaw-pg-source-store
        ↓
PersistedWorkbenchProjection
        ↓
sensiblaw-legal-runtime
        ↓
ComparativeWorkbenchProjection
        ↓
itir-dioxus
        ↓
GraphIr / Before-After-Delta
        ↓
wgpu
```

The PostgreSQL loader reads normalized typed rows from:

```text
pnf_follow_projection
pnf_follow_node
pnf_follow_edge
pnf_follow_edge_provenance
```

It deliberately does **not** reconstruct semantic graph state from
`pnf_follow_projection.payload JSONB`.

## Typed ownership

### sensiblaw-reader-model

Owns the portable production carriers:

```text
PersistedWorkbenchProjection
PersistedWorkbenchGraph
PersistedWorkbenchNode
PersistedWorkbenchEdge

ComparativeWorkbenchProjection
ThreeWayComparativeWorkbenchProjection
```

Serialization is optional and exists only for diagnostic export, fixture replay,
or explicit offline bundles.

### sensiblaw-pg-source-store

Owns PostgreSQL acquisition:

```text
projection_ref
    ↓
load_persisted_workbench_projection(...)
    ↓
PersistedWorkbenchProjection
```

### sensiblaw-legal-runtime

Owns semantic comparison and typed explanation welding:

```text
PersistedWorkbenchProjection(left)
PersistedWorkbenchProjection(right)
        ↓
project_typed_workbench_comparison(...)
        ↓
ComparativeWorkbenchProjection
```

For W0/W1/W2:

```text
project_typed_three_way_workbench_comparison(...)
```

Pabai D/C explanations are passed **in-process as Rust values**:

```text
TypedAnswerChangingExplanation
    ↓
ComparativeWorkbenchOverlay
    ↓
typed comparative projection
```

No overlay file is required in normal operation.

### itir-dioxus

Desktop production builds enable the `production-data` feature.

Dioxus receives the typed comparative projection and lowers it into the existing
visualisation/interaction IR. The frontend checks that its visual classification
agrees exactly with the runtime-owned typed sets:

```text
shared
changed
left-only
right-only
```

A mismatch is an error.

Dioxus does not become the authority for semantic comparison.

## Normal pair comparison

With `DATABASE_URL` configured:

```bash
cargo run --example m11_postgres_comparative_workbench --   <before-projection-ref>   <after-projection-ref>
```

The carrier is typed Rust throughout.

## Normal three-way comparison

```bash
cargo run --example m11_postgres_comparative_workbench --   <w0-projection-ref>   <w1-projection-ref>   <w2-projection-ref>
```

For the Pabai D/C path:

```bash
cargo run --example m11_postgres_comparative_workbench --   --pabai   <w0-projection-ref>   <w1-projection-ref>   <w2-projection-ref>
```

The Pabai helper runs the existing typed Pabai comparative regression,
constructs D/C overlays in memory, and passes them directly into the typed
three-way projection.

## Production GPU receipt

```bash
cargo run --features gpu   --example m11_postgres_comparative_gpu_receipt --   <before-projection-ref>   <after-projection-ref>
```

The receipt starts from PostgreSQL projection refs and contains no JSON/file
carrier step.

It verifies:

1. typed PostgreSQL workbench acquisition;
2. typed runtime comparative projection;
3. runtime/Dioxus shared/changed/one-sided parity;
4. Before / After / Delta GraphIr construction;
5. all three GPU draw submissions;
6. one shared source/provenance-bearing semantic object;
7. the same canonical VisualObjectId in Before and After;
8. Before GPU pick == After GPU pick == shell reducer state;
9. no semantic-authority/truth promotion;
10. no outcome prediction.

## JSON / file adapters

JSON remains supported for three bounded purposes only:

- reproducible diagnostic/export receipt;
- fixture replay;
- explicit offline bundle.

The reader ABI exposes optional helpers such as:

```text
export_persisted_workbench_json(...)
replay_persisted_workbench_json(...)
export_comparative_workbench_json(...)
```

The Dioxus legacy AU and comparative file adapters are explicitly named
`replay_*`.

Examples such as:

```text
m11_comparative_workbench
m11_comparative_gpu_receipt
au_legal_workbench
au_legal_gpu_receipt
```

are replay/receipt tools. They are not the normal SensibLaw → Dioxus carrier.

Likewise:

```text
m11_pabai_workbench_overlays D|C
```

is an inspection/export utility only. Normal Pabai operation passes the typed
overlay in-process.

## Fail-closed boundaries

The typed runtime rejects:

- non-candidate or authority-promoting workbench projections;
- graph edges whose endpoints are absent;
- typed overlay refs absent from either persisted graph;
- answer-changing refs without source + provenance closure;
- a visual comparison whose classification differs from the runtime projection.

The UI/GPU layer never creates:

```text
semantic authority
claim truth
evidence payment
residual payment
winner selection
outcome prediction
```


## Typed change metadata

The production comparative ABI now carries typed change metadata directly:

```text
semantic_ref
+ ComparativeChangeLayer
+ justification_refs
+ explanation_ref
+ answer_changing
```

The legal runtime derives this from the compiler-owned `ChangeLocus` /
`TypedAnswerChangingExplanation` records and projects it into
`sensiblaw-reader-model::ComparativeChangeAnnotation`.

Dioxus explicitly converts that enum into a presentation enum. It does not
classify changes from labels, colours, graph geometry, node kinds, or textual
explanations.

Compatibility string maps remain only for older replay/export surfaces.

The Delta topology may visually emphasize an answer-changing object only when
the typed annotation says `answer_changing = true`. The semantic ref,
source/provenance refs and runtime-owned comparison class remain unchanged.

The production Postgres CPU/GPU receipts print the typed annotation layer and
justification refs so the empirical Pabai receipt can mechanically establish:

```text
W0 -> W1: D / Applicability / reviewed justification receipts
W1 -> W2: C / Applicability / reviewed justification receipts
```

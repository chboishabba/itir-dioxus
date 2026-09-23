# M11 — Comparative Workbench Empirical Specimens

Status: source-written; focused compiler/GPU receipt still required for this tranche.

## Boundary

M11.1/M11.2 comparison semantics are already compiler-receipted.

This tranche does not add another comparison engine. It consumes:

- persisted AU workbench exports;
- their persisted `legal_follow_graph` projections;
- the existing comparative GraphIr projection;
- the existing DomainCommand/reducer path;
- optional typed presentation overlays emitted by SensibLaw.

The comparative UI remains projection-only and cannot create semantic authority,
claim truth, residual payment, a winning party, or a predicted outcome.

## Pair specimen

Run on two real persisted AU workbench exports:

```bash
cargo run --example m11_comparative_workbench --   /tmp/before.json   /tmp/after.json
```

The command fails closed if either persisted export has no legal-follow graph.

It reports:

- left/right world refs;
- Before / After / Delta node and edge counts;
- shared / changed / left-only / right-only semantic refs;
- source/provenance availability;
- non-promotion flags.

## Three-way specimen

For a Pabai-style W0 -> W1 -> W2 sequence:

```bash
cargo run --example m11_comparative_workbench --   /tmp/w0.json   /tmp/w1.json   /tmp/w2.json
```

This renders two linked comparisons:

```text
W0 -> W1
W1 -> W2
```

No route status is interpreted as a predicted judicial result.

## Typed D/C overlays

SensibLaw can emit the reviewed typed explanation overlays directly:

```bash
cargo run -p sensiblaw-legal-runtime   --example m11_pabai_workbench_overlays -- D   > /tmp/pabai-D-overlay.json

cargo run -p sensiblaw-legal-runtime   --example m11_pabai_workbench_overlays -- C   > /tmp/pabai-C-overlay.json
```

Then run:

```bash
cargo run --example m11_comparative_workbench --   /tmp/w0.json   /tmp/w1.json   /tmp/w2.json   /tmp/pabai-D-overlay.json   /tmp/pabai-C-overlay.json
```

The Dioxus adapter verifies that every overlay semantic ref exists in the
persisted GraphIr. Every declared answer-changing semantic object must reopen
both source and provenance refs.

If SensibLaw's semantic identity does not weld to the persisted graph identity,
the specimen fails rather than remapping or guessing.

## Comparative GPU receipt

With the GPU feature:

```bash
cargo run --no-default-features --features gpu   --example m11_comparative_gpu_receipt --   /tmp/before.json   /tmp/after.json
```

The receipt:

1. projects both real persisted legal-follow graphs;
2. constructs Before / After / Delta using one canonical semantic object-ID space;
3. submits all three graph draw passes;
4. finds a shared visible semantic node with source + provenance closure;
5. runs a real integer GPU pick in Before;
6. runs the same semantic-object GPU pick in After;
7. confirms Before-pick == After-pick == shell selection reducer state;
8. reports non-promotion and non-prediction flags.

The GPU remains a projection/proposal mechanism.

## Dioxus shell

`ComparativeWorkbenchView` is a data-driven component. It exposes:

- left/right/query/consumer/as-at/scope selectors when present;
- Before / After / Delta graph summaries;
- answer-changing semantic refs;
- typed change layers;
- explanation text;
- unresolved comparative items.

It is intentionally not populated with a synthetic default comparison. A real
persisted comparative specimen must be supplied by the runtime/operator flow.

## Acceptance target

A real comparative specimen is acceptable when:

- persisted Before/After graphs are non-empty;
- typed overlay refs resolve to actual graph semantic refs;
- answer-changing refs have source/provenance closure;
- canonical shared IDs are stable across panels;
- shell/GPU pick parity holds;
- the Delta panel renders;
- typed explanations survive into the workbench read model;
- no comparison path creates authority/truth or predicts an outcome.

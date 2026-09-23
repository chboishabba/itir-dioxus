# M10 — Unified Workbench

Status: active production UI tranche.

## Canonical host

Production workbench implementation lives in `chboishabba/itir-dioxus`.

The previous `ITIR-suite/itir-svelte` M10 strip is retained only as a
read-model/fixture regression reference. New UI implementation must not extend
that Svelte surface.

## Semantic progression

```text
Journal -> Timeline -> Handoff -> Matter / Proof -> Research
```

Each stage is one of:

```text
Available
Blocked(reason)
Unavailable(reason)
```

Unavailable is not false and hidden is not discarded.

## Architecture

```text
canonical ITIR/SensibLaw world
        |
        v
UnifiedWorkbenchReadModel
        |
        +--> Dioxus shell
        |
        +--> VisualisationIr --> wgpu renderer
                                |
Dioxus input -------------------+--> DomainCommand --> reducer
GPU pick -----------------------+
```

Dioxus owns ordinary shell UI. wgpu owns charts/graph/proof-topology rendering.
Neither is semantic authority.

## Current Wave-5 calibration

Expected:

- Journal: available with exact fact/source/statement lineage.
- Timeline: unavailable if no assembled event coordinate exists.
- Handoff: blocked while reviewed material lacks a real share-scope receipt.
- Matter/Proof: unavailable when no persisted legal-follow graph exists.
- Research: available while review/follow pressure remains.

## Next empirical weld

Consume a real AU read model containing persisted `legal_follow_graph` nodes
and edges. Matter/Proof becomes available only from those persisted refs.

Do not backfill a graph into an older fixture merely to satisfy the UI.

## GPU tranche

The first GPU slice is intentionally bounded:

- framework-neutral stable visual object IDs;
- GraphIr and ChartIr;
- Dioxus selection and GPU pick decode to the same DomainCommand;
- pure selection reducer;
- wgpu feature boundary;
- no graph reconstruction from the Dioxus component tree.

Next GPU implementation after compile validation:

1. deterministic graph vertices/edges;
2. integer pick-ID target;
3. one proof/source graph specimen;
4. later Sankey/hyperfabric layouts only after parity is green.

# itir-dioxus

Canonical Dioxus + wgpu operator workbench for the ITIR/SensibLaw suite.

This repository is the production UI host for M10 Unified Workbench.  It is a
projection/interaction client over canonical ITIR/SensibLaw state; it is not a
second semantic store.

Architecture:

```text
canonical shared world / SensibLaw runtime
        |
        v
framework-neutral read model + Visualisation IR
        |
        +--> Dioxus ordinary shell UI
        |
        +--> wgpu visual projection
                 |
Dioxus event ----+----> same admitted DomainCommand ----> reducer
GPU pick --------+
```

Permanent boundaries:

- Dioxus event != semantic mutation.
- GPU pick != semantic mutation.
- Visualisation IR != canonical graph store.
- Hidden from a projection != absent from the world.
- Changing workbench depth does not create authority, truth, or payment.
- Timeline/Proof stages may be unavailable; the UI must not fabricate them.
- ITIR-suite remains the cross-suite contract/fixture owner.
- `slr` is the Rust production runtime for SensibLaw.
- `dashi_agda` owns the formal parity/boundary proofs.

The old `ITIR-suite/itir-svelte` workbench strip is a retained read-model
regression/reference only. New Unified Workbench implementation belongs here.

Current M10 target:

```text
Journal -> Timeline -> Handoff -> Matter/Proof -> Research
```

Each stage is typed as Available, Blocked, or Unavailable.

See `docs/M10_UNIFIED_WORKBENCH.md`.

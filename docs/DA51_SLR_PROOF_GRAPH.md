# DA51 × SLR proof graph projection boundary

Date: 2026-10-17

`itir-dioxus` consumes the same persisted Matter / reviewed evidence / investigation / proof state that SLR owns and may project cross-repo DA51/formal-routing/artifact coordinates alongside it. It does not become a semantic or proof-payment owner.

A Matter-scoped view may compose:

```text
SLR reviewed PNF / proposition / residual / proof obligation
+ dashi-repo route/history and exact/equivalent/similar matches
+ DA51 identity/evidence
+ independently checked formal evidence
+ zkperf/eBPF runtime evidence
+ ITIR artifact/shard/HF/IPFS refs
```

Producer-owned IDs remain canonical. UI sorting, hiding, selecting, filtering, graph layout and GPU picking are presentation-only.

Hard boundaries:

```text
UI visibility != truth/absence
selection != review/admission
route display != theorem proof
formal evidence display != SLR payment recomputation
DA51 status/address != proof authority
runtime trace != security conclusion
artifact/CID link != semantic identity
```

For formal work, the UI should display the exact payment coordinate `demand_ref + obligation_ref + statement_digest` and the independent checker/certification state. It must not collapse changed statement digests, similar symbols, translated declarations, or shared DA51 addresses into one paid obligation.

The preferred integration surface remains backend/native mediation over read-only repository/MCP tools such as `proof_route_for_obligation`, `formal_evidence_for_obligation`, `da51_resolve`, `da51_evidence`, and `da51_frontier`.

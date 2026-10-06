# AI Agent Readiness for VibeScript

**Status:** Requirements overlay — **not** the language specification.  
**Normative language:** [`vibescript-core.md`](vibescript-core.md) (`vibe-0.1`).  
**Copyright © 2026 Timothy Charles Holborn.** All rights reserved.  
**Principal / inventor:** Timothy Charles Holborn &lt;timothy.holborn@gmail.com&gt;  
Assignment: [`COPYRIGHT.md`](../COPYRIGHT.md) · Licence: [`LICENSE`](../LICENSE)

In-repo publish copy: `docs/manuals/ai-agent-vibescript-readiness.md`  
Public hub (when Pages deploys): `/qualiaDB/vibe/`

This file says what agents need **in addition to** the 0.1 grammar. It does **not** add keywords, backends, or Quin literals. If this document and `vibescript-core.md` disagree, **the core spec wins**.

---

## 0. What this is, and what it is not

Foundation models have no pre-training on Vibe. Agents that author or repair scripts need:

1. A machine-readable map of the **actual** grammar, diagnostics, and `capability.invoke` ids.
2. Optional constrained-decoding artifacts derived from that grammar.
3. A small golden corpus of programs that already pass `poet-vibe` tests.
4. The existing diagnostic loop (`E001`…`E600` + UTF-8 spans) — not a second validator crate.

It is **not**:

- a second language spec
- a reason to implement `<<[ s p o g prov ]>>` (illegal — core §13 N3)
- a reason to add Ollama / llama.cpp (Qualia inference is in-process)
- a reason to invent `pulse.broadcast` or `aura.apply_schema` (0.1 is `pulse.publish` / `aura.validate`)
- a claim that `.d10` is the 10D extension (canonical is `.10d`)
- a mandate for new crates `poet-grammar` / `poet-validator` (those jobs live in `poet-vibe`)

---

## 1. Binding names that are real (0.1 + invoke)

Use these in examples and ontologies. Do not teach agents the struck names.

| Teach | Do not teach | Why |
|---|---|---|
| `quin.statement(...)` | `<<[ s p o g prov ]>>` | Host seals parity; overlay is illegal |
| `<<( s p o )>>` and `<< s p o ~ r >>` | `<< id \| s p o >>` | RDF 1.2 Turtle only |
| `pulse.publish` | `pulse.broadcast` | 0.1 binding table |
| `aura.validate` | `aura.apply_schema` | SHACL subset + `SHACL.validate` |
| `capability.invoke("…")` | new keywords per family | D15 |
| `graph.query(..., take: n)` | unbounded query | N5 / N7 |
| `GraphDatabase.sparql` | inventing SPARQL syntax in Vibe | SPARQL-star stays SPARQL |
| `SHACL.extensions` | undocumented constraint IRIs | catalog first |
| `.10d` | `.d10` as canonical | core §10 |
| `E001` `E100` `E200` `E300` `E400` `E500` `E600` | `VIBE_E0412` | core §9 |

Engine reach beyond 0.1 bindings is **`capability.invoke`**, listed at runtime by `CapabilityDiscovery.list` / `CapabilityDiscovery.coverage`.

---

## 2. Pillars (still wanted — after alignment)

The original four-layer stack stands, with corrected artifacts:

| # | Pillar | Artifact now | Status |
|---|---|---|---|
| 1 | Language knowledge graph | Derive TTL from `vibescript-core` + `ids::ALL_BOUND`, not a parallel API list | **not built** |
| 2 | Constrained decoding | EBNF is already in core §3; GBNF/JSON-Schema **export from that EBNF** | **not built** |
| 3 | Golden corpus | `crates/poet-vibe/fixtures/` + `tests/conformance.rs` (§12 / §13) | **partial** (examples exist; not 100 NL pairs) |
| 4 | Diagnostic loop | `poet-vibe` parse/check/eval already returns `Diagnostic { code, span, message }` | **partial** (API exists; no separate WASM “validator product”; no auto `suggested_fix` yet) |

A fifth pillar (skill file) is still useful: `skills/vibescript/SKILL.md` pointing at core §3–§13 and the invoke table — not at this file’s old examples.

---

## 3. Ontology (M1) — when written

Namespaces (keep):

- `vibe:` `https://qualiadb.org/schema/vibe#` — syntax / AST
- `pulse:` `https://qualiadb.org/schema/pulse#` — events
- `aura:` `https://qualiadb.org/schema/aura#` — shapes
- `poet:` `https://qualiadb.org/schema/poet#` — host / diagnostics

Each **real** 0.1 binding and each `ALL_BOUND` invoke id becomes one `vibe:BuiltinFunction` / `vibe:InvokeId`. Turtle must match live signatures. Example that is **legal**:

```turtle
vibe:PulsePublish a vibe:BuiltinFunction ;
    rdfs:label "pulse.publish" ;
    vibe:namespace "pulse" ;
    vibe:effect "External" ;
    vibe:example """pulse.publish("clinic/overheat", n);""" .
```

Do not emit a `vibe:PulseBroadcast` node.

---

## 4. Constrained decoding (M2)

Export **from** `vibescript-core.md` §3 EBNF:

1. GBNF for Qualia’s **in-process** local model (`gguf_bridge` / Webizen LLM) — not Ollama.
2. JSON Schema for remote structured-output APIs **if** Hybrid/Remote is granted (same three backends: Local / Remote / Hybrid).
3. Hard locks: RDF 1.2 spellings only; `requires [ capability(...) ]`; forbid `<<[`.

Place: `crates/poet-vibe/grammar/` (folder in the existing crate). **No** new workspace crate until asked.

---

## 5. Golden corpus (M3)

Start from what already passes:

- `crates/poet-vibe/fixtures/12_*.vibe` — §12 must-accept
- `crates/poet-vibe/fixtures/n*.vibe` — §13 must-reject
- Invoke samples in the Poet harness

Then add NL → source pairs **only** for those programs. Do not invent a 100-file corpus of illegal APIs. Target categories that match **implemented** seams first (cells, graph query, SHACL, SPARQL, sheet, manifold), then NLP gazetteer, then later 3D/IK / grapheme work.

---

## 6. Diagnostics (M4)

Use `poet_vibe::Diagnostic` / `DiagCode`. A WASM check of `poet-vibe` is already required. Do not stand up `crates/poet-validator`.

Still missing (honest):

- `suggested_fix` field (core §9 allows a safe fix; not filled yet)
- SHACL violations array on the same JSON as parse errors
- Agent-facing JSON export helper (one function is enough)

---

## 7. Skill (M5)

`skills/vibescript/SKILL.md` should tell agents:

- Read `vibescript-core.md` first.
- Call `CapabilityDiscovery.coverage` before inventing an id.
- Never write Quin `parity`.
- Classic UI stays default (D7).

Grapheme / unencoded-language patterns belong with NLP + HCF docs, not as fake Vibe keywords.

---

## 8. Implementation order (do not start M1–M5 from the old table)

| Order | Work | Depends on |
|---|---|---|
| 0 | This alignment (done in this revision) | core spec + D1–D18 |
| 1 | Publish hub `docs/vibe/` on Pages | manuals already in `docs/` |
| 2 | JSON diagnostic export + optional `suggested_fix` | existing `Diagnostic` |
| 3 | GBNF/JSON-Schema **generated from** §3 EBNF | 2 |
| 4 | Turtle catalog of 0.1 bindings + `ALL_BOUND` | live ids |
| 5 | Skill + more fixture/NL pairs | 1 + 4 |
| later | Grapheme / 3D IK / GeoSPARQL as invoke ids | those kernels |

**Do not** implement the pre-alignment milestone table (ontology of illegal APIs, Ollama GBNF, 100 synthetic pairs, separate validator crate).

---

## 9. Completeness verdict

| Question | Answer |
|---|---|
| Is this file a complete **language** spec? | **No.** That is `vibescript-core.md`. |
| Is this file a complete **agent-readiness** spec? | **No, not as first written.** After this revision it is complete enough to schedule M2–M5 **without** contradicting 0.1. |
| Must it be updated before implementing the readiness stack? | **Yes.** Implementing the first draft as-written would teach agents illegal syntax and the wrong inference stack. |
| Is 0.1 language implementation blocked on M1–M5? | **No.** Interpreter + invoke table already exist. Agent enablement is a parallel track. |

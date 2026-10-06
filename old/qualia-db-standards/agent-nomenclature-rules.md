# Agent Nomenclature Rules — STEM-Precise Term Isolation

**Copyright © 2026 Timothy Charles Holborn.** All rights reserved.
**Mandatory project rule.** All agents (human, software, or otherwise) working in this project tree must adhere to these nomenclature rules.

---

## 1. Principle

The project seeks to be **scientifically exacting** in its use of nomenclature. Terms that describe the qualities, capacities, or experiences of one agent type must not be applied to another agent type where the underlying STEM mechanism is fundamentally different.

**Semantic degradation** occurs when a term from one domain is applied to another:
- It anthropomorphises software agents, obscuring the actual statistical/computational mechanisms at work.
- It naturalises legal persons, attributing mental states to entities that have no mind.
- It computationalises natural persons, reducing conscious experience to processing.

This makes it impossible to evaluate what is actually happening — which is the entire point of the exercise.

---

## 2. Agent Type Classification

Three distinct agent types, each with its own nomenclature:

### 2.1 Natural Agent

A living creature with a mind — capable of consciousness, perception, subjective experience (qualia), emotion, belief, intent, and deliberate action.

- **Examples:** human, animal
- **Has:** mind, consciousness, qualia, subjective experience, embodiment, sensory perception, emotion, belief, intent, free will (debated but legally operative), habit, instinct
- **Does not have:** legal personhood (unless separately granted), computational determinism

### 2.2 Legal Person

An entity with legal personhood — capable of holding rights, duties, and liabilities, but having no mind. Acts through natural agents (officers, employees, delegates) or software agents (automated systems).

- **Examples:** company, corporation, government, institution, trust, partnership
- **Has:** legal personhood, procedural decision-making, corporate intent (through decision-makers), institutional position, routine, policy, duty of care, liability
- **Does not have:** mind, consciousness, qualia, emotion, belief (in the mental-state sense), instinct, sensory perception

### 2.3 Software Agent

A computational system that processes inputs and produces outputs through algorithms — including AI models, sensors, automated pipelines, and programs. No mind, no consciousness, no subjective experience, no legal personhood (unless separately granted).

- **Examples:** AI model (LLM, transformer), sensor, algorithm, automated pipeline, program, daemon
- **Has:** computation, signal acquisition, model assertions, default behaviour, context allocation, training data, parameters, sampling, enumeration
- **Does not have:** mind, consciousness, qualia, emotion, belief, intent, free will, perception (in the conscious sense), habit (in the automatic-behaviour sense), instinct

---

## 3. Nomenclature Isolation Table

Terms are scoped to the agent type whose STEM mechanism they accurately describe. Using a term outside its scope is a nomenclature violation.

| Concept | Natural Agent | Legal Person | Software Agent | Violation Notes |
|:-------|:-------------|:-------------|:---------------|:----------------|
| mental processing | cognition | decision-making process | computation | "cognitive" → natural only; "computation" → software only |
| sensory input | perception | observation (through procedure) | signal acquisition | "perception" implies conscious experience |
| held proposition | belief | institutional position | model assertion | "belief" requires a mind; "assertion" is a statistical output |
| affective state | emotion | — (not applicable) | — (not applicable) | emotion is a qualia; no equivalent for legal/software |
| automatic behaviour | habit | routine | default behaviour | "habit" is unconscious repetition by a mind |
| innate response | instinct | — (not applicable) | — (not applicable) | instinct is biological; no equivalent |
| focus allocation | attention | oversight | context allocation | "attention" is cognitive; "context allocation" is computational |
| false output | mistake | procedural error | enumeration failure | "mistake" implies a mind that erred; "enumeration failure" is statistical |
| intentional false output | fabrication | misrepresentation | — (not applicable) | "fabrication" requires intent; software has no intent |
| output without grounding | — (use fabrication if deliberate) | — (use misrepresentation if deliberate) | ungrounded generation | "ungrounded generation" is the STEM term for statistical output without grounding |
| perceptual error | perception error | observational error | signal error | "perception error" implies a mind that misperceived |
| awareness failure | attention failure | oversight failure | context loss | "attention failure" implies cognitive attention |
| intent | intent | corporate intent (through agents) | — (not applicable) | software has no intent; may have "programmed objective" |
| knowingly wrong | deliberate | wilful (through agents) | — (not applicable) | "deliberate" requires will |
| failure to prevent harm | negligent | institutional negligence | — (not applicable) | negligence requires foresight capacity |
| disregard of known risk | reckless | institutional recklessness | — (not applicable) | recklessness requires risk awareness |
| forced action | coerced | directed (by governance) | overridden | "coerced" requires free will to constrain |
| false belief | mistaken | misinformed (through agents) | misinformed (bad input) | "mistaken" implies holding a false belief |
| seeing things | hallucination | — (not applicable) | ungrounded generation | "hallucination" is a perceptual qualia of minds |
| pattern of error | cognitive bias | institutional bias | algorithmic bias / statistical bias | each bias type has a distinct mechanism |

---

## 4. Rules for Agents Working on This Project

1. **Never use mind-dependent terms for software agents.** Terms like "cognitive", "perception", "belief", "emotion", "instinct", "habit", "hallucination", "deliberate", "coerced", "negligent", "reckless", "mistaken" describe phenomena of conscious minds. Software agents compute; they do not think.

2. **Never use mind-dependent terms for legal persons** unless describing the natural agents acting on their behalf. A company does not "believe" — it "holds a position". A company does not "perceive" — it "observes through procedure".

3. **Never use intent-dependent terms for software agents.** Terms like "fabricated", "deliberate", "deceptive" require intent. Software agents generate output through statistical processes; the correct term is "enumerated", "generated", or "ungrounded" depending on context.

4. **Use "enumeration failure" for software agent errors** where the output does not correspond to reality. This is a statistical process failure — the model enumerated tokens that don't ground in verifiable reality. It is not a "mistake" (which implies a mind that erred) or a "fabrication" (which implies intent).

5. **Records are appendable.** When the nature of an axiom or output is unclear at the time of recording, the record should be appendable — any agent (same or different, natural or software) may clarify the nature of the axiom at a future datetime. The original record is not overwritten; a clarifying annotation is appended with provenance.

6. **When a term could apply to multiple agent types, scope it.** Use `applicableAgentType` or equivalent property to indicate which agent types the term applies to. If a term is universal (e.g. "error" in the general sense), use the most general form and provide agent-type-specific sub-types.

7. **When in doubt, use the most mechanically precise term.** If you can describe the STEM mechanism, do so. "The model's attention weights allocated low probability to the relevant context tokens" is better than "the model wasn't paying attention".

---

## 5. Enforcement

- **Ontology files:** All classes and properties must declare `applicableAgentType` where the term is agent-type-scoped. Terms without this property are universal.
- **Code:** Variable names, function names, and type names must follow the nomenclature isolation table. Code review should flag violations.
- **Documentation:** All documentation must use the correct term for the agent type being discussed. The term "hallucination" must not appear in any project artefact as a description of software agent behaviour.
- **Records:** Records that are unclear about agent type or causal mechanism should be marked as `epistemicMode: ambiguous` and remain appendable for future clarification.

---

## 6. Reference

- **Ontology:** [`qualia-ui/ontologies/epistemics.n3`](../qualia-ui/ontologies/epistemics.n3) — §11 Agent-Type Nomenclature Scoping
- **Ungrounded generation:** [`qualia-ui/ontologies/ungrounded-generation.n3`](../qualia-ui/ontologies/ungrounded-generation.n3) — software-agent-specific consequence terminology
- **Epistemics toolbox:** [`qualia-ui/tool-chest/TOOLBOX_EPISTEMICS_SPEC.md`](../qualia-ui/tool-chest/TOOLBOX_EPISTEMICS_SPEC.md)

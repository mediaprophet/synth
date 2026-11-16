# Swarm orchestrator launch brief

**Version 0.1 · 16 November 2026 · Ready for review; implementation not started**

This package converts the proposed specification into an implementation plan. It does not itself authorize execution. When the user authorizes implementation, use the following brief as the initial orchestrator task, subject to current instructions and tool availability.

## Objective and scope

Implement the accepted production core of `qualia-etalon-specification-competition-v0.2.md` through the 24 packets in `swarm-work-packets-v0.1.json`. First reconcile any winning competition proposal with the baseline; record every requirement delta and re-estimate affected packets. Do not assume a winner has already been selected.

The proposed target is an accepted core in 20 working days, with a 12–35-day judgment envelope and recalibration after two days. The day-four tracer demonstrates existing integration, not full qualification. Optional R01–R08 are separately gated and remain uncommissioned unless requested. Never redefine “complete” to include unimplemented research, unexecuted evidence or a list of source files alone.

## Read first

1. The applicable repository `AGENTS.md` instructions and supplied Synth/Hypermedia production contract.
2. `qualia-etalon-swarm-implementation-plan-v0.1.md` for scope, offer, model/resource rules and release gates.
3. `swarm-packet-register-v0.1.md`, `swarm-work-packets-v0.1.json` and `swarm-requirement-matrix-v0.1.md`.
4. Specification v0.2, `research-dossier-v0.2.md` and `source-evidence-v0.2.json`.
5. `swarm-schedule-v0.1.csv` and `swarm-cost-model-v0.1.json`; these are assumptions and calculations, not runtime observations.

## First dispatch

Start P00. Snapshot repository heads and uncommitted-file hashes without resetting them. Confirm accepted geometry, arithmetic, resource, hardware and quality profiles. Freeze exact write scopes and cross-packet contracts. Then release P01/P02 as dependencies allow. Use the existing function registry and source before claiming a capability is absent or proposing replacement.

At most four agents may be active: one coordinator and three workers/reviewers. Default to Sol/high for difficult implementation and integration, Terra/medium for the bounded packets specified, Astra/high for independent high-risk review, and optional Luna/medium for mechanical metadata subtasks. These model assignments are a proposed policy; confirm selectors are available and explicitly authorized before dispatch. Reforecast if fallback models materially change performance or review requirements.

Each agent gets only its packet, requirement excerpts, relevant source hashes, accepted interfaces, permitted files, tests and resource lease. Use a fresh minimal context when choosing a distinct model; obey the current tool's model/fork constraints. Do not repeatedly forward the entire research history.

## Required dispatch payload

```json
{
  "packet_id": "P06",
  "objective": "Implement hard collapse admission against the frozen contract",
  "specification_version": "0.2",
  "interface_version": "to be frozen by P02",
  "baseline_revisions_and_hashes": "filled from current workspace",
  "accepted_dependency_receipts": ["P01", "P03", "P05"],
  "allowed_write_paths": ["exact absolute paths assigned by coordinator"],
  "forbidden_shared_writes": ["Cargo.lock", "shared exports", "catalogs"],
  "model": "gpt-6.1-sol",
  "reasoning_effort": "high",
  "independent_reviewer_model": "gpt-6-astra",
  "slice_budget_hours": 4,
  "gpu_lease": false,
  "acceptance_commands_and_thresholds": "filled from frozen profile",
  "required_return": "diff summary, tests actually run, evidence paths, remaining failures"
}
```

This is an illustrative orchestration record, not a claim that such an API already exists. Populate every placeholder before dispatch. P06 also relies on the P02 contract transitively through its accepted dependencies; give the worker the actual contract and test fixtures.

## Required worker return

Return packet ID, author agent/model, exact touched paths, source baseline, interface changes, actual commands/exit codes, fixture hashes, measured results, evidence paths and unresolved defects. Separate `passed`, `failed`, `inconclusive`, `unsupported`, `cancelled` and `not_executed`. Never replace an unmeasured metric with zero.

Then dispatch a different reviewer agent against the same baseline and artifacts. Record reviewer agent/model and its counterexamples, independent checks and acceptance decision. Review consumes a worker slot. A packet becomes `accepted` only after review and integration of the accepted diff; downstream packets cannot start from an unreviewed success message.

After two failed bounded repair attempts, escalate with the minimal failing fixture and hypotheses. Stop dependent work on a broken interface or invalid mathematical assumption. Do not relax thresholds, invent morphisms or reinterpret binary layouts to clear a test.

## Integration rules

The coordinator exclusively owns shared manifests/exports and cross-repository integration. One writer per exact file. Use isolated worktrees or explicit leases; preserve unrelated modifications. Use separate packet build targets and unique create-only outputs. Do not kill live MCP or build Synth into `target`/`target-studio`. Serialize GPU qualification and, if RAM requires, heavy link jobs.

P17/P20 use state-machine fault injection without a GPU lease; P22 exercises actual runtime/device lifecycle and records the limits of simulated versus actual device-loss tests. P19 initially tunes kernel schedules; integrated decode/upload/compaction/draw accounting is confirmed in P22 after P18.

## Reporting and finish

Report accepted packets, newly discovered risks, failed gates, critical-path movement, cumulative token usage by model/tier and the next executable packet. Recalibrate at M0/M1. The proposed API-equivalent budget is a planning aid, not the user's actual Codex bill or automatic authority to spend.

At release, deliver all 38 requirement rows with evidence and reviewer, frozen support matrix, reproducible commands, source/file hashes, numeric and visual results, fair baselines/ablations, retained artifacts and recovery instructions. Do not publish, deploy, merge or commit merely because the planning package mentions release; follow the user's implementation authorization and applicable repository rules.

If only a plan review was requested, review this package and stop before implementation.

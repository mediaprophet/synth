# qualia-ui Health / Wellbeing / Document / Credentials UI Requirements

**Date:** 2026-08-18
**Principal / copyright holder:** Timothy Charles Holborn <timothy.holborn@gmail.com>
**Target UI:** `C:\Projects\NLP\qualia-ui` (WASM HyperCanvas — the primary UI target)
**Companion documents:**
- `C:\Projects\qualia-27062026\docs\plans\qualia-health-wellbeing-document-credentials-todo-2026-08-18.md` (qualiaDB engine to-do — **assumed completed** for the purposes of this requirements doc)
- `C:\Projects\qualia-27062026\docs\manuals\standards\qualia-embedded-credential-pdf-standard.md` (QECP standard)

---

## 1. Purpose and scope

This document specifies the qualia-ui functionality required to deliver **health and wellbeing (mental and physical), diagnostics, lab results, diet and related analysis, hypothesis generation for the natural person and for clinical professionals / social workers / government, biometrics and zero-knowledge derivatives, document processing with correct NLP extraction, and signed-credential PDF generation/parsing**.

It assumes the engine to-do (companion doc) is completed: every engine record, command, ontology, and capability referenced here is treated as available via the desktop Tauri command bus, the MCP tool surface, and the WASM runtime exports.

### 1.1 Honesty labels

- **live** — fully implemented and functional in qualia-ui today
- **present** — UI structure exists but backend not wired
- **missing** — no UI surface at all
- **planned** — referenced in specs/manifold seeds but not implemented

### 1.2 Current qualia-ui state (relevant surfaces)

| Surface | File | Status |
|---------|------|--------|
| Sanctuary manifold seed | `tool_chest/manifolds/sanctuary.rs` | **present** — seed only |
| Rights & Wallet manifold (credentials, wallet) | `tool_chest/manifolds/rights.rs` | **present** — seed only |
| No health/wellbeing manifold | — | **missing** |
| No document/lab-results/biometrics surfaces | — | **missing** |

---

## 2. Health & Wellbeing manifold (new manifold seed: "Health")

A dedicated health-and-wellbeing manifold with containers:

| Container | Type | Purpose | Engine | Priority |
|-----------|------|---------|--------|----------|
| Health Overview | `dashboard` | Summary dashboard: current conditions, active medications, recent vitals, recent lab results, mood trend, sleep debt, pending hypotheses | wellfare-core (conditions, medication, vitals, lab_results, mood_timeline, sleep_analytics, hypothesis) | P0 |
| Conditions | `list` | ConditionReport + AllergyReport list (add/edit/dispute) | `wellfare-core/conditions.rs` | P0 |
| Clinical Reports | `list` | ClinicalReport list (Pathology/Imaging/Discharge/Referral) with ClaimStatus badges (Draft/Submitted/ClinicianConfirmed/Disputed/Superseded) | `wellfare-core/clinical.rs` | P0 |
| Lab Results | `table` | LabResultGroup list → expand to LabResultRecord rows (test_name, value, unit, reference_range, flag) with trend sparklines | LAB-1..LAB-5 | P0 |
| Medications | `list` | MedicationCatalogEntry list + MedicationAdministration log (Taken/Skipped/Overdue) + adherence rate | `wellfare-core/medication.rs` + HW-9 | P0 |
| Vitals | `chart` | VitalSign time-series chart (heart_rate, blood_pressure, temperature, etc.) | HW-1 | P1 |
| Mental Wellbeing | `sheet` | WellbeingObservation log + mood timeline + PHQ-9/GAD-7 (and DASS-21/K10 if approved) self-assessment with score + severity band + safety flag + disclaimer | `wellfare-core/mental_wellbeing.rs` + `assessment.rs` + HW-6..HW-8 | P0 |
| Therapy Notes | `list` | TherapyNote list (Classified — Sanctuary only) | `wellfare-core/mental_wellbeing.rs` | P1 |
| Sleep | `chart` | SleepNightSample chart + SleepDebtReport + weekly heatmap | `wellfare-core/sleep_analytics.rs` | P1 |
| Diet | `list` | DietEntry log + NutritionAnalysisReport (calories, macros, micronutrient gaps) | `wellfare-core/medication.rs` DietEntry + DIET-1..DIET-2 | P1 |
| Physical Activity | `list` | PhysicalActivityRecord log | HW-5 | P1 |
| Immunizations | `list` | ImmunizationRecord list | HW-2 | P1 |
| Procedures | `list` | ProcedureRecord list | HW-3 | P1 |
| Family History | `tree` | FamilyHistoryRecord tree (relation → condition) | HW-4 | P1 |
| Hypotheses | `list` | Hypothesis list (Proposed/Supported/Disputed/Rejected) with supporting/contradicting evidence + confidence + disclosure tier selector | DIAG-1..DIAG-6 | P0 |
| Biometrics | `list` | BiometricRecord list (kind, commitment hash, enrolled_at — **never shows the template**) + ZK proof generation/verification actions | BIO-1..BIO-5 + ZK-1..ZK-7 | P1 |
| Documents | `library` | DocumentRecord library (PDF/DOCX/Image with QECP status badge) | DOC-20..DOC-21 | P0 |
| Welfare Support | `list` | AssistanceNeed + WelfareStream + GovernmentLetter | `wellfare-core/welfare_support.rs` | P1 |
| Life Records | `list` | LifeEventReport + WelfareCaseReport + CaseTaskReport | `wellfare-core/life_records.rs` | P1 |
| Authority Attestations | `list` | AuthorityAttestation list (authority type, subject, representation PDF/VC/PDF+VC, requires_action) | `wellfare-core/authority_attestation.rs` | P1 |
| Safeguards | `sheet` | Dead-man switch (arm/touch/attest) + incapacity | `webizen-desktop/commands/wellfair/safeguard.rs` | P1 |
| Disclosure Log | `list` | Disclosure event log (recipient, acting delegate, onward-share, tracing fingerprint) + leak trace | `webizen-desktop/commands/wellfair/disclosure.rs` | P1 |

### 2.1 Health Overview dashboard — detailed requirements

**Layout:** A grid of cards, each summarizing one health domain:

| Card | Content | Source | Priority |
|------|---------|--------|----------|
| Conditions | Active conditions + allergies (count + names) | `conditions.rs` | P0 |
| Medications | Active medications (count + names + next dose) | `medication.rs` | P0 |
| Vitals | Latest vital signs (heart_rate, BP, temperature) with trend arrow | HW-1 | P1 |
| Lab Results | Latest lab results with flag colours (normal=green, low=amber, high=orange, critical=red) | LAB-1..LAB-2 | P0 |
| Mental Wellbeing | Latest mood + latest PHQ-9/GAD-7 score + severity band + safety flag (if any) | `mental_wellbeing.rs` + `assessment.rs` | P0 |
| Sleep | Latest night duration + cumulative debt + chronic flag | `sleep_analytics.rs` | P1 |
| Diet | Today's calories + macro breakdown vs target | DIET-1..DIET-2 | P1 |
| Hypotheses | Pending hypotheses (count + top 3 by confidence) | DIAG-1..DIAG-3 | P0 |
| Documents | Recent documents with QECP status badges | DOC-20..DOC-21 | P0 |
| Welfare | Active welfare streams + open assistance needs | `welfare_support.rs` | P1 |

**Safety flag behaviour:** if a PHQ-9 item 9 (self-harm) flag is active, the Mental Wellbeing card shows a red banner with crisis line information and a "Contact crisis line" action. The banner is never dismissable until the flag is addressed.

---

## 3. Lab Results container — detailed requirements

### 3.1 Lab Results table

**View:** A table of LabResultGroup records, each expandable to show its LabResultRecord rows.

**Columns (group level):**
- Report name / authority
- Collected at
- Reported at
- Claim status (Draft/Submitted/ClinicianConfirmed/Disputed/Superseded)
- QECP badge (if the report was ingested from a QECP-enabled PDF)
- Action: view / dispute / supersede

**Columns (result level, expanded):**
- Test name
- Value
- Unit
- Reference range (low–high)
- Flag (normal=green, low=amber, high=orange, critical=red)
- Trend sparkline (if historical data exists for the same test_name)
- Ontology code (LOINC, if mapped via NLP-8)

**Interactions:**
- Click a group to expand/collapse its results
- Click a result row to see the full detail (clinician notes, source provenance)
- Click "Dispute" on a group → opens DisputedDiagnosisReport form (reuses `personal_records.rs`)
- Click "Trend" on a result → opens a time-series chart for that test_name

### 3.2 Lab Results import

**Requirement:** The user can import a lab results PDF. The UI must:

1. Detect whether the PDF is QECP-enabled (per §6 below)
2. If QECP-enabled: extract the signed credential, verify it, ingest as ClinicianConfirmed (if issuer is a pathology authority)
3. If not QECP-enabled: run NLP extraction, show extracted results with confidence scores, queue for human review
4. Show the import result with appropriate honesty labelling

---

## 4. Hypotheses container — detailed requirements

### 4.1 Hypothesis list

**View:** A list of Hypothesis records, each showing:

| Field | Display | Priority |
|-------|---------|----------|
| Hypothesis text | Full text | P0 |
| Generated by | Engine / Clinician / Self (badge) | P0 |
| Epistemic status | Proposed (blue) / Supported (green) / Disputed (amber) / Rejected (grey) | P0 |
| Confidence | 0.0–1.0 bar | P0 |
| Supporting evidence | List of record links (click → opens the record) | P0 |
| Contradicting evidence | List of record links | P0 |
| Disclosure tier | Current tier selector (Self / Clinician / SocialWorker / Government) | P0 |
| Generated at | Timestamp | P0 |

**Interactions:**
- Click a hypothesis → detail view with full supporting/contradicting evidence
- "Confirm" action (clinician only, requires clinician DID + attestation) → Supported
- "Reject" action (clinician or self) → Rejected with reason
- "Dispute" action → Disputed with reason
- "Generate hypotheses" action → runs the HypothesisEngine on current records → new Proposed hypotheses
- "Set disclosure tier" → changes the visible fields per the tier policy

### 4.2 Hypothesis detail view

**Layout:** Two-column — left: hypothesis text + metadata; right: evidence list (supporting in green, contradicting in red).

**Evidence card:** each evidence item shows:
- Record type (lab result, condition, medication, vital, observation, etc.)
- Record summary (e.g. "Ferritin = 12 µg/L (low, ref 30–400)")
- Record provenance (source, author, timestamp)
- Link to open the full record

### 4.3 Disclosure tier view

**Requirement:** When the user selects a disclosure tier, the hypothesis detail view re-renders with only the fields visible at that tier:

| Tier | Visible | Redacted |
|------|---------|----------|
| Self | Full hypothesis + all evidence + plain-language interpretation | None |
| Clinician | Full hypothesis + all evidence + clinician notes | Plain-language interpretation (clinician writes their own) |
| SocialWorker | Summary only ("medical condition confirmed, follow-up recommended") | Exact values, clinical notes, evidence details |
| Government | Summary only ("medical condition confirmed") | Exact values, clinical notes, evidence details, follow-up recommendations |
| Public | Exists only ("this person has a confirmed medical condition") | Everything else |

**The UI must show a clear banner when the current view is a tiered (redacted) view, not the full view.**

---

## 5. Biometrics & ZK derivatives container — detailed requirements

### 5.1 Biometric enrollment

**Requirement:** The user can enroll a biometric (voiceprint, fingerprint, face, iris, palm, gait, HRV). The UI must:

1. Show a clear statement: "Your biometric template is never stored. Only a mathematical commitment is kept. You can prove facts about your biometric without revealing it."
2. Capture the biometric (via device sensor or file upload)
3. Show the commitment hash (not the template)
4. Store the BiometricRecord (Classified sensitivity — Sanctuary only)

### 5.2 ZK proof generation

**Requirement:** The user can generate a ZK proof from a biometric-bound credential:

| Action | UI | Engine |
|--------|-----|--------|
| "Prove I am the credential subject" | Generates a ZK proof that the user's biometric matches the credential's commitment, without revealing the biometric | ZK-5 |
| "Prove I am over 18" | Generates a ZK proof that the user's DOB implies age >= 18, without revealing DOB | ZK-6 |
| "Prove this credential contains claim X" | Generates a ZK proof that the signed credential contains a specific claim, without revealing other claims | ZK-7 |
| "Selectively disclose fields" | Generates a selective-disclosure presentation showing only the selected fields, proving they were signed by the issuer | ZK-3 |

**The UI must show the proof result (verified/not verified) and the public inputs (what the verifier learns), and explicitly state what is NOT revealed.**

### 5.3 Biometric record list

**View:** A list of BiometricRecord entries:

| Field | Display |
|-------|---------|
| Kind | voiceprint / fingerprint / face / iris / palm / gait / HRV |
| Commitment hash | Hex (first 16 chars + ellipsis) |
| Enrolled at | Timestamp |
| Device | Device ID |
| Actions | Generate ZK proof / Verify / Revoke |

**The UI must never display, store, or transmit the raw biometric template. The commitment hash is the only representation.**

---

## 6. Document processing container — detailed requirements

### 6.1 Document library

**View:** A list of DocumentRecord entries:

| Field | Display | Priority |
|-------|---------|----------|
| Name | File name | P0 |
| Kind | PDF / DOCX / HTML / Image / Audio (icon) | P0 |
| QECP status | Badge: green "Signed credential" / amber "Credential present, unverified" / red "Content binding broken" / grey "No embedded credential" | P0 |
| Content hash | Hex (first 16 chars) | P0 |
| Extracted records | Count + types (e.g. "3 lab results, 1 clinical note") | P0 |
| Provenance | Source URI + ingest timestamp | P0 |
| Actions | View / Parse / Extract / Generate QECP PDF | P0 |

### 6.2 PDF import flow

**Requirement:** When a user imports a PDF, the UI must:

1. **Detect QECP status** (DOC-5): scan for JUMBF box → C2PA Manifest Store → `qualia:credential` assertion
2. **Show QECP badge** immediately:
   - Green "Signed credential" → the PDF has a verified signed credential
   - Amber "Credential present, unverified" → the PDF has a credential but the proof did not verify (or C2PA manifest is ParsedOnly)
   - Red "Content binding broken" → the PDF's text has been edited after the credential was embedded
   - Grey "No embedded credential" → the PDF has no embedded credential; NLP extraction is the only path
3. **If QECP-enabled + verified**: extract the credential, ingest as a first-class record with ClinicianConfirmed epistemic (if issuer is pathology authority) — no human review needed for the structured data (the credential's signature is authoritative)
4. **If QECP-enabled + unverified**: extract the credential, ingest as a first-class record with Submitted epistemic — flag for clinician review
5. **If not QECP-enabled**: run NLP extraction (NLP-1..NLP-2), show extracted records with confidence scores (NLP-3), queue for human review (NLP-4)
6. **Show extraction results** with honesty labels:
   - "Extracted from signed credential" (green) — authoritative
   - "NLP-extracted, unverified" (amber) — needs review
   - "NLP-extracted, low confidence" (red) — needs review, may be wrong

### 6.3 PDF viewer

**Requirement:** The UI must render the PDF and, for QECP-enabled PDFs, overlay the extracted credential fields on the rendered PDF (highlighting the regions that correspond to credential claims). For non-QECP PDFs, the overlay shows the NLP-extracted fields with confidence colouring (green = high confidence, amber = medium, red = low).

### 6.4 NLP extraction review

**Requirement:** For NLP-extracted records, the UI must show a review queue:

| Field | Display |
|-------|---------|
| Extracted field | Field name + value |
| Confidence | 0.0–1.0 bar (green/amber/red) |
| Source | PDF name + byte range (click → highlights the source text in the PDF viewer) |
| Extractor | Method + version |
| Reviewer | DID (once reviewed) |
| Status | Pending / Confirmed / Rejected |
| Actions | Confirm / Reject / Edit |

**The UI must never allow an NLP-extracted record to be presented as ClinicianObserved. Only a clinician confirmation (via ClaimStatus lifecycle) can promote to ClinicianObserved.**

### 6.5 QECP PDF generation

**Requirement:** The user can generate a QECP-enabled PDF (e.g. a clinician generating a signed lab-results PDF, or a person generating a signed summary PDF for a social worker). The UI must:

1. Select a credential (from the Credentials container in the Rights & Wallet manifold)
2. Select a disclosure profile (Self / Clinician / SocialWorker / Government)
3. Select a PDF template (or use the default)
4. Generate the PDF with the embedded signed credential + content binding + disclosure profile
5. Show the result with a green "Signed credential" QECP badge

**Engine:** DOC-11..DOC-17.

---

## 7. Clinical professional / social worker / government surfaces

### 7.1 Clinician portal

**Requirement:** A clinician (with a clinician DID + attestation) can:

| Action | UI | Engine | Priority |
|--------|-----|--------|----------|
| View a patient's shared records | Filtered by the disclosure tier the patient consented to | DIAG-7..DIAG-11 | P0 |
| Confirm a clinical report | ClaimStatus: Submitted → ClinicianConfirmed | `clinical.rs` | P0 |
| Confirm a hypothesis | Hypothesis: Proposed → Supported (becomes a ConditionReport) | DIAG-4 | P0 |
| Reject / dispute a hypothesis | Hypothesis: Proposed → Rejected/Disputed | DIAG-4 | P0 |
| Issue a signed credential | Generate a QECP-enabled PDF with a signed ClinicalReportCredential | DOC-11..DOC-17 | P0 |
| View the disclosure log | See who has accessed the patient's records | `disclosure.rs` | P1 |

### 7.2 Social worker portal

**Requirement:** A social worker (with a social-worker DID + attestation) can:

| Action | UI | Engine | Priority |
|--------|-----|--------|----------|
| View a client's summary records | Filtered by the SocialWorker disclosure tier (summary only — no exact values) | DIAG-7..DIAG-11 | P0 |
| View welfare support records | AssistanceNeed + WelfareStream + GovernmentLetter | `welfare_support.rs` | P1 |
| View housing/safety | HousingSafetyReport (if consented) | `personal_records.rs` | P1 |
| Add an assistance need | On behalf of the client (with proxy DID) | `welfare_support.rs` | P1 |
| View the disclosure log | See what they have accessed | `disclosure.rs` | P1 |

### 7.3 Government portal

**Requirement:** A government agent (with a government DID + attestation) can:

| Action | UI | Engine | Priority |
|--------|-----|--------|----------|
| View a citizen's summary records | Filtered by the Government disclosure tier (summary only — "medical condition confirmed", no clinical notes) | DIAG-7..DIAG-11 | P0 |
| Issue a welfare decision | Generate a QECP-enabled PDF with a signed WelfareDecisionCredential | DOC-11..DOC-17 | P1 |
| View welfare streams | WelfareStream list (if consented) | `welfare_support.rs` | P1 |
| View the disclosure log | See what they have accessed | `disclosure.rs` | P1 |

---

## 8. Cross-cutting UI requirements

### 8.1 Honesty labels

Every health record, hypothesis, document, and biometric shows an honesty badge:
- **Signed credential** (green) — from a verified QECP credential
- **Clinician confirmed** (green) — ClaimStatus: ClinicianConfirmed
- **NLP-extracted, unverified** (amber) — from NLP extraction, pending review
- **NLP-extracted, low confidence** (red) — from NLP extraction, confidence < 0.7
- **Self-reported** (grey) — owner-entered, SelfReported epistemic
- **Disputed** (red) — Disputed epistemic status

### 8.2 Provenance and audit

Every health record, hypothesis, document, and biometric shows its provenance: author DID, timestamp, predecessor_id, blob_hash, epistemic_status, sensitivity_class, source (PDF/credential/manual), byte range (if extracted). The Pulse stream shows the live audit trail. The Disclosure Log shows every access and onward-share.

### 8.3 Sensitivity class badges

| Class | Badge | Default for |
|-------|-------|-------------|
| Public | green | — |
| Restricted | amber | Conditions, allergies, clinical reports, lab results, vitals, medications, diet, welfare support, life records |
| Classified | red | Therapy notes, biometric records, crisis episodes |

The UI must never allow a Classified record to be disclosed to anyone other than the self (in Sanctuary mode) or a clinician (with explicit consent). The UI must never allow a Restricted record to be published to Permissive Commons.

### 8.4 Sanctuary mode

In Sanctuary mode:
- Therapy notes are visible
- Biometric records are visible (commitment hashes only)
- Crisis episodes are visible
- All external disclosure is disabled
- The Health Overview dashboard shows everything
- The Disclosure Log is visible (to audit who has accessed the person's records)

Outside Sanctuary mode (e.g. in a clinician portal):
- Therapy notes are hidden (unless the therapist specifically consented)
- Biometric records are hidden (only ZK proofs are shown)
- Crisis episodes are hidden (unless consented)
- The Health Overview dashboard shows only what the disclosure tier permits

### 8.5 Consent gate

Every disclosure to an external party (clinician, social worker, government) requires explicit consent:
1. The recipient requests access (with their DID + attestation + purpose)
2. The holder sees the request and approves/denies
3. If approved, a consent credential is issued (`consent_creds.rs`)
4. The disclosure is recorded in the Disclosure Log (`disclosure.rs`)
5. The recipient sees only the fields permitted by the disclosure tier

### 8.6 Capability gating

Every operation that requires a capability (ZK proof generation, QECP embedding, PDF parsing, C2PA crypto verification) checks the capability at call time and shows a disabled state with an explanation if unavailable.

### 8.7 Accessibility

- All health charts (vitals, lab trends, mood timeline, sleep heatmap) must have text alternatives (e.g. "Heart rate: 72 bpm, trending down over 7 days")
- The lab results table must be screen-reader accessible (test name, value, unit, flag announced)
- The hypothesis list must be screen-reader accessible
- Colour-coded flags (normal/low/high/critical) must have text labels (not colour alone)
- Crisis banners must be announced immediately by screen readers

### 8.8 Multi-language

All health/wellbeing/document/credential UI strings must be translatable. Medical terminology should be available in plain-language alternatives (e.g. "Ferritin (iron stores)" not just "Ferritin").

### 8.9 File size constraints

Per the qualia-ui handover, no UI file should exceed 800 lines. The Health manifold containers must be split into focused submodules when they approach the limit.

---

## 9. Use-case UI walkthroughs

### 9.1 Homeless person with health, financial, and legal challenges

1. The person opens the Health manifold → Health Overview
2. They see: active conditions (iron deficiency, hypertension), active medications (Metformin, iron supplement), latest lab results (Ferritin low, flagged amber), mood trend (declining), sleep debt (chronic flag), pending hypotheses ("iron deficiency anaemia — Proposed, confidence 0.7")
3. They import a pathology PDF → QECP-enabled (green badge) → the lab results are ingested as ClinicianConfirmed
4. They share their records with a social worker: set disclosure tier to SocialWorker → consent → the social worker sees "medical condition confirmed, follow-up recommended" (no exact values)
5. They share with a doctor: set disclosure tier to Clinician → consent → the doctor sees full results + hypotheses
6. The doctor confirms the hypothesis → "iron deficiency anaemia" becomes a ConditionReport with ClinicianObserved evidence
7. The doctor issues a signed clinical report → QECP-enabled PDF → the person stores it in their Documents library
8. The social worker views the welfare support records (AssistanceNeed: "emergency housing", WelfareStream: "housing benefit — Active")
9. The government agent views the Government disclosure tier → "medical condition confirmed" → processes the welfare benefit

### 9.2 Person building a house with health monitoring

1. The person opens the Health manifold → logs vitals daily (heart_rate, BP) via the Vitals container
2. They log diet via the Diet container → NutritionAnalysisReport shows calorie/macro balance
3. They log physical activity via the Physical Activity container
4. The HypothesisEngine generates: "BP trending up — Proposed, confidence 0.5, supporting: vital signs, contradicting: none"
5. They share with their GP: Clinician tier → the GP sees the BP trend + hypothesis → confirms or rejects
6. The GP issues a QECP-enabled PDF with a ClinicalReportCredential

### 9.3 Mental health support

1. The person opens the Health manifold → Mental Wellbeing container
2. They log a WellbeingObservation (mood: "low", intensity: 7/10)
3. They complete a PHQ-9 self-assessment → score 15 (moderately severe) → safety flag (item 9) → red banner with crisis line
4. They contact the crisis line (action button)
5. They share with a therapist: Clinician tier → the therapist sees the mood timeline + PHQ-9 scores + therapy notes
6. The therapist adds a TherapyNote (Classified — Sanctuary only for the person)
7. The HypothesisEngine generates: "depression screening positive — Proposed, confidence 0.8, supporting: PHQ-9 score, contradicting: none"
8. The therapist confirms → becomes a ConditionReport

### 9.4 Biometric identity proof

1. The person opens the Health manifold → Biometrics container
2. They enroll a voiceprint → commitment hash stored (template discarded)
3. They bind the voiceprint to their identity credential (BiometricCredentialBinding)
4. They need to prove their identity to a government agency:
   - They generate a ZK proof: "Prove I am the credential subject" (proves voiceprint matches credential commitment without revealing the voiceprint)
   - The government agent verifies the ZK proof → confirmed
5. The government agent sees only the Government disclosure tier (no clinical data)

### 9.5 Clinician issuing a signed lab report

1. The clinician opens the Clinician portal → sees the patient's shared records (Clinician tier)
2. They review the pending lab results (Submitted status)
3. They confirm the report → ClinicianConfirmed
4. They issue a signed ClinicalReportCredential → QECP-enabled PDF (DOC-11..DOC-17)
5. The PDF is sent to the patient → the patient imports it → green "Signed credential" badge → ingested as ClinicianConfirmed

### 9.6 Document processing with NLP extraction

1. The person imports a non-QECP pathology PDF (grey badge)
2. The NLP extractor runs: LabResultExtractor detects lab result tables, extracts test_name/value/unit/reference_range with confidence scores
3. The UI shows the extracted results with confidence bars (some green, some amber, one red)
4. The person reviews the extraction: confirms the green ones, edits an amber one, rejects the red one
5. The confirmed records become first-class LabResultRecord entries with SelfReported epistemic + ExtractionProvenance
6. The person shares with a clinician → the clinician confirms the values → ClinicianObserved

---

## 10. Implementation priority for qualia-ui

### Phase 0 — Foundation (P0, must precede everything)

- Reuse the collaborative doc's Phase 0 (IntentBus, transport, ontology loader, toolbox registration)
- Real capability detection for ZK proof generation, QECP embedding/extraction, PDF parsing

### Phase 1 — Core health + lab + hypothesis + document + QECP (P0)

- Health manifold seed + Health Overview dashboard
- Conditions, Clinical Reports, Lab Results, Medications, Mental Wellbeing, Hypotheses, Documents containers
- Lab Results import (QECP detection + NLP extraction + review queue)
- Hypothesis list + detail + disclosure tier view
- QECP PDF generation flow
- Clinician / Social Worker / Government portals
- Tool family: `health_wellbeing` (Vitals, Sleep, Diet, Mood Timeline, Assessment Instruments, Biometrics, ZK Proofs)

### Phase 2 — Extended health surfaces (P1)

- Vitals chart, Sleep chart, Diet list, Physical Activity list, Immunizations, Procedures, Family History tree
- Therapy Notes, Crisis Episodes, Medication Adherence
- Nutrition Analysis Report, Diet ↔ Lab correlation
- Biometric enrollment + ZK proof generation/verification
- Welfare Support, Life Records, Authority Attestations, Safeguards, Disclosure Log
- DASS-21 / K10 assessments (if approved)

### Phase 3 — Polish (P2)

- Sleep episodes, Sleep-mood correlation
- Hydration tracking
- .10d document section

---

## 11. File structure (proposed additions to qualia-ui)

```
qualia-ui/src/
├── tool_chest/
│   ├── manifolds/
│   │   ├── health.rs               (new — health & wellbeing manifold)
│   │   └── ...
│   └── tools/
│       ├── health_wellbeing/       (new — Vitals, Sleep, Diet, Mood Timeline, Assessment Instruments, Biometrics, ZK Proofs)
│       ├── documents/              (new — PDF Import, PDF Viewer, QECP Generator, Extraction Review)
│       └── ...
├── browser/
│   ├── containers.rs               (extend: lab_results, hypothesis, biometrics, document_library, pdf_viewer, extraction_review, qecp_generator container types)
│   └── ...
└── ontologies/
    ├── wellfair-health.ttl         (new — vital, immunization, procedure, family_history, physical_activity, lab_result, hypothesis, biometric, disclosure_tier)
    └── document.ttl                (new — document_record, pdf_structure, extraction_provenance)
```

---

## 12. Open questions for the principal

1. **Health manifold vs Sanctuary manifold**: should health records live in a new "Health" manifold or extend the existing "Sanctuary" manifold? New manifold is cleaner; Sanctuary extension avoids manifold proliferation but conflates health with crisis/safety.
2. **Clinician portal location**: should the clinician/social-worker/government portals be separate UI modes (a "portal" toggle) or separate manifolds within the same UI? Separate modes are cleaner for role-based access; same-UI manifolds are more flexible for multi-role users (e.g. a person who is also a clinician).
3. **Hypothesis disclosure tier default**: when a person views their own hypotheses, the default tier is Self (full). When they share, they must explicitly choose a tier. Should the UI default to the most restrictive tier (Government) and require the user to escalate, or default to the least restrictive (Self) and require de-escalation? Default-restrictive is safer; default-permissive is more convenient.
4. **Biometric enrollment UI**: should biometric enrollment be in the Health manifold or the Rights & Wallet manifold (since it's an identity credential)? Health makes sense for HRV/gait; Rights & Wallet makes sense for voiceprint/fingerprint/face. Recommendation: enrollment in Rights & Wallet, health-related biometrics (HRV) also linked from Health.
5. **QECP PDF generation scope**: should the UI allow any person to generate a QECP-enabled PDF (self-attesting their own records), or only clinicians/authorities? Self-attesting is useful for self-reported summaries; clinician/authority-only preserves the "ClinicianObserved requires a clinician" honesty boundary. Recommendation: self-attesting allowed but the credential type is "SelfReportedCredential", not "ClinicalReportCredential".
6. **NLP extraction review workflow**: should the review queue be in the Documents container or the Health manifold? Documents is where the import happens; Health is where the records live. Recommendation: review in Documents (where the PDF is), confirmed records appear in Health.

---

## 13. References

| Document | Path |
|----------|------|
| qualiaDB engine to-do (companion) | `C:\Projects\qualia-27062026\docs\plans\qualia-health-wellbeing-document-credentials-todo-2026-08-18.md` |
| QECP standard (companion) | `C:\Projects\qualia-27062026\docs\manuals\standards\qualia-embedded-credential-pdf-standard.md` |
| Collaborative UI requirements | `C:\Projects\NLP\consult\20260818_qualia-collaborative-ui-requirements.md` |
| 3D/animation/dataset/audio UI requirements | `C:\Projects\NLP\consult\20260818_qualia-3d-animation-dataset-audio-ui-requirements.md` |
| wellfare-core source | `C:\Projects\qualia-27062026\crates\wellfare-core\src\` |
| Desktop wellfair commands | `C:\Projects\qualia-27062026\crates\webizen-desktop\src\commands\wellfair\` |
| Native VC | `C:\Projects\qualia-27062026\crates\qualia-core-db\src\crypto\verifiable_credential.rs` |
| W3C VC runtime | `C:\Projects\qualia-27062026\crates\qualia-core-db\src\identity\credentials\mod.rs` |
| ZK proofs | `C:\Projects\qualia-27062026\crates\qualia-core-db\src\crypto\zk_proofs.rs` |
| C2PA vocabulary | `C:\Projects\qualia-27062026\crates\qualia-core-db\src\sparql_library\sparql_mm.rs` |
| NLP | `C:\Projects\qualia-27062026\crates\qualia-core-db\src\nlp\` |
| C2PA Technical Specification | https://spec.c2pa.org/ |

---

_End of UI requirements. Companion engine to-do: `C:\Projects\qualia-27062026\docs\plans\qualia-health-wellbeing-document-credentials-todo-2026-08-18.md`. Companion standard: `C:\Projects\qualia-27062026\docs\manuals\standards\qualia-embedded-credential-pdf-standard.md`._

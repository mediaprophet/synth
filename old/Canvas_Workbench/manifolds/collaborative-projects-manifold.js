/**
 * Collaborative Projects Nexus Manifold Definition
 * Optimized for cross-project dependency DAGs, contributor resource/equity allocation,
 * milestone delivery verification, and inter-dependent manifold causal linking.
 */

export const collaborativeProjectsManifold = {
  id: 'manifold-project-nexus-01',
  uuid: 'urn:uuid:7c8d9e0f-1a2b-3c4d-5e6f-7a8b9c0d1e2f',
  title: 'Collaborative Projects Nexus & Work DAG',
  type: 'project',
  layout: 'workbench',
  icon: '🚀',
  graphIri: 'graph:manifold:collaborative_projects_01',
  category: 'qualia:CollaborativeProjects',
  topic: 'topic:InterDependentProjectGovernance',
  rights: {
    rightsHolder: 'did:qualia:timothy_charles_holborn',
    fiduciaryDutyLevel: 'fiduciary:GateLevel3',
    permissionCircle: 'circle:ProjectConsortiumAuthors',
    merkleRootHash: '0x7C8D9E0F1A2B3C4D',
    licence: 'https://qualia.io/licences/fiduciary-duty-of-care-v1'
  },
  created: '2026-08-15T00:00:00Z',
  modified: '2026-08-16T17:20:00Z',
  defaultDimension: '2D',
  activeStrata: ['social', 'technical', 'financial', 'legal'],
  activeEpistemicModality: 'all',
  activeToolboxes: ['communication', 'finance_ledger', 'office', 'code', 'ai'],

  containers: [
    {
      id: 'container-project-dag-01',
      type: 'project_dag',
      strata: 'technical',
      epistemic: 'objective',
      title: 'Inter-Dependent Project Milestone DAG',
      x: 80,
      y: 70,
      width: 480,
      height: 330,
      z: 100,
      d: 1.0,
      validFrom: '2026-08-15T00:00:00',
      validTo: '2026-08-16T00:00:00',
      projects: [
        {
          id: 'proj:hydrology_p2',
          name: '1. Catchment Hydrology Phase 2',
          lead: 'Timothy Charles Holborn',
          milestone: 'M3: Acoustic Sensor Deployment',
          progress: 85,
          status: 'Ready for Escrow Release',
          budgetLinked: '$50,000 (Finance Manifold)'
        },
        {
          id: 'proj:acoustic_studio',
          name: '2. 10D Acoustic & Speech Studio',
          lead: 'Q42 Linguistic Consortium',
          milestone: 'M2: Vocal Tract Kinematic Tensor Mesh',
          progress: 60,
          status: 'In Progress',
          budgetLinked: '$35,000 (Finance Manifold)'
        },
        {
          id: 'proj:rights_shacl',
          name: '3. Water-Rights SHACL Validation Guard',
          lead: 'Fiduciary Legal AI Team',
          milestone: 'M4: Statutory Duty Alignment Matrix',
          progress: 95,
          status: 'SHACL Validated',
          budgetLinked: '$30,000 (Finance Manifold)'
        }
      ],
      graphTriples: []
    },
    {
      id: 'container-contributors-01',
      type: 'social',
      strata: 'social',
      epistemic: 'intersubjective',
      title: 'Project Team, Fiduciary Roles & Allocations',
      x: 590,
      y: 70,
      width: 420,
      height: 330,
      z: 100,
      d: 1.0,
      validFrom: '2026-08-15T00:00:00',
      validTo: '2026-08-16T00:00:00',
      messages: [
        { id: 'cm1', sender: 'Timothy Charles Holborn', isAi: false, time: '16:00', text: 'Catchment sensor telemetry confirmed. Milestone M3 is ready for verification.', epistemic: 'subjective' },
        { id: 'cm2', sender: 'Fiduciary Treasurer AI', isAi: true, time: '16:02', text: 'Verified SHACL attestation. Escrow release of $15,000 queued for multi-sig signoff.', epistemic: 'normative' }
      ],
      subAgents: [
        { id: 'agent:TreasurerAI', name: 'Fiduciary Treasurer AI', role: 'Escrow & Budget Custodian', status: 'Online' },
        { id: 'agent:CoordinatorAI', name: 'DAG Project Coordinator', role: 'Milestone Dependency Tracking', status: 'Online' }
      ],
      graphTriples: []
    },
    {
      id: 'container-nested-finance-manifold',
      type: 'manifold',
      strata: 'financial',
      epistemic: 'normative',
      title: 'Nested Manifold: Financial Economics & Ledgers',
      x: 80,
      y: 430,
      width: 480,
      height: 290,
      z: 50,
      d: 1.0,
      validFrom: '2026-08-15T00:00:00',
      validTo: '2026-08-16T00:00:00',
      targetManifoldId: 'manifold-finance-01',
      targetManifoldTitle: 'Financial Economics & Double-Entry Ledger',
      targetManifoldUuid: 'urn:uuid:6b7c8d9e-0f1a-2b3c-4d5e-6f7a8b9c0d1e',
      targetCategory: 'qualia:FinancialEconomics',
      targetTopic: 'topic:DoubleEntryLedgersAndEscrowGovernance',
      nestedOccupantsCount: 4,
      nestedWiresCount: 3,
      merkleHash: '0x6B7C8D9E',
      graphTriples: []
    },
    {
      id: 'container-nested-research-manifold',
      type: 'manifold',
      strata: 'environmental',
      epistemic: 'objective',
      title: 'Nested Manifold: Research & Epistemology',
      x: 590,
      y: 430,
      width: 420,
      height: 290,
      z: 50,
      d: 1.0,
      validFrom: '2026-08-15T00:00:00',
      validTo: '2026-08-16T00:00:00',
      targetManifoldId: 'manifold-research-01',
      targetManifoldTitle: 'Research & Epistemological Mindware Manifold',
      targetManifoldUuid: 'urn:uuid:7f3a8b21-4d1e-4a6c-9e28-1b5e8c3a9d01',
      targetCategory: 'qualia:ResearchAndEpistemology',
      targetTopic: 'topic:CatchmentHydrologyAndBioTelemetry',
      nestedOccupantsCount: 4,
      nestedWiresCount: 3,
      merkleHash: '0x8F3A29B4',
      graphTriples: []
    }
  ],

  connections: [
    {
      id: 'wire-p1',
      from: 'container-project-dag-01',
      to: 'container-contributors-01',
      type: 'intersubjective',
      label: 'proj:assignsWorkPackage',
      predicate: 'proj:assignsWorkPackage',
      bridgeType: 'Milestone-to-Team Assignment'
    },
    {
      id: 'wire-p2',
      from: 'container-project-dag-01',
      to: 'container-nested-finance-manifold',
      type: 'normative',
      label: 'fin:triggersMilestoneEscrow',
      predicate: 'fin:triggersMilestoneEscrow',
      bridgeType: 'Cross-Manifold Escrow Release Causal Wire'
    },
    {
      id: 'wire-p3',
      from: 'container-project-dag-01',
      to: 'container-nested-research-manifold',
      type: 'objective',
      label: 'proj:groundsTelemetryDeliverable',
      predicate: 'proj:groundsTelemetryDeliverable',
      bridgeType: 'Cross-Manifold Empirical Grounding Wire'
    }
  ]
};

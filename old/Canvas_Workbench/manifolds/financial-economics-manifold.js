/**
 * Financial Economics & Double-Entry Ledger Manifold Definition
 * Optimized for double-entry linked accounting quads, milestone-locked escrow pools,
 * multi-currency capital flows, statutory fiscal compliance, and computational gas metering.
 */

export const financialEconomicsManifold = {
  id: 'manifold-finance-01',
  uuid: 'urn:uuid:6b7c8d9e-0f1a-2b3c-4d5e-6f7a8b9c0d1e',
  title: 'Financial Economics & Double-Entry Ledger Manifold',
  type: 'finance',
  layout: 'workbench',
  icon: '💰',
  graphIri: 'graph:manifold:financial_economics_01',
  category: 'qualia:FinancialEconomics',
  topic: 'topic:DoubleEntryLedgersAndEscrowGovernance',
  rights: {
    rightsHolder: 'did:qualia:timothy_charles_holborn',
    fiduciaryDutyLevel: 'fiduciary:GateLevel3',
    permissionCircle: 'circle:FiduciaryTreasurerOnly',
    merkleRootHash: '0x6B7C8D9E0F1A2B3C',
    licence: 'https://qualia.io/licences/fiduciary-duty-of-care-v1'
  },
  created: '2026-08-15T00:00:00Z',
  modified: '2026-08-16T17:20:00Z',
  defaultDimension: '2D',
  activeStrata: ['financial', 'legal', 'technical'],
  activeEpistemicModality: 'all',
  activeToolboxes: ['finance_ledger', 'sheet', 'rights', 'code', 'office'],

  containers: [
    {
      id: 'container-ledger-01',
      type: 'ledger',
      strata: 'financial',
      epistemic: 'normative',
      title: 'Double-Entry Linked Accounting Quad Ledger',
      x: 80,
      y: 70,
      width: 460,
      height: 330,
      z: 300,
      d: 1.0,
      validFrom: '2026-08-15T00:00:00',
      validTo: '2026-08-16T00:00:00',
      currency: 'AUD / USDC / QUALIA-GAS',
      totalBalance: '$142,500.00',
      auditHash: '0x4F8A9C2E1D7B',
      entries: [
        { id: 'tx-01', date: '2026-08-15 09:00', accountDr: 'asset:Escrow_Catchment_Phase2', accountCr: 'equity:EcoGrant_Pool_A', amount: '$50,000.00', memo: 'EcoGrant Catchment tranche allocation', quad: '<<[ tx:01 fin:debit asset:Escrow_Catchment <graph:ledger> ]>>' },
        { id: 'tx-02', date: '2026-08-15 11:30', accountDr: 'expense:Sensor_Hardware_Deploy', accountCr: 'asset:Treasury_Reserve', amount: '$12,500.00', memo: 'UAV acoustic sensor array acquisition', quad: '<<[ tx:02 fin:debit expense:Hardware <graph:ledger> ]>>' },
        { id: 'tx-03', date: '2026-08-15 14:40', accountDr: 'expense:VibeScript_Gas_Compute', accountCr: 'liability:Gas_Pool_Sentinel', amount: '$340.00', memo: 'Merkle-CRDT verification gas settlement', quad: '<<[ tx:03 fin:debit expense:GasCompute <graph:ledger> ]>>' },
        { id: 'tx-04', date: '2026-08-16 10:15', accountDr: 'asset:Milestone_M3_Released', accountCr: 'asset:Escrow_Catchment_Phase2', amount: '$15,000.00', memo: 'Milestone M3 delivery escrow release to Project Team', quad: '<<[ tx:04 fin:transfer asset:MilestoneM3 <graph:ledger> ]>>' }
      ],
      graphTriples: []
    },
    {
      id: 'container-budget-01',
      type: 'sheet',
      strata: 'financial',
      epistemic: 'objective',
      title: 'Multi-Project Capital Flows & Escrow Pools',
      x: 570,
      y: 70,
      width: 440,
      height: 330,
      z: 300,
      d: 1.0,
      validFrom: '2026-08-15T00:00:00',
      validTo: '2026-08-16T00:00:00',
      sheetData: [
        ['Project Stream', 'Allocated Pool', 'Committed', 'Escrow Locked', 'Disbursed', 'Status'],
        ['1. Hydrology Catchment P2', '$50,000', '$12,500', '$22,500', '$15,000', 'Active (M3 Passed)'],
        ['2. 10D Acoustic Studio', '$35,000', '$8,000', '$17,000', '$10,000', 'In Progress'],
        ['3. Water-Rights Fiduciary', '$30,000', '$6,500', '$15,500', '$8,000', 'SHACL Validated'],
        ['4. Cybernetic VM Gas Pool', '$27,500', '$3,200', '$18,300', '$6,000', 'Operational']
      ],
      graphTriples: []
    },
    {
      id: 'container-gas-accounting-01',
      type: 'code',
      strata: 'technical',
      epistemic: 'objective',
      title: 'VibeScript VM Compute & Gas Metering',
      x: 1040,
      y: 70,
      width: 380,
      height: 330,
      z: 400,
      d: 1.1,
      validFrom: '2026-08-15T00:00:00',
      validTo: '2026-08-16T00:00:00',
      codeSnippet: `// VibeScript Fiscal Gas Metering VM
on transaction:settle_gas(tx: GasTransaction) {
    let gas_used = tx.compute_units * 0.00001;
    let storage_fee = tx.storage_bytes * 0.000002;
    let total_cost = gas_used + storage_fee;
    
    fiduciary.assert_spend_cap(total_cost, "cap:hourly_compute");
    ledger.post_debit("expense:VibeScript_Gas", total_cost);
    return { status: "Settled", digest: tx.digest };
}`,
      graphTriples: []
    },
    {
      id: 'container-nested-projects-manifold',
      type: 'manifold',
      strata: 'social',
      epistemic: 'objective',
      title: 'Nested Manifold: Collaborative Projects Nexus',
      x: 80,
      y: 430,
      width: 460,
      height: 290,
      z: 50,
      d: 1.0,
      validFrom: '2026-08-15T00:00:00',
      validTo: '2026-08-16T00:00:00',
      targetManifoldId: 'manifold-project-nexus-01',
      targetManifoldTitle: 'Collaborative Projects Nexus & Work DAG',
      targetManifoldUuid: 'urn:uuid:7c8d9e0f-1a2b-3c4d-5e6f-7a8b9c0d1e2f',
      targetCategory: 'qualia:CollaborativeProjects',
      targetTopic: 'topic:InterDependentProjectGovernance',
      nestedOccupantsCount: 4,
      nestedWiresCount: 3,
      merkleHash: '0x7C8D9E0F',
      graphTriples: []
    }
  ],

  connections: [
    {
      id: 'wire-f1',
      from: 'container-ledger-01',
      to: 'container-budget-01',
      type: 'normative',
      label: 'fin:reconcilesPool',
      predicate: 'fin:reconcilesPool',
      bridgeType: 'Fiscal Double-Entry Reconciliation'
    },
    {
      id: 'wire-f2',
      from: 'container-gas-accounting-01',
      to: 'container-ledger-01',
      type: 'objective',
      label: 'fin:recordsGasCost',
      predicate: 'fin:recordsGasCost',
      bridgeType: 'Automated Micro-Billing Flow'
    },
    {
      id: 'wire-f3',
      from: 'container-budget-01',
      to: 'container-nested-projects-manifold',
      type: 'normative',
      label: 'fin:fundsProjectTranche',
      predicate: 'fin:fundsProjectTranche',
      bridgeType: 'Inter-Manifold Capital Escrow Bridge'
    }
  ]
};

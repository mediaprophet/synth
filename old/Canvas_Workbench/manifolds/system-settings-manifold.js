/**
 * System Settings & Cybernetic Governance Manifold Definition
 * Optimized for Fiduciary Sentinel permissions, VibeScript VM execution, Merkle-CRDT DAG, and embedded sub-manifolds.
 */

export const systemSettingsManifold = {
  id: 'manifold-settings-01',
  uuid: 'urn:uuid:5e6f7a8b-9c0d-1e2f-3a4b-5c6d7e8f9a0b',
  title: 'Cybernetic Settings & Sub-Manifold Hub',
  type: 'settings',
  icon: '⚙️',
  graphIri: 'graph:manifold:system_settings_01',
  category: 'qualia:CyberneticGovernance',
  topic: 'topic:VibeScriptVMAndFiduciarySentinel',
  rights: {
    rightsHolder: 'did:qualia:timothy_charles_holborn',
    fiduciaryDutyLevel: 'fiduciary:GateLevel3',
    permissionCircle: 'circle:ProjectAuthors',
    merkleRootHash: '0x5E6F7A8B9C0D1E2F',
    licence: 'https://qualia.io/licences/fiduciary-duty-of-care-v1'
  },
  created: '2026-08-15T00:00:00Z',
  modified: '2026-08-16T15:35:00Z',
  defaultDimension: '2D',
  activeStrata: ['technical', 'financial', 'legal'],
  activeEpistemicModality: 'all',
  activeToolboxes: ['rights', 'code', 'communication', 'office', 'ai'],

  containers: [
    {
      id: 'container-code-01',
      type: 'code',
      strata: 'technical',
      epistemic: 'objective',
      title: 'VibeScript: Fiduciary Sentinel VM',
      x: 80,
      y: 70,
      width: 420,
      height: 310,
      z: 400,
      d: 1.1,
      validFrom: '2026-08-15T06:00:00',
      validTo: '2026-08-15T18:00:00',
      code: `requires [graph, pulse, motor];\n\nfn on_water_flow_shift(reading: f64) -> Result<(), Error> {\n    let site = graph::node("concept:FreshwaterSpring");\n    if reading < 60.0 {\n        pulse::emit("pulse:alert_low_flow", { "reading": reading, "strata": "legal" });\n    }\n    graph::assert(<<[ site hydro:flow reading <graph:sacred> ]>>);\n    Ok(())\n}`,
      gasUsed: 420,
      graphTriples: []
    },
    {
      id: 'container-subcanvas-01',
      type: 'subcanvas',
      strata: 'technical',
      epistemic: 'objective',
      title: 'Recursive Sub-Manifold Component (Research Manifold)',
      x: 530,
      y: 70,
      width: 440,
      height: 310,
      z: 100,
      d: 1.0,
      validFrom: '2026-08-15T00:00:00',
      validTo: '2026-08-16T00:00:00',
      targetCanvasId: 'manifold-research-01',
      graphTriples: []
    },
    {
      id: 'container-portal-01',
      type: 'portal',
      strata: 'financial',
      epistemic: 'intersubjective',
      title: 'Sovereign Commons Economic Wormhole',
      x: 1000,
      y: 70,
      width: 360,
      height: 260,
      z: 300,
      d: 1.5,
      validFrom: '2026-08-15T00:00:00',
      validTo: '2026-08-16T00:00:00',
      targetCanvas: 'canvas://sovereign-commons/reserve_fund_01',
      graphTriples: []
    }
  ],

  connections: [
    {
      id: 'wire-st1',
      from: 'container-code-01',
      to: 'container-subcanvas-01',
      type: 'data-pipe',
      label: 'vibe:orchestratesSubCanvas',
      predicate: 'vibe:orchestratesSubCanvas',
      bridgeType: 'Sub-Manifold Orchestration'
    }
  ]
};

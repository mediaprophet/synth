/**
 * HyperCanvas: System Settings & Cybernetic Governance Canvas Definition
 * Optimized for Fiduciary Sentinel permissions, VibeScript VM execution, Merkle-CRDT DAG, and Embedded Sub-Canvases.
 */

export const systemSettingsCanvas = {
  id: 'canvas-settings-01',
  title: 'Cybernetic Settings & Sub-Canvas Hub',
  type: 'settings',
  icon: '⚙️',
  graphIri: 'graph:canvas:system_settings_01',
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
      title: 'Recursive Sub-Canvas Component (Research Canvas)',
      x: 530,
      y: 70,
      width: 440,
      height: 310,
      z: 100,
      d: 1.0,
      validFrom: '2026-08-15T00:00:00',
      validTo: '2026-08-16T00:00:00',
      targetCanvasId: 'canvas-research-01',
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
      bridgeType: 'Sub-Canvas Orchestration'
    }
  ]
};

/**
 * HyperCanvas: Social Governance & Multi-Agent Communication Canvas
 * Optimized for human-AI collaborative chat graphs, WebRTC live streams, and project field notes.
 */

export const socialGovernanceCanvas = {
  id: 'canvas-social-01',
  title: 'Social Governance & Multi-Agent Collaboration',
  type: 'social',
  icon: '👥',
  graphIri: 'graph:canvas:social_governance_01',
  defaultDimension: '2D',
  activeStrata: ['social', 'legal'],
  activeEpistemicModality: 'all',
  activeToolboxes: ['communication', 'rights', 'office', 'ai'],

  containers: [
    {
      id: 'container-social-01',
      type: 'social',
      strata: 'social',
      epistemic: 'intersubjective',
      title: 'Project Team & AI Sub-Agent Chat Graph',
      x: 100,
      y: 70,
      width: 420,
      height: 320,
      z: 100,
      d: 1.0,
      validFrom: '2026-08-15T00:00:00',
      validTo: '2026-08-16T00:00:00',
      subAgents: [
        { id: 'agent:SentinelPlanner', name: 'Sentinel Planner AI', role: 'Telemetry Coordinator', status: 'Online' },
        { id: 'agent:LegalCounsel', name: 'Fiduciary Rights AI', role: 'Water Rights Guardian', status: 'Online' },
        { id: 'agent:LinguistAI', name: 'Phonology Grounder', role: 'Q42 Grapheme Grounding', status: 'Active' }
      ],
      messages: [
        { id: 'm1', sender: 'Timothy Charles Holborn', isAi: false, time: '14:38', text: 'North Spring looks clearer after last night’s rain; the field notes should capture that.', epistemic: 'subjective', graphNode: 'node:chat_01_holborn' },
        { id: 'm2', sender: 'Sentinel Planner AI', isAi: true, time: '14:39', text: 'Telemetry sensor confirms flow at 142.5 L/m with 221.5 Hz acoustic resonance.', epistemic: 'objective', graphNode: 'node:chat_02_sentinel' },
        { id: 'm3', sender: 'Fiduciary Rights AI', isAi: true, time: '14:40', text: 'Asserted legal quad: <<[ site:NorthSpring hydro:status "Monitored" <graph:rights> ]>>.', epistemic: 'normative', graphNode: 'node:chat_03_rights' }
      ],
      graphTriples: []
    },
    {
      id: 'container-webrtc-01',
      type: 'webrtc',
      strata: 'social',
      epistemic: 'intersubjective',
      title: 'Encrypted WebRTC Audio/Video Stream',
      x: 550,
      y: 70,
      width: 380,
      height: 290,
      z: 100,
      d: 1.1,
      validFrom: '2026-08-15T00:00:00',
      validTo: '2026-08-16T00:00:00',
      peerId: 'did:qualia:peer_catchment_gateway',
      bitrate: '2.4 Mbps (AV1 / Opus Encrypted)',
      graphTriples: []
    },
    {
      id: 'container-doc-01',
      type: 'doc',
      strata: 'social',
      epistemic: 'subjective',
      title: 'Catchment Field Notes & Observation Record',
      x: 960,
      y: 70,
      width: 400,
      height: 280,
      z: 100,
      d: 1.0,
      validFrom: '2026-08-15T00:00:00',
      validTo: '2026-08-16T00:00:00',
      content: `<h2>North Spring Catchment Field Notes</h2>
<p>Author: <strong>Timothy Charles Holborn</strong></p>
<p>Epistemic Status: <span class="modality-badge modality-subjective">🧠 Subjective Qualia</span></p>
<p>The primary monitoring spring (<q-entity data-id="concept:FreshwaterSpring" style="color: #00f2a9; border-bottom: 2px solid #00f2a9;">North Spring</q-entity>) is the project’s reference site for seasonal flow and water quality.</p>`,
      graphTriples: []
    }
  ],

  connections: [
    {
      id: 'wire-s1',
      from: 'container-social-01',
      to: 'container-webrtc-01',
      type: 'social-link',
      label: 'qualia:transmitsMedia',
      predicate: 'qualia:transmitsMedia',
      bridgeType: 'Live Stream Bridge'
    },
    {
      id: 'wire-s2',
      from: 'container-social-01',
      to: 'container-doc-01',
      type: 'subjective',
      label: 'qualia:groundsSubjectiveQualia',
      predicate: 'qualia:groundsSubjectiveQualia',
      bridgeType: 'Qualia Grounding Link'
    }
  ]
};

/**
 * HyperCanvas: "Toolbox of Toolboxes" Registry & Modular Action Palettes
 * Supports 4-Way Docking and Toolboxes for:
 * - Documents (📝)
 * - Epistemic & Qualia Modalities (🎭)
 * - Image & Media (🎨)
 * - Spreadsheets & Tensors (📊)
 * - Mapping & GIS (🗺️)
 * - Social & Communication (📞)
 * - Rights & Fiduciary Permissions (🔒)
 * - Health & Life Sciences (🩺)
 * - Coding & VibeScript (⚡)
 * - AI Mindware (🧠)
 */

import { canvasState } from './state.js';

export class ToolboxRegistry {
  constructor(dockEl, paletteEl, containerManager) {
    this.dock = dockEl;
    this.palette = paletteEl;
    this.containerManager = containerManager;

    this.toolboxes = {
      epistemic: {
        title: 'Epistemic & Qualia Modalities',
        icon: '🎭',
        sections: [
          {
            title: 'Objective vs Subjective Grounding',
            tools: [
              { label: '🔬 Tag Objective Telemetry', action: () => alert('Tagged selected container as qualia:ObjectiveObservation (Empirically Verifiable).') },
              { label: '🧠 Tag Subjective Qualia', action: () => alert('Tagged selected container as qualia:PhenomenologicalQualia (First-Person Narrative).') },
              { label: '🌊 Tag Intersubjective Agreement', action: () => alert('Tagged selected container as qualia:IntersubjectiveConsensus.') },
              { label: '⚖️ Tag Normative Mandate', action: () => alert('Tagged selected container as qualia:NormativeStatute (Fiduciary Law).') }
            ]
          },
          {
            title: 'Epistemic Calibration & Radars',
            tools: [
              { label: '📊 Qualia Affect Radar', action: () => alert('Visualized 5-dimensional qualia affect spectrum.') },
              { label: '🪞 Subjectivity Calibration Dial', action: () => alert('Calibrated subjective confidence interval (0.88 σ).') }
            ]
          }
        ]
      },
      office: {
        title: 'Document & Office Toolbox',
        icon: '📝',
        sections: [
          {
            title: 'Containers & Templates',
            tools: [
              { label: '+ Document Container', action: () => this.addDocContainer() },
              { label: '+ Ontology Mapping Node', action: () => this.addOntologyContainer() },
              { label: '+ Presentation Slide', action: () => this.addPresentationSlide() }
            ]
          },
          {
            title: 'Semantic Context Markup',
            tools: [
              { label: '🏷️ Tag Ontological Entity', action: () => this.tagEntity() },
              { label: 'ᛯ Insert Grapheme Glyph', action: () => this.insertGrapheme() }
            ]
          }
        ]
      },
      image: {
        title: 'Image, Visual & Media Toolbox',
        icon: '🎨',
        sections: [
          {
            title: 'Generative Assets',
            tools: [
              { label: '+ Visual Asset Card', action: () => this.addImageContainer() },
              { label: '✨ Prompt Image Generator', action: () => alert('Prompted multi-modal vision synthesis.') }
            ]
          },
          {
            title: 'Canvas Annotations & Filters',
            tools: [
              { label: '🖌️ Draw Canvas Marker', action: () => alert('Activated freehand visual annotation tool.') },
              { label: '🔍 Spectral Heatmap Filter', action: () => alert('Applied U3 AcousticPlane spectral heatmap.') }
            ]
          }
        ]
      },
      sheet: {
        title: 'Spreadsheets & Tensor Toolbox',
        icon: '📊',
        sections: [
          {
            title: 'Data Containers',
            tools: [
              { label: '+ Columnar Tensor Sheet', action: () => this.addSheetContainer() },
              { label: '📥 Import CSV/CBOR-LD', action: () => alert('Imported CBOR-LD tensor table.') }
            ]
          },
          {
            title: 'P64 Latent Transforms',
            tools: [
              { label: '🧮 Compute Resonance', action: () => alert('Computed hydrological acoustic resonance.') },
              { label: '📐 64-Aligned Embedding', action: () => alert('Aligned continuous EnCodec latents.') }
            ]
          }
        ]
      },
      spatial: {
        title: 'Mapping, GIS & 3D Design',
        icon: '🗺️',
        sections: [
          {
            title: 'Spatial & GIS Containers',
            tools: [
              { label: '+ Geospatial Case Map', action: () => this.addMapContainer() },
              { label: '+ 3D Mesh Container', action: () => this.add3DContainer() },
              { label: '+ Wormhole Portal', action: () => this.addPortalContainer() }
            ]
          },
          {
            title: 'Multi-Agent Investigation',
            tools: [
              { label: '📍 Pin Case Incident Node', action: () => alert('Pinned new incident event on map timeline.') },
              { label: '🚁 Add Agent Spatiotemporal Track', action: () => alert('Added new Agent trajectory path.') }
            ]
          }
        ]
      },
      communication: {
        title: 'Social, WebRTC & Webview',
        icon: '📞',
        sections: [
          {
            title: 'Collaborative Containers',
            tools: [
              { label: '+ Social AI Chat Graph', action: () => this.addSocialContainer() },
              { label: '+ WebRTC Live Video', action: () => this.addWebRtcContainer() },
              { label: '+ Webview Browser Frame', action: () => this.addWebviewContainer() }
            ]
          },
          {
            title: 'Multi-Agent Sub-Teams',
            tools: [
              { label: '🤖 Spawn AI Sub-Agent', action: () => alert('Spawned Sub-Agent: Fiduciary Compliance AI to chat graph.') },
              { label: '🌐 Mesh Peer Connect', action: () => alert('Connected to peer did:qualia:peer_catchment_gateway.') }
            ]
          }
        ]
      },
      rights: {
        title: 'Rights, Fiduciary & Permissions',
        icon: '🔒',
        sections: [
          {
            title: 'Permission Groups & Scopes',
            tools: [
              { label: '👥 Project Authors Group', action: () => alert('Assigned node to Project Authors Circle (Full Sovereign Access).') },
              { label: '🛡️ Fiduciary Duty Scope', action: () => alert('Attached fiduciary duty-of-care gate Level 3.') },
              { label: '🏷️ Tag #catchment-compliance', action: () => alert('Tagged container with #catchment-compliance.') }
            ]
          },
          {
            title: 'Cryptographic Credentials',
            tools: [
              { label: '✍️ Sign with DID Key', action: () => alert('Signed container with did:qualia:timothy_charles_holborn.') },
              { label: '📜 Export Fiduciary Audit Log', action: () => alert('Exported Merkle DAG tamper-proof audit trail.') }
            ]
          }
        ]
      },
      health: {
        title: 'Health, Clinical & Anatomy',
        icon: '🩺',
        sections: [
          {
            title: 'Clinical Containers',
            tools: [
              { label: '+ Health Telemetry Node', action: () => this.addHealthContainer() },
              { label: '🧪 Pathology Lab Profile', action: () => alert('Loaded water & acoustic pathology profile.') },
              { label: '🫀 10D Anatomy Resonator', action: () => alert('Loaded 10D vocal tract anatomical mesh.') }
            ]
          }
        ]
      },
      code: {
        title: 'Coding & VibeScript Toolbox',
        icon: '⚡',
        sections: [
          {
            title: 'DSL Containers & Cells',
            tools: [
              { label: '+ VibeScript Container', action: () => this.addCodeContainer() },
              { label: '+ Reactive Calc Cell', action: () => this.addCalcCell() }
            ]
          },
          {
            title: 'Fiduciary & Graph Tools',
            tools: [
              { label: '🔒 Declare Capabilities', action: () => alert('Inserted `requires [graph, pulse, motor];`') },
              { label: '<<[ Nquin Assertion ]>>', action: () => alert('Asserted 48-byte Nquin to local Merkle DAG.') }
            ]
          }
        ]
      },
      finance: {
        title: 'Financial Economics & Ledgers',
        icon: '💰',
        sections: [
          {
            title: 'Double-Entry Quad Ledgers',
            tools: [
              { label: '+ Quad Transaction', action: () => alert('➕ Inserted new double-entry accounting quad.') },
              { label: '⚖️ Reconcile Accounts', action: () => alert('⚖️ Account reconciliation verified (0 diff).') },
              { label: '🛡️ Audit Digest', action: () => alert('🛡️ Generated SHA-256 Merkle audit root.') }
            ]
          },
          {
            title: 'Milestone Escrows & Gas',
            tools: [
              { label: '🔒 Lock Milestone Escrow', action: () => alert('🔒 Locked $15,000 in Milestone Escrow Pool.') },
              { label: '✍️ Multi-Sig Release', action: () => alert('✍️ Multi-Sig Escrow release authorized.') },
              { label: '⚡ Settle Compute Gas', action: () => alert('⚡ Settled 340 VibeScript gas units.') }
            ]
          }
        ]
      },
      ai: {
        title: 'AI Mindware & Triad Toolboxes',
        icon: '🧠',
        sections: [
          {
            title: 'Mindware Co-Authors & Agents',
            tools: [
              { label: '🤖 LLM Semantic Co-Author', isAi: true, action: () => this.runAiCoAuthor() },
              { label: '🎯 Structured Graph Extractor', isAi: true, action: () => this.runGraphExtractor() },
              { label: '🛡️ Sentinel Goal Planner', isAi: true, action: () => this.runSentinelPlanner() }
            ]
          },
          {
            title: 'Auditory Triad & GIS Intelligence',
            tools: [
              { label: '🎙️ Speech Synthesizer (q42↔d10)', isAi: true, action: () => this.runSpeechTriad() },
              { label: '🛰️ Multi-Agent Trajectory Predictor', isAi: true, action: () => alert('AI Agent: Simulated spatiotemporal encounter trajectory.') }
            ]
          }
        ]
      }
    };

    this.renderDock();

    canvasState.subscribe('dockPositionChanged', (pos) => {
      this.updateDockClasses(pos);
    });

    canvasState.subscribe('canvasChanged', () => {
      this.renderDock();
    });
  }

  updateDockClasses(pos) {
    this.dock.className = `toolbox-dock dock-${pos}`;
    this.palette.className = `floating-toolbox-palette dock-${pos} ${canvasState.state.activeToolbox ? 'open' : ''}`;
  }

  renderDock() {
    const pos = canvasState.state.dockPosition || 'left';
    this.updateDockClasses(pos);

    // Check manifold active toolboxes
    const activeList = canvasState.state.activeToolboxes || [];

    const allKeys = [
      { id: 'epistemic', title: 'Epistemic & Qualia Modalities', icon: '🎭', color: 'var(--modality-subjective)' },
      { id: 'office', title: 'Document & Office Toolbox', icon: '📝', color: 'var(--color-doc)' },
      { id: 'image', title: 'Image & Visual Media', icon: '🎨', color: 'var(--color-3d)' },
      { id: 'sheet', title: 'Spreadsheets & Tensors', icon: '📊', color: 'var(--color-sheet)' },
      { id: 'finance', title: 'Financial Economics & Ledgers', icon: '💰', color: 'var(--strata-fin)' },
      { id: 'spatial', title: 'Mapping, GIS & 3D Design', icon: '🗺️', color: 'var(--color-map)' },
      { id: 'communication', title: 'Social, WebRTC & Webview', icon: '📞', color: 'var(--color-social)' },
      { id: 'rights', title: 'Rights & Fiduciary Permissions', icon: '🔒', color: 'var(--color-rights)' },
      { id: 'health', title: 'Health, Clinical & Anatomy', icon: '🩺', color: 'var(--color-health)' },
      { id: 'code', title: 'Coding & VibeScript Toolbox', icon: '⚡', color: 'var(--accent-emerald)' },
      { id: 'ai', title: 'AI Mindware & Triad Toolboxes', icon: '🧠', color: 'var(--accent-purple)' }
    ];

    this.dock.innerHTML = `
      <div class="dock-header-controls" title="Change Toolbox Dock Position">
        <button class="dock-pos-btn ${pos === 'left' ? 'active' : ''}" data-pos="left">◀</button>
        <button class="dock-pos-btn ${pos === 'top' ? 'active' : ''}" data-pos="top">▲</button>
        <button class="dock-pos-btn ${pos === 'bottom' ? 'active' : ''}" data-pos="bottom">▼</button>
        <button class="dock-pos-btn ${pos === 'right' ? 'active' : ''}" data-pos="right">▶</button>
      </div>

      ${allKeys.map(tb => {
        const isPrioritized = activeList.length === 0 || activeList.includes(tb.id) || activeList.includes('all');
        const glowStyle = isPrioritized ? `border-left: 2px solid ${tb.color}; box-shadow: inset 0 0 10px rgba(0,210,255,0.08);` : 'opacity: 0.6;';
        return `
          <button class="toolbox-dock-btn ${isPrioritized ? 'manifold-active' : ''}" data-id="${tb.id}" data-title="${tb.title}" style="${glowStyle}">
            ${tb.icon}
          </button>
        `;
      }).join('')}
    `;

    this.dock.querySelectorAll('.dock-pos-btn').forEach(btn => {
      btn.addEventListener('click', (e) => {
        e.stopPropagation();
        const p = btn.getAttribute('data-pos');
        canvasState.setDockPosition(p);
      });
    });

    this.dock.querySelectorAll('.toolbox-dock-btn').forEach(btn => {
      btn.addEventListener('click', () => {
        const id = btn.getAttribute('data-id');
        canvasState.setActiveToolbox(id);
      });
    });

    canvasState.subscribe('toolboxChanged', (activeId) => {
      this.dock.querySelectorAll('.toolbox-dock-btn').forEach(btn => {
        btn.classList.toggle('active', btn.getAttribute('data-id') === activeId);
      });
      this.renderPalette(activeId);
    });
  }

  renderPalette(toolboxId) {
    const pos = canvasState.state.dockPosition || 'left';
    this.palette.className = `floating-toolbox-palette dock-${pos} ${toolboxId ? 'open' : ''}`;

    if (!toolboxId || !this.toolboxes[toolboxId]) {
      this.palette.classList.remove('open');
      return;
    }

    const t = this.toolboxes[toolboxId];
    this.palette.innerHTML = `
      <div class="palette-header">
        <span>${t.icon} ${t.title}</span>
        <button class="container-action-btn btn-close-palette">&times;</button>
      </div>
      <div class="palette-body">
        ${t.sections.map(sec => `
          <div class="tool-section">
            <span class="tool-section-title">${sec.title}</span>
            <div class="tool-grid">
              ${sec.tools.map((tool, idx) => `
                <button class="tool-btn ${tool.isAi ? 'highlight-ai' : ''}" data-sec="${sec.title}" data-idx="${idx}">
                  ${tool.label}
                </button>
              `).join('')}
            </div>
          </div>
        `).join('')}
      </div>
    `;

    this.palette.classList.add('open');

    const closeBtn = this.palette.querySelector('.btn-close-palette');
    if (closeBtn) {
      closeBtn.addEventListener('click', () => canvasState.setActiveToolbox(null));
    }

    this.palette.querySelectorAll('.tool-btn').forEach(btn => {
      btn.addEventListener('click', () => {
        const secTitle = btn.getAttribute('data-sec');
        const idx = parseInt(btn.getAttribute('data-idx'));
        const sec = t.sections.find(s => s.title === secTitle);
        if (sec && sec.tools[idx]) {
          sec.tools[idx].action();
        }
      });
    });
  }

  addSocialContainer() {
    const id = `container-social-${Date.now()}`;
    canvasState.addContainer({
      id,
      type: 'social',
      strata: 'social',
      epistemic: 'intersubjective',
      title: 'New Collaborative Chat Graph',
      x: 320,
      y: 180,
      width: 400,
      height: 300,
      z: 100,
      validFrom: '2026-08-15T00:00:00',
      validTo: '2026-08-16T00:00:00',
      subAgents: [
        { id: 'agent:NewAI', name: 'Facilitator AI', role: 'Synthesis Coordinator', status: 'Online' }
      ],
      messages: [
        { id: 'm0', sender: 'Timothy Charles Holborn', isAi: false, time: '12:00', text: 'Opened collaborative session.', epistemic: 'subjective', graphNode: 'node:init' }
      ],
      graphTriples: []
    });
    this.containerManager.renderAll();
  }

  addWebRtcContainer() {
    const id = `container-webrtc-${Date.now()}`;
    canvasState.addContainer({
      id,
      type: 'webrtc',
      strata: 'social',
      epistemic: 'intersubjective',
      title: 'WebRTC Peer Stream',
      x: 360,
      y: 220,
      width: 360,
      height: 280,
      z: 100,
      validFrom: '2026-08-15T00:00:00',
      validTo: '2026-08-16T00:00:00',
      peerId: 'did:qualia:peer_live_02',
      bitrate: '3.1 Mbps (AV1 Stream)',
      graphTriples: []
    });
    this.containerManager.renderAll();
  }

  addHealthContainer() {
    const id = `container-health-${Date.now()}`;
    canvasState.addContainer({
      id,
      type: 'health',
      strata: 'environmental',
      epistemic: 'objective',
      title: 'Clinical & Pathology Telemetry',
      x: 400,
      y: 200,
      width: 420,
      height: 300,
      z: 0,
      validFrom: '2026-08-15T00:00:00',
      validTo: '2026-08-16T00:00:00',
      activeTab: 'pathology',
      pathology: [
        { biomarker: 'Aquifer Purity Index', value: '99.2%', ref: '> 95.0%', status: 'Optimal' }
      ],
      imaging: { modality: 'Spectral Tomography', resolution: '256x256', slice: 16 },
      anatomy: { system: 'Pharyngeal Resonator', meshId: 'mesh:vocal_tract_01' },
      graphTriples: []
    });
    this.containerManager.renderAll();
  }

  addWebviewContainer() {
    const id = `container-webview-${Date.now()}`;
    canvasState.addContainer({
      id,
      type: 'webview',
      strata: 'technical',
      epistemic: 'objective',
      title: 'Web Knowledge Frame',
      x: 420,
      y: 250,
      width: 400,
      height: 280,
      z: 400,
      validFrom: '2026-08-15T00:00:00',
      validTo: '2026-08-16T00:00:00',
      url: 'https://w3c.org/tr/rdf12-concepts',
      titleSnippet: 'RDF 1.2 Concepts and Abstract Syntax',
      graphTriples: []
    });
    this.containerManager.renderAll();
  }

  addImageContainer() {
    const id = `container-media-${Date.now()}`;
    canvasState.addContainer({
      id,
      type: 'media',
      strata: 'social',
      epistemic: 'subjective',
      title: 'Visual Asset & Image Card',
      x: 380,
      y: 220,
      width: 340,
      height: 260,
      z: 100,
      validFrom: '2026-08-15T00:00:00',
      validTo: '2026-08-16T00:00:00',
      content: '🖼️ Visual Grapheme Geometry (.d10 vector mesh)'
    });
    this.containerManager.renderAll();
  }

  addMapContainer() {
    const id = `container-map-${Date.now()}`;
    canvasState.addContainer({
      id,
      type: 'map',
      strata: 'environmental',
      epistemic: 'objective',
      title: 'Geospatial Case Map',
      x: 200 + Math.random() * 60,
      y: 120 + Math.random() * 60,
      width: 420,
      height: 290,
      z: 0,
      validFrom: '2026-08-15T00:00:00',
      validTo: '2026-08-16T00:00:00',
      geoRegion: 'North Spring Catchment Spatiotemporal Grid',
      agents: [
        {
          id: 'agent:NewSentinel',
          name: 'Patrol Agent #02',
          color: '#00f2a9',
          path: [{ t: 0, x: 80, y: 180 }, { t: 1, x: 340, y: 100 }]
        }
      ],
      incidents: []
    });
    this.containerManager.renderAll();
  }

  addDocContainer() {
    const id = `container-doc-${Date.now()}`;
    canvasState.addContainer({
      id,
      type: 'doc',
      strata: 'social',
      epistemic: 'subjective',
      title: 'New Social/Cultural Doc',
      x: 300 + Math.random() * 80,
      y: 200 + Math.random() * 80,
      width: 360,
      height: 240,
      z: 100,
      validFrom: '2026-08-15T00:00:00',
      validTo: '2026-08-16T00:00:00',
      content: '<h2>Untitled Social/Cultural Record</h2><p>Document social relations, governance, and community testimonies. Select any text to ground in RDF.</p>'
    });
    this.containerManager.renderAll();
  }

  addOntologyContainer() {
    const id = `container-ontology-${Date.now()}`;
    canvasState.addContainer({
      id,
      type: 'ontology',
      strata: 'legal',
      epistemic: 'normative',
      title: 'Custom Legal/Ontology Mapping',
      x: 450 + Math.random() * 60,
      y: 150 + Math.random() * 60,
      width: 380,
      height: 280,
      z: 200,
      d: 1.2,
      validFrom: '2026-08-15T00:00:00',
      validTo: '2026-08-16T00:00:00',
      ontologyIri: 'https://qualia.io/ns/custom-alignment#',
      classes: ['EntityMap', 'BridgeConcept'],
      mappings: [
        { source: 'local:ConceptA', relation: 'owl:equivalentClass', target: 'remote:ConceptB' }
      ]
    });
    this.containerManager.renderAll();
  }

  addCodeContainer() {
    const id = `container-code-${Date.now()}`;
    canvasState.addContainer({
      id,
      type: 'code',
      strata: 'technical',
      epistemic: 'objective',
      title: 'VibeScript Agent Node',
      x: 400 + Math.random() * 80,
      y: 250 + Math.random() * 80,
      width: 380,
      height: 260,
      z: 400,
      validFrom: '2026-08-15T00:00:00',
      validTo: '2026-08-16T00:00:00',
      code: `requires [graph, pulse];\n\nfn evaluate_node() -> Result<(), Error> {\n    pulse::emit("pulse:node_ready", {});\n    Ok(())\n}`,
      gasUsed: 350
    });
    this.containerManager.renderAll();
  }

  addSheetContainer() {
    const id = `container-sheet-${Date.now()}`;
    canvasState.addContainer({
      id,
      type: 'sheet',
      strata: 'financial',
      epistemic: 'objective',
      title: 'Financial Ledger & Tensors',
      x: 350,
      y: 350,
      width: 380,
      height: 220,
      z: 300,
      validFrom: '2026-08-15T00:00:00',
      validTo: '2026-08-16T00:00:00',
      rows: [
        { site: 'Reserve Allocation', flow: '100k SOL', resonance: 'A+', acousticLatent: '[0.1, 0.2]' }
      ]
    });
    this.containerManager.renderAll();
  }

  add3DContainer() {
    const id = `container-3d-${Date.now()}`;
    canvasState.addContainer({
      id,
      type: '3d',
      strata: 'social',
      epistemic: 'objective',
      title: '3D Spatial Geometry Node',
      x: 450,
      y: 280,
      width: 340,
      height: 240,
      z: 100,
      validFrom: '2026-08-15T00:00:00',
      validTo: '2026-08-16T00:00:00',
      meshAsset: 'asset://mesh/spatial_node.d10'
    });
    this.containerManager.renderAll();
  }

  addPortalContainer() {
    const id = `container-portal-${Date.now()}`;
    canvasState.addContainer({
      id,
      type: 'portal',
      strata: 'financial',
      epistemic: 'intersubjective',
      title: 'Sub-Canvas Wormhole Portal',
      x: 500,
      y: 300,
      width: 320,
      height: 240,
      z: 300,
      validFrom: '2026-08-15T00:00:00',
      validTo: '2026-08-16T00:00:00',
      targetCanvas: 'canvas://sub-realm/deep_space_03'
    });
    this.containerManager.renderAll();
  }

  runAiCoAuthor() {
    alert('AI Co-Author: Synthesized 3 new semantic paragraphs grounded in the catchment ontology.');
  }

  runGraphExtractor() {
    alert('Structured Graph Extractor: Extracted 8 RDF 1.2 Nquins and asserted to Merkle DAG.');
  }

  runSentinelPlanner() {
    alert('Sentinel Goal Planner: Verified permission scopes and executed multi-step mindware plan.');
  }

  runSpeechTriad() {
    alert('Articulatory Triad: Synthesized phone sequence across q42 -> p64 -> d10 virtual vocal tract.');
  }
}

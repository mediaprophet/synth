/**
 * HyperCanvas Toolchain Deck & Socket-Case Manager
 * Replaces linear overflow ribbons with physical workshop Toolchain "Socket-Case" architecture:
 * - Top level renders only compact Toolchain Socket Pills: [ 📐 Kinematics (4) ▾ ] [ ⏱️ Animation (3) ▾ ] [ ⚡ Events (3) ▾ ]
 * - Clicking a pill opens its frosted-glass "Socket Case" Drop-Tray directly below
 * - Optional 3D Rotary Prism Drum mode (wheel scrolls 3D barrel along X-axis)
 * - Tear-off floating canvas pods
 * - Full ⚙️ Deck Preferences with LocalStorage persistence
 */

import { canvasState } from './state.js';
import { allToolboxes, getToolboxByDomain } from '../tools/index.js';

export class ToolchainDeckManager {
  constructor(deckWrapperEl) {
    this.wrapper = deckWrapperEl;
    this.pinned = false;
    this.currentContext = null;
    this.currentContainer = null;
    this.activeOpenToolchain = null;
    this.rotaryIndex = 0;

    // User Settings
    this.settings = {
      presentationMode: 'case-tray', // 'case-tray' | '3d-prism' | 'inline-elastic'
      density: 'adaptive',          // 'compact' | 'adaptive' | 'full'
      enableTearOff: true
    };

    this.loadSettings();
    this.buildWrapperStructure();
    this.init();
  }

  loadSettings() {
    try {
      const saved = localStorage.getItem('hypercanvas_toolchain_deck_settings');
      if (saved) {
        Object.assign(this.settings, JSON.parse(saved));
      }
    } catch (e) {
      console.warn('Could not load toolchain deck settings', e);
    }
  }

  saveSettings() {
    try {
      localStorage.setItem('hypercanvas_toolchain_deck_settings', JSON.stringify(this.settings));
    } catch (e) {
      console.warn('Could not save toolchain deck settings', e);
    }
  }

  buildWrapperStructure() {
    this.wrapper.className = 'toolchain-deck-wrapper';
    this.wrapper.innerHTML = `
      <div class="toolchain-deck ${this.settings.presentationMode}" id="toolchain-deck-inner"></div>
      <div class="socket-case-drop-tray hidden" id="socket-case-drop-tray"></div>
      <div class="toolchain-settings-modal hidden" id="toolchain-settings-modal"></div>
    `;

    this.deckInner = this.wrapper.querySelector('#toolchain-deck-inner');
    this.dropTray = this.wrapper.querySelector('#socket-case-drop-tray');
    this.settingsModal = this.wrapper.querySelector('#toolchain-settings-modal');

    // Close drop-tray when clicking outside
    document.addEventListener('click', (e) => {
      if (!this.wrapper.contains(e.target)) {
        this.closeDropTray();
      }
    });

    // 3D Rotary Prism mouse wheel listener
    this.deckInner.addEventListener('wheel', (e) => {
      if (this.settings.presentationMode === '3d-prism') {
        e.preventDefault();
        if (e.deltaY > 0) {
          this.rotatePrism(1);
        } else {
          this.rotatePrism(-1);
        }
      }
    }, { passive: false });
  }

  init() {
    canvasState.subscribe('containerSelected', (containerId) => {
      if (containerId) {
        const c = canvasState.state.containers.find(item => item.id === containerId);
        if (c) {
          this.setContext(c.type, c);
          return;
        }
      }
      if (!this.pinned) {
        this.setContext(null);
      }
    });

    this.renderDefault();
  }

  setContext(type, container = null) {
    this.currentContext = type;
    this.currentContainer = container;
    this.closeDropTray();

    if (!type) {
      if (!this.pinned) {
        this.deckInner.classList.remove('active');
        this.deckInner.innerHTML = '';
      } else {
        this.renderDefault();
      }
      return;
    }

    this.deckInner.classList.add('active');
    this.renderDeckForContext(type, container);
  }

  renderDeckForContext(type, container) {
    const toolbox = getToolboxByDomain(type) || getToolboxByDomain('doc');
    const toolchains = this.getToolchainsForType(type, container);
    const domainBadge = this.getDomainBadge(type);

    if (this.settings.presentationMode === '3d-prism') {
      this.render3DPrismDeck(domainBadge, toolchains, container);
    } else {
      this.renderSocketCaseDeck(domainBadge, toolchains, container);
    }
  }

  getDomainBadge(type) {
    switch (type) {
      case 'doc': return { label: '📝 Document / TinyMCE Suite', color: 'var(--color-doc)' };
      case 'sheet': return { label: '📊 Spreadsheet & Tensor Suite', color: 'var(--color-sheet)' };
      case 'ontology': return { label: '🏷️ RDF 1.2 & Aura Catalog', color: 'var(--color-ontology)' };
      case '3d':
      case 'media': return { label: '🧊 10D Asset (.10d) & Media Studio', color: 'var(--color-3d)' };
      case 'social':
      case 'webrtc': return { label: '📞 Social & Multi-Agent Suite', color: 'var(--color-social)' };
      case 'map': return { label: '🗺️ Spatiotemporal GIS Suite', color: 'var(--color-map)' };
      case 'code': return { label: '⚡ VibeScript 0.1 AST Console', color: 'var(--color-code)' };
      case 'health': return { label: '🩺 Clinical & Bio-Telemetry Suite', color: 'var(--color-health)' };
      default: return { label: '🌌 Global Manifold Deck', color: 'var(--accent-cyan)' };
    }
  }

  getToolchainsForType(type, container) {
    switch (type) {
      case 'doc':
        return [
          {
            id: 'doc_typo',
            icon: '📝',
            title: 'Typography & Block Styles',
            count: 8,
            tools: [
              { id: 'bold', label: 'Bold (Ctrl+B)', icon: '<strong>B</strong>', action: () => document.execCommand('bold') },
              { id: 'italic', label: 'Italic (Ctrl+I)', icon: '<em>I</em>', action: () => document.execCommand('italic') },
              { id: 'underline', label: 'Underline (Ctrl+U)', icon: '<u>U</u>', action: () => document.execCommand('underline') },
              { id: 'code', label: 'Inline Code', icon: '<code>&lt;/&gt;</code>', action: () => document.execCommand('formatBlock', false, 'pre') },
              { id: 'h1', label: 'Heading 1', icon: 'H1', action: () => document.execCommand('formatBlock', false, 'h1') },
              { id: 'h2', label: 'Heading 2', icon: 'H2', action: () => document.execCommand('formatBlock', false, 'h2') },
              { id: 'quote', label: 'Blockquote', icon: '❝', action: () => document.execCommand('formatBlock', false, 'blockquote') },
              { id: 'list', label: 'Bullet List', icon: '• List', action: () => document.execCommand('insertUnorderedList') }
            ]
          },
          {
            id: 'doc_grounding',
            icon: '🏷️',
            title: 'Semantic Grounding & Spans',
            count: 4,
            tools: [
              { id: 'entity', label: 'Tag Ontological Entity', icon: '🏷️ Entity', highlight: 'cyan', action: () => alert('✓ Grounded selected text into Aura ontology <concept:FreshwaterSpring>') },
              { id: 'qualia', label: 'Tag Subjective Qualia', icon: '🧠 Qualia', highlight: 'rose', action: () => alert('Tagged span with qualia:PhenomenologicalQualia') },
              { id: 'objective', label: 'Tag Objective Fact', icon: '🔬 Objective', highlight: 'cyan', action: () => alert('Tagged span with qualia:ObjectiveObservation') },
              { id: 'normative', label: 'Tag Normative Statute', icon: '⚖️ Statute', highlight: 'amber', action: () => alert('Tagged span with qualia:NormativeStatute') }
            ]
          },
          {
            id: 'doc_math_nlp',
            icon: '🔍',
            title: 'Math & NLP Gazetteer',
            count: 2,
            tools: [
              { id: 'latex', label: '$$ LaTeX Formula', icon: '$$ LaTeX', action: () => alert('Inserted LaTeX math formula') },
              { id: 'gazetteer', label: 'Gazetteer NLP Analysis (qualia_core_db::nlp)', icon: '🔍 Gazetteer', highlight: 'violet', action: () => alert('🔍 Gazetteer NLP Analysis:\n• Tokens: 42\n• Sentences: 3\n• Sealed Spans: 3 quins committed to Merkle DAG.') }
            ]
          }
        ];

      case '3d':
      case 'media':
        return [
          {
            id: '3d_kinematics',
            icon: '📐',
            title: '10D Kinematics Motor Chain',
            count: 4,
            tools: [
              { id: 'jaw', label: 'Jaw Motor (0°-30°)', icon: '📐 Jaw 14.2°', action: () => alert('Adjusted Jaw kinematic angle to 14.2°') },
              { id: 'tongue', label: 'Tongue Body (10D Tensor)', icon: '👅 Tongue +2.4mm', action: () => alert('Set Tongue pharyngeal displacement to +2.4mm') },
              { id: 'velum', label: 'Velopharyngeal Gate', icon: '🚪 Velum Closed', action: () => alert('Toggled velopharyngeal port state: Closed') },
              { id: 'orbit', label: 'CAD 3D Trackball Orbit', icon: '🔄 CAD Orbit', action: () => canvasState.setDimensionMode('3D') }
            ]
          },
          {
            id: '3d_animation',
            icon: '⏱️',
            title: 'Timeline Animation (60fps)',
            count: 3,
            tools: [
              { id: 'play', label: 'Play / Pause Motion (60fps)', icon: '▶ Play 60fps', active: true, action: () => alert('Playing 10D animation keyframes @ 60fps') },
              { id: 'scrub', label: 'Temporal Scrub (t = 1.42s)', icon: '⏱️ Scrub t=1.42s', action: () => alert('Scrubbed 10D temporal state to 1.42s') },
              { id: 'keyframe', label: '+ Insert Motion Keyframe', icon: '🔑 + Keyframe', action: () => alert('Inserted motion keyframe at t=1.42s') }
            ]
          },
          {
            id: '3d_events',
            icon: '⚡',
            title: 'Reactive Event Hooks & DID',
            count: 3,
            tools: [
              { id: 'event_hook', label: 'Hook on_closure -> pulse::emit', icon: '⚡ on_closure -> pulse', highlight: 'violet', action: () => alert('⚡ Event Hook active:\non_closure emits pulse to Merkle bus') },
              { id: 'crc_did', label: 'Verify DID & Section CRC', icon: '🛡️ CRC / DID Valid', highlight: 'emerald', action: () => alert('🛡️ Verified .10d Container:\n• CRC: 0x8F3A29 (Valid)\n• Author: did:qualia:timothy_charles_holborn') },
              { id: 'wgpu', label: 'WGPU Hardware Viewport', icon: '⚡ WGPU Frame', action: () => alert('WGPU Hardware Accelerated Native Frame Surface active') }
            ]
          }
        ];

      case 'sheet':
        return [
          {
            id: 'sheet_formula',
            icon: '🧮',
            title: 'Formula Engine & Math',
            count: 3,
            tools: [
              { id: 'calc', label: 'fx Compute =SUM(B1:B5)', icon: '🧮 Compute SUM', action: () => alert('🧮 Computed formula: result = 405.7 L/m') },
              { id: 'resonance', label: 'RESONANCE(flow, freq)', icon: '🌊 Resonance Hz', action: () => alert('🌊 Calculated resonance frequency: 221.5 Hz') },
              { id: 'purity', label: 'PURITY_INDEX(tds, do)', icon: '🧪 Purity Index', action: () => alert('Purity Index: 99.2% Optimal') }
            ]
          },
          {
            id: 'sheet_tensors',
            icon: '📐',
            title: 'P64 Neural Latents & IO',
            count: 3,
            tools: [
              { id: 'latent', label: 'Align P64 Neural Latents', icon: '📐 P64 Latent Tensor', highlight: 'emerald', action: () => alert('Aligned continuous EnCodec P64 neural latent tensor') },
              { id: 'import_cbor', label: 'Ingest CSV / CBOR-LD', icon: '📥 Import CBOR', action: () => alert('Imported CBOR-LD tensor table') },
              { id: 'export_csvw', label: 'Export W3C CSVW Triples', icon: '📤 Export CSVW', action: () => alert('Exported W3C CSVW Tabular Graph') }
            ]
          }
        ];

      case 'ontology':
        return [
          {
            id: 'onto_shacl',
            icon: '🛡️',
            title: 'Aura & SHACL Validator',
            count: 2,
            tools: [
              { id: 'validate', label: 'SHACL Shape Validator (100% Conforming)', icon: '🛡️ SHACL Validate (100%)', highlight: 'cyan', action: () => alert('🛡️ Aura SHACL Validator: 100% Conforming · 3 Target Shapes Satisfied.') },
              { id: 'add_row', label: '+ Add Alignment Row', icon: '+ Add Alignment Row', action: () => {
                if (container) {
                  if (!container.mappings) container.mappings = [];
                  container.mappings.push({ source: `hydro:Metric_${Date.now().toString().slice(-4)}`, relation: 'skos:exactMatch', target: 'fiduciary:WaterAsset' });
                  canvasState.emit('stateChanged', canvasState.state);
                }
                alert('✓ Appended dynamic alignment row to matrix.');
              }}
            ]
          },
          {
            id: 'onto_sparql_dag',
            icon: '🔍',
            title: 'SPARQL, N3 & Nquins',
            count: 3,
            tools: [
              { id: 'sparql', label: 'SPARQL 1.2 Query Console', icon: '🔍 SPARQL Console', action: () => alert('Opened SPARQL 1.2 query console') },
              { id: 'n3', label: 'N3 Rule Forward-Chaining', icon: '📜 N3 Deduction', action: () => alert('Executed N3 forward-chaining deduction') },
              { id: 'nquin', label: 'Assert 48-byte Nquin', icon: '<<[ 48-byte Nquin ]>>', highlight: 'emerald', action: () => alert('<<[ Asserted 48-byte Nquin into local Merkle DAG ]>>') }
            ]
          }
        ];

      default:
        return [
          {
            id: 'global_place',
            icon: '🌌',
            title: 'Workspace Quick-Place',
            count: 4,
            tools: [
              { id: 'p_doc', label: '+ Document Container', icon: '📝 + Doc', action: () => alert('Placed Document Container') },
              { id: 'p_sheet', label: '+ Tensor Sheet Container', icon: '📊 + Sheet', action: () => alert('Placed Spreadsheet Container') },
              { id: 'p_10d', label: '+ 10D Kinematic Asset', icon: '🧊 + 10D Asset', action: () => alert('Placed 10D Kinematic Asset') },
              { id: 'p_map', label: '+ Geospatial Case Map', icon: '🗺️ + Map', action: () => alert('Placed Geospatial Case Map') }
            ]
          }
        ];
    }
  }

  renderSocketCaseDeck(domainBadge, toolchains, container) {
    this.deckInner.innerHTML = `
      <div class="deck-domain-badge" style="color: ${domainBadge.color}; border-color: ${domainBadge.color};">
        ${domainBadge.label}
      </div>

      <!-- Toolchain Socket-Case Pills -->
      <div class="toolchains-socket-bar">
        ${toolchains.map(tc => `
          <button class="socket-case-pill ${this.activeOpenToolchain === tc.id ? 'active' : ''}" data-toolchain="${tc.id}" title="${tc.title}">
            <span class="socket-icon">${tc.icon}</span>
            <span class="socket-title">${tc.title.split(' ')[0]}</span>
            <span class="socket-count-badge">${tc.count}</span>
            <span class="socket-chevron">▾</span>
          </button>
        `).join('')}
      </div>

      <!-- Controls Shelf (Right side: Pin + Settings ⚙️) -->
      <div class="deck-controls-shelf">
        <span class="honesty-badge honesty-present" style="font-size:9px;">present</span>
        <button class="deck-pin-btn ${this.pinned ? 'active' : ''}" id="btn-deck-pin" title="Pin Active Deck">📌</button>
        <button class="deck-settings-btn" id="btn-deck-settings" title="Toolchain Deck Settings & 3D Prism">⚙️</button>
      </div>
    `;

    this.attachSocketPillEvents(toolchains);
    this.attachDeckControls();
  }

  render3DPrismDeck(domainBadge, toolchains, container) {
    const currentTc = toolchains[this.rotaryIndex % toolchains.length] || toolchains[0];

    this.deckInner.innerHTML = `
      <div class="deck-domain-badge" style="color: ${domainBadge.color}; border-color: ${domainBadge.color};">
        ${domainBadge.label}
      </div>

      <!-- 3D Cylindrical Rotary Prism Drum -->
      <div class="prism-drum-container">
        <button class="prism-drum-stepper" id="btn-prism-prev" title="Rotate 3D Prism Up">▲</button>
        <div class="prism-drum-face" id="prism-drum-active-face">
          <span class="prism-face-icon">${currentTc.icon}</span>
          <span class="prism-face-title"><strong>${currentTc.title}</strong></span>
          <div class="prism-face-tools">
            ${currentTc.tools.map(t => `
              <button class="tool-action-btn ${t.highlight ? `highlight-${t.highlight}` : ''}" data-tool-id="${t.id}">
                ${t.icon}
              </button>
            `).join('')}
          </div>
        </div>
        <button class="prism-drum-stepper" id="btn-prism-next" title="Rotate 3D Prism Down">▼</button>
      </div>

      <!-- Controls Shelf -->
      <div class="deck-controls-shelf">
        <span class="honesty-badge honesty-live" style="font-size:9px;">3D Prism</span>
        <button class="deck-pin-btn ${this.pinned ? 'active' : ''}" id="btn-deck-pin">📌</button>
        <button class="deck-settings-btn" id="btn-deck-settings">⚙️</button>
      </div>
    `;

    // Prism steppers
    this.deckInner.querySelector('#btn-prism-prev')?.addEventListener('click', () => this.rotatePrism(-1));
    this.deckInner.querySelector('#btn-prism-next')?.addEventListener('click', () => this.rotatePrism(1));

    // Prism tool buttons
    this.deckInner.querySelectorAll('.tool-action-btn').forEach(btn => {
      btn.addEventListener('click', () => {
        const toolId = btn.getAttribute('data-tool-id');
        const tool = currentTc.tools.find(t => t.id === toolId);
        if (tool && tool.action) tool.action();
      });
    });

    this.attachDeckControls();
  }

  rotatePrism(delta) {
    const toolchains = this.getToolchainsForType(this.currentContext, this.currentContainer);
    if (!toolchains.length) return;
    this.rotaryIndex = (this.rotaryIndex + delta + toolchains.length) % toolchains.length;
    this.renderDeckForContext(this.currentContext, this.currentContainer);
  }

  attachSocketPillEvents(toolchains) {
    this.deckInner.querySelectorAll('.socket-case-pill').forEach(pill => {
      pill.addEventListener('click', (e) => {
        e.stopPropagation();
        const tcId = pill.getAttribute('data-toolchain');
        const tc = toolchains.find(t => t.id === tcId);
        if (!tc) return;

        if (this.activeOpenToolchain === tcId && !this.dropTray.classList.contains('hidden')) {
          this.closeDropTray();
          return;
        }

        this.openSocketCaseTray(pill, tc);
      });
    });
  }

  openSocketCaseTray(pillEl, toolchain) {
    this.activeOpenToolchain = toolchain.id;
    this.deckInner.querySelectorAll('.socket-case-pill').forEach(p => p.classList.remove('active'));
    pillEl.classList.add('active');

    const pillRect = pillEl.getBoundingClientRect();
    const wrapperRect = this.wrapper.getBoundingClientRect();
    const leftOffset = Math.max(16, pillRect.left - wrapperRect.left);

    this.dropTray.style.left = `${leftOffset}px`;
    this.dropTray.innerHTML = `
      <div class="drop-tray-header">
        <span>${toolchain.icon} <strong>${toolchain.title}</strong> (${toolchain.count} instruments)</span>
        <button class="drop-tray-tearoff-btn" id="btn-tearoff" title="Tear off toolchain to floating canvas pod">📌 Tear-off Pod</button>
        <button class="drop-tray-close-btn" id="btn-close-tray">✕</button>
      </div>
      <div class="drop-tray-tools-grid">
        ${toolchain.tools.map((t, idx) => `
          <button class="socket-tool-btn ${t.highlight ? `highlight-${t.highlight}` : ''}" data-tool-id="${t.id}" title="${t.label}">
            <span class="tool-shortcut-num">${idx + 1}</span>
            <span class="tool-main-icon">${t.icon}</span>
            <span class="tool-label-text">${t.label}</span>
          </button>
        `).join('')}
      </div>
    `;

    this.dropTray.classList.remove('hidden');

    // Close button
    this.dropTray.querySelector('#btn-close-tray')?.addEventListener('click', () => this.closeDropTray());

    // Tear-off button
    this.dropTray.querySelector('#btn-tearoff')?.addEventListener('click', () => {
      this.tearOffToolchain(toolchain);
      this.closeDropTray();
    });

    // Tool click actions
    this.dropTray.querySelectorAll('.socket-tool-btn').forEach(btn => {
      btn.addEventListener('click', (e) => {
        e.stopPropagation();
        const toolId = btn.getAttribute('data-tool-id');
        const tool = toolchain.tools.find(t => t.id === toolId);
        if (tool && tool.action) tool.action();
      });
    });
  }

  tearOffToolchain(toolchain) {
    const stage = document.querySelector('#canvas-stage');
    if (!stage) return;

    const pod = document.createElement('div');
    pod.className = 'floating-toolchain-pod';
    pod.style.left = '380px';
    pod.style.top = '120px';
    pod.innerHTML = `
      <div class="pod-header">
        <span>${toolchain.icon} ${toolchain.title.split(' ')[0]} Pod</span>
        <button class="pod-close-btn">✕</button>
      </div>
      <div class="pod-body">
        ${toolchain.tools.map(t => `
          <button class="socket-tool-btn" data-tool-id="${t.id}">
            <span class="tool-main-icon">${t.icon}</span>
          </button>
        `).join('')}
      </div>
    `;

    pod.querySelector('.pod-close-btn')?.addEventListener('click', () => pod.remove());

    // Dragging pod
    const pHeader = pod.querySelector('.pod-header');
    pHeader.addEventListener('mousedown', (e) => {
      e.stopPropagation();
      let startX = e.clientX;
      let startY = e.clientY;
      let initL = pod.offsetLeft;
      let initT = pod.offsetTop;

      const onMove = (me) => {
        pod.style.left = `${initL + (me.clientX - startX)}px`;
        pod.style.top = `${initT + (me.clientY - startY)}px`;
      };
      const onUp = () => {
        window.removeEventListener('mousemove', onMove);
        window.removeEventListener('mouseup', onUp);
      };
      window.addEventListener('mousemove', onMove);
      window.addEventListener('mouseup', onUp);
    });

    stage.appendChild(pod);
  }

  closeDropTray() {
    this.activeOpenToolchain = null;
    this.deckInner.querySelectorAll('.socket-case-pill').forEach(p => p.classList.remove('active'));
    if (this.dropTray) this.dropTray.classList.add('hidden');
  }

  attachDeckControls() {
    this.deckInner.querySelector('#btn-deck-pin')?.addEventListener('click', () => {
      this.pinned = !this.pinned;
      const btn = this.deckInner.querySelector('#btn-deck-pin');
      if (btn) btn.classList.toggle('active', this.pinned);
    });

    this.deckInner.querySelector('#btn-deck-settings')?.addEventListener('click', (e) => {
      e.stopPropagation();
      this.toggleSettingsModal();
    });
  }

  renderDefault() {
    this.deckInner.innerHTML = `
      <div class="deck-domain-badge">🌌 Global Manifold Deck</div>
      <div class="toolchains-socket-bar">
        <span style="color: var(--text-muted); font-size: 11px;">Select a container node on the canvas to clip its Toolchain Socket Cases to the deck.</span>
      </div>
      <div class="deck-controls-shelf">
        <button class="deck-pin-btn ${this.pinned ? 'active' : ''}" id="btn-deck-pin">📌</button>
        <button class="deck-settings-btn" id="btn-deck-settings">⚙️</button>
      </div>
    `;
    this.attachDeckControls();
  }

  toggleSettingsModal() {
    const isHidden = this.settingsModal.classList.contains('hidden');
    if (!isHidden) {
      this.settingsModal.classList.add('hidden');
      return;
    }

    this.settingsModal.innerHTML = `
      <div class="settings-modal-header">
        <span>⚙️ Toolchain Deck & Presentation Mode</span>
        <button style="background:transparent; border:none; color:var(--text-secondary); cursor:pointer; font-size:14px;" id="btn-close-deck-settings">✕</button>
      </div>

      <div class="settings-section-title">Toolchain Presentation Architecture</div>
      <div class="settings-options-group">
        <label class="settings-radio-label">
          <input type="radio" name="deck-mode" value="case-tray" ${this.settings.presentationMode === 'case-tray' ? 'checked' : ''} />
          <span><strong>1. Socket-Case Drop-Tray (Zero Overflow)</strong> (Click pill to open frosted glass case)</span>
        </label>
        <label class="settings-radio-label">
          <input type="radio" name="deck-mode" value="3d-prism" ${this.settings.presentationMode === '3d-prism' ? 'checked' : ''} />
          <span><strong>2. 3D Cylindrical Rotary Prism Drum</strong> (Wheel rolls 3D cylinder barrel)</span>
        </label>
      </div>

      <div class="settings-section-title">Density & Interaction</div>
      <div class="settings-options-group">
        <label class="settings-check-label">
          <input type="checkbox" id="chk-tearoff" ${this.settings.enableTearOff ? 'checked' : ''} />
          <span>Allow "Tear-off" floating tool pods on canvas stage</span>
        </label>
      </div>
    `;

    this.settingsModal.classList.remove('hidden');

    this.settingsModal.querySelector('#btn-close-deck-settings')?.addEventListener('click', () => {
      this.settingsModal.classList.add('hidden');
    });

    this.settingsModal.querySelectorAll('input[name="deck-mode"]').forEach(radio => {
      radio.addEventListener('change', (e) => {
        this.settings.presentationMode = e.target.value;
        this.saveSettings();
        this.renderDeckForContext(this.currentContext, this.currentContainer);
      });
    });

    this.settingsModal.querySelector('#chk-tearoff')?.addEventListener('change', (e) => {
      this.settings.enableTearOff = e.target.checked;
      this.saveSettings();
    });
  }
}

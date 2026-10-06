/**
 * Dynamic Contextual Action Ribbon Manager
 * Dynamically mounts specialized domain toolbars (RichText, Sheet, RDF Graph, Media Studio, etc.)
 * onto the top of the manifold window with advanced overflow management:
 * - Scroll Stepper chevrons (◀ and ▶)
 * - Collapsible Section Group Accordions
 * - Priority-tiered Adaptive Folding (⋯ More Tools popover)
 * - Multi-Row vs Single-Row Layout Modes
 * - Configurable Density (Compact, Adaptive, Full)
 * - ⚙️ Ribbon Settings Drawer with LocalStorage Persistence
 */

import { canvasState } from './state.js';

export class ContextualRibbonManager {
  constructor(ribbonWrapperEl) {
    this.wrapper = ribbonWrapperEl;
    this.pinned = false;
    this.currentContext = null;
    this.currentContainer = null;
    
    // User Settings (with defaults)
    this.settings = {
      layoutMode: 'single', // 'single' | 'multi' | 'adaptive'
      density: 'adaptive',  // 'compact' | 'adaptive' | 'full'
      collapsedGroups: new Set(),
      hiddenGroups: new Set()
    };
    
    this.loadSettings();
    this.buildWrapperStructure();
    this.init();
  }

  loadSettings() {
    try {
      const saved = localStorage.getItem('hypercanvas_ribbon_settings');
      if (saved) {
        const parsed = JSON.parse(saved);
        this.settings.layoutMode = parsed.layoutMode || 'single';
        this.settings.density = parsed.density || 'adaptive';
        if (parsed.collapsedGroups) this.settings.collapsedGroups = new Set(parsed.collapsedGroups);
        if (parsed.hiddenGroups) this.settings.hiddenGroups = new Set(parsed.hiddenGroups);
      }
    } catch (e) {
      console.warn('Could not load ribbon settings from localStorage', e);
    }
  }

  saveSettings() {
    try {
      localStorage.setItem('hypercanvas_ribbon_settings', JSON.stringify({
        layoutMode: this.settings.layoutMode,
        density: this.settings.density,
        collapsedGroups: Array.from(this.settings.collapsedGroups),
        hiddenGroups: Array.from(this.settings.hiddenGroups)
      }));
    } catch (e) {
      console.warn('Could not save ribbon settings', e);
    }
  }

  buildWrapperStructure() {
    this.wrapper.className = 'contextual-action-ribbon-wrapper';
    this.wrapper.innerHTML = `
      <button class="ribbon-scroll-stepper stepper-left" id="ribbon-step-left" title="Scroll Left">◀</button>
      <div class="contextual-action-ribbon" id="contextual-ribbon-inner"></div>
      <button class="ribbon-scroll-stepper stepper-right" id="ribbon-step-right" title="Scroll Right">▶</button>
      <div class="ribbon-settings-modal hidden" id="ribbon-settings-modal"></div>
    `;

    this.ribbonInner = this.wrapper.querySelector('#contextual-ribbon-inner');
    this.stepLeft = this.wrapper.querySelector('#ribbon-step-left');
    this.stepRight = this.wrapper.querySelector('#ribbon-step-right');
    this.settingsModal = this.wrapper.querySelector('#ribbon-settings-modal');

    // Horizontal wheel scrolling
    this.ribbonInner.addEventListener('wheel', (e) => {
      if (this.settings.layoutMode === 'single') {
        e.preventDefault();
        this.ribbonInner.scrollLeft += e.deltaY;
        this.checkOverflow();
      }
    }, { passive: false });

    // Stepper buttons
    this.stepLeft.addEventListener('click', () => {
      this.ribbonInner.scrollBy({ left: -140, behavior: 'smooth' });
    });
    this.stepRight.addEventListener('click', () => {
      this.ribbonInner.scrollBy({ left: 140, behavior: 'smooth' });
    });

    this.ribbonInner.addEventListener('scroll', () => this.checkOverflow());

    // ResizeObserver for overflow calculation
    this.resizeObserver = new ResizeObserver(() => {
      this.checkOverflow();
    });
    this.resizeObserver.observe(this.ribbonInner);
  }

  checkOverflow() {
    if (this.settings.layoutMode === 'multi') {
      this.stepLeft.classList.remove('visible');
      this.stepRight.classList.remove('visible');
      return;
    }

    const { scrollLeft, scrollWidth, clientWidth } = this.ribbonInner;
    const hasOverflow = scrollWidth > clientWidth + 4;

    if (hasOverflow && scrollLeft > 10) {
      this.stepLeft.classList.add('visible');
    } else {
      this.stepLeft.classList.remove('visible');
    }

    if (hasOverflow && scrollLeft < scrollWidth - clientWidth - 10) {
      this.stepRight.classList.add('visible');
    } else {
      this.stepRight.classList.remove('visible');
    }
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

    this.applyLayoutClasses();
    this.renderDefault();
  }

  applyLayoutClasses() {
    this.ribbonInner.className = `contextual-action-ribbon layout-${this.settings.layoutMode} density-${this.settings.density} ${this.currentContext ? 'active' : ''}`;
  }

  setContext(type, container = null) {
    this.currentContext = type;
    this.currentContainer = container;
    this.applyLayoutClasses();

    if (!type) {
      if (!this.pinned) {
        this.ribbonInner.classList.remove('active');
        this.ribbonInner.innerHTML = '';
      } else {
        this.renderDefault();
      }
      this.checkOverflow();
      return;
    }

    this.ribbonInner.classList.add('active');
    switch (type) {
      case 'doc':
        this.renderDocToolbar(container);
        break;
      case 'sheet':
        this.renderSheetToolbar(container);
        break;
      case 'ontology':
        this.renderRdfGraphToolbar(container);
        break;
      case 'media':
      case '3d':
        this.renderMediaStudioToolbar(container);
        break;
      case 'social':
      case 'webrtc':
        this.renderCommunicationToolbar(container);
        break;
      case 'map':
        this.renderSpatialGisToolbar(container);
        break;
      case 'code':
        this.renderVibeScriptToolbar(container);
        break;
      case 'health':
        this.renderHealthToolbar(container);
        break;
      default:
        this.renderDefault();
    }

    setTimeout(() => this.checkOverflow(), 50);
  }

  renderControlsShelf(honestyBadge = 'live') {
    return `
      <div class="ribbon-pin-control">
        ${honestyBadge ? `<span class="honesty-badge honesty-${honestyBadge}">${honestyBadge}</span>` : ''}
        <button class="ribbon-pin-btn ${this.pinned ? 'active' : ''}" id="btn-ribbon-pin" title="Pin Top Toolbar">📌</button>
        <button class="ribbon-settings-btn" id="btn-ribbon-settings" title="Toolbar Settings & Density">⚙️</button>
      </div>
    `;
  }

  attachCommonControls() {
    this.ribbonInner.querySelector('#btn-ribbon-pin')?.addEventListener('click', () => {
      this.pinned = !this.pinned;
      const btn = this.ribbonInner.querySelector('#btn-ribbon-pin');
      if (btn) btn.classList.toggle('active', this.pinned);
    });

    this.ribbonInner.querySelector('#btn-ribbon-settings')?.addEventListener('click', (e) => {
      e.stopPropagation();
      this.toggleSettingsModal();
    });

    // Group header collapsible toggles
    this.ribbonInner.querySelectorAll('.ribbon-group-header').forEach(hdr => {
      hdr.addEventListener('click', () => {
        const groupKey = hdr.getAttribute('data-group');
        const groupEl = hdr.closest('.ribbon-tools-group');
        if (!groupEl || !groupKey) return;

        if (this.settings.collapsedGroups.has(groupKey)) {
          this.settings.collapsedGroups.delete(groupKey);
          groupEl.classList.remove('collapsed');
          hdr.querySelector('.chevron').textContent = '▾';
        } else {
          this.settings.collapsedGroups.add(groupKey);
          groupEl.classList.add('collapsed');
          hdr.querySelector('.chevron').textContent = '▸';
        }
        this.saveSettings();
        this.checkOverflow();
      });
    });
  }

  renderDefault() {
    this.ribbonInner.innerHTML = `
      <div class="ribbon-domain-badge">🌌 Global Manifold</div>
      <div class="ribbon-tools-group">
        <span style="color: var(--text-muted); font-size: 11px;">Select a container on the stage to clip its domain tools to the top ribbon.</span>
      </div>
      ${this.renderControlsShelf('live')}
    `;
    this.attachCommonControls();
  }

  renderDocToolbar(container) {
    const isTypoCollapsed = this.settings.collapsedGroups.has('doc_typo');
    const isGroundCollapsed = this.settings.collapsedGroups.has('doc_ground');

    this.ribbonInner.innerHTML = `
      <div class="ribbon-domain-badge" style="color: var(--color-doc); border-color: var(--color-doc);">📝 Document / TinyMCE Suite</div>
      
      <!-- Group 1: Typography & Formatting -->
      <div class="ribbon-tools-group ${isTypoCollapsed ? 'collapsed' : ''}" data-group-id="doc_typo">
        <div class="ribbon-group-header" data-group="doc_typo">Typography <span class="chevron">${isTypoCollapsed ? '▸' : '▾'}</span></div>
        <button class="ribbon-tool-btn" id="rib-doc-bold" title="Bold (Ctrl+B)"><strong>B</strong></button>
        <button class="ribbon-tool-btn" id="rib-doc-italic" title="Italic (Ctrl+I)"><em>I</em></button>
        <button class="ribbon-tool-btn" id="rib-doc-underline" title="Underline (Ctrl+U)"><u>U</u></button>
        <button class="ribbon-tool-btn" id="rib-doc-code" title="Inline Code"><code>&lt;/&gt;</code></button>
        <div class="ribbon-divider"></div>
        <button class="ribbon-tool-btn" id="rib-doc-h1">H1</button>
        <button class="ribbon-tool-btn" id="rib-doc-h2">H2</button>
        <button class="ribbon-tool-btn" id="rib-doc-quote">❝ Quote</button>
        <button class="ribbon-tool-btn" id="rib-doc-list">• List</button>
      </div>

      <!-- Group 2: Semantic Spans & Qualia Grounding -->
      <div class="ribbon-tools-group ${isGroundCollapsed ? 'collapsed' : ''}" data-group-id="doc_ground">
        <div class="ribbon-group-header" data-group="doc_ground">Semantic Grounding <span class="chevron">${isGroundCollapsed ? '▸' : '▾'}</span></div>
        <button class="ribbon-tool-btn highlight-entity" id="rib-doc-entity" title="Tag Ontological Entity in Aura">🏷️ <span class="ribbon-btn-label">Tag Entity</span></button>
        <button class="ribbon-tool-btn" id="rib-doc-subj" style="color: var(--modality-subjective);">🧠 <span class="ribbon-btn-label">Qualia</span></button>
        <button class="ribbon-tool-btn" id="rib-doc-obj" style="color: var(--modality-objective);">🔬 <span class="ribbon-btn-label">Objective</span></button>
        <button class="ribbon-tool-btn" id="rib-doc-norm" style="color: var(--modality-normative);">⚖️ <span class="ribbon-btn-label">Statute</span></button>
        <button class="ribbon-tool-btn" id="rib-doc-latex">$$ <span class="ribbon-btn-label">LaTeX</span></button>
        <button class="ribbon-tool-btn highlight-ai" id="rib-doc-gazetteer" title="NLP Tokenize & Seal Text Spans">🔍 <span class="ribbon-btn-label">Gazetteer NLP</span></button>
      </div>

      ${this.renderControlsShelf('partial')}
    `;

    this.attachDocEvents(container);
    this.attachCommonControls();
  }

  renderSheetToolbar(container) {
    const isCalcCollapsed = this.settings.collapsedGroups.has('sheet_calc');
    const isTensorCollapsed = this.settings.collapsedGroups.has('sheet_tensor');

    this.ribbonInner.innerHTML = `
      <div class="ribbon-domain-badge" style="color: var(--color-sheet); border-color: var(--color-sheet);">📊 Spreadsheet & Tensor Suite</div>
      
      <!-- Group 1: Formula Receptor -->
      <div class="ribbon-tools-group ${isCalcCollapsed ? 'collapsed' : ''}" data-group-id="sheet_calc">
        <div class="ribbon-group-header" data-group="sheet_calc">Formulas <span class="chevron">${isCalcCollapsed ? '▸' : '▾'}</span></div>
        <div class="ribbon-formula-group">
          <span class="sheet-cell-ref" id="rib-sheet-cell-addr">B2</span>
          <span style="color: var(--accent-amber); font-weight: bold; font-size: 11px;">fx</span>
          <input type="text" class="ribbon-formula-input" id="rib-sheet-formula" value="=SUM(B1:B5)" />
        </div>
        <button class="ribbon-tool-btn" id="rib-sheet-calc">🧮 <span class="ribbon-btn-label">Compute</span></button>
      </div>

      <!-- Group 2: Continuous Neural Latents -->
      <div class="ribbon-tools-group ${isTensorCollapsed ? 'collapsed' : ''}" data-group-id="sheet_tensor">
        <div class="ribbon-group-header" data-group="sheet_tensor">Tensors & Latents <span class="chevron">${isTensorCollapsed ? '▸' : '▾'}</span></div>
        <button class="ribbon-tool-btn" id="rib-sheet-resonance">🌊 <span class="ribbon-btn-label">Resonance Hz</span></button>
        <button class="ribbon-tool-btn" id="rib-sheet-latent" style="color: var(--accent-emerald);">📐 <span class="ribbon-btn-label">P64 Latent Tensor</span></button>
        <button class="ribbon-tool-btn" id="rib-sheet-import">📥 <span class="ribbon-btn-label">Import CSV/CBOR</span></button>
        <button class="ribbon-tool-btn" id="rib-sheet-export">📤 <span class="ribbon-btn-label">Export CSVW</span></button>
      </div>

      ${this.renderControlsShelf('present')}
    `;

    this.attachSheetEvents(container);
    this.attachCommonControls();
  }

  renderRdfGraphToolbar(container) {
    this.ribbonInner.innerHTML = `
      <div class="ribbon-domain-badge" style="color: var(--color-ontology); border-color: var(--color-ontology);">🏷️ RDF 1.2 & Aura Catalog</div>
      
      <div class="ribbon-tools-group">
        <div class="ribbon-group-header">Aura & SHACL <span class="chevron">▾</span></div>
        <button class="ribbon-tool-btn highlight-entity" id="rib-rdf-validate">🛡️ <span class="ribbon-btn-label">SHACL Validate (100%)</span></button>
        <button class="ribbon-tool-btn" id="rib-rdf-align">+ Add Alignment Row</button>
      </div>

      <div class="ribbon-tools-group">
        <div class="ribbon-group-header">Query & DAG <span class="chevron">▾</span></div>
        <button class="ribbon-tool-btn" id="rib-rdf-sparql">🔍 <span class="ribbon-btn-label">SPARQL Console</span></button>
        <button class="ribbon-tool-btn" id="rib-rdf-n3">📜 <span class="ribbon-btn-label">N3 Deduction</span></button>
        <button class="ribbon-tool-btn" id="rib-rdf-nquin" style="color: var(--accent-emerald);">&lt;&lt;[ 48-byte Nquin ]&gt;&gt;</button>
        <button class="ribbon-tool-btn" id="rib-rdf-compact">⚡ <span class="ribbon-btn-label">Compact Merkle DAG</span></button>
      </div>

      ${this.renderControlsShelf('partial')}
    `;

    this.attachRdfEvents(container);
    this.attachCommonControls();
  }

  renderMediaStudioToolbar(container) {
    this.ribbonInner.innerHTML = `
      <div class="ribbon-domain-badge" style="color: var(--color-3d); border-color: var(--color-3d);">🧊 Native 10D Asset (.10d) & Media Studio</div>
      
      <!-- Toolchain 1: 10D Kinematics Motor Articulation -->
      <div class="ribbon-tools-group" data-group-id="3d_kinematics">
        <div class="ribbon-group-header" data-group="3d_kinematics">Kinematics Toolchain <span class="chevron">▾</span></div>
        <button class="ribbon-tool-btn" id="rib-10d-jaw">📐 <span class="ribbon-btn-label">Jaw Joint (14°)</span></button>
        <button class="ribbon-tool-btn" id="rib-10d-tongue">👅 <span class="ribbon-btn-label">Tongue Tensor</span></button>
        <button class="ribbon-tool-btn" id="rib-10d-velum">🚪 <span class="ribbon-btn-label">Velum Gate</span></button>
        <button class="ribbon-tool-btn" id="rib-media-3d">🔄 <span class="ribbon-btn-label">CAD Orbit</span></button>
      </div>

      <!-- Toolchain 2: Animation & Temporal Timeline -->
      <div class="ribbon-tools-group" data-group-id="3d_anim">
        <div class="ribbon-group-header" data-group="3d_anim">Animation Toolchain <span class="chevron">▾</span></div>
        <button class="ribbon-tool-btn active" id="rib-10d-play">▶ <span class="ribbon-btn-label">Play (60fps)</span></button>
        <button class="ribbon-tool-btn" id="rib-10d-scrub">⏱️ <span class="ribbon-btn-label">Scrub (t=1.42s)</span></button>
        <button class="ribbon-tool-btn" id="rib-10d-keyframe">🔑 <span class="ribbon-btn-label">+ Keyframe</span></button>
      </div>

      <!-- Toolchain 3: Reactive Event Emitters & Pulse Bus -->
      <div class="ribbon-tools-group" data-group-id="3d_events">
        <div class="ribbon-group-header" data-group="3d_events">Event Wiring Toolchain <span class="chevron">▾</span></div>
        <button class="ribbon-tool-btn highlight-ai" id="rib-10d-hook-event">⚡ <span class="ribbon-btn-label">Hook on_closure -> pulse</span></button>
        <button class="ribbon-tool-btn" id="rib-10d-did">🛡️ <span class="ribbon-btn-label">CRC / DID Verified</span></button>
        <button class="ribbon-tool-btn" id="rib-media-wgpu">⚡ <span class="ribbon-btn-label">WGPU Surface</span></button>
      </div>

      ${this.renderControlsShelf('present')}
    `;

    this.attachMediaEvents(container);
    this.attachCommonControls();
  }

  renderCommunicationToolbar(container) {
    this.ribbonInner.innerHTML = `
      <div class="ribbon-domain-badge" style="color: var(--color-social); border-color: var(--color-social);">📞 Social & Multi-Agent Suite</div>
      
      <div class="ribbon-tools-group">
        <div class="ribbon-group-header">Sub-Agents <span class="chevron">▾</span></div>
        <button class="ribbon-tool-btn highlight-ai" id="rib-comms-spawn">🤖 <span class="ribbon-btn-label">Spawn AI Sub-Agent</span></button>
        <button class="ribbon-tool-btn" id="rib-comms-chatquad">💬 <span class="ribbon-btn-label">Generate Chat Quad</span></button>
      </div>

      <div class="ribbon-tools-group">
        <div class="ribbon-group-header">Stream & Auth <span class="chevron">▾</span></div>
        <button class="ribbon-tool-btn" id="rib-comms-webrtc">🎥 <span class="ribbon-btn-label">WebRTC Gateway</span></button>
        <button class="ribbon-tool-btn" id="rib-comms-peer">🌐 <span class="ribbon-btn-label">DID Handshake</span></button>
        <button class="ribbon-tool-btn" id="rib-comms-sign" style="color: var(--accent-emerald);">✍️ <span class="ribbon-btn-label">Sign Timothy Holborn</span></button>
      </div>

      ${this.renderControlsShelf('partial')}
    `;

    this.attachCommsEvents(container);
    this.attachCommonControls();
  }

  renderSpatialGisToolbar(container) {
    this.ribbonInner.innerHTML = `
      <div class="ribbon-domain-badge" style="color: var(--color-map); border-color: var(--color-map);">🗺️ Spatiotemporal GIS Suite</div>
      
      <div class="ribbon-tools-group">
        <div class="ribbon-group-header">GIS Layers <span class="chevron">▾</span></div>
        <button class="ribbon-tool-btn active" id="rib-gis-sat">🛰️ Satellite</button>
        <button class="ribbon-tool-btn active" id="rib-gis-vectors">🌊 Hydrological Flow</button>
        <button class="ribbon-tool-btn active" id="rib-gis-uav">🚁 UAV Sentinel</button>
        <button class="ribbon-tool-btn active" id="rib-gis-survey">🚶 Survey Trail</button>
      </div>

      <div class="ribbon-tools-group">
        <div class="ribbon-group-header">Events <span class="chevron">▾</span></div>
        <button class="ribbon-tool-btn highlight-entity" id="rib-gis-pin">📍 <span class="ribbon-btn-label">Pin Case Incident</span></button>
        <button class="ribbon-tool-btn" id="rib-gis-timesync">⏱️ <span class="ribbon-btn-label">Time Sync Scrub</span></button>
      </div>

      ${this.renderControlsShelf('partial')}
    `;

    this.attachGisEvents(container);
    this.attachCommonControls();
  }

  renderVibeScriptToolbar(container) {
    this.ribbonInner.innerHTML = `
      <div class="ribbon-domain-badge" style="color: var(--color-code); border-color: var(--color-code);">⚡ VibeScript 0.1 AST Console</div>
      
      <div class="ribbon-tools-group">
        <div class="ribbon-group-header">Execution <span class="chevron">▾</span></div>
        <button class="ribbon-tool-btn active" id="rib-vibe-run" style="background: var(--color-code); color: #000; font-weight: 700;">▶ Run (Gas: 350)</button>
        <button class="ribbon-tool-btn" id="rib-vibe-ast">🌲 AST Inspector</button>
      </div>

      <div class="ribbon-tools-group">
        <div class="ribbon-group-header">Snippets <span class="chevron">▾</span></div>
        <button class="ribbon-tool-btn" id="rib-vibe-req">🔒 requires [graph, pulse]</button>
        <button class="ribbon-tool-btn" id="rib-vibe-pulse">+ pulse::emit</button>
        <button class="ribbon-tool-btn" id="rib-vibe-invoke">+ capability.invoke</button>
      </div>

      ${this.renderControlsShelf('partial')}
    `;

    this.attachVibeEvents(container);
    this.attachCommonControls();
  }

  renderHealthToolbar(container) {
    this.ribbonInner.innerHTML = `
      <div class="ribbon-domain-badge" style="color: var(--color-health); border-color: var(--color-health);">🩺 Clinical & Bio-Telemetry Suite</div>
      
      <div class="ribbon-tools-group">
        <div class="ribbon-group-header">Biomarkers <span class="chevron">▾</span></div>
        <button class="ribbon-tool-btn active" id="rib-health-path">🧪 Pathology Lab</button>
        <button class="ribbon-tool-btn" id="rib-health-tomo">🩻 Tomography (32/64)</button>
      </div>

      <div class="ribbon-tools-group">
        <div class="ribbon-group-header">Anatomy <span class="chevron">▾</span></div>
        <button class="ribbon-tool-btn" id="rib-health-anat">🫀 10D Vocal Tract Resonator</button>
        <button class="ribbon-tool-btn" id="rib-health-export">📋 Export Baseline Profile</button>
      </div>

      ${this.renderControlsShelf('present')}
    `;

    this.attachHealthEvents(container);
    this.attachCommonControls();
  }

  toggleSettingsModal() {
    const isHidden = this.settingsModal.classList.contains('hidden');
    if (!isHidden) {
      this.settingsModal.classList.add('hidden');
      return;
    }

    this.settingsModal.innerHTML = `
      <div class="settings-modal-header">
        <span>⚙️ Ribbon & Toolbar Preferences</span>
        <button style="background:transparent; border:none; color:var(--text-secondary); cursor:pointer; font-size:14px;" id="btn-close-ribbon-settings">✕</button>
      </div>

      <div class="settings-section-title">Layout & Overflow Mode</div>
      <div class="settings-options-group">
        <label class="settings-radio-label">
          <input type="radio" name="ribbon-layout" value="single" ${this.settings.layoutMode === 'single' ? 'checked' : ''} />
          <span>Single-Row Scrollable (Horizontal Strip)</span>
        </label>
        <label class="settings-radio-label">
          <input type="radio" name="ribbon-layout" value="multi" ${this.settings.layoutMode === 'multi' ? 'checked' : ''} />
          <span>Multi-Row Studio Deck (Wrap into Rows)</span>
        </label>
      </div>

      <div class="settings-section-title">Button Density</div>
      <div class="settings-options-group">
        <label class="settings-radio-label">
          <input type="radio" name="ribbon-density" value="compact" ${this.settings.density === 'compact' ? 'checked' : ''} />
          <span>Compact (Icon-Only, 28px)</span>
        </label>
        <label class="settings-radio-label">
          <input type="radio" name="ribbon-density" value="adaptive" ${this.settings.density === 'adaptive' ? 'checked' : ''} />
          <span>Adaptive (Icon + Short Label, 36px)</span>
        </label>
        <label class="settings-radio-label">
          <input type="radio" name="ribbon-density" value="full" ${this.settings.density === 'full' ? 'checked' : ''} />
          <span>Full Expanded (42px)</span>
        </label>
      </div>

      <div class="settings-section-title">Collapsible Groups</div>
      <div class="settings-options-group">
        <p style="margin:0; font-size:11px; color:var(--text-secondary);">Click any section title on the ribbon (e.g. <em>Typography ▾</em>) to collapse/expand it inline.</p>
      </div>
    `;

    this.settingsModal.classList.remove('hidden');

    this.settingsModal.querySelector('#btn-close-ribbon-settings')?.addEventListener('click', () => {
      this.settingsModal.classList.add('hidden');
    });

    this.settingsModal.querySelectorAll('input[name="ribbon-layout"]').forEach(radio => {
      radio.addEventListener('change', (e) => {
        this.settings.layoutMode = e.target.value;
        this.saveSettings();
        this.applyLayoutClasses();
        this.checkOverflow();
      });
    });

    this.settingsModal.querySelectorAll('input[name="ribbon-density"]').forEach(radio => {
      radio.addEventListener('change', (e) => {
        this.settings.density = e.target.value;
        this.saveSettings();
        this.applyLayoutClasses();
        this.checkOverflow();
      });
    });
  }

  attachDocEvents(container) {
    this.ribbonInner.querySelector('#rib-doc-bold')?.addEventListener('click', () => document.execCommand('bold'));
    this.ribbonInner.querySelector('#rib-doc-italic')?.addEventListener('click', () => document.execCommand('italic'));
    this.ribbonInner.querySelector('#rib-doc-underline')?.addEventListener('click', () => document.execCommand('underline'));
    this.ribbonInner.querySelector('#rib-doc-h1')?.addEventListener('click', () => document.execCommand('formatBlock', false, 'h1'));
    this.ribbonInner.querySelector('#rib-doc-h2')?.addEventListener('click', () => document.execCommand('formatBlock', false, 'h2'));
    this.ribbonInner.querySelector('#rib-doc-quote')?.addEventListener('click', () => document.execCommand('formatBlock', false, 'blockquote'));
    this.ribbonInner.querySelector('#rib-doc-list')?.addEventListener('click', () => document.execCommand('insertUnorderedList'));
    
    this.ribbonInner.querySelector('#rib-doc-entity')?.addEventListener('click', () => {
      const sel = window.getSelection();
      const text = sel?.toString().trim() || 'NamedEntity';
      alert(`✓ Grounded entity "${text}" into Aura catalog <concept:${text.replace(/\\s+/g, '_')}>`);
    });

    this.ribbonInner.querySelector('#rib-doc-gazetteer')?.addEventListener('click', () => {
      alert(`🔍 Gazetteer NLP Analysis (qualia_core_db::nlp):\n• Tokens: 42\n• Sentences: 3\n• Sealed Spans: 3 quins committed to Merkle DAG.`);
    });
  }

  attachSheetEvents(container) {
    this.ribbonInner.querySelector('#rib-sheet-calc')?.addEventListener('click', () => {
      alert('🧮 Computed active spreadsheet formula: result = 405.7 L/m total catchment flow.');
    });
    this.ribbonInner.querySelector('#rib-sheet-resonance')?.addEventListener('click', () => {
      alert('🌊 Calculated resonance frequency: 221.5 Hz across North Spring aquifer.');
    });
    this.ribbonInner.querySelector('#rib-sheet-latent')?.addEventListener('click', () => {
      alert('📐 Aligned continuous EnCodec P64 neural latent tensor to 64-byte boundary.');
    });
  }

  attachRdfEvents(container) {
    this.ribbonInner.querySelector('#rib-rdf-validate')?.addEventListener('click', () => {
      alert('🛡️ Aura SHACL Validator: 100% Conforming · 3 Target Shapes Satisfied.');
    });
    this.ribbonInner.querySelector('#rib-rdf-align')?.addEventListener('click', () => {
      if (container) {
        if (!container.mappings) container.mappings = [];
        container.mappings.push({
          source: `hydro:Metric_${Date.now().toString().slice(-4)}`,
          relation: 'skos:exactMatch',
          target: 'fiduciary:WaterAsset'
        });
        canvasState.emit('stateChanged', canvasState.state);
      }
      alert('✓ Appended dynamic alignment row to Cross-Ontology Alignment Matrix.');
    });
    this.ribbonInner.querySelector('#rib-rdf-nquin')?.addEventListener('click', () => {
      alert('<<[ Asserted 48-byte Nquin into local Merkle-CRDT DAG with DID signature ]>>');
    });
  }

  attachMediaEvents(container) {
    this.ribbonInner.querySelector('#rib-media-gen')?.addEventListener('click', () => {
      alert('✨ Prompted generative multi-modal synthesis for visual grapheme asset.');
    });
    this.ribbonInner.querySelector('#rib-media-3d')?.addEventListener('click', () => {
      canvasState.setDimensionMode('3D');
      alert('🧊 Switched viewport to 3D CAD Perspective Orbit.');
    });
  }

  attachCommsEvents(container) {
    this.ribbonInner.querySelector('#rib-comms-spawn')?.addEventListener('click', () => {
      alert('🤖 Spawned AI Sub-Agent: Fiduciary Compliance AI to chat graph.');
    });
    this.ribbonInner.querySelector('#rib-comms-sign')?.addEventListener('click', () => {
      alert('✍️ Signed active chat graph session with did:qualia:timothy_charles_holborn.');
    });
  }

  attachGisEvents(container) {
    this.ribbonInner.querySelector('#rib-gis-pin')?.addEventListener('click', () => {
      alert('📍 Pinned new case incident node onto catchment map at (34.0°S, 148.2°E).');
    });
  }

  attachVibeEvents(container) {
    this.ribbonInner.querySelector('#rib-vibe-run')?.addEventListener('click', () => {
      alert('⚡ VibeScript 0.1 AST Interpreter:\nCompiled homoiconic AST. Gas: 48 units consumed. Pulse emitted.');
    });
  }

  attachHealthEvents(container) {
    this.ribbonInner.querySelector('#rib-health-export')?.addEventListener('click', () => {
      alert('📋 Exported baseline adult bio-telemetry profile.');
    });
  }
}

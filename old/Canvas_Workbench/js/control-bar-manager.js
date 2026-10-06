/**
 * Top Control Bar "Socket-Case" Pods Manager
 * Replaces the overflowing 30-button horizontal control bar with compact,
 * zero-overflow Socket-Case Pods with frosted glass drop-trays:
 * - 🌿 Strata Pod: [ 🌿 Strata: All (5) ▾ ] -> Multi-select Strata tray
 * - 🔬 Epistemic Lens Pod: [ 🔬 Lens: All Modalities ▾ ] -> Epistemic modality tray
 * - ⏱️ Dimension & 4D Time Pod: [ ⏱️ 2D · 14:40 (24h) ▾ ] -> 2D/3D/4D & temporal slider tray
 */

import { canvasState } from './state.js';

export class ControlBarManager {
  constructor(containerEl) {
    this.container = containerEl;
    this.activeTray = null;
    this.init();
  }

  init() {
    this.render();
    this.bindEvents();
    this.subscribeState();
  }

  render() {
    const s = canvasState.state;
    const activeStrataCount = s.activeStrata.includes('all') ? 5 : s.activeStrata.length;
    const strataLabel = s.activeStrata.includes('all') ? 'All (5)' : `${activeStrataCount} Active`;

    const epistemicIcons = {
      all: '🌐 All',
      objective: '🔬 Objective',
      subjective: '🧠 Subjective',
      intersubjective: '🌊 Shared',
      normative: '⚖️ Normative'
    };
    const epistemicLabel = epistemicIcons[s.activeEpistemicLens] || '🌐 All';

    this.container.innerHTML = `
      <!-- 1. Virtual Desktops / Manifold Pager -->
      <div id="virtual-desktop-container"></div>

      <!-- 2. Manifold Title & Graph ID -->
      <div class="canvas-title-box">
        <input type="text" class="canvas-title-input" id="canvas-title" value="${s.title || 'Research & Epistemological Mindware Manifold'}" title="Click to rename manifold" />
        <span class="graph-address-badge">graph:canvas:research_epistemology_01</span>
      </div>

      <!-- 3. Top-Level Socket-Case Pods (Zero Overflow) -->
      <div class="top-control-pods-bar">
        <!-- Strata Pod -->
        <button class="top-pod-btn" id="btn-strata-pod" title="Filter by Social & Ecological Strata">
          <span class="pod-icon">🌿</span>
          <span class="pod-label">Strata:</span>
          <span class="pod-value" style="color: var(--accent-emerald);">${strataLabel}</span>
          <span class="pod-chevron">▾</span>
        </button>

        <!-- Epistemic Lens Pod -->
        <button class="top-pod-btn" id="btn-epistemic-pod" title="Filter by Epistemic Lens (Objective / Subjective Qualia)">
          <span class="pod-icon">🔬</span>
          <span class="pod-label">Lens:</span>
          <span class="pod-value" style="color: var(--accent-cyan);">${epistemicLabel}</span>
          <span class="pod-chevron">▾</span>
        </button>

        <!-- Dimension & 4D Spatiotemporal Time Pod -->
        <button class="top-pod-btn" id="btn-time-dim-pod" title="Spatial Dimension (2D/3D/4D) & Time Span">
          <span class="pod-icon">⏱️</span>
          <span class="pod-label">${s.dimensionMode || '2D'}</span>
          <span class="pod-value" style="color: var(--accent-amber);">${s.time?.spanPreset || '24h'}</span>
          <span class="pod-chevron">▾</span>
        </button>
      </div>

      <!-- Action Buttons -->
      <div class="top-actions-shelf" style="display: flex; gap: 6px; margin-left: auto;">
        <button class="btn" id="btn-toggle-a11y" style="background: var(--surface-panel); border: 1px solid var(--border-medium); color: var(--text-primary); font-size: 11px; padding: 4px 8px; border-radius: var(--radius-sm); cursor: pointer;">
          <span>♿</span> SME | a11y
        </button>
        <button class="btn" id="btn-toggle-tech-sidebar" style="background: var(--surface-panel); border: 1px solid var(--border-medium); color: var(--text-primary); font-size: 11px; padding: 4px 8px; border-radius: var(--radius-sm); cursor: pointer;">
          <span>⚙️</span> Telemetry & DAG
        </button>
      </div>

      <!-- Floating Frosted-Glass Drop-Trays for Top Pods -->
      <div class="top-pod-drop-tray hidden" id="top-pod-drop-tray"></div>
    `;

    this.dropTray = this.container.querySelector('#top-pod-drop-tray');
  }

  bindEvents() {
    // Strata Pod click
    this.container.querySelector('#btn-strata-pod')?.addEventListener('click', (e) => {
      e.stopPropagation();
      this.togglePodTray('strata', this.container.querySelector('#btn-strata-pod'));
    });

    // Epistemic Pod click
    this.container.querySelector('#btn-epistemic-pod')?.addEventListener('click', (e) => {
      e.stopPropagation();
      this.togglePodTray('epistemic', this.container.querySelector('#btn-epistemic-pod'));
    });

    // Dimension & Time Pod click
    this.container.querySelector('#btn-time-dim-pod')?.addEventListener('click', (e) => {
      e.stopPropagation();
      this.togglePodTray('time_dim', this.container.querySelector('#btn-time-dim-pod'));
    });

    // Telemetry button
    this.container.querySelector('#btn-toggle-tech-sidebar')?.addEventListener('click', () => {
      canvasState.toggleTechSidebar();
    });

    // Title rename input
    const titleInput = this.container.querySelector('#canvas-title');
    titleInput?.addEventListener('change', (e) => {
      canvasState.updateCanvasTitle(e.target.value);
    });

    // Close drop-tray on clicking outside
    document.addEventListener('click', (e) => {
      if (this.dropTray && !this.container.contains(e.target)) {
        this.closeTray();
      }
    });
  }

  togglePodTray(type, anchorBtn) {
    if (this.activeTray === type && !this.dropTray.classList.contains('hidden')) {
      this.closeTray();
      return;
    }

    this.activeTray = type;
    this.container.querySelectorAll('.top-pod-btn').forEach(b => b.classList.remove('active'));
    anchorBtn.classList.add('active');

    const btnRect = anchorBtn.getBoundingClientRect();
    const containerRect = this.container.getBoundingClientRect();
    const leftOffset = Math.max(12, btnRect.left - containerRect.left);

    this.dropTray.style.left = `${leftOffset}px`;

    if (type === 'strata') {
      this.renderStrataTray();
    } else if (type === 'epistemic') {
      this.renderEpistemicTray();
    } else if (type === 'time_dim') {
      this.renderTimeDimTray();
    }

    this.dropTray.classList.remove('hidden');
  }

  renderStrataTray() {
    const s = canvasState.state;
    const isAll = s.activeStrata.includes('all');

    this.dropTray.innerHTML = `
      <div class="pod-tray-header">
        <span>🌿 <strong>Social & Ecological Strata Deck</strong></span>
        <button class="pod-tray-close-btn" id="btn-close-pod-tray">✕</button>
      </div>
      <div class="pod-tray-body">
        <div style="font-size: 11px; color: var(--text-muted); margin-bottom: 6px;">Select active strata planes to visualize on the manifold:</div>
        <div style="display: flex; flex-wrap: wrap; gap: 6px;">
          <button class="strata-deck-btn ${isAll ? 'active' : ''}" data-strata="all">All Strata</button>
          <button class="strata-deck-btn ${s.activeStrata.includes('environmental') || isAll ? 'active' : ''}" data-strata="environmental">🌿 Environmental</button>
          <button class="strata-deck-btn ${s.activeStrata.includes('social') || isAll ? 'active' : ''}" data-strata="social">👥 Social</button>
          <button class="strata-deck-btn ${s.activeStrata.includes('legal') || isAll ? 'active' : ''}" data-strata="legal">⚖️ Legal / Fiduciary</button>
          <button class="strata-deck-btn ${s.activeStrata.includes('financial') || isAll ? 'active' : ''}" data-strata="financial">💰 Financial</button>
          <button class="strata-deck-btn ${s.activeStrata.includes('technical') || isAll ? 'active' : ''}" data-strata="technical">⚙️ Technical</button>
        </div>
      </div>
    `;

    this.dropTray.querySelector('#btn-close-pod-tray')?.addEventListener('click', () => this.closeTray());

    this.dropTray.querySelectorAll('.strata-deck-btn').forEach(btn => {
      btn.addEventListener('click', (e) => {
        e.stopPropagation();
        const strata = btn.getAttribute('data-strata');
        canvasState.toggleStrata(strata);
        this.renderStrataTray();
        this.updatePodLabels();
      });
    });
  }

  renderEpistemicTray() {
    const s = canvasState.state;
    const current = s.activeEpistemicLens;

    this.dropTray.innerHTML = `
      <div class="pod-tray-header">
        <span>🔬 <strong>Epistemic Modality Lens</strong></span>
        <button class="pod-tray-close-btn" id="btn-close-pod-tray">✕</button>
      </div>
      <div class="pod-tray-body">
        <div style="font-size: 11px; color: var(--text-muted); margin-bottom: 6px;">Filter knowledge containers by epistemological grounding:</div>
        <div style="display: flex; flex-direction: column; gap: 4px;">
          <button class="epistemic-btn ${current === 'all' ? 'active' : ''}" data-epistemic="all" style="justify-content: flex-start; padding: 5px 8px;">
            🌐 <strong>All Modalities</strong> (Show full knowledge ecosystem)
          </button>
          <button class="epistemic-btn ${current === 'objective' ? 'active' : ''}" data-epistemic="objective" style="justify-content: flex-start; padding: 5px 8px; color: var(--modality-objective);">
            🔬 <strong>Objective Telemetry</strong> (Empirical sensors, flow meters, facts)
          </button>
          <button class="epistemic-btn ${current === 'subjective' ? 'active' : ''}" data-epistemic="subjective" style="justify-content: flex-start; padding: 5px 8px; color: var(--modality-subjective);">
            🧠 <strong>Subjective Qualia</strong> (First-person oral testimony, lived experience)
          </button>
          <button class="epistemic-btn ${current === 'intersubjective' ? 'active' : ''}" data-epistemic="intersubjective" style="justify-content: flex-start; padding: 5px 8px; color: var(--modality-intersubjective);">
            🌊 <strong>Intersubjective Consensus</strong> (Cultural agreements, shared terms)
          </button>
          <button class="epistemic-btn ${current === 'normative' ? 'active' : ''}" data-epistemic="normative" style="justify-content: flex-start; padding: 5px 8px; color: var(--modality-normative);">
            ⚖️ <strong>Normative Statutes</strong> (Statutory duty-of-care, legal mandates)
          </button>
        </div>
      </div>
    `;

    this.dropTray.querySelector('#btn-close-pod-tray')?.addEventListener('click', () => this.closeTray());

    this.dropTray.querySelectorAll('.epistemic-btn').forEach(btn => {
      btn.addEventListener('click', (e) => {
        e.stopPropagation();
        const ep = btn.getAttribute('data-epistemic');
        canvasState.setEpistemicLens(ep);
        this.renderEpistemicTray();
        this.updatePodLabels();
      });
    });
  }

  renderTimeDimTray() {
    const s = canvasState.state;
    const currentPreset = s.time?.spanPreset || '24h';

    this.dropTray.innerHTML = `
      <div class="pod-tray-header">
        <span>⏱️ <strong>Spatial Dimension & 4D Spatiotemporal Time</strong></span>
        <button class="pod-tray-close-btn" id="btn-close-pod-tray">✕</button>
      </div>
      <div class="pod-tray-body">
        <!-- 1. Spatial Dimension Switch -->
        <div style="display: flex; gap: 6px; align-items: center; margin-bottom: 10px;">
          <span style="font-size: 11px; color: var(--text-muted);">Dimension:</span>
          <button class="dim-btn ${s.dimensionMode === '2D' ? 'active' : ''}" data-dim="2D">2D Flat</button>
          <button class="dim-btn ${s.dimensionMode === '3D' ? 'active' : ''}" data-dim="3D">3D Orbit</button>
          <button class="dim-btn ${s.dimensionMode === '4D' ? 'active' : ''}" data-dim="4D">4D Matrix</button>
        </div>

        <!-- 2. Temporal Time Scrubber -->
        <div style="font-size: 11px; color: var(--text-muted); margin-bottom: 4px;">4D Spatiotemporal Time Span:</div>
        <div style="display: flex; align-items: center; gap: 8px; background: var(--surface-base); padding: 6px 10px; border-radius: var(--radius-sm); border: 1px solid var(--border-subtle);">
          <span class="datetime-badge">08-15 00:00</span>
          <button class="play-pause-btn" id="tray-play-time">▶</button>
          <input type="range" class="time-slider" id="tray-time-slider" min="0" max="100" value="61" style="flex: 1;" />
          <span class="datetime-badge active-time" id="tray-current-time">14:40:00</span>
          <span class="datetime-badge">08-15 23:59</span>
        </div>

        <!-- 3. Span Presets -->
        <div style="display: flex; gap: 6px; margin-top: 8px; align-items: center;">
          <span style="font-size: 11px; color: var(--text-muted);">Span Presets:</span>
          <button class="span-preset-btn ${currentPreset === '1h' ? 'active' : ''}" data-span="1h">1h</button>
          <button class="span-preset-btn ${currentPreset === '24h' ? 'active' : ''}" data-span="24h">24h</button>
          <button class="span-preset-btn ${currentPreset === '7d' ? 'active' : ''}" data-span="7d">7d</button>
          <button class="span-preset-btn ${currentPreset === 'season' ? 'active' : ''}" data-span="season">Season</button>
        </div>
      </div>
    `;

    this.dropTray.querySelector('#btn-close-pod-tray')?.addEventListener('click', () => this.closeTray());

    // Dimension buttons
    this.dropTray.querySelectorAll('.dim-btn').forEach(btn => {
      btn.addEventListener('click', () => {
        const dim = btn.getAttribute('data-dim');
        canvasState.setDimensionMode(dim);
        this.renderTimeDimTray();
        this.updatePodLabels();
      });
    });

    // Span preset buttons
    this.dropTray.querySelectorAll('.span-preset-btn').forEach(btn => {
      btn.addEventListener('click', () => {
        const span = btn.getAttribute('data-span');
        canvasState.setTimeSpanPreset(span);
        this.renderTimeDimTray();
        this.updatePodLabels();
      });
    });

    // Time slider
    const slider = this.dropTray.querySelector('#tray-time-slider');
    slider?.addEventListener('input', (e) => {
      const val = parseInt(e.target.value, 10);
      const hours = Math.floor((val / 100) * 24);
      const mins = Math.floor(((val / 100) * 24 - hours) * 60);
      const timeStr = `${String(hours).padStart(2, '0')}:${String(mins).padStart(2, '0')}:00`;
      const timeDisplay = this.dropTray.querySelector('#tray-current-time');
      if (timeDisplay) timeDisplay.innerText = timeStr;
    });

    // Play/Pause button
    const playBtn = this.dropTray.querySelector('#tray-play-time');
    let isPlaying = false;
    let timer = null;
    playBtn?.addEventListener('click', () => {
      isPlaying = !isPlaying;
      playBtn.innerText = isPlaying ? '⏸' : '▶';
      if (isPlaying) {
        timer = setInterval(() => {
          if (!slider) return;
          let v = (parseInt(slider.value, 10) + 1) % 100;
          slider.value = v;
          slider.dispatchEvent(new Event('input'));
        }, 200);
      } else {
        clearInterval(timer);
      }
    });
  }

  updatePodLabels() {
    const s = canvasState.state;
    const activeStrataCount = s.activeStrata.includes('all') ? 5 : s.activeStrata.length;
    const strataLabel = s.activeStrata.includes('all') ? 'All (5)' : `${activeStrataCount} Active`;
    const strataValEl = this.container.querySelector('#btn-strata-pod .pod-value');
    if (strataValEl) strataValEl.innerText = strataLabel;

    const epistemicIcons = {
      all: '🌐 All',
      objective: '🔬 Objective',
      subjective: '🧠 Subjective',
      intersubjective: '🌊 Shared',
      normative: '⚖️ Normative'
    };
    const epistemicValEl = this.container.querySelector('#btn-epistemic-pod .pod-value');
    if (epistemicValEl) epistemicValEl.innerText = epistemicIcons[s.activeEpistemicLens] || '🌐 All';

    const timeDimValEl = this.container.querySelector('#btn-time-dim-pod .pod-value');
    const timeDimLabelEl = this.container.querySelector('#btn-time-dim-pod .pod-label');
    if (timeDimValEl) timeDimValEl.innerText = s.time?.spanPreset || '24h';
    if (timeDimLabelEl) timeDimLabelEl.innerText = s.dimensionMode || '2D';
  }

  closeTray() {
    this.activeTray = null;
    this.container.querySelectorAll('.top-pod-btn').forEach(b => b.classList.remove('active'));
    if (this.dropTray) this.dropTray.classList.add('hidden');
  }

  subscribeState() {
    canvasState.subscribe('stateChanged', () => {
      this.updatePodLabels();
    });
  }
}


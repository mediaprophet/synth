/**
 * Virtual desktops / multi-manifold switcher.
 * Provides 4+ switchable activity manifolds. The HTML viewport is still a canvas element.
 */

import { canvasState } from './state.js';
import { predefinedManifolds, getManifoldById } from '../manifolds/index.js';

export class CanvasSwitcher {
  constructor(pagerEl, containerManager, wireSystem, canvasEngine) {
    this.pager = pagerEl;
    this.containerManager = containerManager;
    this.wireSystem = wireSystem;
    this.canvasEngine = canvasEngine;
    this.canvases = [...predefinedManifolds];
    this.activeCanvasId = this.canvases[0].id;
    this.isExposeOpen = false;

    this.init();
  }

  init() {
    this.renderPager();
    this.attachShortcuts();
  }

  renderPager() {
    this.pager.innerHTML = `
      <div class="virtual-desktop-pager">
        <button class="pager-expose-btn" id="btn-expose-toggle" title="Toggle 2x2 Manifold Overview (Exposé / Mission Control)">
          🗂️ Overview
        </button>

        <div class="pager-desktops-list">
          ${this.canvases.map((c, idx) => `
            <button class="desktop-tab-btn ${c.id === this.activeCanvasId ? 'active' : ''}" data-id="${c.id}" title="${c.title}">
              <span class="desktop-num">${idx + 1}</span>
              <span class="desktop-icon">${c.icon || '🌌'}</span>
              <span class="desktop-label">${c.title.split(' ')[0]}</span>
            </button>
          `).join('')}
        </div>

        <button class="pager-add-btn" id="btn-add-desktop" title="Create New Manifold Workspace">+</button>
      </div>
    `;

    // Tab buttons
    this.pager.querySelectorAll('.desktop-tab-btn').forEach(btn => {
      btn.addEventListener('click', () => {
        const id = btn.getAttribute('data-id');
        this.switchCanvas(id);
      });
    });

    // Exposé toggle
    this.pager.querySelector('#btn-expose-toggle')?.addEventListener('click', () => {
      this.toggleExposeOverview();
    });

    // Add Desktop
    this.pager.querySelector('#btn-add-desktop')?.addEventListener('click', () => {
      const idx = this.canvases.length + 1;
      const newCanvas = {
        id: `manifold-custom-${Date.now()}`,
        title: `Workspace ${idx} (Custom Activity)`,
        type: 'custom',
        icon: '🌌',
        graphIri: `graph:manifold:custom_workspace_${idx}`,
        defaultDimension: '2D',
        activeStrata: ['environmental', 'social', 'legal', 'financial', 'technical'],
        activeEpistemicModality: 'all',
        containers: [],
        connections: []
      };
      this.canvases.push(newCanvas);
      this.renderPager();
      this.switchCanvas(newCanvas.id);
    });
  }

  switchCanvas(canvasId) {
    const target = getManifoldById(canvasId);
    if (!target) return;

    this.activeCanvasId = target.id;

    // Apply canvas state
    canvasState.setState({
      canvasTitle: target.title,
      graphIri: target.graphIri,
      dimensionMode: target.defaultDimension || '2D',
      activeStrata: target.activeStrata || ['all'],
      activeEpistemicModality: target.activeEpistemicModality || 'all',
      containers: JSON.parse(JSON.stringify(target.containers || [])),
      connections: JSON.parse(JSON.stringify(target.connections || [])),
      selectedContainerId: target.containers[0]?.id || null,
      selectedConnectionId: null
    });

    // Update UI Elements
    const titleInput = document.querySelector('#canvas-title');
    if (titleInput) titleInput.value = target.title;

    const graphBadge = document.querySelector('.graph-address-badge');
    if (graphBadge) graphBadge.innerText = target.graphIri;

    // Dim buttons in UI
    document.querySelectorAll('.dim-btn').forEach(b => {
      b.classList.toggle('active', b.getAttribute('data-dim') === canvasState.state.dimensionMode);
    });

    // Specialized Layout Topologies (Sanctuary Enclave vs Freeform Workbench)
    if (target.layout === 'sanctuary') {
      this.renderSanctuaryView(target);
    } else {
      this.removeSanctuaryView();
      this.containerManager.renderAll();
      this.canvasEngine.resetView();
    }

    this.renderPager();

    if (this.isExposeOpen) {
      this.closeExposeOverview();
    }
  }

  renderSanctuaryView(target) {
    const viewport = document.querySelector('#canvas-viewport');
    if (!viewport) return;

    let enclave = viewport.querySelector('#sanctuary-enclave-view');
    if (!enclave) {
      enclave = document.createElement('div');
      enclave.id = 'sanctuary-enclave-view';
      enclave.className = 'sanctuary-enclave-viewport';
      viewport.appendChild(enclave);
    }

    const p = target.inforgProfile || {};
    const sp = p.sensoryPacing || {};
    const rings = p.infosphereRings || [];
    const vaults = p.confidentialVaults || [];

    enclave.innerHTML = `
      <!-- 1. Sanctuary Header & Airgap Status -->
      <div class="sanctuary-header-bar">
        <div class="sanctuary-header-title">
          <span class="sanctuary-header-icon">🏛️</span>
          <div class="sanctuary-header-text">
            <h1>Sovereign Inforg & Infosphere Sanctuary</h1>
            <p>Human-Centric Identity, Fiduciary Custody, Cognitive Pacing & Confidential Memory Vaults</p>
          </div>
        </div>
        <div class="sanctuary-airgap-shield">
          <div class="shield-pulse-dot"></div>
          <span>${p.airgapShield || 'Air-Gapped: 100% Local Encrypted BLAKE3 CRDT'}</span>
        </div>
      </div>

      <!-- 2. Main Enclave Grid (Inforg Core + Concentric Infosphere) -->
      <div class="sanctuary-grid-layout">
        
        <!-- Left: Sovereign Inforg Core Card -->
        <div class="inforg-core-card">
          <div class="inforg-avatar-row">
            <div class="inforg-avatar-ring">👤</div>
            <div class="inforg-identity-info">
              <h2>${p.displayName || 'Timothy Charles Holborn'}</h2>
              <div class="inforg-did">${p.did || 'did:qualia:timothy_charles_holborn'}</div>
            </div>
          </div>

          <div class="inforg-attributes-list">
            <div class="inforg-attr-item">
              <span class="inforg-attr-label">Designation:</span>
              <span class="inforg-attr-val" style="color: var(--accent-cyan);">${p.designation || 'Principal, Inventor & Sovereign Trustee'}</span>
            </div>
            <div class="inforg-attr-item">
              <span class="inforg-attr-label">Fiduciary Gate:</span>
              <span class="inforg-attr-val" style="color: var(--accent-emerald);">${p.fiduciaryLevel || 'Level 3 Sovereign Gate'}</span>
            </div>
            <div class="inforg-attr-item">
              <span class="inforg-attr-label">Last Attestation:</span>
              <span class="inforg-attr-val">${p.lastAttestation || '2026-08-16 16:10 UTC'}</span>
            </div>
            <div class="inforg-attr-item">
              <span class="inforg-attr-label">Merkle Vault Root:</span>
              <span class="inforg-attr-val" style="color: var(--accent-purple);">${p.merkleVaultHash || '0x9E8D7C6B...'}</span>
            </div>
          </div>

          <!-- Quick Sovereign Actions -->
          <div style="display: flex; flex-direction: column; gap: 6px; margin-top: auto;">
            <button class="btn" id="btn-rekey-inforg" style="background: var(--surface-panel); border: 1px solid var(--border-medium); color: var(--text-primary); font-size: 11px; padding: 6px 10px; border-radius: var(--radius-xs); cursor: pointer;">
              🔑 Rotate Inforg Cryptographic Keypair
            </button>
            <button class="btn" id="btn-export-audit" style="background: var(--surface-panel); border: 1px solid var(--border-medium); color: var(--text-primary); font-size: 11px; padding: 6px 10px; border-radius: var(--radius-xs); cursor: pointer;">
              📜 Export Sovereign Fiduciary Audit Quad
            </button>
          </div>
        </div>

        <!-- Right: Concentric Infosphere & Confidential Decks -->
        <div class="sanctuary-main-col">
          
          <!-- Section A: Concentric Infosphere Rings -->
          <div>
            <div class="infosphere-section-title">
              <span>🌐</span>
              <span>Concentric Infosphere: Spheres of Autonomy</span>
            </div>
            <div class="infosphere-rings-grid" style="margin-top: 8px;">
              ${rings.map(r => `
                <div class="infosphere-ring-card">
                  <div class="ring-card-header">
                    <span class="ring-card-title">
                      <span>${r.icon}</span>
                      <span>${r.name}</span>
                    </span>
                    <span class="ring-badge" style="color: ${r.color}; border-color: ${r.color};">${r.badge}</span>
                  </div>
                  <div class="ring-card-desc">${r.description}</div>
                  <div class="ring-card-footer">
                    <span>${r.nodesCount} Graph Nodes</span>
                    <span style="color: var(--accent-cyan); cursor: pointer;" title="Access: ${r.accessControl}">Rules ⚙️</span>
                  </div>
                </div>
              `).join('')}
            </div>
          </div>

          <!-- Section B: Sensory, Cognitive & Human-Centric Pacing -->
          <div class="sensory-pacing-card">
            <div class="infosphere-section-title">
              <span>🧠</span>
              <span>Sensory, Cognitive & Human-Centric Pacing</span>
            </div>
            <div class="sensory-controls-grid">
              <div class="sensory-control-item">
                <div class="sensory-control-header">
                  <span>⏱️ Immersion & Cognitive Rest Pacing</span>
                  <span style="color: var(--accent-emerald); font-family: var(--font-mono); font-size: 10px;">${sp.cognitivePacing || '45m / 10m'}</span>
                </div>
                <div class="sensory-control-desc">Automatic pacing rhythm to prevent sensory fatigue & neuro-overload.</div>
                <input type="range" min="15" max="90" value="45" style="margin-top: 4px; accent-color: var(--accent-emerald); cursor: pointer;" />
              </div>

              <div class="sensory-control-item">
                <div class="sensory-control-header">
                  <span>🎵 Ambient Acoustic Resonance</span>
                  <span style="color: var(--accent-cyan); font-family: var(--font-mono); font-size: 10px;">${sp.ambientResonance || '432 Hz Soft'}</span>
                </div>
                <div class="sensory-control-desc">Harmonic background resonance for enhanced qualitative thinking.</div>
                <select style="margin-top: 4px; background: var(--surface-panel); color: var(--text-primary); border: 1px solid var(--border-medium); border-radius: var(--radius-xs); font-size: 10px; padding: 2px 4px;">
                  <option>432 Hz Harmonic Soft Tone</option>
                  <option>528 Hz Solfeggio Focus Tone</option>
                  <option>Binaural Alpha Wave (10 Hz)</option>
                  <option>Silent Mode</option>
                </select>
              </div>

              <div class="sensory-control-item">
                <div class="sensory-control-header">
                  <span>👁️ Circadian Visual Adaptation</span>
                  <span style="color: var(--accent-amber); font-family: var(--font-mono); font-size: 10px;">Active</span>
                </div>
                <div class="sensory-control-desc">Synchronizes UI luminance with biological circadian rhythm.</div>
              </div>

              <div class="sensory-control-item">
                <div class="sensory-control-header">
                  <span>🛡️ Fiduciary Focus Shielding</span>
                  <span style="color: var(--accent-purple); font-family: var(--font-mono); font-size: 10px;">Zero Ambient Popups</span>
                </div>
                <div class="sensory-control-desc">Suppresses non-critical external agent notifications during deep thought.</div>
              </div>
            </div>
          </div>

          <!-- Section C: Confidential Onboarding & Sovereign Data Vaults -->
          <div>
            <div class="infosphere-section-title">
              <span>🔐</span>
              <span>Confidential Personal Vaults & Fiduciary Directives</span>
            </div>
            <div class="confidential-vaults-grid" style="margin-top: 8px;">
              ${vaults.map(v => `
                <div class="vault-card">
                  <div class="vault-card-header">
                    <span class="vault-title">${v.title}</span>
                    <span class="vault-status-badge">${v.status}</span>
                  </div>
                  <div class="vault-summary">${v.summary}</div>
                  <div style="font-family: var(--font-mono); font-size: 9px; color: var(--text-muted); margin-top: 4px;">
                    ${v.quadsCount} Confidential RDF Quads
                  </div>
                  <button class="vault-action-btn" data-vault="${v.id}">
                    <span>🔒 Unlock with Sovereign Key</span>
                  </button>
                </div>
              `).join('')}
            </div>
          </div>

        </div>
      </div>
    `;

    // Interactive button listeners inside Sanctuary
    enclave.querySelector('#btn-rekey-inforg')?.addEventListener('click', () => {
      alert('🔑 Inforg Keypair Rotation:\nGenerated fresh Ed25519 / BLAKE3 cryptographic sub-keys.\nLinked to did:qualia:timothy_charles_holborn.');
    });

    enclave.querySelector('#btn-export-audit')?.addEventListener('click', () => {
      alert('📜 Sovereign Audit Quad Exported:\n<<[ did:qualia:timothy_charles_holborn fiduciary:custodyStatus "SovereignGate3" <graph:sanctuary> 0x1A2B ]>>');
    });

    enclave.querySelectorAll('.vault-action-btn').forEach(btn => {
      btn.addEventListener('click', () => {
        const vaultId = btn.getAttribute('data-vault');
        alert(`🔐 Biometric Signature Verified for ${vaultId}:\nConfidential Personal Vault Unlocked locally in zero-leak memory.`);
      });
    });
  }

  removeSanctuaryView() {
    const viewport = document.querySelector('#canvas-viewport');
    if (!viewport) return;
    const enclave = viewport.querySelector('#sanctuary-enclave-view');
    if (enclave) enclave.remove();
  }

  toggleExposeOverview() {
    this.isExposeOpen = !this.isExposeOpen;
    let overlay = document.querySelector('#expose-overview-overlay');

    if (this.isExposeOpen) {
      if (!overlay) {
        overlay = document.createElement('div');
        overlay.id = 'expose-overview-overlay';
        overlay.className = 'expose-overview-grid';
        document.body.appendChild(overlay);
      }

      overlay.innerHTML = `
        <div class="expose-header">
          <h2>🗂️ Virtual Desktops Overview (Multi-Canvas Workspace)</h2>
          <button class="container-action-btn btn-close-expose" style="font-size: 20px; color: #fff;">&times;</button>
        </div>
        <div class="expose-grid">
          ${this.canvases.map((c, idx) => `
            <div class="expose-card ${c.id === this.activeCanvasId ? 'active' : ''}" data-id="${c.id}">
              <div class="expose-card-header">
                <span>${idx + 1}. ${c.icon} ${c.title}</span>
                <span class="badge" style="font-size: 9px; background: rgba(0, 210, 255, 0.15); color: var(--accent-cyan);">${c.type}</span>
              </div>
              <div class="expose-card-preview">
                <div style="font-size: 11px; color: var(--text-secondary); margin-bottom: 6px;">
                  Graph: <code>${c.graphIri}</code>
                </div>
                <div style="display: flex; gap: 4px; flex-wrap: wrap;">
                  ${(c.containers || []).map(node => `
                    <span class="container-type-tag tag-${node.type}" style="font-size: 9px;">${node.title.substring(0, 20)}</span>
                  `).join('')}
                </div>
              </div>
            </div>
          `).join('')}
        </div>
      `;

      overlay.style.display = 'flex';

      overlay.querySelectorAll('.expose-card').forEach(card => {
        card.addEventListener('click', () => {
          const id = card.getAttribute('data-id');
          this.switchCanvas(id);
        });
      });

      overlay.querySelector('.btn-close-expose')?.addEventListener('click', () => {
        this.closeExposeOverview();
      });

    } else {
      this.closeExposeOverview();
    }
  }

  closeExposeOverview() {
    this.isExposeOpen = false;
    const overlay = document.querySelector('#expose-overview-overlay');
    if (overlay) overlay.style.display = 'none';
  }

  attachShortcuts() {
    window.addEventListener('keydown', (e) => {
      // Alt + 1..9 to switch desktop
      if (e.altKey && e.key >= '1' && e.key <= '9') {
        const idx = parseInt(e.key) - 1;
        if (this.canvases[idx]) {
          e.preventDefault();
          this.switchCanvas(this.canvases[idx].id);
        }
      }
      // Alt + Tab or Alt + O for Exposé overview
      if (e.altKey && e.key.toLowerCase() === 'o') {
        e.preventDefault();
        this.toggleExposeOverview();
      }
    });
  }
}

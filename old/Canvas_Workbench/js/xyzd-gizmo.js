/**
 * HyperCanvas: 3D CAD-Style Trackball & XYZD Spatial Manifold Gizmo
 * ─────────────────────────────────────────────────────────────────
 * Dual-mode gizmo:
 *   • ORBIT mode  — rotates the camera/canvas view
 *   • POSITION mode — moves the selected container's x/y/z/d in canvas space
 *
 * Professional 3D orientation sphere with projected axis arrows,
 * arcball rotation, momentum/inertia, touch support and snap-to-view.
 * Inspired by Blender/Maya/Fusion 360 viewport gizmos.
 */

import { canvasState } from './state.js';

const TAU   = Math.PI * 2;
const DEG   = 180 / Math.PI;
const RAD   = Math.PI / 180;
const CLAMP = (v, lo, hi) => Math.max(lo, Math.min(hi, v));

// ─── Rotation matrix helpers ──────────────────────────────────────────
function rotatePoint3D(px, py, pz, rx, ry) {
  const cosY = Math.cos(ry * RAD), sinY = Math.sin(ry * RAD);
  const cosX = Math.cos(rx * RAD), sinX = Math.sin(rx * RAD);
  let x = px * cosY + pz * sinY;
  let z = -px * sinY + pz * cosY;
  let y = py;
  const y2 = y * cosX - z * sinX;
  const z2 = y * sinX + z * cosX;
  return { x, y: y2, z: z2 };
}

export class XYZDGizmo {
  constructor(containerEl, canvasEngine) {
    this.container = containerEl;
    this.canvasEngine = canvasEngine;

    // Orientation angles (degrees) — for Orbit mode
    this.rotation = { x: 15, y: -25, z: 0 };

    // Interaction state
    this.isDragging  = false;
    this.dragStart   = { x: 0, y: 0 };
    this.startRot    = { x: 0, y: 0 };
    this.startPos    = { x: 0, y: 0 };
    this.velocity    = { x: 0, y: 0 };
    this.momentumRAF = null;

    // Mode: 'orbit' or 'position'
    this.mode = 'orbit';

    this.init();
  }

  init() {
    this.render();
    this.attachEvents();

    canvasState.subscribe('xyzdChanged', () => this.updateGizmoDisplay());
    canvasState.subscribe('dimensionModeChanged', (mode) => {
      this.container.classList.toggle('active-3d', mode === '3D' || mode === '4D');
    });
    // When a container is selected, refresh the position panel
    canvasState.subscribe('containerSelected', () => this._syncPositionPanel());
    canvasState.subscribe('containerMoved', () => this._syncPositionPanel());
  }

  // ─── Get the currently selected container data ──────────────────
  _getSelectedContainer() {
    const id = canvasState.state.selectedContainerId;
    if (!id) return null;
    return canvasState.state.containers.find(c => c.id === id) || null;
  }

  // ─── Build the DOM ───────────────────────────────────────────────
  render() {
    this.container.innerHTML = `
      <div class="gizmo-card">
        <!-- Mode Toggle Header -->
        <div class="gizmo-header">
          <div class="gizmo-mode-toggle">
            <button class="gizmo-mode-btn active" data-mode="orbit" title="Orbit Camera">🎥 Orbit</button>
            <button class="gizmo-mode-btn" data-mode="position" title="Position Selected Container">📦 Position</button>
          </div>
        </div>

        <!-- Orbit Mode Quick Views (hidden in position mode) -->
        <div class="gizmo-quick-views-bar" id="gizmo-orbit-views">
          <button class="gizmo-view-btn" data-view="iso" title="Isometric">ISO</button>
          <button class="gizmo-view-btn" data-view="top" title="Top (+Y)">⬆</button>
          <button class="gizmo-view-btn" data-view="front" title="Front (+Z)">⬤</button>
          <button class="gizmo-view-btn" data-view="right" title="Right (+X)">⬤</button>
          <button class="gizmo-view-btn" data-view="reset" title="Reset">↺</button>
        </div>

        <!-- 3D Trackball Viewport -->
        <div class="gizmo-viewport" id="gizmo-trackball-viewport"
             title="Drag to orbit · Scroll to adjust D-scale · Click axis label to snap">
          <svg class="gizmo-svg" viewBox="-60 -60 120 120" xmlns="http://www.w3.org/2000/svg">
            <defs>
              <radialGradient id="sphere-grad" cx="40%" cy="35%" r="55%">
                <stop offset="0%"  stop-color="rgba(65, 85, 120, 0.85)"/>
                <stop offset="60%" stop-color="rgba(20, 28, 42, 0.92)"/>
                <stop offset="100%" stop-color="rgba(6, 8, 14, 0.97)"/>
              </radialGradient>
              <radialGradient id="sphere-highlight" cx="38%" cy="32%" r="45%">
                <stop offset="0%" stop-color="rgba(140,180,255,0.25)"/>
                <stop offset="100%" stop-color="rgba(0,0,0,0)"/>
              </radialGradient>
              <filter id="glow-x" x="-50%" y="-50%" width="200%" height="200%">
                <feGaussianBlur stdDeviation="2.5" result="blur"/>
                <feMerge><feMergeNode in="blur"/><feMergeNode in="SourceGraphic"/></feMerge>
              </filter>
              <filter id="glow-y" x="-50%" y="-50%" width="200%" height="200%">
                <feGaussianBlur stdDeviation="2.5" result="blur"/>
                <feMerge><feMergeNode in="blur"/><feMergeNode in="SourceGraphic"/></feMerge>
              </filter>
              <filter id="glow-z" x="-50%" y="-50%" width="200%" height="200%">
                <feGaussianBlur stdDeviation="2.5" result="blur"/>
                <feMerge><feMergeNode in="blur"/><feMergeNode in="SourceGraphic"/></feMerge>
              </filter>
            </defs>

            <!-- Wireframe latitude/longitude lines -->
            <g class="gizmo-wireframe" opacity="0.12">
              <ellipse cx="0" cy="0" rx="40" ry="40" fill="none" stroke="#fff" stroke-width="0.5"/>
              <ellipse cx="0" cy="0" rx="40" ry="14" fill="none" stroke="#fff" stroke-width="0.4"/>
              <ellipse cx="0" cy="0" rx="14" ry="40" fill="none" stroke="#fff" stroke-width="0.4"/>
            </g>

            <!-- Sphere body -->
            <circle cx="0" cy="0" r="40" fill="url(#sphere-grad)" stroke="rgba(255,255,255,0.08)" stroke-width="0.8"/>
            <circle cx="0" cy="0" r="40" fill="url(#sphere-highlight)"/>

            <!-- D-Manifold outer ring (4th dimension) -->
            <circle cx="0" cy="0" r="52" fill="none"
                    stroke="var(--accent-violet, #a855f7)" stroke-width="1.2"
                    stroke-dasharray="4 3" opacity="0.55" class="gizmo-d-ring"/>

            <!-- Projected axis lines (updated dynamically) -->
            <g id="gizmo-axes-group">
              <!-- +X axis (red) -->
              <line id="gizmo-ax-x" x1="0" y1="0" x2="38" y2="0"
                    stroke="#ef4444" stroke-width="2" stroke-linecap="round" filter="url(#glow-x)"/>
              <circle id="gizmo-tip-x" cx="38" cy="0" r="7"
                      fill="#ef4444" stroke="#fff" stroke-width="0.6" class="gizmo-axis-tip" data-axis="x"/>
              <text id="gizmo-label-x" x="38" y="0" text-anchor="middle" dominant-baseline="central"
                    fill="#fff" font-size="7" font-weight="800" font-family="var(--font-mono, monospace)"
                    class="gizmo-axis-label" data-axis="x" pointer-events="all" cursor="pointer">X</text>

              <!-- -X axis -->
              <line id="gizmo-ax-nx" x1="0" y1="0" x2="-38" y2="0"
                    stroke="#ef4444" stroke-width="1" stroke-linecap="round" opacity="0.3"/>

              <!-- +Y axis (green) -->
              <line id="gizmo-ax-y" x1="0" y1="0" x2="0" y2="-38"
                    stroke="#22c55e" stroke-width="2" stroke-linecap="round" filter="url(#glow-y)"/>
              <circle id="gizmo-tip-y" cx="0" cy="-38" r="7"
                      fill="#22c55e" stroke="#fff" stroke-width="0.6" class="gizmo-axis-tip" data-axis="y"/>
              <text id="gizmo-label-y" x="0" y="-38" text-anchor="middle" dominant-baseline="central"
                    fill="#fff" font-size="7" font-weight="800" font-family="var(--font-mono, monospace)"
                    class="gizmo-axis-label" data-axis="y" pointer-events="all" cursor="pointer">Y</text>

              <!-- -Y axis -->
              <line id="gizmo-ax-ny" x1="0" y1="0" x2="0" y2="38"
                    stroke="#22c55e" stroke-width="1" stroke-linecap="round" opacity="0.3"/>

              <!-- +Z axis (cyan) -->
              <line id="gizmo-ax-z" x1="0" y1="0" x2="0" y2="0"
                    stroke="#06b6d4" stroke-width="2" stroke-linecap="round" filter="url(#glow-z)"/>
              <circle id="gizmo-tip-z" cx="0" cy="0" r="7"
                      fill="#06b6d4" stroke="#fff" stroke-width="0.6" class="gizmo-axis-tip" data-axis="z"/>
              <text id="gizmo-label-z" x="0" y="0" text-anchor="middle" dominant-baseline="central"
                    fill="#000" font-size="7" font-weight="800" font-family="var(--font-mono, monospace)"
                    class="gizmo-axis-label" data-axis="z" pointer-events="all" cursor="pointer">Z</text>

              <!-- -Z axis -->
              <line id="gizmo-ax-nz" x1="0" y1="0" x2="0" y2="0"
                    stroke="#06b6d4" stroke-width="1" stroke-linecap="round" opacity="0.3"/>

              <!-- Central pivot -->
              <circle cx="0" cy="0" r="3" fill="#fff" opacity="0.9">
                <animate attributeName="opacity" values="0.9;0.5;0.9" dur="3s" repeatCount="indefinite"/>
              </circle>
            </g>
          </svg>

          <!-- Mode indicator overlay -->
          <div class="gizmo-mode-indicator" id="gizmo-mode-indicator">ORBIT</div>
        </div>

        <!-- Orbit Mode Footer: telemetry + D slider -->
        <div class="gizmo-footer" id="gizmo-orbit-footer">
          <div class="gizmo-axis-telemetry">
            <span style="color: #ef4444;">X: <strong id="tele-x">15°</strong></span>
            <span style="color: #22c55e;">Y: <strong id="tele-y">-25°</strong></span>
            <span style="color: #06b6d4;">Z: <strong id="tele-z">0°</strong></span>
          </div>
          <div class="gizmo-d-control">
            <span style="color: var(--accent-violet, #a855f7); font-size: 10px; font-weight: 700;">D:</span>
            <input type="range" class="gizmo-d-slider" id="gizmo-d-slider"
                   min="50" max="250" value="100"
                   title="Adjust 4th-dimensional manifold aperture (D)" />
            <span class="gizmo-d-value" id="tele-d">1.0σ</span>
          </div>
        </div>

        <!-- Position Mode Footer: selected container name + XYZD numeric inputs -->
        <div class="gizmo-footer gizmo-position-footer" id="gizmo-position-footer" style="display: none;">
          <div class="gizmo-pos-target" id="gizmo-pos-target">
            <span class="gizmo-pos-target-label">No container selected</span>
          </div>
          <div class="gizmo-pos-fields">
            <div class="gizmo-pos-row">
              <label class="gizmo-pos-label" style="color: #ef4444;">X</label>
              <input type="number" class="gizmo-pos-input" id="gizmo-pos-x" step="10" value="0" />
              <span class="gizmo-pos-unit">px</span>
            </div>
            <div class="gizmo-pos-row">
              <label class="gizmo-pos-label" style="color: #22c55e;">Y</label>
              <input type="number" class="gizmo-pos-input" id="gizmo-pos-y" step="10" value="0" />
              <span class="gizmo-pos-unit">px</span>
            </div>
            <div class="gizmo-pos-row">
              <label class="gizmo-pos-label" style="color: #06b6d4;">Z</label>
              <input type="number" class="gizmo-pos-input" id="gizmo-pos-z" step="10" value="0" />
              <span class="gizmo-pos-unit">px</span>
            </div>
            <div class="gizmo-pos-row">
              <label class="gizmo-pos-label" style="color: var(--accent-violet, #a855f7);">D</label>
              <input type="number" class="gizmo-pos-input" id="gizmo-pos-d" step="0.1" min="0.1" max="5.0" value="1.0" />
              <span class="gizmo-pos-unit">σ</span>
            </div>
          </div>
          <button class="gizmo-pos-apply-btn" id="gizmo-pos-center" title="Center container in view">
            ⊙ Center in View
          </button>
        </div>
      </div>
    `;

    this._cacheElements();
    this.projectAxes();
    this.updateTelemetry();
  }

  _cacheElements() {
    const q = (sel) => this.container.querySelector(sel);
    this.els = {
      viewport: q('#gizmo-trackball-viewport'),
      modeIndicator: q('#gizmo-mode-indicator'),
      orbitViews: q('#gizmo-orbit-views'),
      orbitFooter: q('#gizmo-orbit-footer'),
      positionFooter: q('#gizmo-position-footer'),
      posTarget: q('#gizmo-pos-target'),
      // Axis lines/tips
      axX: q('#gizmo-ax-x'), tipX: q('#gizmo-tip-x'), lblX: q('#gizmo-label-x'),
      axNX: q('#gizmo-ax-nx'),
      axY: q('#gizmo-ax-y'), tipY: q('#gizmo-tip-y'), lblY: q('#gizmo-label-y'),
      axNY: q('#gizmo-ax-ny'),
      axZ: q('#gizmo-ax-z'), tipZ: q('#gizmo-tip-z'), lblZ: q('#gizmo-label-z'),
      axNZ: q('#gizmo-ax-nz'),
      // Orbit telemetry
      teleX: q('#tele-x'), teleY: q('#tele-y'), teleZ: q('#tele-z'), teleD: q('#tele-d'),
      dSlider: q('#gizmo-d-slider'),
      dRing: q('.gizmo-d-ring'),
      // Position numeric inputs
      posX: q('#gizmo-pos-x'), posY: q('#gizmo-pos-y'),
      posZ: q('#gizmo-pos-z'), posD: q('#gizmo-pos-d'),
      posCenterBtn: q('#gizmo-pos-center'),
    };
  }

  // ─── 3D → 2D Projection of axis endpoints ──────────────────────
  projectAxes() {
    const R = 36;
    const { x: rx, y: ry } = this.rotation;

    const axes = [
      { dir: [ 1, 0, 0], el: 'X',  sign: '+' },
      { dir: [-1, 0, 0], el: 'NX', sign: '-' },
      { dir: [ 0, 1, 0], el: 'Y',  sign: '+' },
      { dir: [ 0,-1, 0], el: 'NY', sign: '-' },
      { dir: [ 0, 0, 1], el: 'Z',  sign: '+' },
      { dir: [ 0, 0,-1], el: 'NZ', sign: '-' },
    ];

    const projected = axes.map(a => {
      const p = rotatePoint3D(a.dir[0] * R, a.dir[1] * R, a.dir[2] * R, rx, ry);
      return { ...a, px: p.x, py: -p.y, pz: p.z };
    });

    projected.sort((a, b) => a.pz - b.pz);

    const group = this.container.querySelector('#gizmo-axes-group');
    if (!group) return;
    const pivotDot = group.querySelector('circle:last-child');

    for (const item of projected) {
      const key = item.el;
      if (key.startsWith('N')) {
        const lineEl = this.els[`ax${key}`];
        if (lineEl) {
          lineEl.setAttribute('x2', item.px.toFixed(1));
          lineEl.setAttribute('y2', item.py.toFixed(1));
          const depthFade = CLAMP(0.15 + (item.pz + R) / (2 * R) * 0.2, 0.08, 0.35);
          lineEl.setAttribute('opacity', depthFade.toFixed(2));
          group.insertBefore(lineEl, pivotDot);
        }
      } else {
        const lineEl = this.els[`ax${key}`];
        const tipEl  = this.els[`tip${key}`];
        const lblEl  = this.els[`lbl${key}`];
        if (lineEl) {
          lineEl.setAttribute('x2', item.px.toFixed(1));
          lineEl.setAttribute('y2', item.py.toFixed(1));
          const lineOpacity = CLAMP(0.4 + (item.pz + R) / (2 * R) * 0.6, 0.3, 1.0);
          lineEl.setAttribute('opacity', lineOpacity.toFixed(2));
          group.insertBefore(lineEl, pivotDot);
        }
        if (tipEl) {
          tipEl.setAttribute('cx', item.px.toFixed(1));
          tipEl.setAttribute('cy', item.py.toFixed(1));
          const scale = CLAMP(0.7 + (item.pz + R) / (2 * R) * 0.5, 0.55, 1.2);
          tipEl.setAttribute('r', (7 * scale).toFixed(1));
          tipEl.setAttribute('opacity', CLAMP(0.5 + (item.pz + R) / (2 * R) * 0.5, 0.35, 1.0).toFixed(2));
          group.insertBefore(tipEl, pivotDot);
        }
        if (lblEl) {
          lblEl.setAttribute('x', item.px.toFixed(1));
          lblEl.setAttribute('y', item.py.toFixed(1));
          lblEl.setAttribute('opacity', CLAMP(0.5 + (item.pz + R) / (2 * R) * 0.5, 0.3, 1.0).toFixed(2));
          group.insertBefore(lblEl, pivotDot);
        }
      }
    }
  }

  // ─── Mode switching ─────────────────────────────────────────────
  setMode(mode) {
    this.mode = mode;
    const isOrbit = mode === 'orbit';

    // Toggle visual state
    this.container.querySelectorAll('.gizmo-mode-btn').forEach(b => {
      b.classList.toggle('active', b.getAttribute('data-mode') === mode);
    });

    // Show/hide orbit vs position panels
    if (this.els.orbitViews) this.els.orbitViews.style.display = isOrbit ? 'flex' : 'none';
    if (this.els.orbitFooter) this.els.orbitFooter.style.display = isOrbit ? 'flex' : 'none';
    if (this.els.positionFooter) this.els.positionFooter.style.display = isOrbit ? 'none' : 'flex';

    // Update mode indicator
    if (this.els.modeIndicator) {
      this.els.modeIndicator.textContent = isOrbit ? 'ORBIT' : 'POSITION';
      this.els.modeIndicator.classList.toggle('mode-position', !isOrbit);
    }

    // Update viewport cursor
    if (this.els.viewport) {
      this.els.viewport.classList.toggle('position-mode', !isOrbit);
    }

    if (!isOrbit) {
      this._syncPositionPanel();
    }
  }

  // ─── Sync the Position panel inputs with the selected container ─
  _syncPositionPanel() {
    const c = this._getSelectedContainer();
    if (this.els.posTarget) {
      if (c) {
        this.els.posTarget.innerHTML = `
          <span class="container-type-tag tag-${c.type}" style="font-size: 9px; padding: 1px 5px;">${c.type}</span>
          <span class="gizmo-pos-target-label" title="${c.id}">${c.title}</span>
        `;
      } else {
        this.els.posTarget.innerHTML = `<span class="gizmo-pos-target-label" style="opacity:0.5;">No container selected</span>`;
      }
    }
    if (c) {
      if (this.els.posX) this.els.posX.value = Math.round(c.x || 0);
      if (this.els.posY) this.els.posY.value = Math.round(c.y || 0);
      if (this.els.posZ) this.els.posZ.value = Math.round(c.z || 0);
      if (this.els.posD) this.els.posD.value = (c.d || 1.0).toFixed(1);
    }
  }

  // ─── Apply position from numeric fields to the container ────────
  _applyPositionFromInputs() {
    const c = this._getSelectedContainer();
    if (!c) return;

    const x = parseFloat(this.els.posX?.value) || 0;
    const y = parseFloat(this.els.posY?.value) || 0;
    const z = parseFloat(this.els.posZ?.value) || 0;
    const d = parseFloat(this.els.posD?.value) || 1.0;

    c.x = x;
    c.y = y;
    c.z = z;
    c.d = d;

    canvasState.updateContainerPos(c.id, x, y, z, d);
    this._updateContainerDOM(c);
  }

  // ─── Update the container's DOM element to match new position ───
  _updateContainerDOM(c) {
    const el = document.getElementById(c.id);
    if (!el) return;
    el.style.left = `${c.x}px`;
    el.style.top = `${c.y}px`;
    if ((canvasState.state.dimensionMode === '3D' || canvasState.state.dimensionMode === '4D') && c.z !== undefined) {
      el.style.transform = `translateZ(${c.z}px) scale(${c.d || 1.0})`;
    }
    // Update the xyzd badge in the header
    const badge = el.querySelector('.container-xyzd-badge');
    if (badge) badge.textContent = `[z:${c.z || 0}, d:${(c.d || 1.0).toFixed(1)}]`;

    // Redraw wires
    if (this.canvasEngine && typeof this.canvasEngine.wireSystem?.render === 'function') {
      this.canvasEngine.wireSystem.render();
    }
    // Fallback: try global wireSystem via canvas stage
    const wiresLayer = document.querySelector('#wires-layer');
    if (wiresLayer) {
      const event = new CustomEvent('containerMoved');
      wiresLayer.dispatchEvent(event);
    }
  }

  // ─── Events ─────────────────────────────────────────────────────
  attachEvents() {
    const vp = this.els.viewport;
    if (!vp) return;

    // ── Mode toggle buttons ──
    this.container.querySelectorAll('.gizmo-mode-btn').forEach(btn => {
      btn.addEventListener('click', (e) => {
        e.stopPropagation();
        this.setMode(btn.getAttribute('data-mode'));
      });
    });

    // ── Mouse drag (dual-mode) ──
    const onDown = (e) => {
      e.preventDefault();
      e.stopPropagation();
      this.isDragging = true;
      this.dragStart  = { x: e.clientX, y: e.clientY };
      this.velocity   = { x: 0, y: 0 };
      if (this.momentumRAF) cancelAnimationFrame(this.momentumRAF);

      if (this.mode === 'orbit') {
        this.startRot = { x: this.rotation.x, y: this.rotation.y };
      } else {
        const c = this._getSelectedContainer();
        if (c) {
          this.startPos = { x: c.x, y: c.y };
        }
      }

      vp.classList.add('dragging');
      window.addEventListener('mousemove', onMove);
      window.addEventListener('mouseup', onUp);
    };

    const onMove = (e) => {
      if (!this.isDragging) return;
      const dx = e.clientX - this.dragStart.x;
      const dy = e.clientY - this.dragStart.y;
      this.velocity = {
        x: (e.clientX - (this._lastMX || e.clientX)),
        y: (e.clientY - (this._lastMY || e.clientY)),
      };
      this._lastMX = e.clientX;
      this._lastMY = e.clientY;

      if (this.mode === 'orbit') {
        this.rotation.y = this.startRot.y + dx * 0.7;
        this.rotation.x = CLAMP(this.startRot.x - dy * 0.7, -89, 89);
        this.applyOrbit();
      } else {
        // Position mode — translate the selected container
        const c = this._getSelectedContainer();
        if (c) {
          // Scale movement by 3x to make the small gizmo drag translate to larger canvas movements
          const sensitivity = 3.0;
          c.x = this.startPos.x + dx * sensitivity;
          c.y = this.startPos.y + dy * sensitivity;
          this._updateContainerDOM(c);
          this._syncPositionPanel();
        }
      }
    };

    const onUp = () => {
      this.isDragging = false;
      this._lastMX = this._lastMY = undefined;
      vp.classList.remove('dragging');
      window.removeEventListener('mousemove', onMove);
      window.removeEventListener('mouseup', onUp);

      if (this.mode === 'orbit') {
        this._startMomentum();
      } else {
        // Persist final position
        const c = this._getSelectedContainer();
        if (c) {
          canvasState.updateContainerPos(c.id, c.x, c.y, c.z, c.d);
        }
      }
    };

    vp.addEventListener('mousedown', onDown);

    // ── Touch (dual-mode) ──
    vp.addEventListener('touchstart', (e) => {
      if (e.touches.length !== 1) return;
      e.preventDefault();
      this.isDragging = true;
      this.dragStart  = { x: e.touches[0].clientX, y: e.touches[0].clientY };
      this.velocity   = { x: 0, y: 0 };
      if (this.momentumRAF) cancelAnimationFrame(this.momentumRAF);
      if (this.mode === 'orbit') {
        this.startRot = { x: this.rotation.x, y: this.rotation.y };
      } else {
        const c = this._getSelectedContainer();
        if (c) this.startPos = { x: c.x, y: c.y };
      }
    }, { passive: false });

    vp.addEventListener('touchmove', (e) => {
      if (!this.isDragging || e.touches.length !== 1) return;
      e.preventDefault();
      const dx = e.touches[0].clientX - this.dragStart.x;
      const dy = e.touches[0].clientY - this.dragStart.y;
      this.velocity = {
        x: (e.touches[0].clientX - (this._lastMX || e.touches[0].clientX)),
        y: (e.touches[0].clientY - (this._lastMY || e.touches[0].clientY)),
      };
      this._lastMX = e.touches[0].clientX;
      this._lastMY = e.touches[0].clientY;

      if (this.mode === 'orbit') {
        this.rotation.y = this.startRot.y + dx * 0.7;
        this.rotation.x = CLAMP(this.startRot.x - dy * 0.7, -89, 89);
        this.applyOrbit();
      } else {
        const c = this._getSelectedContainer();
        if (c) {
          c.x = this.startPos.x + dx * 3.0;
          c.y = this.startPos.y + dy * 3.0;
          this._updateContainerDOM(c);
          this._syncPositionPanel();
        }
      }
    }, { passive: false });

    vp.addEventListener('touchend', () => {
      this.isDragging = false;
      this._lastMX = this._lastMY = undefined;
      if (this.mode === 'orbit') {
        this._startMomentum();
      } else {
        const c = this._getSelectedContainer();
        if (c) canvasState.updateContainerPos(c.id, c.x, c.y, c.z, c.d);
      }
    });

    // ── Scroll wheel ──
    vp.addEventListener('wheel', (e) => {
      e.preventDefault();
      const delta = e.deltaY > 0 ? -5 : 5;
      if (this.mode === 'orbit') {
        const dSlider = this.els.dSlider;
        if (dSlider) {
          dSlider.value = CLAMP(parseInt(dSlider.value) + delta, 50, 250);
          this._applyDScale(parseInt(dSlider.value) / 100);
        }
      } else {
        // In position mode, scroll adjusts Z depth
        const c = this._getSelectedContainer();
        if (c) {
          c.z = (c.z || 0) + delta * 4;
          this._updateContainerDOM(c);
          this._syncPositionPanel();
          canvasState.updateContainerPos(c.id, c.x, c.y, c.z, c.d);
        }
      }
    }, { passive: false });

    // ── Axis label click ──
    this.container.querySelectorAll('.gizmo-axis-label, .gizmo-axis-tip').forEach(el => {
      el.addEventListener('click', (e) => {
        e.stopPropagation();
        const axis = el.getAttribute('data-axis');
        if (this.mode === 'orbit') {
          this.snapToAxis(axis);
        } else {
          // In position mode, clicking an axis nudges the container along that axis
          const c = this._getSelectedContainer();
          if (c) {
            const nudge = 50;
            if (axis === 'x') c.x += nudge;
            if (axis === 'y') c.y += nudge;
            if (axis === 'z') c.z = (c.z || 0) + nudge;
            this._updateContainerDOM(c);
            this._syncPositionPanel();
            canvasState.updateContainerPos(c.id, c.x, c.y, c.z, c.d);
          }
        }
      });
    });

    // ── Quick view buttons (orbit mode only) ──
    this.container.querySelectorAll('.gizmo-view-btn').forEach(btn => {
      btn.addEventListener('click', (e) => {
        e.stopPropagation();
        const v = btn.getAttribute('data-view');
        const views = {
          iso:   [25, -35],
          top:   [89, 0],
          front: [0, 0],
          right: [0, -90],
          reset: [0, 0],
        };
        const [rx, ry] = views[v] || [0, 0];
        this.animateTo(rx, ry);
      });
    });

    // ── D-slider (orbit mode) ──
    this.els.dSlider?.addEventListener('input', (e) => {
      this._applyDScale(parseInt(e.target.value) / 100);
    });

    // ── Position numeric inputs ──
    ['posX', 'posY', 'posZ', 'posD'].forEach(key => {
      this.els[key]?.addEventListener('change', () => this._applyPositionFromInputs());
      this.els[key]?.addEventListener('input', () => this._applyPositionFromInputs());
    });

    // ── Center in view button ──
    this.els.posCenterBtn?.addEventListener('click', (e) => {
      e.stopPropagation();
      const c = this._getSelectedContainer();
      if (c) {
        // Center the container in the viewport
        const viewport = document.querySelector('#canvas-viewport');
        if (viewport) {
          const vr = viewport.getBoundingClientRect();
          const zoom = canvasState.state.zoom || 1;
          c.x = (vr.width / 2 / zoom) - (c.width || 400) / 2;
          c.y = (vr.height / 2 / zoom) - (c.height || 300) / 2;
          this._updateContainerDOM(c);
          this._syncPositionPanel();
          canvasState.updateContainerPos(c.id, c.x, c.y, c.z, c.d);
        }
      }
    });
  }

  // ─── Momentum / Inertia (orbit mode only) ──────────────────────
  _startMomentum() {
    const friction = 0.92;
    const threshold = 0.15;
    let vx = this.velocity.x * 0.7;
    let vy = this.velocity.y * 0.7;

    const step = () => {
      if (Math.abs(vx) < threshold && Math.abs(vy) < threshold) return;
      this.rotation.y += vx * 0.5;
      this.rotation.x = CLAMP(this.rotation.x - vy * 0.5, -89, 89);
      vx *= friction;
      vy *= friction;
      this.applyOrbit();
      this.momentumRAF = requestAnimationFrame(step);
    };
    this.momentumRAF = requestAnimationFrame(step);
  }

  // ─── Smooth animated snap (orbit mode) ─────────────────────────
  animateTo(targetRx, targetRy, duration = 350) {
    if (this.momentumRAF) cancelAnimationFrame(this.momentumRAF);
    const startRx = this.rotation.x;
    const startRy = this.rotation.y;
    const startT = performance.now();

    const ease = (t) => t < 0.5 ? 4 * t * t * t : 1 - Math.pow(-2 * t + 2, 3) / 2;

    const tick = (now) => {
      const t = CLAMP((now - startT) / duration, 0, 1);
      const e = ease(t);
      this.rotation.x = startRx + (targetRx - startRx) * e;
      this.rotation.y = startRy + (targetRy - startRy) * e;
      this.applyOrbit();
      if (t < 1) requestAnimationFrame(tick);
    };
    requestAnimationFrame(tick);
  }

  snapToAxis(axis) {
    const snaps = { x: [0, -90], y: [89, 0], z: [0, 0] };
    const [rx, ry] = snaps[axis] || [0, 0];
    this.animateTo(rx, ry);
  }

  // ─── Apply orbit to gizmo + canvas ─────────────────────────────
  applyOrbit() {
    this.projectAxes();
    this.updateTelemetry();

    if (canvasState.state.dimensionMode === '2D') {
      canvasState.setDimensionMode('3D');
      document.querySelectorAll('.dim-btn').forEach(b => {
        b.classList.toggle('active', b.getAttribute('data-dim') === '3D');
      });
    }

    const stage = document.querySelector('#canvas-stage');
    if (stage) {
      const { zoom, pan, xyzd } = canvasState.state;
      stage.style.transform = `
        translate(${pan.x + xyzd.x}px, ${pan.y + xyzd.y}px)
        scale(${zoom * (xyzd.d || 1.0)})
        rotateX(${this.rotation.x}deg)
        rotateY(${this.rotation.y}deg)
      `;
    }
  }

  // ─── D-scale (orbit mode) ──────────────────────────────────────
  _applyDScale(d) {
    canvasState.setXYZD({ d });
    if (this.els.teleD) this.els.teleD.innerText = `${d.toFixed(1)}σ`;
    if (this.els.dRing) {
      const r = 48 + d * 6;
      this.els.dRing.setAttribute('r', r.toFixed(0));
    }
    this.applyOrbit();
  }

  // ─── Update readouts ──────────────────────────────────────────
  updateTelemetry() {
    const { teleX, teleY, teleZ } = this.els;
    if (teleX) teleX.innerText = `${Math.round(this.rotation.x)}°`;
    if (teleY) teleY.innerText = `${Math.round(this.rotation.y)}°`;
    if (teleZ) teleZ.innerText = `${Math.round(this.rotation.z)}°`;
  }

  updateGizmoDisplay() {
    this.projectAxes();
    this.updateTelemetry();
    if (this.mode === 'position') {
      this._syncPositionPanel();
    }
  }
}

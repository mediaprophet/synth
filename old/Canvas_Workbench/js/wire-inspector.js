/**
 * js/wire-inspector.js — Semantic Connector Relationship Inspector
 * Slide-out inspector panel to define, inspect, and modify RDF 1.2 semantic
 * relationships, predicates, directional causality, and epistemic modalities between containers.
 */

import { canvasState } from './state.js';

export class WireInspector {
  constructor(panelEl, wireSystem, containerManager) {
    this.panel = panelEl;
    this.wireSystem = wireSystem;
    this.containerManager = containerManager;
    this.activeWire = null;

    this.commonPredicates = [
      { uri: 'qualia:causallyPrecedes', label: '⚡ qualia:causallyPrecedes (Causal Dependency)', category: 'Qualia Causal' },
      { uri: 'qualia:embedsSubGraph', label: '🌌 qualia:embedsSubGraph (Fractal Containment)', category: 'Qualia Structural' },
      { uri: 'fin:triggersMilestoneEscrow', label: '💰 fin:triggersMilestoneEscrow (Fiscal Escrow Release)', category: 'Financial Economics' },
      { uri: 'fin:reconcilesPool', label: '⚖️ fin:reconcilesPool (Double-Entry Balance)', category: 'Financial Economics' },
      { uri: 'prov:wasDerivedFrom', label: '📜 prov:wasDerivedFrom (Data Provenance)', category: 'W3C PROV-O' },
      { uri: 'dcterms:references', label: '📚 dcterms:references (Bibliographic Reference)', category: 'Dublin Core' },
      { uri: 'skos:related', label: '🔗 skos:related (Semantic Association)', category: 'SKOS' },
      { uri: 'rdfs:seeAlso', label: '👁️ rdfs:seeAlso (Cross-Reference)', category: 'RDFS' }
    ];
  }

  open(conn) {
    if (!conn) return;
    this.activeWire = conn;

    const fromNode = canvasState.state.containers.find(c => c.id === conn.from) || { title: conn.from, type: 'node' };
    const toNode = canvasState.state.containers.find(c => c.id === conn.to) || { title: conn.to, type: 'node' };

    this.panel.classList.remove('hidden');
    this.panel.innerHTML = `
      <div class="wire-inspector-header">
        <div style="display: flex; align-items: center; gap: 8px;">
          <span style="font-size: 16px;">🔗</span>
          <strong style="color: #fff; font-size: 13px;">Semantic Connector Inspector</strong>
        </div>
        <button class="btn btn-sm btn-close-inspector" style="background: transparent; border: none; color: #fff; font-size: 16px; cursor: pointer;">&times;</button>
      </div>

      <div class="wire-inspector-body">
        <!-- Wire ID & Status -->
        <div style="display: flex; justify-content: space-between; align-items: center; font-size: 10px; font-family: var(--font-mono); color: var(--text-muted); border-bottom: 1px solid var(--border-subtle); padding-bottom: 6px;">
          <span>Wire ID: <strong style="color: var(--accent-cyan);">${conn.id}</strong></span>
          <span class="honesty-badge honesty-live">RDF 1.2 Quad</span>
        </div>

        <!-- Connection Endpoints -->
        <div style="display: flex; flex-direction: column; gap: 8px; background: var(--surface-base); padding: 10px; border-radius: var(--radius-sm); border: 1px solid var(--border-subtle);">
          <div>
            <span style="font-size: 10px; color: var(--text-secondary); text-transform: uppercase;">Origin / Source Node (Subject):</span>
            <div style="display: flex; align-items: center; gap: 6px; margin-top: 2px;">
              <span class="container-type-tag tag-${fromNode.type}" style="font-size: 9px;">${fromNode.type}</span>
              <strong style="font-size: 11px; color: #fff;">${fromNode.title || fromNode.id}</strong>
            </div>
            <div style="font-family: var(--font-mono); font-size: 9px; color: var(--text-muted); margin-top: 1px;">IRI: <code>:${conn.from}</code></div>
          </div>

          <div style="text-align: center; color: var(--accent-cyan); font-size: 14px;">⬇️</div>

          <div>
            <span style="font-size: 10px; color: var(--text-secondary); text-transform: uppercase;">Destination / Target Node (Object):</span>
            <div style="display: flex; align-items: center; gap: 6px; margin-top: 2px;">
              <span class="container-type-tag tag-${toNode.type}" style="font-size: 9px;">${toNode.type}</span>
              <strong style="font-size: 11px; color: #fff;">${toNode.title || toNode.id}</strong>
            </div>
            <div style="font-family: var(--font-mono); font-size: 9px; color: var(--text-muted); margin-top: 1px;">IRI: <code>:${conn.to}</code></div>
          </div>
        </div>

        <!-- Semantic Predicate Selector -->
        <div style="display: flex; flex-direction: column; gap: 4px;">
          <label style="font-size: 11px; font-weight: 600; color: #fff;">Semantic Predicate / Relationship:</label>
          <select id="wire-predicate-select" style="background: var(--surface-panel); color: #fff; border: 1px solid var(--border-medium); border-radius: var(--radius-xs); padding: 6px; font-size: 11px;">
            ${this.commonPredicates.map(p => `
              <option value="${p.uri}" ${conn.predicate === p.uri || conn.label === p.uri ? 'selected' : ''}>${p.label}</option>
            `).join('')}
          </select>
        </div>

        <!-- Epistemic Modality for Wire -->
        <div style="display: flex; flex-direction: column; gap: 4px;">
          <label style="font-size: 11px; font-weight: 600; color: #fff;">Epistemic Modality (Wire Visual Style):</label>
          <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 6px;">
            <button class="btn btn-sm wire-mod-btn ${conn.type === 'objective' ? 'active' : ''}" data-mod="objective" style="border: 1px solid var(--modality-objective); color: var(--modality-objective);">🔬 Objective</button>
            <button class="btn btn-sm wire-mod-btn ${conn.type === 'subjective' ? 'active' : ''}" data-mod="subjective" style="border: 1px dashed var(--modality-subjective); color: var(--modality-subjective);">🧠 Subjective</button>
            <button class="btn btn-sm wire-mod-btn ${conn.type === 'intersubjective' ? 'active' : ''}" data-mod="intersubjective" style="border: 1px dotted var(--modality-intersubjective); color: var(--modality-intersubjective);">🌊 Intersubjective</button>
            <button class="btn btn-sm wire-mod-btn ${conn.type === 'normative' ? 'active' : ''}" data-mod="normative" style="border: 2px double var(--modality-normative); color: var(--modality-normative);">⚖️ Normative</button>
          </div>
        </div>

        <!-- Live RDF 1.2 Quad Preview -->
        <div style="display: flex; flex-direction: column; gap: 4px;">
          <label style="font-size: 11px; font-weight: 600; color: var(--text-secondary);">Generated RDF 1.2 Quad Assertion:</label>
          <div id="wire-quad-preview" style="background: #020408; border: 1px solid var(--border-medium); border-radius: var(--radius-xs); padding: 8px; font-family: var(--font-mono); font-size: 10px; color: var(--accent-cyan); word-break: break-all; line-height: 1.4;">
            &lt;&lt;[ :${conn.from} ${conn.predicate || conn.label || 'qualia:causallyPrecedes'} :${conn.to} &lt;${canvasState.state.graphIri}&gt; ]&gt;&gt;
          </div>
        </div>

        <!-- Actions -->
        <div style="display: flex; gap: 8px; margin-top: auto; border-top: 1px solid var(--border-subtle); padding-top: 10px;">
          <button id="btn-save-wire-relationship" class="btn" style="flex: 1; background: var(--accent-cyan); color: #000; font-weight: 700; font-size: 11px; padding: 6px 12px; border-radius: var(--radius-xs); cursor: pointer;">
            💾 Save Quad Relationship
          </button>
          <button id="btn-delete-wire" class="btn" style="background: rgba(239, 68, 68, 0.15); border: 1px solid rgba(239, 68, 68, 0.4); color: #ef4444; font-size: 11px; padding: 6px 10px; border-radius: var(--radius-xs); cursor: pointer;" title="Delete Connection Wire">
            🗑️ Sever
          </button>
        </div>
      </div>
    `;

    this.bindEvents(conn);
  }

  bindEvents(conn) {
    this.panel.querySelector('.btn-close-inspector')?.addEventListener('click', () => {
      this.close();
    });

    const predSelect = this.panel.querySelector('#wire-predicate-select');
    predSelect?.addEventListener('change', (e) => {
      conn.predicate = e.target.value;
      conn.label = e.target.value;
      this.updateQuadPreview(conn);
    });

    this.panel.querySelectorAll('.wire-mod-btn').forEach(btn => {
      btn.addEventListener('click', () => {
        this.panel.querySelectorAll('.wire-mod-btn').forEach(b => b.classList.remove('active'));
        btn.classList.add('active');
        conn.type = btn.getAttribute('data-mod');
        this.updateQuadPreview(conn);
      });
    });

    this.panel.querySelector('#btn-save-wire-relationship')?.addEventListener('click', () => {
      this.wireSystem.renderWires();
      alert(`💾 Semantic Wire Relationship Committed:\n<<[ :${conn.from} ${conn.predicate || conn.label} :${conn.to} <${canvasState.state.graphIri}> ]>>\nModality: ${conn.type}`);
      this.close();
    });

    this.panel.querySelector('#btn-delete-wire')?.addEventListener('click', () => {
      if (confirm(`Sever connection wire "${conn.id}"?`)) {
        canvasState.removeConnection(conn.id);
        this.close();
      }
    });
  }

  updateQuadPreview(conn) {
    const quadBox = this.panel.querySelector('#wire-quad-preview');
    if (quadBox) {
      quadBox.textContent = `<<[ :${conn.from} ${conn.predicate || conn.label || 'qualia:causallyPrecedes'} :${conn.to} <${canvasState.state.graphIri}> ]>>`;
    }
  }

  close() {
    this.panel.classList.add('hidden');
    this.activeWire = null;
  }
}

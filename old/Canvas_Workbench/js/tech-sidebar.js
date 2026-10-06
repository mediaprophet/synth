/**
 * HyperCanvas: Slide-out Technical Telemetry & Connection Ontology Sidebar
 * Renders graph telemetry, 48-byte Nquins, Merkle-CRDT DAG, and interactive Connection Ontology editor.
 */

import { canvasState } from './state.js';
import { GraphGenerator } from './graph-generator.js';

export class TechSidebar {
  constructor(sidebarEl) {
    this.sidebar = sidebarEl;
    this.init();
  }

  init() {
    canvasState.subscribe('techSidebarToggled', (isOpen) => {
      this.sidebar.classList.toggle('open', isOpen);
      if (isOpen) this.render();
    });

    canvasState.subscribe('containerSelected', () => {
      if (this.sidebar.classList.contains('open')) {
        this.render();
      }
    });

    canvasState.subscribe('connectionSelected', () => {
      this.sidebar.classList.add('open');
      this.render();
    });
  }

  render() {
    const isConnMode = canvasState.state.sidebarTab === 'connection-ontology' || !!canvasState.state.selectedConnectionId;

    if (isConnMode && canvasState.state.selectedConnectionId) {
      this.renderConnectionOntology();
    } else {
      this.renderContainerTelemetry();
    }
  }

  renderConnectionOntology() {
    const wireId = canvasState.state.selectedConnectionId;
    const conn = canvasState.state.connections.find(c => c.id === wireId);
    if (!conn) {
      this.renderContainerTelemetry();
      return;
    }

    const src = canvasState.state.containers.find(c => c.id === conn.from);
    const tgt = canvasState.state.containers.find(c => c.id === conn.to);

    this.sidebar.innerHTML = `
      <div class="tech-sidebar-header">
        <div style="display: flex; align-items: center; gap: 8px;">
          <span>🔗</span>
          <span>Connection Ontology Inspector</span>
        </div>
        <button class="container-action-btn btn-close-tech" style="font-size: 16px;">&times;</button>
      </div>

      <div class="tech-sidebar-content">
        <!-- Connection Endpoints Card -->
        <div style="background: var(--surface-panel-elevated); border: 1px solid var(--accent-violet); border-radius: var(--radius-sm); padding: 12px;">
          <div style="font-size: 10px; font-weight: 700; color: var(--accent-violet); text-transform: uppercase; margin-bottom: 6px;">
            Connection Link Endpoints
          </div>
          <div style="display: flex; align-items: center; justify-content: space-between; font-size: 12px; font-weight: 600;">
            <span style="color: var(--accent-cyan);">${src ? src.title : conn.from}</span>
            <span style="color: var(--accent-amber);">➔</span>
            <span style="color: var(--accent-emerald);">${tgt ? tgt.title : conn.to}</span>
          </div>
          <div style="font-family: var(--font-mono); font-size: 9px; color: var(--text-muted); margin-top: 4px;">
            Wire ID: ${conn.id} • Type: ${conn.type}
          </div>
        </div>

        <!-- Semantic Predicate & Relation Editor -->
        <div style="background: var(--surface-panel-elevated); border: 1px solid var(--border-subtle); border-radius: var(--radius-sm); padding: 12px;">
          <div style="font-size: 10px; font-weight: 700; color: var(--accent-cyan); text-transform: uppercase; margin-bottom: 8px;">
            Ontological Predicate & Bridge
          </div>
          
          <div style="display: flex; flex-direction: column; gap: 8px;">
            <div>
              <label style="font-size: 10px; color: var(--text-muted); display: block; margin-bottom: 2px;">Predicate IRI / Property:</label>
              <input type="text" id="conn-predicate-input" value="${conn.predicate || 'qualia:connectsTo'}" style="width: 100%; background: var(--canvas-bg); border: 1px solid var(--border-medium); color: var(--accent-cyan); font-family: var(--font-mono); font-size: 11px; padding: 4px 8px; border-radius: 4px;" />
            </div>

            <div>
              <label style="font-size: 10px; color: var(--text-muted); display: block; margin-bottom: 2px;">Semantic Bridge Type:</label>
              <select id="conn-bridge-select" style="width: 100%; background: var(--canvas-bg); border: 1px solid var(--border-medium); color: var(--text-primary); font-size: 11px; padding: 4px 8px; border-radius: 4px;">
                <option value="Schema Grounding" ${conn.bridgeType === 'Schema Grounding' ? 'selected' : ''}>Schema Grounding (qualia:modelsOntology)</option>
                <option value="Ontology Alignment" ${conn.bridgeType === 'Ontology Alignment' ? 'selected' : ''}>Ontology Alignment (owl:equivalentClass / skos:closeMatch)</option>
                <option value="Type-Safe DSL Contract" ${conn.bridgeType === 'Type-Safe DSL Contract' ? 'selected' : ''}>Type-Safe DSL Contract (vibe:bindsSchema)</option>
                <option value="Tensor Projection" ${conn.bridgeType === 'Tensor Projection' ? 'selected' : ''}>Tensor Projection (p64:groundsTensor)</option>
                <option value="Multi-Tenant Wormhole" ${conn.bridgeType === 'Multi-Tenant Wormhole' ? 'selected' : ''}>Multi-Tenant Wormhole (portal:crossCanvasBridge)</option>
              </select>
            </div>

            <button class="btn btn-sm btn-primary" id="btn-save-conn-ontology" style="margin-top: 4px;">Save Relation Ontology</button>
          </div>
        </div>

        <!-- RDF 1.2 Reifier Statement -->
        <div style="background: var(--surface-panel-elevated); border: 1px solid var(--border-subtle); border-radius: var(--radius-sm); padding: 12px;">
          <div style="font-size: 10px; font-weight: 700; color: var(--accent-amber); text-transform: uppercase; margin-bottom: 4px;">
            W3C RDF 1.2 Triple Reifier
          </div>
          <div style="font-family: var(--font-mono); font-size: 10px; color: var(--accent-amber); background: var(--canvas-bg); padding: 6px; border-radius: 4px; word-break: break-all;">
            ${conn.reifier || `<< ${conn.from} ${conn.predicate || 'qualia:connectsTo'} ${conn.to} ~ reifier:${conn.id} >>`}
          </div>
        </div>

        <!-- 48-Byte Qualia Nquin -->
        <div style="background: var(--surface-panel-elevated); border: 1px solid var(--border-subtle); border-radius: var(--radius-sm); padding: 12px;">
          <div style="font-size: 10px; font-weight: 700; color: var(--accent-emerald); text-transform: uppercase; margin-bottom: 4px;">
            Qualia 48-byte Nquin Assertion
          </div>
          <div style="font-family: var(--font-mono); font-size: 10px; color: var(--accent-emerald); background: var(--canvas-bg); padding: 6px; border-radius: 4px; word-break: break-all;">
            ${conn.nquin || `<<[ ${conn.from} ${conn.predicate || 'qualia:connectsTo'} ${conn.to} ${canvasState.state.graphIri} did:qualia:timothy_charles_holborn ]>>`}
          </div>
        </div>

        <!-- Back to Container Telemetry Button -->
        <button class="btn btn-sm" id="btn-switch-to-telemetry" style="margin-top: 4px;">
          View Node Telemetry & Merkle DAG ➔
        </button>
      </div>
    `;

    this.attachConnectionEvents(conn);
  }

  attachConnectionEvents(conn) {
    const closeBtn = this.sidebar.querySelector('.btn-close-tech');
    if (closeBtn) {
      closeBtn.addEventListener('click', () => canvasState.toggleTechSidebar());
    }

    const saveBtn = this.sidebar.querySelector('#btn-save-conn-ontology');
    if (saveBtn) {
      saveBtn.addEventListener('click', () => {
        const predicate = this.sidebar.querySelector('#conn-predicate-input').value;
        const bridgeType = this.sidebar.querySelector('#conn-bridge-select').value;
        
        canvasState.updateConnectionOntology(conn.id, {
          predicate,
          label: predicate,
          bridgeType,
          reifier: `<< ${conn.from} ${predicate} ${conn.to} ~ reifier:${conn.id} >>`,
          nquin: `<<[ ${conn.from} ${predicate} ${conn.to} ${canvasState.state.graphIri} did:qualia:timothy_charles_holborn ]>>`
        });

        alert(`✓ Connection Ontology Updated!\nPredicate: ${predicate}\nBridge: ${bridgeType}`);
      });
    }

    const switchBtn = this.sidebar.querySelector('#btn-switch-to-telemetry');
    if (switchBtn) {
      switchBtn.addEventListener('click', () => {
        canvasState.setState({ selectedConnectionId: null, sidebarTab: 'telemetry' });
        this.renderContainerTelemetry();
      });
    }
  }

  renderContainerTelemetry() {
    const selectedId = canvasState.state.selectedContainerId;
    const container = canvasState.state.containers.find(c => c.id === selectedId);
    const subGraph = GraphGenerator.getContainerSubGraph(selectedId);

    this.sidebar.innerHTML = `
      <div class="tech-sidebar-header">
        <div style="display: flex; align-items: center; gap: 8px;">
          <span>⚙️</span>
          <span>Technical Telemetry & DAG</span>
        </div>
        <button class="container-action-btn btn-close-tech" style="font-size: 16px;">&times;</button>
      </div>

      <div class="tech-sidebar-content">
        <!-- Selected Container Graph Addressing -->
        <div style="background: var(--surface-panel-elevated); border: 1px solid var(--border-subtle); border-radius: var(--radius-sm); padding: 12px;">
          <div style="font-size: 10px; font-weight: 700; color: var(--accent-cyan); text-transform: uppercase; margin-bottom: 4px;">
            Container Sub-Graph
          </div>
          <div style="font-size: 13px; font-weight: 700; color: #fff;">${container ? container.title : 'No Container Selected'}</div>
          <div style="font-family: var(--font-mono); font-size: 10px; color: var(--text-muted); margin-bottom: 10px;">
            IRI: container:${selectedId || 'none'}
          </div>

          <!-- Triples / Nquins in Container -->
          <div style="font-size: 10px; font-family: var(--font-mono); color: var(--text-secondary); max-height: 140px; overflow-y: auto; background: var(--canvas-bg); padding: 8px; border-radius: var(--radius-xs);">
            ${subGraph.map(t => `
              <div style="margin-bottom: 4px;">
                <span style="color: var(--accent-cyan);">${t.p}</span>: <span style="color: var(--text-primary);">${t.o}</span>
              </div>
            `).join('')}
          </div>
        </div>

        <!-- Merkle-CRDT Root & Provenance -->
        <div style="background: var(--surface-panel-elevated); border: 1px solid var(--border-subtle); border-radius: var(--radius-sm); padding: 12px;">
          <div style="font-size: 10px; font-weight: 700; color: var(--accent-amber); text-transform: uppercase; margin-bottom: 6px;">
            Merkle-CRDT DAG Status
          </div>
          <div style="font-family: var(--font-mono); font-size: 11px; color: #fff;">
            Root: 0x8f4c2b9a71e06d33...a42
          </div>
          <div style="font-size: 11px; color: var(--text-muted); margin-top: 4px;">
            Total Quads/Nquins in Canvas: <strong>32</strong> (State-Based CRDT Compacted)
          </div>
        </div>

        <!-- Fiduciary Sentinel Permissions Scopes -->
        <div style="background: var(--surface-panel-elevated); border: 1px solid var(--border-subtle); border-radius: var(--radius-sm); padding: 12px;">
          <div style="display: flex; align-items: center; justify-content: space-between; margin-bottom: 6px;">
            <span style="font-size: 10px; font-weight: 700; color: var(--accent-emerald); text-transform: uppercase;">Sentinel Permissions</span>
            <span class="badge" style="background: rgba(0, 242, 169, 0.15); color: var(--accent-emerald); border: 1px solid var(--accent-emerald); font-size: 9px; padding: 1px 6px; border-radius: 4px;">Level 3 Verified</span>
          </div>
          <div style="font-size: 11px; color: var(--text-secondary);">
            • <code>graph:read/write</code>: Scope Granted<br>
            • <code>pulse:collaborate</code>: Multi-tenant Enabled<br>
            • <code>ontology:map</code>: Cross-Ontology Alignment Active
          </div>
        </div>

        <!-- Live Collaborative Pulse Feed -->
        <div style="background: var(--surface-panel-elevated); border: 1px solid var(--border-subtle); border-radius: var(--radius-sm); padding: 12px; flex: 1;">
          <div style="font-size: 10px; font-weight: 700; color: var(--accent-violet); text-transform: uppercase; margin-bottom: 6px;">
            Collaborative Pulse Feed
          </div>
          <div style="font-size: 10px; font-family: var(--font-mono); display: flex; flex-direction: column; gap: 6px;">
            ${canvasState.state.pulses.map(p => `
              <div style="padding: 6px; background: var(--canvas-bg); border-radius: var(--radius-xs);">
                <div style="color: var(--accent-cyan); font-weight: 600;">[${p.time}] ${p.source}</div>
                <div style="color: var(--text-primary); margin-top: 2px;">${p.msg}</div>
              </div>
            `).join('')}
          </div>
        </div>
      </div>
    `;

    const closeBtn = this.sidebar.querySelector('.btn-close-tech');
    if (closeBtn) {
      closeBtn.addEventListener('click', () => canvasState.toggleTechSidebar());
    }
  }
}

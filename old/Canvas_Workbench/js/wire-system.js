/**
 * HyperCanvas: Bezier Connection Wire System
 * Renders clickable connection wires with interactive ontology inspection and midpoint predicate badges.
 */

import { canvasState } from './state.js';

export class WireSystem {
  constructor(svgElement) {
    this.svg = svgElement;
  }

  render() {
    this.svg.innerHTML = '';
    const { connections, containers, selectedConnectionId } = canvasState.state;

    for (const conn of connections) {
      const src = containers.find(c => c.id === conn.from);
      const tgt = containers.find(c => c.id === conn.to);
      if (!src || !tgt) continue;

      // Calculate output port (right side of src) and input port (left side of tgt)
      const startX = src.x + src.width;
      const startY = src.y + src.height / 2;
      const endX = tgt.x;
      const endY = tgt.y + tgt.height / 2;

      const dx = Math.abs(endX - startX) * 0.5;
      const pathD = `M ${startX} ${startY} C ${startX + dx} ${startY}, ${endX - dx} ${endY}, ${endX} ${endY}`;

      const isSelected = conn.id === selectedConnectionId;

      // Wire Path Element
      const path = document.createElementNS('http://www.w3.org/2000/svg', 'path');
      path.setAttribute('d', pathD);
      path.setAttribute('class', `connection-wire wire-${conn.type || 'active'} ${isSelected ? 'selected-wire' : ''}`);
      path.setAttribute('data-id', conn.id);

      // Click wire to open Semantic Connector Relationship Inspector
      path.addEventListener('click', (e) => {
        e.stopPropagation();
        canvasState.selectConnection(conn.id);
        this.render();
        if (window.hyperWireInspector) {
          window.hyperWireInspector.open(conn);
        }
      });

      this.svg.appendChild(path);

      // Midpoint Predicate Label
      if (conn.label) {
        const midX = (startX + endX) / 2;
        const midY = (startY + endY) / 2;

        const text = document.createElementNS('http://www.w3.org/2000/svg', 'text');
        text.setAttribute('x', midX);
        text.setAttribute('y', midY - 6);
        text.setAttribute('class', 'wire-label-text');
        text.textContent = conn.label;

        text.addEventListener('click', (e) => {
          e.stopPropagation();
          canvasState.selectConnection(conn.id);
          this.render();
          if (window.hyperWireInspector) {
            window.hyperWireInspector.open(conn);
          }
        });

        this.svg.appendChild(text);
      }
    }
  }
}

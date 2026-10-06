/**
 * HyperCanvas: Container Sub-Graph Generator
 * Derives RDF 1.2 Nquins and graph triples from container content, ports, and connections.
 */

import { canvasState } from './state.js';

export class GraphGenerator {
  static getContainerSubGraph(containerId) {
    const container = canvasState.state.containers.find(c => c.id === containerId);
    if (!container) return [];

    const baseIri = `container:${container.id}`;
    const triples = [
      { s: baseIri, p: 'rdf:type', o: `qualia:${container.type.toUpperCase()}_Container` },
      { s: baseIri, p: 'qualia:title', o: `"${container.title}"` },
      { s: baseIri, p: 'qualia:spatialCoord', o: `"[x:${container.x}, y:${container.y}, z:${container.z || 0}]"` },
      { s: baseIri, p: 'qualia:inCanvas', o: canvasState.state.graphIri }
    ];

    if (container.graphTriples) {
      triples.push(...container.graphTriples);
    }

    // Add connection edges as Nquins
    const outgoing = canvasState.state.connections.filter(conn => conn.from === containerId);
    for (const edge of outgoing) {
      triples.push({
        s: baseIri,
        p: `qualia:connectsTo_${edge.type}`,
        o: `container:${edge.to}`,
        nquin: `<<[ ${baseIri} qualia:connectsTo container:${edge.to} ${canvasState.state.graphIri} did:qualia:timothy_charles_holborn ]>>`
      });
    }

    return triples;
  }

  static getGlobalCanvasGraph() {
    const all = [];
    for (const c of canvasState.state.containers) {
      all.push(...this.getContainerSubGraph(c.id));
    }
    return all;
  }
}

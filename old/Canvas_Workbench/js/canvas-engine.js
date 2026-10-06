/**
 * HyperCanvas: 2D / 3D / 4D Spatial Canvas Engine
 * Handles viewport transformations, pan/zoom, XYZD 3D/4D orbit projections,
 * Strata multi-deck elevations, and DateTime Span scrubbing with live GIS agent pin tracking.
 */

import { canvasState } from './state.js';

export class CanvasEngine {
  constructor(viewportEl, stageEl, containerManager) {
    this.viewport = viewportEl;
    this.stage = stageEl;
    this.containerManager = containerManager;
    this.isPanning = false;
    this.panStart = { x: 0, y: 0 };
    this.timePlayInterval = null;

    this.initEvents();
  }

  initEvents() {
    // Pan & Zoom
    this.viewport.addEventListener('mousedown', (e) => {
      if (e.target === this.viewport || e.target === this.stage || e.target.classList.contains('canvas-grid-svg')) {
        this.isPanning = true;
        this.panStart = { x: e.clientX - canvasState.state.pan.x, y: e.clientY - canvasState.state.pan.y };
      }
    });

    window.addEventListener('mousemove', (e) => {
      if (this.isPanning) {
        const newX = e.clientX - this.panStart.x;
        const newY = e.clientY - this.panStart.y;
        canvasState.setState({ pan: { x: newX, y: newY } });
        this.updateTransform();
      }
    });

    window.addEventListener('mouseup', () => {
      this.isPanning = false;
    });

    // Zoom Wheel
    this.viewport.addEventListener('wheel', (e) => {
      e.preventDefault();
      const zoomFactor = e.deltaY < 0 ? 1.1 : 0.9;
      const newZoom = Math.min(Math.max(canvasState.state.zoom * zoomFactor, 0.3), 3.0);
      canvasState.setState({ zoom: newZoom });
      this.updateTransform();
    });

    // Handle Dimension Mode Changes (2D / 3D / 4D)
    canvasState.subscribe('dimensionModeChanged', (mode) => {
      this.setDimensionMode(mode);
    });

    // Handle XYZD Coordinate Changes
    canvasState.subscribe('xyzdChanged', () => {
      this.updateTransform();
      this.containerManager.renderAll();
    });

    // Handle DateTime Changes
    canvasState.subscribe('timeChanged', (timeData) => {
      this.updateDateTimeUI(timeData);
      this.highlightTemporalContainers(timeData.currentDateTime);
      this.updateLiveAgentPins(timeData.progress);
    });

    // Handle Time Play / Pause
    canvasState.subscribe('playTimeToggled', (isPlaying) => {
      if (isPlaying) {
        this.startTimePlayback();
      } else {
        this.stopTimePlayback();
      }
    });

    this.updateTransform();
  }

  setDimensionMode(mode) {
    this.viewport.classList.toggle('mode-3d', mode === '3D' || mode === '4D');
    this.updateTransform();
    this.containerManager.renderAll();
  }

  updateTransform() {
    const { pan, zoom, dimensionMode, xyzd } = canvasState.state;
    const totalX = pan.x + (xyzd.x || 0);
    const totalY = pan.y + (xyzd.y || 0);
    const totalZ = (xyzd.z || 0) - 100;
    const dScale = xyzd.d || 1.0;

    if (dimensionMode === '3D') {
      this.stage.style.transform = `translate3d(${totalX}px, ${totalY}px, ${totalZ}px) scale(${zoom * dScale}) rotateX(32deg) rotateZ(-12deg)`;
    } else if (dimensionMode === '4D') {
      this.stage.style.transform = `translate3d(${totalX}px, ${totalY}px, ${totalZ}px) scale(${zoom * dScale}) rotateX(18deg) rotateY(6deg)`;
    } else {
      this.stage.style.transform = `translate(${totalX}px, ${totalY}px) scale(${zoom})`;
    }

    const zoomText = document.querySelector('#zoom-display');
    if (zoomText) zoomText.innerText = `${Math.round(zoom * 100)}%`;
  }

  startTimePlayback() {
    if (this.timePlayInterval) clearInterval(this.timePlayInterval);
    this.timePlayInterval = setInterval(() => {
      let next = canvasState.state.time.progress + 0.01;
      if (next > 1.0) next = 0.0;
      canvasState.setTimeProgress(next);
    }, 120);
  }

  stopTimePlayback() {
    if (this.timePlayInterval) {
      clearInterval(this.timePlayInterval);
      this.timePlayInterval = null;
    }
  }

  updateDateTimeUI(timeData) {
    const activeTimeEl = document.querySelector('#current-datetime-display');
    const spanStartEl = document.querySelector('#span-start-display');
    const spanEndEl = document.querySelector('#span-end-display');
    const slider = document.querySelector('#time-span-slider');

    const formatDt = (d) => {
      if (!(d instanceof Date) || isNaN(d)) return '';
      return d.toISOString().replace('T', ' ').substring(0, 19);
    };

    if (activeTimeEl) activeTimeEl.innerText = formatDt(timeData.currentDateTime).substring(11, 19);
    if (spanStartEl) spanStartEl.innerText = formatDt(timeData.spanStart).substring(5, 16);
    if (spanEndEl) spanEndEl.innerText = formatDt(timeData.spanEnd).substring(5, 16);
    if (slider) slider.value = Math.round(timeData.progress * 100);
  }

  updateLiveAgentPins(progress) {
    const mapContainer = canvasState.state.containers.find(c => c.type === 'map');
    if (!mapContainer || !mapContainer.agents) return;

    const mapNode = document.querySelector('#container-map-01');
    if (!mapNode) return;

    const getAgentPos = (path, p) => {
      if (!path || path.length === 0) return { x: 50, y: 50 };
      const idx = Math.min(Math.floor(p * (path.length - 1)), path.length - 2);
      const segP = (p * (path.length - 1)) - idx;
      const p1 = path[idx];
      const p2 = path[idx + 1] || p1;
      return {
        x: p1.x + (p2.x - p1.x) * segP,
        y: p1.y + (p2.y - p1.y) * segP
      };
    };

    const pins = mapNode.querySelectorAll('.agent-pin-marker');
    if (pins.length >= 2) {
      const uavPos = getAgentPos(mapContainer.agents[0]?.path, progress);
      const surveyorPos = getAgentPos(mapContainer.agents[1]?.path, progress);

      pins[0].setAttribute('transform', `translate(${uavPos.x}, ${uavPos.y})`);
      pins[1].setAttribute('transform', `translate(${surveyorPos.x}, ${surveyorPos.y})`);
    }
  }

  highlightTemporalContainers(currentDate) {
    const curMs = currentDate.getTime();
    document.querySelectorAll('.canvas-container-node').forEach(node => {
      const c = canvasState.state.containers.find(x => x.id === node.id);
      if (c && c.validFrom && c.validTo) {
        const fromMs = new Date(c.validFrom).getTime();
        const toMs = new Date(c.validTo).getTime();
        const isActive = curMs >= fromMs && curMs <= toMs;
        node.classList.toggle('temporally-inactive', !isActive);
      }
    });
  }

  resetView() {
    canvasState.setState({ zoom: 0.95, pan: { x: 90, y: 50 } });
    canvasState.setXYZD({ x: 0, y: 0, z: 0, d: 1.0 });
    this.updateTransform();
  }
}

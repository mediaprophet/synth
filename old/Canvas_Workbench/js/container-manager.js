/**
 * HyperCanvas: Container Manager & Multi-Type Renderer
 * Handles rendering, dragging (8px snap), resizing (8px snap), dynamic z-indexing,
 * multi-strata filtering, epistemic modalities, honesty badges, and in-container authoring instruments.
 */

import { canvasState } from './state.js';
import { getManifoldById } from '../manifolds/index.js';

const HONESTY_MAP = {
  doc: { level: 'partial', desc: 'WYSIWYG authoring + RDF spans' },
  code: { level: 'partial', desc: 'Vibe 0.1 AST interpreter preview' },
  ontology: { level: 'partial', desc: 'Aura / SHACL alignment matrix' },
  map: { level: 'partial', desc: 'Geospatial vector + UAV tracks' },
  social: { level: 'partial', desc: 'Chat graph + Nquin generator' },
  subcanvas: { level: 'partial', desc: 'Recursive manifold component' },
  sheet: { level: 'present', desc: 'P64 tensor table (formula: present)' },
  health: { level: 'present', desc: 'Generic adult bio-telemetry profile' },
  webrtc: { level: 'present', desc: 'Encrypted stream panel (mock stream)' },
  webview: { level: 'present', desc: 'Sandboxed RDF clipping frame' },
  '3d': { level: 'present', desc: '10D kinematic vocal tract mesh' },
  media: { level: 'partial', desc: 'Grapheme vector artwork card' },
  portal: { level: 'present', desc: 'Multi-tenant wormhole bridge' }
};

export class ContainerManager {
  constructor(stageElement, wireSystem) {
    this.stage = stageElement;
    this.wireSystem = wireSystem;
    this.draggingContainer = null;
    this.resizingContainer = null;
    this.dragOffset = { x: 0, y: 0 };
    this.highestZIndex = 25;
    this.popoverEl = null;

    this.initPopover();
  }

  initPopover() {
    this.popoverEl = document.createElement('div');
    this.popoverEl.className = 'contextual-rdf-popover';
    this.popoverEl.style.display = 'none';
    document.body.appendChild(this.popoverEl);

    document.addEventListener('mousedown', (e) => {
      if (!this.popoverEl.contains(e.target) && !e.target.closest('.container-body')) {
        this.hidePopover();
      }
    });
  }

  showPopover(x, y, data) {
    this.popoverEl.style.left = `${Math.min(x, window.innerWidth - 320)}px`;
    this.popoverEl.style.top = `${Math.min(y, window.innerHeight - 200)}px`;
    this.popoverEl.style.display = 'flex';

    if (data.type === 'text') {
      this.popoverEl.innerHTML = `
        <div style="font-size: 10px; font-weight: 700; color: var(--accent-cyan); text-transform: uppercase;">
          Contextual RDF & Epistemic Grounding
        </div>
        <div class="popover-sel-text">"${data.text}"</div>
        <div class="popover-actions-grid">
          <button class="popover-action-btn" id="pop-tag-subj" style="color: var(--modality-subjective);">🧠 Tag Subjective Qualia</button>
          <button class="popover-action-btn" id="pop-tag-obj" style="color: var(--modality-objective);">🔬 Tag Objective Fact</button>
          <button class="popover-action-btn" id="pop-tag-entity">🏷️ Tag RDF Entity</button>
          <button class="popover-action-btn" id="pop-ai-enrich" style="color: var(--accent-cyan);">🤖 AI Enrich</button>
        </div>
      `;

      this.popoverEl.querySelector('#pop-tag-subj')?.addEventListener('click', () => {
        alert(`✓ Tagged "${data.text}" with Epistemic Modality qualia:SubjectiveQualia (First-Person Narrative)`);
        this.hidePopover();
      });
      this.popoverEl.querySelector('#pop-tag-obj')?.addEventListener('click', () => {
        alert(`✓ Tagged "${data.text}" with Epistemic Modality qualia:ObjectiveObservation (Empirically Verifiable)`);
        this.hidePopover();
      });
      this.popoverEl.querySelector('#pop-tag-entity')?.addEventListener('click', () => {
        alert(`✓ Tagged "${data.text}" as RDF Entity <concept:${data.text.replace(/\\s+/g, '_')}>`);
        this.hidePopover();
      });
      this.popoverEl.querySelector('#pop-ai-enrich')?.addEventListener('click', () => {
        alert(`🤖 AI Mindware: Grounded "${data.text}" across Subjective Qualia and Objective sensor telemetry.`);
        this.hidePopover();
      });

    } else if (data.type === 'cell') {
      this.popoverEl.innerHTML = `
        <div style="font-size: 10px; font-weight: 700; color: var(--accent-amber); text-transform: uppercase;">
          Columnar Cell RDF Grounding
        </div>
        <div class="popover-sel-text">Cell: ${data.col} = "<strong>${data.val}</strong>" (${data.row})</div>
        <div class="popover-actions-grid">
          <button class="popover-action-btn" id="pop-cell-rdf">🏷️ Ground in RDF Literal</button>
          <button class="popover-action-btn" id="pop-cell-latent">📐 P64 Neural Latent</button>
          <button class="popover-action-btn" id="pop-cell-formula">🧮 Compute Resonance</button>
        </div>
      `;

      this.popoverEl.querySelector('#pop-cell-rdf')?.addEventListener('click', () => {
        alert(`✓ Grounded cell "${data.val}" as datatype "142.5 L/m"^^hydro:FlowRate`);
        this.hidePopover();
      });
      this.popoverEl.querySelector('#pop-cell-latent')?.addEventListener('click', () => {
        alert(`✓ Aligned continuous EnCodec tensor [0.42, 0.88, 1.24] to 64-byte boundary.`);
        this.hidePopover();
      });
      this.popoverEl.querySelector('#pop-cell-formula')?.addEventListener('click', () => {
        alert(`🧮 Computed resonance frequency: 221.5 Hz across North Spring aquifer.`);
        this.hidePopover();
      });
    }
  }

  hidePopover() {
    if (this.popoverEl) {
      this.popoverEl.style.display = 'none';
    }
  }

  renderAll() {
    this.stage.querySelectorAll('.canvas-container-node').forEach(el => el.remove());
    for (const c of canvasState.state.containers) {
      this.renderContainer(c);
    }
    this.wireSystem.render();
  }

  renderContainer(c) {
    const isSelected = c.id === canvasState.state.selectedContainerId;
    const activeStrata = canvasState.state.activeStrata || [];
    const activeModality = canvasState.state.activeEpistemicModality || 'all';

    const isStrataDimmed = !activeStrata.includes('all') && c.strata && !activeStrata.includes(c.strata);
    const isModalityDimmed = activeModality !== 'all' && c.epistemic && c.epistemic !== activeModality;

    const honesty = HONESTY_MAP[c.type] || { level: 'partial', desc: 'Mock Component' };

    const node = document.createElement('div');
    node.className = `canvas-container-node ${isSelected ? 'selected' : ''} ${isStrataDimmed || isModalityDimmed ? 'strata-dimmed' : ''}`;
    node.id = c.id;
    node.style.left = `${c.x}px`;
    node.style.top = `${c.y}px`;
    node.style.width = `${c.width}px`;
    node.style.height = `${c.height}px`;

    if ((canvasState.state.dimensionMode === '3D' || canvasState.state.dimensionMode === '4D') && c.z !== undefined) {
      node.style.transform = `translateZ(${c.z}px) scale(${c.d || 1.0})`;
    }

    const strataClass = c.strata ? `strata-${c.strata}` : '';
    const epistemicClass = c.epistemic ? `modality-${c.epistemic}` : '';
    const epistemicIcon = c.epistemic === 'objective' ? '🔬' : (c.epistemic === 'subjective' ? '🧠' : (c.epistemic === 'intersubjective' ? '🌊' : '⚖️'));

    // Header with Honesty Badge & Epistemic Status
    const header = document.createElement('div');
    header.className = 'container-header';
    header.innerHTML = `
      <div class="container-title-group">
        <span class="container-type-tag tag-${c.type}">${c.type}</span>
        <span class="honesty-badge honesty-${honesty.level}" title="Honesty level: ${honesty.level} (${honesty.desc})">${honesty.level}</span>
        ${c.strata ? `<span class="strata-badge ${strataClass}">${c.strata}</span>` : ''}
        ${c.epistemic ? `<span class="modality-badge ${epistemicClass}">${epistemicIcon} ${c.epistemic}</span>` : ''}
        <span class="container-title">${c.title}</span>
      </div>
      <div class="container-actions">
        <span class="container-xyzd-badge" title="Spatial XYZD coordinates">[z:${c.z || 0}, d:${c.d || 1.0}]</span>
        <button class="container-action-btn btn-expand" title="Expand to Full Viewport (or Double-Click Header)">⛶</button>
        <button class="container-action-btn btn-focus" title="Inspect Node in Technical Sidebar">⚙️</button>
        <button class="container-action-btn btn-close" title="Close Container">&times;</button>
      </div>
    `;

    // In-Container Toolbar (for authoring containers)
    const inToolbar = this.getInContainerToolbar(c);

    // Body based on container type
    const body = document.createElement('div');
    body.className = 'container-body';
    body.innerHTML = this.getContainerBodyHtml(c);

    // Port pins
    const portIn = document.createElement('div');
    portIn.className = 'container-port port-in';
    portIn.title = 'Input Data / Event Port';

    const portOut = document.createElement('div');
    portOut.className = 'container-port port-out';
    portOut.title = 'Output Data / Event Port';

    // Corner Resizer
    const resizer = document.createElement('div');
    resizer.className = 'container-resizer';
    resizer.title = 'Drag corner to resize (8px snap)';

    // Multi-Edge Resizers (Left, Right, Bottom)
    const resizerLeft = document.createElement('div');
    resizerLeft.className = 'resizer-left';
    resizerLeft.title = 'Drag left edge to resize (8px snap)';

    const resizerRight = document.createElement('div');
    resizerRight.className = 'resizer-right';
    resizerRight.title = 'Drag right edge to resize (8px snap)';

    const resizerBottom = document.createElement('div');
    resizerBottom.className = 'resizer-bottom';
    resizerBottom.title = 'Drag bottom edge to resize (8px snap)';

    node.appendChild(header);
    if (inToolbar) node.appendChild(inToolbar);
    node.appendChild(body);
    node.appendChild(portIn);
    node.appendChild(portOut);
    node.appendChild(resizer);
    node.appendChild(resizerLeft);
    node.appendChild(resizerRight);
    node.appendChild(resizerBottom);

    this.stage.appendChild(node);
    this.attachContainerEvents(node, c, header, resizer, resizerLeft, resizerRight, resizerBottom);
    this.attachTypeSpecificHandlers(node, c, body, inToolbar);
  }

  getInContainerToolbar(c) {
    const bar = document.createElement('div');
    bar.className = 'in-container-toolbar';

    if (c.type === 'doc') {
      bar.innerHTML = `
        <div class="in-container-btn-group">
          <button class="in-container-btn btn-doc-bold" title="Bold"><strong>B</strong></button>
          <button class="in-container-btn btn-doc-italic" title="Italic"><em>I</em></button>
          <button class="in-container-btn btn-doc-code" title="Inline Code"><code>&lt;/&gt;</code></button>
          <button class="in-container-btn btn-doc-entity" title="Tag Ontological Entity" style="color: var(--accent-emerald);">🏷️ Entity</button>
          <button class="in-container-btn btn-doc-subj" title="Tag Subjective Qualia" style="color: var(--modality-subjective);">🧠 Qualia</button>
        </div>
        <div class="in-container-btn-group">
          <button class="in-container-btn active btn-doc-view" data-view="editor">Visual</button>
          <button class="in-container-btn btn-doc-view" data-view="markdown">Markdown</button>
          <button class="in-container-btn btn-doc-view" data-view="triples">Triples</button>
        </div>
      `;
      return bar;
    }

    if (c.type === 'code') {
      bar.innerHTML = `
        <div class="in-container-btn-group">
          <button class="in-container-btn active btn-code-tab" data-tab="editor">⚡ VibeScript 0.1</button>
          <button class="in-container-btn btn-code-tab" data-tab="ast">🌲 AST View</button>
          <button class="in-container-btn btn-code-tab" data-tab="gas">⛽ Gas: ${c.gasUsed || 350}</button>
        </div>
        <div class="in-container-btn-group">
          <button class="in-container-btn btn-insert-snippet" data-snippet="pulse" title="Insert pulse::emit">+ pulse</button>
          <button class="in-container-btn btn-insert-snippet" data-snippet="capability" title="Insert capability.invoke">+ invoke</button>
        </div>
      `;
      return bar;
    }

    if (c.type === 'ontology') {
      bar.innerHTML = `
        <div class="in-container-btn-group">
          <button class="in-container-btn active btn-onto-tab" data-tab="matrix">🔀 Alignment Matrix</button>
          <button class="in-container-btn btn-onto-tab" data-tab="classes">🏷️ Classes (${(c.classes || []).length})</button>
          <button class="in-container-btn btn-onto-tab" data-tab="shacl">🛡️ SHACL (100%)</button>
        </div>
        <div class="in-container-btn-group">
          <button class="in-container-btn btn-onto-add-mapping" style="color: var(--color-ontology);">+ Add Mapping</button>
        </div>
      `;
      return bar;
    }

    if (c.type === 'map') {
      bar.innerHTML = `
        <div class="in-container-btn-group">
          <button class="in-container-btn active btn-map-layer" data-layer="vectors">🌊 Flow</button>
          <button class="in-container-btn active btn-map-layer" data-layer="pins">📍 Pins</button>
          <button class="in-container-btn active btn-map-layer" data-layer="uav">🚁 UAV</button>
          <button class="in-container-btn active btn-map-layer" data-layer="trail">🚶 Survey</button>
        </div>
        <div class="in-container-btn-group">
          <span style="font-family: var(--font-mono); font-size: 9px; color: var(--accent-cyan);">34.0°S, 148.2°E</span>
        </div>
      `;
      return bar;
    }

    return null;
  }

  attachTypeSpecificHandlers(node, c, body, inToolbar) {
    // 1. Document / Observation Record In-Container Authoring & View Switching
    if (c.type === 'doc') {
      const editorArea = body.querySelector('.doc-editor-surface');
      const markdownArea = body.querySelector('.doc-markdown-surface');
      const triplesArea = body.querySelector('.doc-triples-surface');

      if (inToolbar) {
        inToolbar.querySelectorAll('.btn-doc-view').forEach(btn => {
          btn.addEventListener('click', () => {
            inToolbar.querySelectorAll('.btn-doc-view').forEach(b => b.classList.remove('active'));
            btn.classList.add('active');
            const view = btn.getAttribute('data-view');
            
            if (editorArea) editorArea.style.display = view === 'editor' ? 'block' : 'none';
            if (markdownArea) markdownArea.style.display = view === 'markdown' ? 'block' : 'none';
            if (triplesArea) triplesArea.style.display = view === 'triples' ? 'block' : 'none';
          });
        });

        inToolbar.querySelector('.btn-doc-bold')?.addEventListener('click', () => {
          document.execCommand('bold', false, null);
        });
        inToolbar.querySelector('.btn-doc-italic')?.addEventListener('click', () => {
          document.execCommand('italic', false, null);
        });
        inToolbar.querySelector('.btn-doc-code')?.addEventListener('click', () => {
          const sel = window.getSelection();
          if (sel && sel.rangeCount > 0) {
            const range = sel.getRangeAt(0);
            const codeEl = document.createElement('code');
            codeEl.style.cssText = 'background: rgba(0,210,255,0.12); color: var(--accent-cyan); padding: 1px 4px; border-radius: 2px; font-family: var(--font-mono);';
            codeEl.textContent = sel.toString() || 'entity';
            range.deleteContents();
            range.insertNode(codeEl);
          }
        });
        inToolbar.querySelector('.btn-doc-entity')?.addEventListener('click', () => {
          const sel = window.getSelection();
          if (sel && sel.rangeCount > 0) {
            const range = sel.getRangeAt(0);
            const text = sel.toString().trim() || 'NamedEntity';
            const entityEl = document.createElement('span');
            entityEl.className = 'q-entity-highlight';
            entityEl.style.cssText = 'color: #00f2a9; border-bottom: 2px solid #00f2a9; cursor: pointer;';
            entityEl.setAttribute('data-id', `concept:${text.replace(/\\s+/g, '_')}`);
            entityEl.textContent = text;
            range.deleteContents();
            range.insertNode(entityEl);
          }
        });
        inToolbar.querySelector('.btn-doc-subj')?.addEventListener('click', () => {
          const sel = window.getSelection();
          if (sel && sel.rangeCount > 0) {
            const range = sel.getRangeAt(0);
            const text = sel.toString().trim() || 'QualiaReflection';
            const badgeEl = document.createElement('span');
            badgeEl.className = 'modality-badge modality-subjective';
            badgeEl.style.cssText = 'font-size: 9px; padding: 1px 5px; border-radius: 4px; margin: 0 2px;';
            badgeEl.textContent = `🧠 ${text}`;
            range.deleteContents();
            range.insertNode(badgeEl);
          }
        });
      }

      body.addEventListener('mouseup', () => {
        const sel = window.getSelection();
        const text = sel?.toString().trim();
        if (text && text.length > 1) {
          const range = sel.getRangeAt(0);
          const rect = range.getBoundingClientRect();
          this.showPopover(rect.left, rect.bottom + 6, {
            containerId: c.id,
            type: 'text',
            text
          });
        }
      });
    }

    // 2. Spreadsheet Interactive Formula Bar & Cell Editor
    if (c.type === 'sheet') {
      const formulaInput = body.querySelector('.sheet-formula-input');
      const cellRef = body.querySelector('.sheet-cell-ref');
      let activeCell = null;

      body.querySelectorAll('td.sheet-data-cell').forEach(td => {
        td.addEventListener('click', (e) => {
          e.stopPropagation();
          body.querySelectorAll('td.sheet-data-cell').forEach(el => el.style.outline = 'none');
          td.style.outline = '2px solid var(--accent-amber)';
          activeCell = td;

          const col = td.getAttribute('data-col') || 'B';
          const row = td.getAttribute('data-row') || '1';
          if (cellRef) cellRef.textContent = `${col}${row}`;
          if (formulaInput) {
            formulaInput.value = td.textContent.trim();
            formulaInput.focus();
          }

          const rect = td.getBoundingClientRect();
          const th = td.getAttribute('data-colname') || 'Field';
          const site = td.closest('tr')?.cells[0]?.textContent || 'Row';

          this.showPopover(rect.left, rect.bottom + 6, {
            containerId: c.id,
            type: 'cell',
            col: th,
            val: td.textContent.trim(),
            row: site
          });
        });
      });

      if (formulaInput) {
        formulaInput.addEventListener('input', (e) => {
          if (activeCell) {
            activeCell.textContent = e.target.value;
          }
        });
      }
    }

    // 3. VibeScript 0.1 AST Inspector & Snippet Inserter
    if (c.type === 'code') {
      const editorSection = body.querySelector('.code-editor-section');
      const astSection = body.querySelector('.code-ast-section');
      const gasSection = body.querySelector('.code-gas-section');
      const textarea = body.querySelector('.vibescript-code-area');

      if (inToolbar) {
        inToolbar.querySelectorAll('.btn-code-tab').forEach(btn => {
          btn.addEventListener('click', () => {
            inToolbar.querySelectorAll('.btn-code-tab').forEach(b => b.classList.remove('active'));
            btn.classList.add('active');
            const tab = btn.getAttribute('data-tab');

            if (editorSection) editorSection.style.display = tab === 'editor' ? 'flex' : 'none';
            if (astSection) astSection.style.display = tab === 'ast' ? 'flex' : 'none';
            if (gasSection) gasSection.style.display = tab === 'gas' ? 'flex' : 'none';
          });
        });

        inToolbar.querySelectorAll('.btn-insert-snippet').forEach(btn => {
          btn.addEventListener('click', () => {
            const snip = btn.getAttribute('data-snippet');
            if (textarea) {
              if (snip === 'pulse') {
                textarea.value += '\n    pulse::emit("pulse:stream_active", { level: 3 });';
              } else if (snip === 'capability') {
                textarea.value += '\n    capability.invoke("graph.query", { query: "qualia:active" });';
              }
              c.code = textarea.value;
            }
          });
        });
      }

      const runBtn = body.querySelector('.btn-run-code');
      runBtn?.addEventListener('click', () => {
        alert('⚡ VibeScript 0.1 AST Interpreter:\nAST compiled successfully.\nGas consumed: 48 units. Pulse emitted to Merkle DAG.');
      });
    }

    // 4. Ontology Alignment Matrix Interactive Inserter
    if (c.type === 'ontology') {
      const matrixSec = body.querySelector('.onto-matrix-section');
      const classesSec = body.querySelector('.onto-classes-section');
      const shaclSec = body.querySelector('.onto-shacl-section');

      if (inToolbar) {
        inToolbar.querySelectorAll('.btn-onto-tab').forEach(btn => {
          btn.addEventListener('click', () => {
            inToolbar.querySelectorAll('.btn-onto-tab').forEach(b => b.classList.remove('active'));
            btn.classList.add('active');
            const tab = btn.getAttribute('data-tab');

            if (matrixSec) matrixSec.style.display = tab === 'matrix' ? 'block' : 'none';
            if (classesSec) classesSec.style.display = tab === 'classes' ? 'block' : 'none';
            if (shaclSec) shaclSec.style.display = tab === 'shacl' ? 'block' : 'none';
          });
        });

        inToolbar.querySelector('.btn-onto-add-mapping')?.addEventListener('click', () => {
          if (!c.mappings) c.mappings = [];
          c.mappings.push({
            source: `hydro:Metric_${Date.now().toString().slice(-4)}`,
            relation: 'skos:exactMatch',
            target: 'fiduciary:WaterAsset'
          });
          this.renderAll();
        });
      }

      body.querySelector('.btn-add-mapping-inline')?.addEventListener('click', () => {
        if (!c.mappings) c.mappings = [];
        c.mappings.push({
          source: `hydro:Metric_${Date.now().toString().slice(-4)}`,
          relation: 'skos:exactMatch',
          target: 'fiduciary:WaterAsset'
        });
        this.renderAll();
      });
    }

    // 5. GIS Map Layer Toggles
    if (c.type === 'map' && inToolbar) {
      inToolbar.querySelectorAll('.btn-map-layer').forEach(btn => {
        btn.addEventListener('click', () => {
          btn.classList.toggle('active');
          const layer = btn.getAttribute('data-layer');
          const svgPath = body.querySelector(`.map-layer-${layer}`);
          if (svgPath) {
            svgPath.style.display = btn.classList.contains('active') ? 'block' : 'none';
          }
        });
      });
    }

    // 6. Social & AI Chat Graph Handler
    if (c.type === 'social') {
      const sendBtn = body.querySelector('.btn-send-chat');
      const inputField = body.querySelector('.chat-input-field');
      const agentSelect = body.querySelector('.chat-agent-select');
      const modalitySelect = body.querySelector('.chat-modality-select');

      const handleSend = () => {
        const text = inputField?.value?.trim();
        if (text) {
          const sender = agentSelect ? agentSelect.value : 'Timothy Charles Holborn';
          const epistemic = modalitySelect ? modalitySelect.value : 'subjective';
          canvasState.addChatMessage(c.id, text, sender, epistemic);
          inputField.value = '';
          this.renderAll();
        }
      };

      sendBtn?.addEventListener('click', handleSend);
      inputField?.addEventListener('keydown', (e) => {
        if (e.key === 'Enter') handleSend();
      });
    }

    // 7. Sub-Manifold & Nested Graph Navigation Handler
    if (c.type === 'subcanvas' || c.type === 'manifold' || c.type === 'portal') {
      const btnEnter = body.querySelector('.btn-traverse-manifold, .btn-enter-subcanvas');
      btnEnter?.addEventListener('click', () => {
        const targetId = btnEnter.getAttribute('data-target') || c.targetManifoldId || c.targetCanvasId || c.targetCanvas;
        if (window.hyperCanvasSwitcher && targetId) {
          window.hyperCanvasSwitcher.switchCanvas(targetId);
        } else {
          alert(`Traversing to Sub-Manifold: ${targetId}`);
        }
      });
    }

    // 8. WebRTC Live Stream Controls Handler
    if (c.type === 'webrtc') {
      const btnMute = body.querySelector('#webrtc-btn-mute');
      const btnCam = body.querySelector('#webrtc-btn-cam');
      const btnShare = body.querySelector('#webrtc-btn-share');

      btnMute?.addEventListener('click', () => {
        btnMute.classList.toggle('active');
        btnMute.innerText = btnMute.classList.contains('active') ? '🔇 Muted' : '🎙️ Mic On';
      });
      btnCam?.addEventListener('click', () => {
        btnCam.classList.toggle('active');
        btnCam.innerText = btnCam.classList.contains('active') ? '📷 Cam Off' : '🎥 Cam On';
      });
      btnShare?.addEventListener('click', () => {
        alert('WebRTC: Screen share feed established with Project Authors Mesh.');
      });
    }

    // 9. Health Sub-panel Tab Switcher Handler
    if (c.type === 'health') {
      body.querySelectorAll('.health-tab-btn').forEach(btn => {
        btn.addEventListener('click', () => {
          body.querySelectorAll('.health-tab-btn').forEach(b => b.classList.remove('active'));
          btn.classList.add('active');
          const tab = btn.getAttribute('data-tab');
          c.activeTab = tab;
          
          body.querySelectorAll('.health-tab-section').forEach(sec => {
            sec.style.display = sec.getAttribute('data-tab') === tab ? 'block' : 'none';
          });
        });
      });
    }

    // 10. Webview Navigation & RDF Clipping Handler
    if (c.type === 'webview') {
      const addrInput = body.querySelector('.webview-address-bar');
      const btnGo = body.querySelector('.btn-webview-go');
      const btnReload = body.querySelector('.btn-webview-reload');
      const btnClip = body.querySelector('.btn-webview-clip');

      btnGo?.addEventListener('click', () => {
        c.url = addrInput.value;
        alert(`Webview navigated to: ${c.url}`);
      });
      btnReload?.addEventListener('click', () => {
        alert(`Reloaded sandboxed frame: ${c.url}`);
      });
      btnClip?.addEventListener('click', () => {
        alert(`✓ Webview RDF Extractor: Extracted 6 semantic entities from "${c.url}" into Merkle DAG.`);
      });
    }

    // 11. Double-Entry Ledger Transaction Posting Handler
    if (c.type === 'ledger') {
      const btnPost = body.querySelector('.btn-post-quad-tx');
      btnPost?.addEventListener('click', () => {
        const newId = `tx-0${(c.entries?.length || 0) + 1}`;
        c.entries = c.entries || [];
        const newTx = {
          id: newId,
          date: '2026-08-16 17:25',
          accountDr: 'asset:Catchment_Field_Hardware',
          accountCr: 'liability:Accounts_Payable_Vendor',
          amount: '$4,200.00',
          memo: 'Telemetry sensor deployment quad commitment',
          quad: `<<[ ${newId} fin:debit asset:Hardware <graph:ledger> ]>>`
        };
        c.entries.push(newTx);

        const tbody = body.querySelector('tbody');
        if (tbody) {
          const row = document.createElement('tr');
          row.style.cssText = 'border-bottom: 1px solid rgba(255,255,255,0.04);';
          row.title = newTx.memo;
          row.innerHTML = `
            <td style="padding: 4px 6px; color: var(--accent-cyan);">${newTx.id}</td>
            <td style="padding: 4px 6px; color: #fff;">${newTx.accountDr}</td>
            <td style="padding: 4px 6px; color: var(--text-muted);">${newTx.accountCr}</td>
            <td style="padding: 4px 6px; text-align: right; color: var(--accent-emerald); font-weight: 600;">${newTx.amount}</td>
          `;
          tbody.appendChild(row);
        }
        alert(`➕ Posted Double-Entry RDF Quad Transaction: ${newId}\nDebit: asset:Catchment_Field_Hardware ($4,200.00)\nCredit: liability:Accounts_Payable_Vendor\nStatus: SHACL Validated & Committed to Merkle DAG.`);
      });
    }

    // 12. Project Milestone DAG & Cross-Manifold Escrow Trigger Handler
    if (c.type === 'project_dag') {
      const btnEscrow = body.querySelector('.btn-trigger-escrow');
      btnEscrow?.addEventListener('click', () => {
        if (c.projects && c.projects[0]) {
          c.projects[0].status = '✅ Escrow Released ($15,000 Disbursed)';
          c.projects[0].progress = 100;
          body.innerHTML = this.getContainerBodyHtml(c);
          this.attachTypeSpecificHandlers(node, c, body, inToolbar);
        }
        alert(`⚡ Cross-Manifold Causal Wire Triggered!\n\nSource: Milestone M3: Catchment Sensor Deployment (Collaborative Projects Nexus)\nTarget: Financial Economics Manifold (Escrow Pool: $15,000.00)\n\nFiduciary Status: 3-of-5 Multi-Sig Authorized.\nFunds Disbursed to Catchment Field Research Team.`);
      });
    }
  }

  getContainerBodyHtml(c) {
    const progress = canvasState.state.time.progress;

    switch (c.type) {
      case 'doc':
        return `
          <div class="doc-container-views" style="flex: 1; display: flex; flex-direction: column;">
            <!-- Visual / WYSIWYG Editable Surface -->
            <div class="doc-editor-surface" contenteditable="true" style="flex: 1; line-height: 1.6; color: var(--text-primary); outline: none; padding: 4px; user-select: text;">
              ${c.content || '<h2>North Spring Observation Record</h2><p>Author: <strong>Timothy Charles Holborn</strong></p><p>Epistemic Status: <span class="modality-badge modality-subjective">🧠 Subjective Qualia</span></p><p>The primary monitoring spring (<span class="q-entity-highlight" style="color: #00f2a9; border-bottom: 2px solid #00f2a9;">North Spring</span>) is the project’s reference site for seasonal flow, water quality, and field observation.</p>'}
            </div>

            <!-- Markdown Source View (Hidden by default) -->
            <div class="doc-markdown-surface" style="display: none; flex: 1;">
              <textarea style="width: 100%; height: 100%; background: var(--canvas-bg); color: var(--color-doc); font-family: var(--font-mono); font-size: 11px; border: 1px solid var(--border-subtle); padding: 8px; border-radius: var(--radius-xs); resize: none;"># North Spring Catchment Field Notes\n\nAuthor: **Timothy Charles Holborn**\nEpistemic: qualia:PhenomenologicalQualia\n\nSite [North Spring](concept:FreshwaterSpring) flow monitoring record.</textarea>
            </div>

            <!-- RDF Triples View (Hidden by default) -->
            <div class="doc-triples-surface" style="display: none; flex: 1; background: var(--canvas-bg); border: 1px solid var(--border-subtle); border-radius: var(--radius-xs); padding: 8px; font-family: var(--font-mono); font-size: 10px; color: var(--accent-cyan); overflow-y: auto;">
              <div>&lt;doc:CatchmentNotes&gt; qualia:author &lt;entity:Timothy_Charles_Holborn&gt; .</div>
              <div>&lt;doc:CatchmentNotes&gt; qualia:epistemicStatus qualia:PhenomenologicalQualia .</div>
              <div>&lt;doc:CatchmentNotes&gt; qualia:referencesSite &lt;concept:NorthSpring&gt; .</div>
            </div>
          </div>
        `;

      case 'sheet':
        return `
          <div style="flex: 1; display: flex; flex-direction: column;">
            <!-- Formula Bar -->
            <div class="sheet-formula-bar">
              <span class="sheet-cell-ref">B1</span>
              <span style="color: var(--text-muted); font-size: 10px; font-weight: bold;">fx</span>
              <input type="text" class="sheet-formula-input" value="142.5 L/m" />
            </div>

            <!-- Columnar Table -->
            <div style="flex: 1; overflow-x: auto; margin-top: 4px;">
              <table style="width: 100%; border-collapse: collapse; font-family: var(--font-mono); font-size: 11px; text-align: left;">
                <thead>
                  <tr style="background: var(--surface-panel-elevated); color: var(--accent-amber); border-bottom: 1px solid var(--border-subtle);">
                    <th style="padding: 4px 6px; width: 24px; color: var(--text-muted);">#</th>
                    <th style="padding: 4px 6px;">Site (A)</th>
                    <th style="padding: 4px 6px;">Flow (B)</th>
                    <th style="padding: 4px 6px;">Resonance (C)</th>
                    <th style="padding: 4px 6px;">P64 Latent (D)</th>
                  </tr>
                </thead>
                <tbody>
                  ${(c.rows || [
                    { site: 'North Spring Flow', flow: '142.5 L/m', resonance: '221.5 Hz', acousticLatent: '[0.42, 0.88, 1.24]' },
                    { site: 'Upper Lagoon', flow: '98.2 L/m', resonance: '184.0 Hz', acousticLatent: '[0.35, 0.72, 1.10]' },
                    { site: 'Coastal Gauge', flow: '165.0 L/m', resonance: '246.8 Hz', acousticLatent: '[0.58, 0.94, 1.32]' }
                  ]).map((r, i) => `
                    <tr style="border-bottom: 1px solid var(--border-subtle);">
                      <td style="padding: 4px 6px; color: var(--text-muted); font-size: 9px;">${i + 1}</td>
                      <td class="sheet-data-cell" data-col="A" data-row="${i+1}" data-colname="Site" style="padding: 4px 6px; color: #fff; cursor: pointer;">${r.site}</td>
                      <td class="sheet-data-cell" data-col="B" data-row="${i+1}" data-colname="Flow" style="padding: 4px 6px; color: var(--accent-cyan); cursor: pointer;">${r.flow}</td>
                      <td class="sheet-data-cell" data-col="C" data-row="${i+1}" data-colname="Resonance" style="padding: 4px 6px; color: var(--accent-emerald); cursor: pointer;">${r.resonance}</td>
                      <td class="sheet-data-cell" data-col="D" data-row="${i+1}" data-colname="P64 Latent" style="padding: 4px 6px; color: var(--text-muted); cursor: pointer;">${r.acousticLatent}</td>
                    </tr>
                  `).join('')}
                </tbody>
              </table>
            </div>
          </div>
        `;

      case 'code':
        return `
          <div style="flex: 1; display: flex; flex-direction: column;">
            <!-- 1. Code Editor View -->
            <div class="code-editor-section" style="flex: 1; display: flex; flex-direction: column;">
              <textarea class="vibescript-code-area" style="flex: 1; background: var(--canvas-bg); color: var(--color-code); font-family: var(--font-mono); font-size: 11px; border: 1px solid var(--border-subtle); padding: 8px; border-radius: var(--radius-xs); resize: none;" spellcheck="false">${c.code || 'requires [graph, pulse];\n\nfn evaluate_node() -> Result<(), Error> {\n    pulse::emit("pulse:node_ready", {});\n    Ok(())\n}'}</textarea>
              <div style="display: flex; align-items: center; justify-content: space-between; margin-top: 6px;">
                <button class="btn btn-sm btn-run-code" style="background: var(--color-code); color: #000; font-weight: 700; border: none; padding: 4px 10px; border-radius: var(--radius-xs); cursor: pointer;">▶ Run (Gas: ${c.gasUsed || 350})</button>
                <span style="font-family: var(--font-mono); font-size: 9px; color: var(--text-muted);">AST: Homoiconic · interpreter: poet-vibe 0.1</span>
              </div>
            </div>

            <!-- 2. AST View (Hidden by default) -->
            <div class="code-ast-section code-ast-container" style="display: none;">
              <div class="ast-node-row" style="color: var(--accent-cyan);">📁 Program (VibeScript 0.1 AST)</div>
              <div class="ast-node-row" style="padding-left: 14px; color: var(--accent-amber);">🔒 RequiresDeclaration [graph, pulse]</div>
              <div class="ast-node-row" style="padding-left: 14px; color: var(--accent-emerald);">⚡ FunctionDecl: evaluate_node() -> Result&lt;(), Error&gt;</div>
              <div class="ast-node-row" style="padding-left: 28px; color: var(--text-secondary);">↳ PulseEmitStmt("pulse:node_ready", {})</div>
              <div class="ast-node-row" style="padding-left: 28px; color: var(--text-secondary);">↳ ReturnExpr(Ok(()))</div>
            </div>

            <!-- 3. Gas Section (Hidden by default) -->
            <div class="code-gas-section" style="display: none; background: var(--canvas-bg); border: 1px solid var(--border-subtle); border-radius: var(--radius-xs); padding: 8px; font-family: var(--font-mono); font-size: 10px;">
              <div style="color: var(--accent-amber); font-weight: bold; margin-bottom: 4px;">⛽ Fiduciary Gas Metering</div>
              <div>Base Invocation: 120 gas</div>
              <div>Graph Triple Traversal: 180 gas</div>
              <div>Pulse Emit Dispatch: 50 gas</div>
              <div style="border-top: 1px solid var(--border-subtle); margin-top: 4px; padding-top: 4px; color: #fff;">Total: 350 / 1000 Units Available</div>
            </div>
          </div>
        `;

      case 'ontology':
        return `
          <div class="ontology-card" style="flex: 1; display: flex; flex-direction: column;">
            <!-- Matrix Tab -->
            <div class="onto-matrix-section" style="flex: 1; display: flex; flex-direction: column;">
              <div class="ontology-uri-badge">${c.ontologyIri || 'https://qualia.io/ns/hydro-catchment#'}</div>
              
              <div style="font-size: 10px; font-weight: 700; color: var(--text-muted); text-transform: uppercase; margin: 4px 0 2px 0;">
                Cross-Ontology Alignment Matrix (Aura / SHACL)
              </div>
              
              <div style="flex: 1; max-height: 120px; overflow-y: auto; background: var(--canvas-bg); border-radius: var(--radius-xs); border: 1px solid var(--border-subtle);">
                <table class="mapping-table" style="width: 100%; border-collapse: collapse; font-family: var(--font-mono); font-size: 10px;">
                  <thead>
                    <tr style="background: var(--surface-panel-elevated); color: var(--text-muted);">
                      <th style="padding: 2px 4px;">Source Term</th>
                      <th style="padding: 2px 4px;">Bridge</th>
                      <th style="padding: 2px 4px;">Target Term</th>
                    </tr>
                  </thead>
                  <tbody>
                    ${(c.mappings || [
                      { source: 'concept:NorthSpring', relation: 'owl:equivalentClass', target: 'hydro:FreshwaterResurgence' },
                      { source: 'qualia:author', relation: 'rdfs:subPropertyOf', target: 'fiduciary:dutyOfCare' }
                    ]).map(m => `
                      <tr style="border-bottom: 1px solid var(--border-subtle);">
                        <td style="padding: 2px 4px; color: var(--accent-cyan);">${m.source}</td>
                        <td style="padding: 2px 4px; color: var(--accent-amber); font-size: 9px;">${m.relation}</td>
                        <td style="padding: 2px 4px; color: var(--accent-emerald);">${m.target}</td>
                      </tr>
                    `).join('')}
                  </tbody>
                </table>
              </div>

              <div style="display: flex; justify-content: space-between; align-items: center; margin-top: 4px;">
                <button class="btn btn-sm btn-add-mapping-inline" style="font-size: 9px; background: rgba(139, 92, 246, 0.2); color: var(--color-ontology); border: 1px solid var(--color-ontology); padding: 2px 6px; border-radius: 2px; cursor: pointer;">+ Add Alignment Row</button>
                <span style="font-family: var(--font-mono); font-size: 9px; color: var(--text-muted);">SHACL Conformance: 100%</span>
              </div>
            </div>

            <!-- Classes Tab (Hidden by default) -->
            <div class="onto-classes-section" style="display: none; flex: 1; background: var(--canvas-bg); border-radius: var(--radius-xs); border: 1px solid var(--border-subtle); padding: 8px;">
              <div style="font-size: 10px; font-weight: bold; color: var(--color-ontology); margin-bottom: 4px;">Declared Aura Classes:</div>
              <div style="display: flex; gap: 4px; flex-wrap: wrap;">
                ${(c.classes || ['MonitoringSite', 'FreshwaterResurgence', 'WaterRight']).map(cls => `
                  <span class="badge" style="background: rgba(139, 92, 246, 0.2); color: var(--color-ontology); border: 1px solid var(--color-ontology); font-size: 9px; padding: 2px 6px;">${cls}</span>
                `).join('')}
              </div>
            </div>

            <!-- SHACL Tab (Hidden by default) -->
            <div class="onto-shacl-section" style="display: none; flex: 1; background: var(--canvas-bg); border-radius: var(--radius-xs); border: 1px solid var(--border-subtle); padding: 8px; font-family: var(--font-mono); font-size: 10px;">
              <div style="color: var(--accent-emerald); font-weight: bold; margin-bottom: 4px;">🛡️ SHACL Shape Validation</div>
              <div>• Shape: hydro:CatchmentShape (Conforming)</div>
              <div>• Constraint: qualia:author minCount 1 (Satisfied)</div>
              <div>• Constraint: hydro:flowRate datatype xsd:decimal (Satisfied)</div>
            </div>
          </div>
        `;

      case 'map':
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

        const uavPos = getAgentPos(c.agents?.[0]?.path, progress);
        const surveyorPos = getAgentPos(c.agents?.[1]?.path, progress);

        return `
          <div class="map-canvas-container" style="flex: 1; display: flex; flex-direction: column;">
            <div class="map-hud-overlay">
              📍 ${c.geoRegion || 'North Spring Catchment'} • Spatiotemporal GIS
            </div>
            
            <svg class="map-svg-viewport" viewBox="0 0 420 250" style="flex: 1; width: 100%; height: 100%;">
              <path class="map-layer-vectors" d="M 0 40 Q 60 20 120 70 T 240 90 T 360 40 T 420 80 L 420 250 L 0 250 Z" fill="#0f1f33" stroke="#1e3a5f" stroke-width="1.5" />
              <path class="map-layer-vectors" d="M 60 200 Q 140 160 200 180 T 320 150 L 360 250 L 50 250 Z" fill="#0d2847" />
              
              <path class="map-layer-uav" d="M 70 190 Q 160 140 260 110 T 380 80" class="agent-trail-path" stroke="#00d2ff" fill="none" stroke-width="1.5" stroke-dasharray="4 2" />
              <path class="map-layer-trail" d="M 120 220 Q 200 170 250 135 T 320 160" class="agent-trail-path" stroke="#38bdf8" fill="none" stroke-width="1.5" />

              <g class="map-layer-pins" transform="translate(260, 110)">
                <circle r="14" fill="none" stroke="var(--accent-rose)" stroke-width="1.5" opacity="0.6">
                  <animate attributeName="r" values="6;20" dur="2s" repeatCount="indefinite" />
                  <animate attributeName="opacity" values="0.8;0" dur="2s" repeatCount="indefinite" />
                </circle>
                <circle r="5" fill="var(--accent-rose)" />
                <text x="10" y="4" fill="var(--accent-rose)" font-size="9" font-family="monospace" font-weight="bold">Incident #01: Low Flow</text>
              </g>

              <g transform="translate(${uavPos.x}, ${uavPos.y})" class="agent-pin-marker">
                <circle r="7" fill="#00d2ff" filter="drop-shadow(0 0 6px rgba(0, 210, 255, 0.8))" />
                <circle r="3" fill="#fff" />
                <text x="10" y="-4" fill="#00d2ff" font-size="9" font-family="monospace">UAV Sentinel-01</text>
              </g>

              <g transform="translate(${surveyorPos.x}, ${surveyorPos.y})" class="agent-pin-marker">
                <circle r="7" fill="#38bdf8" filter="drop-shadow(0 0 6px rgba(56, 189, 248, 0.8))" />
                <circle r="3" fill="#fff" />
                <text x="10" y="12" fill="#38bdf8" font-size="9" font-family="monospace">Timothy Holborn</text>
              </g>
            </svg>
          </div>
        `;

      case 'subcanvas':
        const target = getManifoldById(c.targetCanvasId);
        return `
          <div style="flex: 1; background: #0c121e; border-radius: var(--radius-xs); border: 1px dashed var(--accent-cyan); padding: 10px; display: flex; flex-direction: column; gap: 8px;">
            <div style="display: flex; align-items: center; justify-content: space-between;">
              <span style="font-size: 11px; font-weight: 700; color: var(--accent-cyan);">${target ? target.icon : '🌌'} ${target ? target.title : 'Embedded Sub-Manifold'}</span>
              <span class="badge" style="font-size: 8px; background: rgba(0, 210, 255, 0.15); color: var(--accent-cyan);">Recursive Manifold Component</span>
            </div>

            <div style="flex: 1; background: var(--canvas-bg); border-radius: var(--radius-xs); border: 1px solid var(--border-subtle); padding: 8px; display: flex; flex-direction: column; justify-content: center; gap: 6px;">
              <div style="font-size: 10px; color: var(--text-muted);">
                Embedded Graph: <code>${target ? target.graphIri : 'graph:subcanvas'}</code>
              </div>
              <div style="display: flex; gap: 4px; flex-wrap: wrap;">
                ${(target?.containers || []).map(n => `
                  <span class="container-type-tag tag-${n.type}" style="font-size: 8px;">${n.title.substring(0, 18)}</span>
                `).join('')}
              </div>
            </div>

            <button class="btn btn-sm btn-enter-subcanvas" style="background: var(--surface-panel-elevated); border: 1px solid var(--accent-cyan); color: var(--accent-cyan); font-weight: 700; font-size: 10px; cursor: pointer; border-radius: var(--radius-xs); padding: 4px 8px;">
              Enter Sub-Manifold Viewport ➔
            </button>
          </div>
        `;

      case 'social':
        return `
          <div class="social-chat-container">
            <div class="chat-subagents-bar">
              <span style="font-size: 9px; font-weight: 700; color: var(--color-social); text-transform: uppercase;">Sub-Agents:</span>
              ${(c.subAgents || [
                { id: 'agent:SentinelPlanner', name: 'Sentinel Planner AI', role: 'Telemetry Coordinator', status: 'Online' },
                { id: 'agent:LegalCounsel', name: 'Fiduciary Rights AI', role: 'Water Rights Guardian', status: 'Online' }
              ]).map(a => `
                <div class="agent-chip ${a.status === 'Active' || a.status === 'Online' ? 'active' : ''}" title="${a.role}">
                  <span>🤖</span>
                  <span>${a.name}</span>
                </div>
              `).join('')}
            </div>

            <div class="chat-messages-log">
              ${(c.messages || [
                { id: 'm1', sender: 'Timothy Charles Holborn', isAi: false, time: '14:38', text: 'North Spring looks clearer after last night’s rain; the field notes should capture that.', epistemic: 'subjective', graphNode: 'node:chat_01_holborn' },
                { id: 'm2', sender: 'Sentinel Planner AI', isAi: true, time: '14:39', text: 'Telemetry sensor confirms flow at 142.5 L/m with 221.5 Hz acoustic resonance.', epistemic: 'objective', graphNode: 'node:chat_02_sentinel' }
              ]).map(m => {
                const icon = m.epistemic === 'objective' ? '🔬' : (m.epistemic === 'subjective' ? '🧠' : '⚖️');
                const badgeClass = `modality-${m.epistemic || 'subjective'}`;
                return `
                  <div class="chat-message-bubble ${m.isAi ? 'is-ai' : ''}">
                    <div class="chat-msg-header">
                      <strong style="color: ${m.isAi ? 'var(--accent-cyan)' : '#fff'};">${m.sender}</strong>
                      <span style="display: flex; align-items: center; gap: 4px;">
                        <span class="modality-badge ${badgeClass}" style="font-size: 8px;">${icon} ${m.epistemic || 'subjective'}</span>
                        <span class="chat-graph-pill">${m.graphNode}</span>
                        <span style="color: var(--text-muted); font-size: 9px;">${m.time}</span>
                      </span>
                    </div>
                    <div style="color: var(--text-primary); margin-top: 2px;">${m.text}</div>
                  </div>
                `;
              }).join('')}
            </div>

            <div class="chat-input-bar">
              <select class="chat-agent-select" style="background: var(--surface-panel-elevated); color: var(--text-secondary); border: 1px solid var(--border-medium); font-size: 10px; border-radius: var(--radius-xs); padding: 0 4px;">
                <option value="Timothy Charles Holborn">👤 Timothy Charles Holborn</option>
                <option value="Sentinel Planner AI">🤖 Sentinel Planner AI</option>
                <option value="Fiduciary Rights AI">⚖️ Fiduciary Rights AI</option>
              </select>
              <select class="chat-modality-select" title="Select Epistemic Modality" style="background: var(--surface-panel-elevated); color: var(--accent-cyan); border: 1px solid var(--border-medium); font-size: 10px; border-radius: var(--radius-xs); padding: 0 4px;">
                <option value="subjective">🧠 Subjective Qualia</option>
                <option value="objective">🔬 Objective Telemetry</option>
                <option value="normative">⚖️ Normative Statute</option>
                <option value="intersubjective">🌊 Intersubjective</option>
              </select>
              <input type="text" class="chat-input-field" placeholder="Collaborate with users & AI sub-agents (generates chat graph quads)..." />
              <button class="btn btn-sm btn-send-chat" style="background: var(--color-social); color: #fff; border: none; font-weight: 700; border-radius: var(--radius-xs); padding: 0 10px; cursor: pointer;">Send</button>
            </div>
          </div>
        `;

      case 'webrtc':
        return `
          <div class="webrtc-stream-container">
            <div class="webrtc-video-viewport">
              <span style="font-size: 38px; filter: drop-shadow(0 0 12px var(--color-webrtc));">🎥</span>
              <span style="font-family: var(--font-mono); font-size: 10px; color: #fff; margin-top: 6px;">Encrypted Audio/Video Channel</span>
              <span style="font-family: var(--font-mono); font-size: 9px; color: var(--color-webrtc);">${c.peerId || 'did:qualia:peer_catchment_gateway'}</span>
              <div style="position: absolute; bottom: 6px; left: 8px; font-family: var(--font-mono); font-size: 8px; color: var(--text-muted);">
                ${c.bitrate || '2.4 Mbps (AV1 / Opus Encrypted)'} · Fidelity: Present
              </div>
            </div>

            <div class="webrtc-controls-row">
              <button class="webrtc-btn" id="webrtc-btn-mute">🎙️ Mic On</button>
              <button class="webrtc-btn" id="webrtc-btn-cam">🎥 Cam On</button>
              <button class="webrtc-btn" id="webrtc-btn-share">🖥️ Share</button>
              <button class="webrtc-btn active" style="background: rgba(239, 68, 68, 0.3); border-color: var(--color-webrtc);">🔴 End Call</button>
            </div>
          </div>
        `;

      case 'health':
        const activeTab = c.activeTab || 'pathology';
        return `
          <div class="health-panel-container">
            <div class="health-tab-bar">
              <button class="health-tab-btn ${activeTab === 'pathology' ? 'active' : ''}" data-tab="pathology">🧪 Pathology</button>
              <button class="health-tab-btn ${activeTab === 'imaging' ? 'active' : ''}" data-tab="imaging">🩻 Imaging</button>
              <button class="health-tab-btn ${activeTab === 'anatomy' ? 'active' : ''}" data-tab="anatomy">🫀 Anatomy</button>
            </div>

            <div class="health-tab-content">
              <!-- 1. Pathology Section -->
              <div class="health-tab-section" data-tab="pathology" style="display: ${activeTab === 'pathology' ? 'block' : 'none'};">
                <table style="width: 100%; border-collapse: collapse; font-family: var(--font-mono); font-size: 10px;">
                  <thead>
                    <tr style="background: var(--surface-panel-elevated); color: var(--color-health); text-align: left;">
                      <th style="padding: 4px;">Biomarker Reading</th>
                      <th style="padding: 4px;">Value</th>
                      <th style="padding: 4px;">Reference</th>
                      <th style="padding: 4px;">Status</th>
                    </tr>
                  </thead>
                  <tbody>
                    ${(c.pathology || [
                      { biomarker: 'Aquifer Dissolved Oxygen', value: '8.4 mg/L', ref: '6.5 - 9.0', status: 'Optimal' },
                      { biomarker: 'Salinity (TDS)', value: '110 ppm', ref: '< 250 ppm', status: 'Normal' },
                      { biomarker: 'Pharyngeal Resonance Index', value: '0.94 σ', ref: '0.80 - 1.00', status: 'Normal' }
                    ]).map(p => `
                      <tr style="border-bottom: 1px solid var(--border-subtle);">
                        <td style="padding: 4px; color: #fff;">${p.biomarker}</td>
                        <td style="padding: 4px; color: var(--accent-cyan); font-weight: bold;">${p.value}</td>
                        <td style="padding: 4px; color: var(--text-muted);">${p.ref}</td>
                        <td style="padding: 4px;"><span class="badge" style="background: rgba(20, 184, 166, 0.2); color: var(--color-health); border: 1px solid var(--color-health); font-size: 8px; padding: 1px 4px; border-radius: 2px;">${p.status}</span></td>
                      </tr>
                    `).join('')}
                  </tbody>
                </table>
              </div>

              <!-- 2. Imaging Section -->
              <div class="health-tab-section" data-tab="imaging" style="display: ${activeTab === 'imaging' ? 'block' : 'none'};">
                <div style="background: #000; border-radius: var(--radius-xs); border: 1px solid var(--border-subtle); padding: 10px; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 6px;">
                  <span style="font-size: 28px;">🩻</span>
                  <span style="font-size: 11px; font-weight: bold; color: var(--color-health);">${c.imaging?.modality || 'Spectral Acoustic Tomography (.d10)'}</span>
                  <span style="font-family: var(--font-mono); font-size: 9px; color: var(--text-muted);">${c.imaging?.resolution || '512 x 512 x 64'} • Slice ${c.imaging?.slice || 32}/64</span>
                  <input type="range" min="1" max="64" value="${c.imaging?.slice || 32}" style="width: 80%; accent-color: var(--color-health); cursor: pointer;" />
                </div>
              </div>

              <!-- 3. Anatomy Section -->
              <div class="health-tab-section" data-tab="anatomy" style="display: ${activeTab === 'anatomy' ? 'block' : 'none'};">
                <div style="background: radial-gradient(circle at center, #134e4a 0%, #042f2e 100%); border-radius: var(--radius-xs); border: 1px solid var(--color-health); padding: 12px; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 6px;">
                  <span style="font-size: 28px;">🗣️</span>
                  <strong style="color: #fff; font-size: 11px;">10D Vocal Tract & Physiological Resonators</strong>
                  <span style="font-family: var(--font-mono); font-size: 9px; color: var(--color-health);">${c.anatomy?.meshId || 'mesh:vocal_tract_generic'}</span>
                  <span style="font-size: 10px; color: var(--text-secondary); text-align: center;">Aligned with Q42 phonological articulators and D10 kinematic motors.</span>
                </div>
              </div>
            </div>
          </div>
        `;

      case 'media':
        return `
          <div style="flex: 1; background: radial-gradient(circle at center, #1e293b 0%, #090d16 100%); border-radius: var(--radius-xs); border: 1px solid var(--border-subtle); display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 8px; padding: 10px;">
            <span style="font-size: 32px;">🎨</span>
            <strong style="color: #fff; font-size: 12px;">Visual Grapheme & Media Asset</strong>
            <span style="font-family: var(--font-mono); font-size: 10px; color: var(--color-media);">asset://visual/grapheme_q42.png</span>
            <button style="margin-top: 4px; background: var(--surface-panel-elevated); border: 1px solid var(--border-medium); color: var(--text-primary); padding: 3px 8px; border-radius: var(--radius-xs); font-size: 10px; cursor: pointer;">Generate Variant ✨</button>
          </div>
        `;

      case '3d':
        return `
          <div style="flex: 1; background: radial-gradient(circle at center, #1e1b4b 0%, #07090e 100%); border-radius: var(--radius-xs); display: flex; flex-direction: column; padding: 10px; gap: 8px; overflow: hidden;">
            <div style="display: flex; justify-content: space-between; align-items: center; border-bottom: 1px solid rgba(255,255,255,0.06); padding-bottom: 4px;">
              <span style="font-family: var(--font-mono); font-size: 11px; color: var(--color-3d); font-weight: 700;">🧊 Asset: ${c.meshAsset || 'vocal_tract_model_01.10d'}</span>
              <span class="honesty-badge honesty-live" style="font-size: 9px;">CRC: Valid · 10D Asset</span>
            </div>

            <!-- Interactive 3D Wireframe / Kinematic Mesh Viewport -->
            <div style="flex: 1; min-height: 120px; background: rgba(0,0,0,0.4); border-radius: 4px; border: 1px solid var(--border-subtle); display: flex; align-items: center; justify-content: center; position: relative;">
              <svg width="200" height="110" viewBox="0 0 200 110" style="animation: spin3D 14s linear infinite;">
                <polygon points="100,10 165,45 165,85 100,102 35,85 35,45" fill="none" stroke="var(--color-3d)" stroke-width="2" />
                <line x1="100" y1="10" x2="100" y2="102" stroke="var(--color-3d)" stroke-width="1.5" stroke-dasharray="3 3" />
                <line x1="165" y1="45" x2="35" y2="85" stroke="var(--accent-cyan)" stroke-width="1" />
                <line x1="35" y1="45" x2="165" y2="85" stroke="var(--accent-cyan)" stroke-width="1" />
                <circle cx="100" cy="55" r="5" fill="var(--accent-rose)" />
              </svg>
              <div style="position: absolute; top: 6px; left: 8px; font-family: var(--font-mono); font-size: 9px; color: var(--text-muted);">
                Sections: 6 (Mesh, 10D Tensor, Anim, Events, BVH, DID)
              </div>
              <div style="position: absolute; bottom: 6px; right: 8px; font-family: var(--font-mono); font-size: 9px; color: var(--accent-cyan);">
                WGPU Native Surface
              </div>
            </div>

            <!-- Kinematics Motor Articulation & Event Monitor -->
            <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 6px; font-size: 10px; font-family: var(--font-mono);">
              <div style="display: flex; align-items: center; gap: 4px; background: var(--surface-panel); padding: 3px 6px; border-radius: 3px;">
                <span>Jaw Angle:</span>
                <span style="color: var(--accent-amber);">14.2°</span>
              </div>
              <div style="display: flex; align-items: center; gap: 4px; background: var(--surface-panel); padding: 3px 6px; border-radius: 3px;">
                <span>Tongue Tensor:</span>
                <span style="color: var(--accent-emerald);">+2.4mm</span>
              </div>
              <div style="display: flex; align-items: center; gap: 4px; background: var(--surface-panel); padding: 3px 6px; border-radius: 3px;">
                <span>Velum Gate:</span>
                <span style="color: var(--accent-cyan);">Closed</span>
              </div>
              <div style="display: flex; align-items: center; gap: 4px; background: var(--surface-panel); padding: 3px 6px; border-radius: 3px;">
                <span>Resonance:</span>
                <span style="color: var(--accent-violet);">221.5 Hz</span>
              </div>
            </div>

            <!-- Timeline Animation & Reactive Events Hook -->
            <div style="display: flex; justify-content: space-between; align-items: center; font-size: 9px; font-family: var(--font-mono); background: rgba(6, 182, 212, 0.08); border: 1px solid rgba(6, 182, 212, 0.2); padding: 3px 6px; border-radius: 3px;">
              <span style="color: var(--accent-cyan);">⏱️ Anim: 60fps (t = 1.42s / 10s)</span>
              <span style="color: var(--accent-emerald);">⚡ on_closure -> pulse::emit</span>
            </div>
          </div>
          <style>
            @keyframes spin3D {
              0% { transform: rotateY(0deg) rotateX(10deg); }
              100% { transform: rotateY(360deg) rotateX(10deg); }
            }
          </style>
        `;

      case 'webview':
        return `
          <div class="webview-container">
            <div class="webview-toolbar">
              <button class="webview-nav-btn" title="Back">◀</button>
              <button class="webview-nav-btn" title="Forward">▶</button>
              <button class="webview-nav-btn btn-webview-reload" title="Reload">🔄</button>
              <input type="text" class="webview-address-bar" value="${c.url || 'https://w3c.org/tr/rdf12-concepts'}" />
              <button class="btn btn-sm btn-webview-go" style="background: var(--color-webview); color: #fff; border: none; font-size: 10px; padding: 2px 6px; border-radius: 2px; cursor: pointer;">Go</button>
              <button class="btn btn-sm btn-webview-clip" style="background: var(--surface-panel); border: 1px solid var(--accent-cyan); color: var(--accent-cyan); font-size: 10px; padding: 2px 6px; border-radius: 2px; cursor: pointer;" title="Clip Web Page Triples into Canvas Graph">📑 Clip RDF</button>
            </div>

            <div class="webview-frame-area">
              <div style="background: #1e293b; color: #f8fafc; padding: 12px; flex: 1; overflow-y: auto; font-family: sans-serif; font-size: 11px; line-height: 1.5;">
                <h3 style="color: #38bdf8; margin-bottom: 6px;">${c.titleSnippet || 'Sandboxed Knowledge Source'}</h3>
                <p>This sandboxed page is a placeholder knowledge source. Clip RDF from it into the canvas graph without leaving the fiduciary web partition.</p>
                <p style="margin-top: 6px; color: #94a3b8; font-size: 10px;">Security Sandbox: Fiduciary Web Partition Active • Origin: <code>${c.url || 'https://w3c.org'}</code></p>
              </div>
            </div>
          </div>
        `;

      case 'manifold':
      case 'subcanvas':
        return `
          <div style="flex: 1; background: radial-gradient(circle at center, #111827 0%, #030712 100%); border-radius: var(--radius-xs); border: 1px solid var(--accent-cyan); display: flex; flex-direction: column; padding: 10px; gap: 8px; overflow: hidden; position: relative;">
            
            <!-- 1. Ontological Metadata Header -->
            <div style="display: flex; justify-content: space-between; align-items: flex-start; border-bottom: 1px solid rgba(255,255,255,0.06); padding-bottom: 6px;">
              <div style="display: flex; flex-direction: column; gap: 2px;">
                <div style="display: flex; align-items: center; gap: 6px;">
                  <span style="font-size: 14px;">🌌</span>
                  <strong style="color: #fff; font-size: 12px;">${c.targetManifoldTitle || c.title}</strong>
                </div>
                <span style="font-family: var(--font-mono); font-size: 9px; color: var(--accent-cyan);">${c.targetManifoldUuid || 'urn:uuid:8a2b3c4d-5e6f-7a8b-9c0d-1e2f3a4b5c6d'}</span>
              </div>
              <span class="honesty-badge honesty-live" style="font-size: 9px;">L3 Fiduciary · Verified</span>
            </div>

            <!-- 2. Interactive Mini-Graph Visualizer (Nested Manifold Occupants) -->
            <div style="flex: 1; min-height: 100px; background: rgba(0,0,0,0.5); border-radius: var(--radius-xs); border: 1px solid var(--border-subtle); display: flex; align-items: center; justify-content: center; position: relative; overflow: hidden;">
              <svg width="100%" height="100%" viewBox="0 0 360 100" style="position: absolute; inset: 0;">
                <!-- Nested Wires -->
                <path d="M 60 50 Q 140 20, 220 40" fill="none" stroke="var(--accent-cyan)" stroke-width="1.5" stroke-dasharray="3 3" />
                <path d="M 220 40 Q 280 60, 310 50" fill="none" stroke="var(--accent-emerald)" stroke-width="1.5" />
                
                <!-- Nested Node Icons -->
                <rect x="30" y="32" width="60" height="36" rx="4" fill="var(--surface-panel)" stroke="var(--color-doc)" stroke-width="1" />
                <text x="60" y="54" fill="#fff" font-size="10" font-family="sans-serif" text-anchor="middle">📝 Doc</text>

                <rect x="190" y="22" width="60" height="36" rx="4" fill="var(--surface-panel)" stroke="var(--color-3d)" stroke-width="1" />
                <text x="220" y="44" fill="#fff" font-size="10" font-family="sans-serif" text-anchor="middle">🧊 10D</text>

                <rect x="280" y="32" width="60" height="36" rx="4" fill="var(--surface-panel)" stroke="var(--color-sheet)" stroke-width="1" />
                <text x="310" y="54" fill="#fff" font-size="10" font-family="sans-serif" text-anchor="middle">📊 Sheet</text>
              </svg>
              <div style="position: absolute; bottom: 4px; left: 8px; font-family: var(--font-mono); font-size: 8px; color: var(--text-muted);">
                Graph IRI: ${c.targetManifoldId || 'graph:manifold:media_01'}
              </div>
            </div>

            <!-- 3. Ontological Taxonomy & Rights Badges -->
            <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 4px; font-size: 9px; font-family: var(--font-mono);">
              <div style="background: var(--surface-panel); padding: 3px 6px; border-radius: 3px; border: 1px solid var(--border-subtle);">
                <span style="color: var(--text-muted);">Category:</span> <span style="color: var(--color-3d);">${c.targetCategory || 'qualia:CreativeMedia'}</span>
              </div>
              <div style="background: var(--surface-panel); padding: 3px 6px; border-radius: 3px; border: 1px solid var(--border-subtle);">
                <span style="color: var(--text-muted);">Topic:</span> <span style="color: var(--accent-amber);">${c.targetTopic?.split(':')[1] || '10DKinematics'}</span>
              </div>
              <div style="background: var(--surface-panel); padding: 3px 6px; border-radius: 3px; border: 1px solid var(--border-subtle);">
                <span style="color: var(--text-muted);">Occupants:</span> <span style="color: var(--text-primary);">${c.nestedOccupantsCount || 3} Nodes (${c.nestedWiresCount || 2} Wires)</span>
              </div>
              <div style="background: var(--surface-panel); padding: 3px 6px; border-radius: 3px; border: 1px solid var(--border-subtle);">
                <span style="color: var(--text-muted);">Merkle DAG:</span> <span style="color: var(--accent-emerald);">${c.merkleHash || '0x9B2C4F18'}</span>
              </div>
            </div>

            <!-- 4. Portal Traversal Action Button -->
            <div style="display: flex; justify-content: space-between; align-items: center; margin-top: 2px;">
              <span style="font-size: 9px; color: var(--text-muted); font-family: var(--font-mono);">Rights: did:qualia:timothy_charles_holborn</span>
              <button class="btn btn-sm btn-traverse-manifold" data-target="${c.targetManifoldId || 'manifold-media-01'}" style="background: linear-gradient(135deg, var(--accent-cyan) 0%, #3b82f6 100%); color: #000; font-weight: 700; font-size: 10px; border: none; border-radius: var(--radius-xs); padding: 4px 10px; cursor: pointer; display: flex; align-items: center; gap: 4px;">
                <span>Traverse into Manifold</span> <span>➔</span>
              </button>
            </div>
          </div>
        `;

      case 'ledger':
        return `
          <div style="flex: 1; display: flex; flex-direction: column; background: var(--surface-base); border-radius: var(--radius-xs); border: 1px solid var(--border-subtle); overflow: hidden; padding: 10px; gap: 8px;">
            <!-- Ledger Header -->
            <div style="display: flex; justify-content: space-between; align-items: center; border-bottom: 1px solid var(--border-subtle); padding-bottom: 6px;">
              <div style="display: flex; align-items: center; gap: 6px;">
                <span style="font-size: 14px;">💰</span>
                <strong style="color: #fff; font-size: 12px;">Total Reconciled Balance:</strong>
                <span style="font-family: var(--font-mono); font-size: 12px; color: var(--accent-emerald); font-weight: 700;">${c.totalBalance || '$142,500.00'}</span>
              </div>
              <span class="honesty-badge honesty-live" style="font-size: 9px;">Audit: ${c.auditHash || '0x4F8A9C'} · SHACL OK</span>
            </div>

            <!-- Double-Entry Transactions Table -->
            <div style="flex: 1; overflow-y: auto; max-height: 180px;">
              <table style="width: 100%; border-collapse: collapse; font-family: var(--font-mono); font-size: 10px;">
                <thead>
                  <tr style="background: var(--surface-panel-elevated); color: var(--text-secondary); text-align: left;">
                    <th style="padding: 4px 6px;">Tx ID</th>
                    <th style="padding: 4px 6px;">Debit (Dr)</th>
                    <th style="padding: 4px 6px;">Credit (Cr)</th>
                    <th style="padding: 4px 6px; text-align: right;">Amount</th>
                  </tr>
                </thead>
                <tbody>
                  ${(c.entries || []).map(tx => `
                    <tr style="border-bottom: 1px solid rgba(255,255,255,0.04);" title="${tx.memo || ''}">
                      <td style="padding: 4px 6px; color: var(--accent-cyan);">${tx.id}</td>
                      <td style="padding: 4px 6px; color: #fff;">${tx.accountDr}</td>
                      <td style="padding: 4px 6px; color: var(--text-muted);">${tx.accountCr}</td>
                      <td style="padding: 4px 6px; text-align: right; color: var(--accent-emerald); font-weight: 600;">${tx.amount}</td>
                    </tr>
                  `).join('')}
                </tbody>
              </table>
            </div>

            <!-- Ledger Action Bar -->
            <div style="display: flex; justify-content: space-between; align-items: center; border-top: 1px solid var(--border-subtle); padding-top: 6px;">
              <span style="font-family: var(--font-mono); font-size: 9px; color: var(--text-muted);">${c.currency || 'AUD / USDC / QUALIA-GAS'}</span>
              <button class="btn btn-sm btn-post-quad-tx" style="background: var(--strata-fin); color: #000; font-weight: 700; font-size: 10px; border: none; padding: 3px 8px; border-radius: var(--radius-xs); cursor: pointer;">
                ➕ Post Quad Entry
              </button>
            </div>
          </div>
        `;

      case 'project_dag':
        return `
          <div style="flex: 1; display: flex; flex-direction: column; background: var(--surface-base); border-radius: var(--radius-xs); border: 1px solid var(--border-subtle); overflow: hidden; padding: 10px; gap: 8px;">
            <div style="display: flex; justify-content: space-between; align-items: center; border-bottom: 1px solid var(--border-subtle); padding-bottom: 6px;">
              <div style="display: flex; align-items: center; gap: 6px;">
                <span style="font-size: 14px;">🚀</span>
                <strong style="color: #fff; font-size: 12px;">Inter-Dependent Project Streams</strong>
              </div>
              <span class="honesty-badge honesty-live" style="font-size: 9px;">3 Streams · Live DAG</span>
            </div>

            <div style="flex: 1; display: flex; flex-direction: column; gap: 8px; overflow-y: auto;">
              ${(c.projects || []).map(p => `
                <div style="background: var(--surface-panel); border: 1px solid var(--border-medium); border-radius: var(--radius-xs); padding: 8px; display: flex; flex-direction: column; gap: 4px;">
                  <div style="display: flex; justify-content: space-between; align-items: center;">
                    <strong style="color: #fff; font-size: 11px;">${p.name}</strong>
                    <span style="font-family: var(--font-mono); font-size: 9px; color: var(--accent-emerald); background: rgba(16,185,129,0.12); padding: 1px 4px; border-radius: 2px;">${p.status}</span>
                  </div>
                  <div style="display: flex; justify-content: space-between; font-size: 10px; color: var(--text-secondary);">
                    <span>${p.milestone}</span>
                    <span style="color: var(--accent-cyan); font-family: var(--font-mono);">${p.progress}%</span>
                  </div>
                  <!-- Progress Bar -->
                  <div style="height: 4px; background: rgba(255,255,255,0.06); border-radius: 2px; overflow: hidden;">
                    <div style="width: ${p.progress}%; height: 100%; background: linear-gradient(90deg, var(--accent-cyan), var(--accent-emerald));"></div>
                  </div>
                  <div style="display: flex; justify-content: space-between; font-size: 9px; font-family: var(--font-mono); color: var(--text-muted); margin-top: 2px;">
                    <span>Lead: ${p.lead}</span>
                    <span style="color: var(--strata-fin);">${p.budgetLinked}</span>
                  </div>
                </div>
              `).join('')}
            </div>

            <!-- Inter-Manifold Trigger Action -->
            <div style="display: flex; justify-content: space-between; align-items: center; border-top: 1px solid var(--border-subtle); padding-top: 6px;">
              <span style="font-size: 9px; color: var(--text-muted); font-family: var(--font-mono);">Causal Wire: M3 ➔ Finance Escrow</span>
              <button class="btn btn-sm btn-trigger-escrow" style="background: linear-gradient(135deg, var(--accent-emerald) 0%, #059669 100%); color: #000; font-weight: 700; font-size: 10px; border: none; padding: 4px 10px; border-radius: var(--radius-xs); cursor: pointer;">
                ⚡ Trigger Inter-Manifold Escrow Release ($15,000)
              </button>
            </div>
          </div>
        `;

      case 'portal':
        return `
          <div class="portal-viewport">
            <span style="font-size: 24px;">🌀</span>
            <strong style="color: #fff; font-size: 13px;">Wormhole to Sub-Manifold</strong>
            <span style="font-family: var(--font-mono); font-size: 10px; color: var(--color-portal);">${c.targetCanvas || 'manifold-media-01'}</span>
            <button class="btn-traverse-manifold" data-target="${c.targetCanvas || 'manifold-media-01'}" style="margin-top: 6px; background: var(--color-portal); color: #fff; border: none; padding: 4px 10px; border-radius: var(--radius-full); font-size: 10px; font-weight: 700; cursor: pointer;">Traverse Portal ➔</button>
          </div>
        `;

      default:
        return `<div style="color: var(--text-muted); padding: 10px; text-align: center;">Empty Container Node (${c.type})</div>`;
    }
  }

  toggleContainerExpansion(node, c, header) {
    const isExpanded = node.classList.toggle('expanded-fullscreen');
    const btnExpand = header?.querySelector('.btn-expand');
    if (btnExpand) {
      btnExpand.innerHTML = isExpanded ? '🔙 Back to Manifold' : '⛶';
      btnExpand.className = isExpanded ? 'btn-back-to-manifold' : 'container-action-btn btn-expand';
      btnExpand.title = isExpanded ? 'Return to Manifold Desk Physics' : 'Expand to Full Viewport';
    }
    this.wireSystem.render();
  }

  attachContainerEvents(node, c, header, resizer, resizerLeft, resizerRight, resizerBottom) {
    // Bring container to front on mousedown
    node.addEventListener('mousedown', () => {
      this.highestZIndex = Math.max(this.highestZIndex, 25) + 1;
      node.style.zIndex = this.highestZIndex;
      canvasState.selectContainer(c.id);
      document.querySelectorAll('.canvas-container-node').forEach(el => el.classList.remove('selected'));
      node.classList.add('selected');
    });

    // Double-click header to toggle fullscreen expansion
    header.addEventListener('dblclick', (e) => {
      e.stopPropagation();
      this.toggleContainerExpansion(node, c, header);
    });

    // Expand button click
    const btnExpand = header.querySelector('.btn-expand');
    if (btnExpand) {
      btnExpand.addEventListener('click', (e) => {
        e.stopPropagation();
        this.toggleContainerExpansion(node, c, header);
      });
    }

    // Close button
    const btnClose = header.querySelector('.btn-close');
    if (btnClose) {
      btnClose.addEventListener('click', (e) => {
        e.stopPropagation();
        canvasState.removeContainer(c.id);
        node.remove();
        this.wireSystem.render();
      });
    }

    // Focus in technical sidebar button
    const btnFocus = header.querySelector('.btn-focus');
    if (btnFocus) {
      btnFocus.addEventListener('click', (e) => {
        e.stopPropagation();
        canvasState.toggleTechSidebar('telemetry');
      });
    }

    // Dragging physics with 8px snapping
    header.addEventListener('mousedown', (e) => {
      if (node.classList.contains('expanded-fullscreen')) return;
      e.stopPropagation();
      this.draggingContainer = { node, data: c };
      const rect = node.getBoundingClientRect();
      this.dragOffset = {
        x: (e.clientX - rect.left) / canvasState.state.zoom,
        y: (e.clientY - rect.top) / canvasState.state.zoom
      };

      const onMouseMove = (moveEvent) => {
        if (!this.draggingContainer || node.classList.contains('expanded-fullscreen')) return;
        const stageRect = this.stage.getBoundingClientRect();
        let rawX = (moveEvent.clientX - stageRect.left) / canvasState.state.zoom - this.dragOffset.x;
        let rawY = (moveEvent.clientY - stageRect.top) / canvasState.state.zoom - this.dragOffset.y;

        // 8px Snap
        const snapX = Math.round(rawX / 8) * 8;
        const snapY = Math.round(rawY / 8) * 8;

        node.style.left = `${snapX}px`;
        node.style.top = `${snapY}px`;
        c.x = snapX;
        c.y = snapY;

        this.wireSystem.render();
      };

      const onMouseUp = () => {
        this.draggingContainer = null;
        window.removeEventListener('mousemove', onMouseMove);
        window.removeEventListener('mouseup', onMouseUp);
        canvasState.updateContainerPos(c.id, c.x, c.y);
      };

      window.addEventListener('mousemove', onMouseMove);
      window.addEventListener('mouseup', onMouseUp);
    });

    // 1. Corner Resizer (Bottom-Right)
    if (resizer) {
      resizer.addEventListener('mousedown', (e) => {
        if (node.classList.contains('expanded-fullscreen')) return;
        e.stopPropagation();
        this.resizingContainer = { node, data: c };
        const startX = e.clientX;
        const startY = e.clientY;
        const startW = c.width;
        const startH = c.height;

        const onResizeMove = (moveEvent) => {
          if (!this.resizingContainer) return;
          const deltaX = (moveEvent.clientX - startX) / canvasState.state.zoom;
          const deltaY = (moveEvent.clientY - startY) / canvasState.state.zoom;

          const newW = Math.max(280, Math.round((startW + deltaX) / 8) * 8);
          const newH = Math.max(180, Math.round((startH + deltaY) / 8) * 8);

          node.style.width = `${newW}px`;
          node.style.height = `${newH}px`;
          c.width = newW;
          c.height = newH;

          this.wireSystem.render();
        };

        const onResizeUp = () => {
          this.resizingContainer = null;
          window.removeEventListener('mousemove', onResizeMove);
          window.removeEventListener('mouseup', onResizeUp);
          canvasState.emit('stateChanged', canvasState.state);
        };

        window.addEventListener('mousemove', onResizeMove);
        window.addEventListener('mouseup', onResizeUp);
      });
    }

    // 2. Right-Edge Resizer
    if (resizerRight) {
      resizerRight.addEventListener('mousedown', (e) => {
        if (node.classList.contains('expanded-fullscreen')) return;
        e.stopPropagation();
        const startX = e.clientX;
        const startW = c.width;

        const onResizeMove = (moveEvent) => {
          const deltaX = (moveEvent.clientX - startX) / canvasState.state.zoom;
          const newW = Math.max(280, Math.round((startW + deltaX) / 8) * 8);
          node.style.width = `${newW}px`;
          c.width = newW;
          this.wireSystem.render();
        };

        const onResizeUp = () => {
          window.removeEventListener('mousemove', onResizeMove);
          window.removeEventListener('mouseup', onResizeUp);
          canvasState.emit('stateChanged', canvasState.state);
        };

        window.addEventListener('mousemove', onResizeMove);
        window.addEventListener('mouseup', onResizeUp);
      });
    }

    // 3. Left-Edge Resizer
    if (resizerLeft) {
      resizerLeft.addEventListener('mousedown', (e) => {
        if (node.classList.contains('expanded-fullscreen')) return;
        e.stopPropagation();
        const startX = e.clientX;
        const startW = c.width;
        const startLeft = c.x;

        const onResizeMove = (moveEvent) => {
          const deltaX = (moveEvent.clientX - startX) / canvasState.state.zoom;
          const newW = Math.max(280, Math.round((startW - deltaX) / 8) * 8);
          const newX = startLeft + (startW - newW);
          node.style.width = `${newW}px`;
          node.style.left = `${newX}px`;
          c.width = newW;
          c.x = newX;
          this.wireSystem.render();
        };

        const onResizeUp = () => {
          window.removeEventListener('mousemove', onResizeMove);
          window.removeEventListener('mouseup', onResizeUp);
          canvasState.updateContainerPos(c.id, c.x, c.y);
          canvasState.emit('stateChanged', canvasState.state);
        };

        window.addEventListener('mousemove', onResizeMove);
        window.addEventListener('mouseup', onResizeUp);
      });
    }

    // 4. Bottom-Edge Resizer
    if (resizerBottom) {
      resizerBottom.addEventListener('mousedown', (e) => {
        if (node.classList.contains('expanded-fullscreen')) return;
        e.stopPropagation();
        const startY = e.clientY;
        const startH = c.height;

        const onResizeMove = (moveEvent) => {
          const deltaY = (moveEvent.clientY - startY) / canvasState.state.zoom;
          const newH = Math.max(180, Math.round((startH + deltaY) / 8) * 8);
          node.style.height = `${newH}px`;
          c.height = newH;
          this.wireSystem.render();
        };

        const onResizeUp = () => {
          window.removeEventListener('mousemove', onResizeMove);
          window.removeEventListener('mouseup', onResizeUp);
          canvasState.emit('stateChanged', canvasState.state);
        };

        window.addEventListener('mousemove', onResizeMove);
        window.addEventListener('mouseup', onResizeUp);
      });
    }
  }
}


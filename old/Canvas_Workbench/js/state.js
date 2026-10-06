/**
 * HyperCanvas: Reactive State Store & Container Registry
 * Supports Objective vs Subjective Epistemic Modalities, Social Chat Graphs,
 * WebRTC, Health, Webview, Rights, Multi-Strata, and Contextual RDF.
 */

class HyperCanvasState {
  constructor() {
    this.listeners = new Map();

    // Default Workbench State
    this.state = {
      canvasTitle: 'Catchment Knowledge 4D Mindware Canvas',
      graphIri: 'graph:canvas:catchment_knowledge_4d',
      dimensionMode: '2D', // '2D' | '3D' | '4D'
      
      // Multi-Selectable Strata Levels
      activeStrata: ['environmental', 'social', 'legal', 'financial', 'technical'],
      
      // Active Epistemic Modality Filter ('all' | 'objective' | 'subjective' | 'intersubjective' | 'normative')
      activeEpistemicModality: 'all',

      // Toolbox Docking Position ('left' | 'top' | 'right' | 'bottom')
      dockPosition: 'left',

      // Rights & Permissions Registry
      rightsRegistry: {
        currentIdentity: 'did:qualia:timothy_charles_holborn',
        activeRole: 'Principal, Inventor & Fiduciary Trustee',
        fiduciaryLevel: 'Level 3 (Full Cryptographic Sovereign Custody)',
        groups: [
          { id: 'group:authors', name: 'Project Authors Circle', members: 5, access: 'Full Sovereign' },
          { id: 'group:researchers', name: 'Ecological & Hydrology Team', members: 12, access: 'Sensor Data Read/Write' },
          { id: 'group:legal', name: 'Water Rights Trustees', members: 4, access: 'Rights Assertion & SHACL Validation' }
        ],
        tags: [
          '#catchment-compliance',
          '#restricted-health-telemetry',
          '#open-water-data',
          '#fiduciary-custody-l3',
          '#webrtc-encrypted-channel'
        ]
      },

      // Strata Dimension Definitions
      strataTaxonomy: [
        { id: 'environmental', label: 'Environmental', icon: '🌿', color: 'var(--strata-env)', elevationZ: 0 },
        { id: 'social', label: 'Social', icon: '👥', color: 'var(--strata-social)', elevationZ: 100 },
        { id: 'legal', label: 'Legal', icon: '⚖️', color: 'var(--strata-legal)', elevationZ: 200 },
        { id: 'financial', label: 'Financial', icon: '💰', color: 'var(--strata-fin)', elevationZ: 300 },
        { id: 'technical', label: 'Technical', icon: '⚙️', color: 'var(--strata-tech)', elevationZ: 400 }
      ],
      
      // 3D XYZD Spatial Coordinates & Manifold
      xyzd: {
        x: 0,
        y: 0,
        z: 0,
        d: 1.0
      },

      // 4D Spatiotemporal DateTime & DateTime Span
      time: {
        currentDateTime: new Date('2026-08-15T14:40:00'),
        spanStart: new Date('2026-08-15T00:00:00'),
        spanEnd: new Date('2026-08-15T23:59:59'),
        spanPreset: '24h',
        progress: 0.61,
        isPlaying: false,
        playbackSpeed: 1
      },

      zoom: 0.9,
      pan: { x: 70, y: 40 },
      activeToolbox: null,
      selectedContainerId: 'container-social-01',
      selectedConnectionId: null,
      techSidebarOpen: false,
      sidebarTab: 'telemetry',

      contextualSelection: null,

      // Canvas Containers Suite with Epistemic Modalities
      containers: [
        {
          id: 'container-social-01',
          type: 'social',
          strata: 'social',
          epistemic: 'intersubjective',
          title: 'Project Team & AI Sub-Agent Chat Graph',
          x: 100,
          y: 70,
          width: 420,
          height: 310,
          z: 100,
          d: 1.0,
          validFrom: '2026-08-15T00:00:00',
          validTo: '2026-08-16T00:00:00',
          subAgents: [
            { id: 'agent:SentinelPlanner', name: 'Sentinel Planner AI', role: 'Telemetry Coordinator', status: 'Online' },
            { id: 'agent:LegalCounsel', name: 'Fiduciary Rights AI', role: 'Water Rights Guardian', status: 'Online' },
            { id: 'agent:LinguistAI', name: 'Phonology Grounder', role: 'Q42 Grapheme Grounding', status: 'Active' }
          ],
          messages: [
            { id: 'm1', sender: 'Timothy Charles Holborn', isAi: false, time: '14:38', text: 'North Spring looks clearer after last night’s rain; the field notes should capture that.', epistemic: 'subjective', graphNode: 'node:chat_01_holborn' },
            { id: 'm2', sender: 'Sentinel Planner AI', isAi: true, time: '14:39', text: 'Telemetry sensor confirms flow at 142.5 L/m with 221.5 Hz acoustic resonance.', epistemic: 'objective', graphNode: 'node:chat_02_sentinel' },
            { id: 'm3', sender: 'Fiduciary Rights AI', isAi: true, time: '14:40', text: 'Asserted legal quad: <<[ site:NorthSpring hydro:status "Monitored" <graph:rights> ]>>.', epistemic: 'normative', graphNode: 'node:chat_03_rights' }
          ],
          graphTriples: [
            { s: 'chat:Thread01', p: 'rdf:type', o: 'qualia:CollaborativeChatGraph' },
            { s: 'chat:Thread01', p: 'qualia:epistemicStatus', o: 'qualia:IntersubjectiveConsensus' }
          ]
        },
        {
          id: 'container-health-01',
          type: 'health',
          strata: 'environmental',
          epistemic: 'objective',
          title: 'Bio-Acoustic & Health Telemetry',
          x: 550,
          y: 70,
          width: 420,
          height: 310,
          z: 0,
          d: 1.0,
          validFrom: '2026-08-15T00:00:00',
          validTo: '2026-08-16T00:00:00',
          activeTab: 'pathology',
          pathology: [
            { biomarker: 'Aquifer Dissolved Oxygen', value: '8.4 mg/L', ref: '6.5 - 9.0', status: 'Optimal' },
            { biomarker: 'Salinity (TDS)', value: '110 ppm', ref: '< 250 ppm', status: 'Normal' },
            { biomarker: 'Pharyngeal Resonance Index', value: '0.94 σ', ref: '0.80 - 1.00', status: 'Normal' }
          ],
          imaging: { modality: 'Spectral Acoustic Tomography (.d10)', resolution: '512 x 512 x 64', slice: 32 },
          anatomy: { system: '10D Vocal Tract & Physiological Resonators', meshId: 'mesh:vocal_tract_holborn' },
          graphTriples: [
            { s: 'health:CatchmentBio', p: 'rdf:type', o: 'qualia:HealthBioTelemetry' },
            { s: 'health:CatchmentBio', p: 'qualia:epistemicStatus', o: 'qualia:EmpiricalTelemetry' }
          ]
        },
        {
          id: 'container-doc-01',
          type: 'doc',
          strata: 'social',
          epistemic: 'subjective',
          title: 'Catchment Field Notes & Observation Record',
          x: 1000,
          y: 70,
          width: 380,
          height: 260,
          z: 100,
          d: 1.0,
          validFrom: '2026-08-15T00:00:00',
          validTo: '2026-08-16T00:00:00',
          content: `<h2>North Spring Catchment Field Notes</h2>
<p>Author: <strong>Timothy Charles Holborn</strong></p>
<p>Epistemic Status: <span class="modality-badge modality-subjective">🧠 Subjective Qualia</span></p>
<p>The primary monitoring spring (<q-entity data-id="concept:FreshwaterSpring" style="color: #00f2a9; border-bottom: 2px solid #00f2a9;">North Spring</q-entity>) is the project’s reference site for seasonal flow, water quality, and field observation.</p>`,
          graphTriples: [
            { s: 'doc:CatchmentNotes', p: 'qualia:author', o: 'entity:Timothy_Charles_Holborn' },
            { s: 'doc:CatchmentNotes', p: 'qualia:epistemicStatus', o: 'qualia:PhenomenologicalQualia' }
          ]
        },
        {
          id: 'container-webrtc-01',
          type: 'webrtc',
          strata: 'social',
          epistemic: 'intersubjective',
          title: 'Encrypted WebRTC Audio/Video Stream',
          x: 100,
          y: 410,
          width: 380,
          height: 280,
          z: 100,
          d: 1.1,
          validFrom: '2026-08-15T00:00:00',
          validTo: '2026-08-16T00:00:00',
          peerId: 'did:qualia:peer_catchment_gateway',
          bitrate: '2.4 Mbps (AV1 / Opus Encrypted)',
          graphTriples: [
            { s: 'webrtc:Stream42', p: 'qualia:epistemicStatus', o: 'qualia:IntersubjectiveStream' }
          ]
        },
        {
          id: 'container-map-01',
          type: 'map',
          strata: 'environmental',
          epistemic: 'objective',
          title: 'Geospatial & Spatiotemporal Case Map',
          x: 520,
          y: 410,
          width: 440,
          height: 290,
          z: 0,
          d: 1.1,
          validFrom: '2026-08-15T00:00:00',
          validTo: '2026-08-16T00:00:00',
          geoRegion: 'North Spring Catchment (34.0°S, 148.2°E)',
          agents: [
            {
              id: 'agent:Sentinel_UAV_01',
              name: 'Autonomous Sentinel UAV #01',
              color: '#00d2ff',
              path: [
                { t: 0.0, x: 70, y: 190, label: '00:00 Gauge Base' },
                { t: 0.3, x: 160, y: 140, label: '07:15 Inflow Survey' },
                { t: 0.61, x: 260, y: 110, label: '14:40 North Spring' },
                { t: 1.0, x: 380, y: 80, label: '23:59 Coastal Gauge' }
              ]
            },
            {
              id: 'agent:Field_Holborn_Trail',
              name: 'Timothy Holborn Survey Path',
              color: '#38bdf8',
              path: [
                { t: 0.0, x: 120, y: 220, label: '00:00 Base Camp' },
                { t: 0.5, x: 200, y: 170, label: '12:00 Midday Sample' },
                { t: 0.61, x: 250, y: 135, label: '14:40 Acoustic Sample' },
                { t: 1.0, x: 320, y: 160, label: '23:59 Evening Camp' }
              ]
            }
          ],
          incidents: [
            { t: 0.61, x: 260, y: 110, title: 'Alert: Low Flow Critical Threshold', strata: 'environmental' }
          ],
          graphTriples: [
            { s: 'map:NorthSpringCase01', p: 'geo:lat', o: '-34.05' },
            { s: 'map:NorthSpringCase01', p: 'qualia:epistemicStatus', o: 'qualia:EmpiricalCoordinates' }
          ]
        },
        {
          id: 'container-ontology-01',
          type: 'ontology',
          strata: 'legal',
          epistemic: 'normative',
          title: 'Hydrology ↔ Water-Rights Alignment',
          x: 1000,
          y: 360,
          width: 380,
          height: 310,
          z: 200,
          d: 1.2,
          validFrom: '2026-08-15T00:00:00',
          validTo: '2026-08-16T00:00:00',
          ontologyIri: 'https://qualia.io/ns/hydro-catchment#',
          prefix: 'hydro:',
          classes: ['MonitoringSite', 'FreshwaterResurgence', 'WaterRight'],
          mappings: [
            { source: 'concept:NorthSpring', relation: 'owl:equivalentClass', target: 'hydro:FreshwaterResurgence' },
            { source: 'qualia:author', relation: 'rdfs:subPropertyOf', target: 'fiduciary:dutyOfCare' },
            { source: 'hydro:SurveyTransect', relation: 'skos:closeMatch', target: 'geo:HydrologicalFlowLine' }
          ],
          graphTriples: [
            { s: 'ontology:HydroCatchment', p: 'qualia:epistemicStatus', o: 'qualia:NormativeStatute' }
          ]
        }
      ],

      // Clickable Connection Wires
      connections: [
        {
          id: 'wire-01',
          from: 'container-social-01',
          to: 'container-doc-01',
          type: 'subjective',
          label: 'qualia:groundsSubjectiveQualia',
          predicate: 'qualia:groundsSubjectiveQualia',
          epistemic: 'subjective',
          bridgeType: 'Qualia Grounding Link',
          reifier: '<< chat:Thread01 qualia:groundsSubjectiveQualia doc:CatchmentNotes ~ reifier:wire-01 >>',
          nquin: '<<[ container:social-01 qualia:groundsSubjectiveQualia container:doc-01 graph:catchment_knowledge_4d did:qualia:timothy_charles_holborn ]>>'
        },
        {
          id: 'wire-02',
          from: 'container-health-01',
          to: 'container-ontology-01',
          type: 'objective',
          label: 'health:groundsBioTelemetry',
          predicate: 'health:groundsBioTelemetry',
          epistemic: 'objective',
          bridgeType: 'Empirical Grounding Link',
          reifier: '<< health:CatchmentBio health:groundsBioTelemetry ontology:HydroCatchment ~ reifier:wire-02 >>',
          nquin: '<<[ container:health-01 health:groundsBioTelemetry container:ontology-01 graph:catchment_knowledge_4d did:qualia:timothy_charles_holborn ]>>'
        },
        {
          id: 'wire-03',
          from: 'container-map-01',
          to: 'container-ontology-01',
          type: 'objective',
          label: 'geo:triggersLegalRight',
          predicate: 'geo:triggersLegalRight',
          epistemic: 'objective',
          bridgeType: 'Empirical ➔ Normative Cascade',
          reifier: '<< map:NorthSpringCase01 geo:triggersLegalRight ontology:HydroCatchment ~ reifier:wire-03 >>',
          nquin: '<<[ container:map-01 geo:triggersLegalRight container:ontology-01 graph:catchment_knowledge_4d did:qualia:timothy_charles_holborn ]>>'
        }
      ],

      // Live Telemetry Pulses
      pulses: [
        { id: 'p1', time: '17:35:10', source: 'Epistemic_Harmonizer', msg: 'Harmonized 3 Objective telemetry feeds with 2 Subjective Qualia narratives.' },
        { id: 'p2', time: '17:35:25', source: 'Chat_Graph', msg: 'Asserted subjective statement quad: node:chat_01_holborn -> qualia:SubjectiveQualia' },
        { id: 'p3', time: '17:35:40', source: 'Health_Sentinel', msg: 'Empirical acoustic reading (221.5 Hz) validates field-sample pitch profile.' }
      ]
    };
  }

  subscribe(event, callback) {
    if (!this.listeners.has(event)) {
      this.listeners.set(event, new Set());
    }
    this.listeners.get(event).add(callback);
    return () => this.listeners.get(event).delete(callback);
  }

  emit(event, data) {
    if (this.listeners.has(event)) {
      for (const cb of this.listeners.get(event)) {
        cb(data, this.state);
      }
    }
  }

  setState(updates) {
    this.state = { ...this.state, ...updates };
    this.emit('stateChanged', this.state);
  }

  selectContainer(id) {
    this.setState({ selectedContainerId: id, selectedConnectionId: null, sidebarTab: 'telemetry' });
    this.emit('containerSelected', id);
  }

  selectConnection(wireId) {
    this.setState({
      selectedConnectionId: wireId,
      sidebarTab: 'connection-ontology',
      techSidebarOpen: true
    });
    this.emit('connectionSelected', wireId);
  }

  setEpistemicModality(modalityKey) {
    this.setState({ activeEpistemicModality: modalityKey });
    this.emit('epistemicModalityChanged', modalityKey);
  }

  toggleStrata(strataKey) {
    let current = [...this.state.activeStrata];
    if (current.includes(strataKey)) {
      current = current.filter(s => s !== strataKey);
      if (current.length === 0) current = ['all'];
    } else {
      current = current.filter(s => s !== 'all');
      current.push(strataKey);
    }
    this.setState({ activeStrata: current });
    this.emit('strataChanged', current);
  }

  selectAllStrata() {
    this.setState({ activeStrata: ['environmental', 'social', 'legal', 'financial', 'technical'] });
    this.emit('strataChanged', this.state.activeStrata);
  }

  setDockPosition(pos) {
    this.setState({ dockPosition: pos });
    this.emit('dockPositionChanged', pos);
  }

  setContextualSelection(sel) {
    this.setState({ contextualSelection: sel });
    this.emit('contextualSelectionChanged', sel);
  }

  addChatMessage(containerId, text, sender = 'User', epistemic = 'subjective') {
    const container = this.state.containers.find(c => c.id === containerId);
    if (container && container.messages) {
      const msgId = `m${Date.now()}`;
      const graphNode = `node:chat_${Date.now()}`;
      const now = new Date();
      const timeStr = `${String(now.getHours()).padStart(2,'0')}:${String(now.getMinutes()).padStart(2,'0')}`;
      
      container.messages.push({
        id: msgId,
        sender,
        isAi: sender.includes('AI') || sender.includes('Agent'),
        time: timeStr,
        text,
        epistemic,
        graphNode
      });

      container.graphTriples.push({
        s: `chat:${msgId}`,
        p: 'qualia:chatContent',
        o: `"${text}"`
      });

      this.emit('chatMessageAdded', { containerId, msgId });
      this.emit('stateChanged', this.state);
    }
  }

  updateConnectionOntology(wireId, updates) {
    const conn = this.state.connections.find(c => c.id === wireId);
    if (conn) {
      Object.assign(conn, updates);
      this.emit('connectionUpdated', conn);
      this.emit('stateChanged', this.state);
    }
  }

  setDimensionMode(mode) {
    this.setState({ dimensionMode: mode });
    this.emit('dimensionModeChanged', mode);
  }

  setXYZD(partialXYZD) {
    const updated = { ...this.state.xyzd, ...partialXYZD };
    this.setState({ xyzd: updated });
    this.emit('xyzdChanged', updated);
  }

  setTimeProgress(progressRatio) {
    const p = Math.max(0, Math.min(1, progressRatio));
    const startMs = this.state.time.spanStart.getTime();
    const endMs = this.state.time.spanEnd.getTime();
    const currentMs = startMs + p * (endMs - startMs);
    const newDateTime = new Date(currentMs);

    const updatedTime = {
      ...this.state.time,
      progress: p,
      currentDateTime: newDateTime
    };

    this.setState({ time: updatedTime });
    this.emit('timeChanged', updatedTime);
  }

  setTimeSpanPreset(presetKey) {
    const now = new Date('2026-08-15T14:40:00');
    let start = new Date(now);
    let end = new Date(now);

    switch (presetKey) {
      case '1h':
        start = new Date(now.getTime() - 30 * 60 * 1000);
        end = new Date(now.getTime() + 30 * 60 * 1000);
        break;
      case '24h':
        start = new Date('2026-08-15T00:00:00');
        end = new Date('2026-08-15T23:59:59');
        break;
      case '7d':
        start = new Date('2026-08-10T00:00:00');
        end = new Date('2026-08-17T23:59:59');
        break;
      case 'season':
        start = new Date('2026-06-01T00:00:00');
        end = new Date('2026-08-31T23:59:59');
        break;
      case 'epoch':
        start = new Date('2026-01-01T00:00:00');
        end = new Date('2026-12-31T23:59:59');
        break;
    }

    const updatedTime = {
      ...this.state.time,
      spanPreset: presetKey,
      spanStart: start,
      spanEnd: end
    };

    this.setState({ time: updatedTime });
    this.emit('timeChanged', updatedTime);
    this.setTimeProgress(this.state.time.progress);
  }

  togglePlayTime() {
    const isPlaying = !this.state.time.isPlaying;
    const updatedTime = { ...this.state.time, isPlaying };
    this.setState({ time: updatedTime });
    this.emit('playTimeToggled', isPlaying);
  }

  setActiveToolbox(toolboxId) {
    const newToolbox = this.state.activeToolbox === toolboxId ? null : toolboxId;
    this.setState({ activeToolbox: newToolbox });
    this.emit('toolboxChanged', newToolbox);
  }

  toggleTechSidebar(tab = 'telemetry') {
    const isOpen = !this.state.techSidebarOpen;
    this.setState({ techSidebarOpen: isOpen, sidebarTab: tab });
    this.emit('techSidebarToggled', isOpen);
  }

  addContainer(container) {
    const containers = [...this.state.containers, container];
    this.setState({ containers, selectedContainerId: container.id });
    this.emit('containerAdded', container);
  }

  removeContainer(id) {
    const containers = this.state.containers.filter(c => c.id !== id);
    const connections = this.state.connections.filter(conn => conn.from !== id && conn.to !== id);
    this.setState({ containers, connections, selectedContainerId: containers[0]?.id || null });
    this.emit('containerRemoved', id);
  }

  updateContainerPos(id, x, y, z, d) {
    const container = this.state.containers.find(c => c.id === id);
    if (container) {
      container.x = x;
      container.y = y;
      if (z !== undefined) container.z = z;
      if (d !== undefined) container.d = d;
      this.emit('containerMoved', { id, x, y, z, d });
    }
  }
}

export const canvasState = new HyperCanvasState();

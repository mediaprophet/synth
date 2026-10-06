/**
 * HyperCanvas: Research & Epistemology Canvas Definition
 * Optimized for cross-strata research, ontology alignment, GIS mapping, and clinical health telemetry.
 */

export const researchCanvas = {
  id: 'canvas-research-01',
  title: 'Research & Epistemological Mindware Canvas',
  type: 'research',
  icon: '🔬',
  graphIri: 'graph:canvas:research_epistemology_01',
  defaultDimension: '2D',
  activeStrata: ['environmental', 'social', 'legal', 'technical'],
  activeEpistemicModality: 'all',
  activeToolboxes: ['epistemic', 'spatial', 'health', 'rights', 'office'],
  
  containers: [
    {
      id: 'container-map-01',
      type: 'map',
      strata: 'environmental',
      epistemic: 'objective',
      title: 'Geospatial & Spatiotemporal Catchment Map',
      x: 80,
      y: 70,
      width: 440,
      height: 310,
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
      graphTriples: []
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
      graphTriples: []
    },
    {
      id: 'container-ontology-01',
      type: 'ontology',
      strata: 'legal',
      epistemic: 'normative',
      title: 'Hydrology ↔ Water-Rights Alignment',
      x: 1000,
      y: 70,
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
      graphTriples: []
    }
  ],

  connections: [
    {
      id: 'wire-r1',
      from: 'container-map-01',
      to: 'container-ontology-01',
      type: 'objective',
      label: 'geo:triggersLegalRight',
      predicate: 'geo:triggersLegalRight',
      bridgeType: 'Empirical ➔ Normative Cascade'
    },
    {
      id: 'wire-r2',
      from: 'container-health-01',
      to: 'container-ontology-01',
      type: 'objective',
      label: 'health:groundsBioTelemetry',
      predicate: 'health:groundsBioTelemetry',
      bridgeType: 'Empirical Grounding Link'
    }
  ]
};

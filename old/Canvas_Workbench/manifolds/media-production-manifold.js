/**
 * Media Production & Creative Design Manifold Definition
 * Optimized for 3D kinematics, generative visual media, speech synthesizer triads, and columnar tensor tables.
 */

export const mediaProductionManifold = {
  id: 'manifold-media-01',
  uuid: 'urn:uuid:8a2b3c4d-5e6f-7a8b-9c0d-1e2f3a4b5c6d',
  title: 'Media Production & Creative 3D Studio',
  type: 'media',
  icon: '🎨',
  graphIri: 'graph:manifold:media_production_01',
  category: 'qualia:CreativeMedia',
  topic: 'topic:PhonologicalGraphemesAnd10DKinematics',
  rights: {
    rightsHolder: 'did:qualia:timothy_charles_holborn',
    fiduciaryDutyLevel: 'fiduciary:GateLevel3',
    permissionCircle: 'circle:ProjectAuthors',
    merkleRootHash: '0x9B2C4F18A3D5E7B1',
    licence: 'https://qualia.io/licences/fiduciary-duty-of-care-v1'
  },
  created: '2026-08-15T00:00:00Z',
  modified: '2026-08-16T15:35:00Z',
  defaultDimension: '3D',
  activeStrata: ['social', 'technical'],
  activeEpistemicModality: 'all',
  activeToolboxes: ['image', 'spatial', 'sheet', 'code', 'ai'],

  containers: [
    {
      id: 'container-3d-01',
      type: '3d',
      strata: 'social',
      epistemic: 'objective',
      title: '3D Virtual Vocal Tract Kinematics (.d10)',
      x: 100,
      y: 80,
      width: 400,
      height: 290,
      z: 100,
      d: 1.3,
      validFrom: '2026-08-15T00:00:00',
      validTo: '2026-08-16T00:00:00',
      meshAsset: 'mesh:vocal_tract_holborn_d10',
      graphTriples: []
    },
    {
      id: 'container-media-01',
      type: 'media',
      strata: 'social',
      epistemic: 'subjective',
      title: 'Visual Grapheme & Artwork Layer',
      x: 540,
      y: 80,
      width: 380,
      height: 290,
      z: 100,
      validFrom: '2026-08-15T00:00:00',
      validTo: '2026-08-16T00:00:00',
      content: '🖼️ Vector Grapheme Artwork & Glyph Render',
      graphTriples: []
    },
    {
      id: 'container-sheet-01',
      type: 'sheet',
      strata: 'technical',
      epistemic: 'objective',
      title: 'EnCodec P64 Neural Acoustic Latents',
      x: 950,
      y: 80,
      width: 420,
      height: 280,
      z: 0,
      d: 1.0,
      validFrom: '2026-08-15T00:00:00',
      validTo: '2026-08-16T00:00:00',
      rows: [
        { site: 'North Spring Flow', flow: '142.5 L/m', resonance: '221.5 Hz', acousticLatent: '[0.42, 0.88, 1.24]' },
        { site: 'Upper Lagoon', flow: '98.2 L/m', resonance: '184.0 Hz', acousticLatent: '[0.35, 0.72, 1.10]' },
        { site: 'Coastal Gauge', flow: '165.0 L/m', resonance: '246.8 Hz', acousticLatent: '[0.58, 0.94, 1.32]' }
      ],
      graphTriples: []
    }
  ],

  connections: [
    {
      id: 'wire-m1',
      from: 'container-sheet-01',
      to: 'container-3d-01',
      type: 'tensor',
      label: 'p64:drivesKinematics',
      predicate: 'p64:drivesKinematics',
      bridgeType: 'Neural Latent Motor Drive'
    },
    {
      id: 'wire-m2',
      from: 'container-3d-01',
      to: 'container-media-01',
      type: 'subjective',
      label: 'qualia:articulatesVisualGrapheme',
      predicate: 'qualia:articulatesVisualGrapheme',
      bridgeType: 'Kinematic ➔ Aesthetic Mapping'
    }
  ]
};

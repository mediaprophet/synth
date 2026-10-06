/**
 * Sanctuary Manifold Definition
 * Human-Centric Sovereign Enclave for Inforg Identity, Infosphere Governance,
 * Sensory Pacing, Confidential Memory Vaults, and Fiduciary Trust Delegation.
 */

export const sanctuaryManifold = {
  id: 'manifold-sanctuary-01',
  uuid: 'urn:uuid:1a2b3c4d-5e6f-7a8b-9c0d-0e1f2a3b4c5d',
  title: 'Sanctuary: Inforg & Sovereign Infosphere Enclave',
  type: 'sanctuary',
  layout: 'sanctuary', // Specialized spatial topology (Concentric Enclave vs Freeform Workbench)
  icon: '🏛️',
  graphIri: 'graph:manifold:sanctuary_inforg_01',
  category: 'qualia:HumanCentricSanctuary',
  topic: 'topic:InforgIdentityAndInfosphereGovernance',
  rights: {
    rightsHolder: 'did:qualia:timothy_charles_holborn',
    fiduciaryDutyLevel: 'fiduciary:GateLevel3',
    permissionCircle: 'circle:SovereignPrincipalOnly',
    merkleRootHash: '0x1A2B3C4D5E6F7A8B',
    licence: 'https://qualia.io/licences/sovereign-confidential-sanctuary-v1'
  },
  created: '2026-08-15T00:00:00Z',
  modified: '2026-08-16T16:15:00Z',
  defaultDimension: '2D',
  activeStrata: ['social', 'legal', 'technical'],
  activeEpistemicModality: 'all',
  activeToolboxes: ['rights', 'communication', 'code', 'office', 'ai'],

  // Sovereign Inforg & Infosphere Profile
  inforgProfile: {
    did: 'did:qualia:timothy_charles_holborn',
    displayName: 'Timothy Charles Holborn',
    designation: 'Principal, Inventor & Sovereign Trustee',
    fiduciaryLevel: 'Level 3 Sovereign Gatekeeper',
    airgapShield: 'Active: 100% Local Encrypted BLAKE3 CRDT (Zero Cloud Exfiltration)',
    lastAttestation: '2026-08-16 16:10 UTC',
    merkleVaultHash: '0x9E8D7C6B5A4F3E2D1C0B',
    
    // 1. Sensory, Cognitive & Human-Centric Pacing
    sensoryPacing: {
      cognitivePacing: '45m Immersion / 10m Biological Rest',
      ambientResonance: '432 Hz Harmonic Soft Tone',
      visualSensoryMode: 'High-Legibility Cyber-Glassmorphism',
      focusShielding: 'Strict Fiduciary Gate (Silent Notifications)',
      ambientLightAdaptation: 'Circadian Sync (Warm Evening Palette)'
    },

    // 2. Concentric Infosphere Spheres of Autonomy
    infosphereRings: [
      {
        ringId: 'ring:core',
        name: 'Inner Core: Sovereign Self',
        badge: 'Private & Air-Gapped',
        color: 'var(--accent-cyan)',
        icon: '🛡️',
        description: 'Biometric signatures, confidential qualia diary, raw cryptographic private keyrings, and personal values.',
        nodesCount: 14,
        accessControl: 'Strictly did:qualia:timothy_charles_holborn (Biometric Key Required)'
      },
      {
        ringId: 'ring:fiduciary',
        name: 'Middle Sphere: Fiduciary Guardians & Trustees',
        badge: 'Statutory Duty-of-Care',
        color: 'var(--accent-emerald)',
        icon: '⚖️',
        description: 'Legal emergency directives, statutory power-of-attorney guardians, water rights trustees, and authorial integrity gates.',
        nodesCount: 6,
        accessControl: 'Multi-Sig Fiduciary Circle (3-of-5 M-of-N Threshold)'
      },
      {
        ringId: 'ring:ambient',
        name: 'Outer Sphere: Ambient Infosphere & AI Agents',
        badge: 'Sentinel Gated',
        color: 'var(--accent-purple)',
        icon: '🤖',
        description: 'AI sub-agents (Sentinel Planner, Linguist, Bio-Telemetry), external WebRTC peer gateways, and sensor telemetry feeds.',
        nodesCount: 9,
        accessControl: 'Capability-Gated via VibeScript Sentinel VM'
      }
    ],

    // 3. Confidential Onboarding & Sovereign Data Vaults
    confidentialVaults: [
      {
        id: 'vault:personal_values',
        title: 'Core Values, Principles & Human Boundaries',
        status: 'Encrypted & Sealed',
        quadsCount: 42,
        summary: 'Non-negotiable ethical directives, privacy boundaries, communication pacing, and authorial consent rules.'
      },
      {
        id: 'vault:fiduciary_delegations',
        title: 'Fiduciary Duty-of-Care & Legal Directives',
        status: 'SHACL Validated',
        quadsCount: 18,
        summary: 'Emergency proxy execution, statutory rights custody, and intellectual property guardianship per COPYRIGHT.md.'
      },
      {
        id: 'vault:biometric_baseline',
        title: 'Physiological & Acoustic Resonance Baselines',
        status: 'Air-Gapped Local',
        quadsCount: 29,
        summary: 'Vocal tract 10D articulatory model, pathology biomarker baseline bounds, and circadian rest curves.'
      }
    ]
  },

  // Fallback workbench containers if switched to freeform node mode
  containers: [
    {
      id: 'container-inforg-core',
      type: 'rights',
      strata: 'legal',
      epistemic: 'objective',
      title: 'Inforg Sovereign Identity & Fiduciary Gate',
      x: 100,
      y: 80,
      width: 440,
      height: 320,
      z: 200,
      d: 1.0,
      validFrom: '2026-08-15T00:00:00',
      validTo: '2026-08-16T00:00:00',
      graphTriples: []
    }
  ],
  connections: []
};

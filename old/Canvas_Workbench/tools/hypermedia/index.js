/**
 * tools/hypermedia/index.js — Hypermedia Library & Archive Manager Toolbox
 * Principle: Sovereign content-addressed asset indexing, HCF/HMC packaging, and multi-tenant archive management.
 */

export const hypermediaToolbox = {
  id: 'hypermedia',
  title: 'Hypermedia Library & Archive Manager',
  icon: '🗄️',
  domain: 'hypermedia',
  honesty: 'partial',
  sections: [
    {
      title: 'Asset Ingestion & Cataloging',
      tools: [
        { id: 'ingest_asset', label: '📥 Ingest Media Asset (BLAKE3 Hashed)', honesty: 'partial' },
        { id: 'clip_vault', label: '📑 Web RDF Clip Vault', honesty: 'present' },
        { id: 'tag_metadata', label: '🏷️ Tag Provenance Metadata', honesty: 'partial' }
      ]
    },
    {
      title: 'HCF / HMC Archive Packaging',
      tools: [
        { id: 'export_hcf', label: '📦 Export HyperCanvas File (.hcf)', honesty: 'partial' },
        { id: 'import_hmc', label: '📂 Open HyperMedia Container (.hmc)', honesty: 'partial' },
        { id: 'codec_telemetry', label: '📊 Codec Telemetry (AV1 / Opus / glTF)', honesty: 'present' }
      ]
    }
  ]
};

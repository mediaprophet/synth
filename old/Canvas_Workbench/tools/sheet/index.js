/**
 * tools/sheet/index.js — Spreadsheets & Columnar Tensors Toolbox
 * Principle: Continuous calculation merging classic spreadsheet math with EnCodec P64 neural latents.
 */

export const sheetToolbox = {
  id: 'sheet',
  title: 'Spreadsheets & Columnar Tensors',
  icon: '📊',
  domain: 'sheet',
  honesty: 'present',
  sections: [
    {
      title: 'Calculation & Formulas',
      tools: [
        { id: 'formula_bar', label: 'fx Formula Receptor', honesty: 'present' },
        { id: 'sum', label: 'SUM / Arithmetic Aggregate', honesty: 'present' },
        { id: 'resonance', label: 'RESONANCE(flow, freq)', honesty: 'present' },
        { id: 'purity', label: 'PURITY_INDEX(tds, do)', honesty: 'present' }
      ]
    },
    {
      title: 'Neural Latents & Import/Export',
      tools: [
        { id: 'p64_align', label: '📐 P64 Neural Latent Tensor Alignment', honesty: 'present' },
        { id: 'import_cbor', label: '📥 Ingest CSV / CBOR-LD', honesty: 'present' },
        { id: 'export_csvw', label: '📤 Export W3C CSVW Tabular Graph', honesty: 'present' }
      ]
    }
  ]
};

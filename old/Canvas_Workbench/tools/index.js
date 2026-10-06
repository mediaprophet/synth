/**
 * tools/index.js — Unified Modular Toolbox Registry
 * Exports all specialized domain toolboxes for the HyperCanvas Workbench.
 */

import { richTextToolbox } from './rich_text/index.js';
import { sheetToolbox } from './sheet/index.js';
import { rdfGraphToolbox } from './rdf_graph/index.js';
import { hypermediaToolbox } from './hypermedia/index.js';
import { mediaStudioToolbox } from './media_studio/index.js';
import { tenDAssetsToolbox } from './ten_d_assets/index.js';
import { audioSpeechToolbox } from './audio_speech/index.js';
import { communicationToolbox } from './communication/index.js';
import { spatialGisToolbox } from './spatial_gis/index.js';
import { vibeScriptCodeToolbox } from './vibescript_code/index.js';
import { rightsFiduciaryToolbox } from './rights_fiduciary/index.js';
import { healthClinicalToolbox } from './health_clinical/index.js';
import { financeLedgerToolbox } from './finance_ledger/index.js';

export const allToolboxes = [
  richTextToolbox,
  sheetToolbox,
  rdfGraphToolbox,
  hypermediaToolbox,
  mediaStudioToolbox,
  tenDAssetsToolbox,
  audioSpeechToolbox,
  communicationToolbox,
  spatialGisToolbox,
  vibeScriptCodeToolbox,
  rightsFiduciaryToolbox,
  healthClinicalToolbox,
  financeLedgerToolbox
];

export function getToolboxById(id) {
  return allToolboxes.find(t => t.id === id);
}

export function getToolboxByDomain(domain) {
  return allToolboxes.find(t => t.domain === domain);
}

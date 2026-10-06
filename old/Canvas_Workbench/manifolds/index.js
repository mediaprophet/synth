/**
 * Unified manifold registry — work surfaces, not HTML canvases.
 */

import { researchManifold } from './research-manifold.js';
import { mediaProductionManifold } from './media-production-manifold.js';
import { socialGovernanceManifold } from './social-governance-manifold.js';
import { sanctuaryManifold } from './sanctuary-manifold.js';
import { financialEconomicsManifold } from './financial-economics-manifold.js';
import { collaborativeProjectsManifold } from './collaborative-projects-manifold.js';

export const predefinedManifolds = [
  researchManifold,
  mediaProductionManifold,
  socialGovernanceManifold,
  financialEconomicsManifold,
  collaborativeProjectsManifold,
  sanctuaryManifold
];

const LEGACY_IDS = {
  'canvas-research-01': 'manifold-research-01',
  'canvas-media-01': 'manifold-media-01',
  'canvas-social-01': 'manifold-social-01',
  'canvas-settings-01': 'manifold-sanctuary-01',
  'manifold-settings-01': 'manifold-sanctuary-01'
};

export function getManifoldById(id) {
  const resolved = LEGACY_IDS[id] || id;
  return predefinedManifolds.find(m => m.id === resolved) || predefinedManifolds[0];
}

/** @deprecated use getManifoldById */
export const getCanvasById = getManifoldById;
/** @deprecated use predefinedManifolds */
export const predefinedCanvases = predefinedManifolds;

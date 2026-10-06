/**
 * tools/spatial_gis/index.js — Spatiotemporal GIS & Catchment Maps Toolbox
 * Principle: 4D geospatial intelligence tracking environmental phenomena and agent tracks over time.
 */

export const spatialGisToolbox = {
  id: 'spatial_gis',
  title: 'Spatiotemporal GIS & Catchment Maps',
  icon: '🗺️',
  domain: 'map',
  honesty: 'partial',
  sections: [
    {
      title: 'Topographic & GIS Layers',
      tools: [
        { id: 'sat_layer', label: '🛰️ Satellite Imagery Layer', honesty: 'partial' },
        { id: 'flow_vectors', label: '🌊 Hydrological Flow Vectors', honesty: 'partial' },
        { id: 'uav_track', label: '🚁 UAV Sentinel Trajectory', honesty: 'partial' },
        { id: 'survey_trail', label: '🚶 Timothy Holborn Survey Trail', honesty: 'partial' }
      ]
    },
    {
      title: 'Incident Pinning & Timeline',
      tools: [
        { id: 'pin_incident', label: '📍 Pin Case Incident Event', honesty: 'partial' },
        { id: 'time_scrub', label: '⏱️ Synchronize 4D Time Scrubber', honesty: 'live' }
      ]
    }
  ]
};

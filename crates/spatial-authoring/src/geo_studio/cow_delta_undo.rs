//! Copy-on-Write (`Arc<Vec<f32>>`) tile heightfield snapshot & delta undo/redo stack
//! with deterministic 64-byte `StrokeCommand` replay journal.
//!
//! Adapted from EarthCraft-main v0.2.0 under MIT License.
//! Copyright (c) EarthCraft contributors.
//! Licensed under the MIT License.

#![forbid(unsafe_code)]

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

/// Errors produced by GeoStudio terrain authoring operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GeoStudioError {
    InvalidParameter(&'static str),
    ByteBudgetExceeded { current_bytes: usize, budget_bytes: usize },
    TileNotFound(i32, i32),
    StrokeAlreadyActive,
    NoActiveStroke,
}

impl std::fmt::Display for GeoStudioError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidParameter(msg) => write!(f, "Invalid parameter: {msg}"),
            Self::ByteBudgetExceeded { current_bytes, budget_bytes } => {
                write!(f, "Byte budget exceeded: {current_bytes} > {budget_bytes}")
            }
            Self::TileNotFound(tx, ty) => write!(f, "Tile ({tx}, {ty}) not found"),
            Self::StrokeAlreadyActive => write!(f, "A stroke is already active"),
            Self::NoActiveStroke => write!(f, "No stroke is currently active"),
        }
    }
}

impl std::error::Error for GeoStudioError {}

/// Falloff curves for terrain brush strokes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrushFalloff {
    SmoothStep = 0,
    Gaussian = 1,
    Linear = 2,
    Plateau = 3,
}

impl BrushFalloff {
    pub fn from_code(code: u8) -> Self {
        match code {
            1 => Self::Gaussian,
            2 => Self::Linear,
            3 => Self::Plateau,
            _ => Self::SmoothStep,
        }
    }

    #[inline]
    pub fn compute_weight(self, dist_m: f32, radius_m: f32) -> f32 {
        if radius_m <= 0.0 || dist_m >= radius_m {
            return 0.0;
        }
        let t = (dist_m / radius_m).clamp(0.0, 1.0);
        match self {
            Self::Linear => 1.0 - t,
            Self::SmoothStep => {
                let u = 1.0 - t;
                u * u * (3.0 - 2.0 * u)
            }
            Self::Gaussian => (-3.0 * t * t).exp(),
            Self::Plateau => {
                if t < 0.5 { 1.0 } else {
                    let u = (1.0 - t) * 2.0;
                    u * u * (3.0 - 2.0 * u)
                }
            }
        }
    }
}

/// Core brush deformation operations.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BrushOperation {
    Raise,
    Lower,
    Flatten { target_height: f32 },
    Smooth,
    Terrace { step_m: f32 },
    HydraulicDrop,
}

impl BrushOperation {
    pub fn from_sample(sample: &StrokeSample) -> Self {
        match sample.op_code {
            1 => Self::Lower,
            2 => Self::Flatten { target_height: sample.target_height_m },
            3 => Self::Smooth,
            4 => Self::Terrace { step_m: sample.step_m },
            5 => Self::HydraulicDrop,
            _ => Self::Raise,
        }
    }
}

/// Exactly 64-byte deterministic replay journal sample.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StrokeSample {
    pub stroke_id: u64,
    pub timestamp_ms: u64,
    pub center_x_m: f32,
    pub center_y_m: f32,
    pub radius_m: f32,
    pub strength: f32,
    pub target_height_m: f32,
    pub step_m: f32,
    pub op_code: u8,
    pub falloff_code: u8,
    pub layer_index: u8,
    pub flags: u8,
    pub seed: u32,
    pub pad: [u64; 2],
}

/// Delta patch capturing before and after states for a single tile.
#[derive(Debug, Clone)]
pub struct TileDeltaPatch {
    pub tile_coord: (i32, i32),
    pub before_tile: Arc<Vec<f32>>,
    pub after_tile: Arc<Vec<f32>>,
}

/// Recorded transaction entry on the undo/redo stack.
#[derive(Debug, Clone)]
pub struct UndoCommitEntry {
    pub commit_id: u64,
    pub label: String,
    pub modified_tiles: Vec<TileDeltaPatch>,
    pub stroke_samples: Vec<StrokeSample>,
}

/// Copy-on-Write tile grid with delta undo/redo history.
#[derive(Debug)]
pub struct CowTerrainTileGrid {
    pub tile_size: usize,
    pub cell_size_m: f32,
    pub tiles: HashMap<(i32, i32), Arc<Vec<f32>>>,
    pub undo_stack: Vec<UndoCommitEntry>,
    pub redo_stack: Vec<UndoCommitEntry>,
    pub active_baseline: HashMap<(i32, i32), Arc<Vec<f32>>>,
    pub active_strokes: Vec<StrokeSample>,
    pub byte_budget: usize,
    next_commit_id: u64,
    is_stroke_active: bool,
}

impl CowTerrainTileGrid {
    pub fn new(tile_size: usize, cell_size_m: f32, byte_budget: usize) -> Result<Self, GeoStudioError> {
        if tile_size == 0 { return Err(GeoStudioError::InvalidParameter("tile_size must be positive")); }
        if cell_size_m <= 0.0 || !cell_size_m.is_finite() {
            return Err(GeoStudioError::InvalidParameter("cell_size_m must be finite and positive"));
        }
        if byte_budget == 0 { return Err(GeoStudioError::InvalidParameter("byte_budget must be positive")); }
        Ok(Self {
            tile_size, cell_size_m, tiles: HashMap::new(), undo_stack: Vec::new(), redo_stack: Vec::new(),
            active_baseline: HashMap::new(), active_strokes: Vec::new(), byte_budget, next_commit_id: 1, is_stroke_active: false,
        })
    }

    pub fn ensure_tile(&mut self, tx: i32, ty: i32, initial_height: f32) -> Result<(), GeoStudioError> {
        if self.tiles.contains_key(&(tx, ty)) { return Ok(()); }
        let data = vec![initial_height; self.tile_size * self.tile_size];
        self.tiles.insert((tx, ty), Arc::new(data));
        let mem = self.estimate_memory_bytes();
        if mem > self.byte_budget {
            self.tiles.remove(&(tx, ty));
            return Err(GeoStudioError::ByteBudgetExceeded { current_bytes: mem, budget_bytes: self.byte_budget });
        }
        Ok(())
    }

    pub fn begin_stroke(&mut self) -> Result<(), GeoStudioError> {
        if self.is_stroke_active { return Err(GeoStudioError::StrokeAlreadyActive); }
        self.is_stroke_active = true;
        self.active_baseline.clear();
        self.active_strokes.clear();
        Ok(())
    }

    pub fn apply_stroke_sample(&mut self, sample: StrokeSample) -> Result<usize, GeoStudioError> {
        if !self.is_stroke_active { return Err(GeoStudioError::NoActiveStroke); }
        if sample.radius_m <= 0.0 || !sample.radius_m.is_finite() { return Ok(0); }
        self.active_strokes.push(sample);
        let tile_span = self.tile_size as f32 * self.cell_size_m;
        let min_tx = ((sample.center_x_m - sample.radius_m) / tile_span).floor() as i32;
        let max_tx = ((sample.center_x_m + sample.radius_m) / tile_span).floor() as i32;
        let min_ty = ((sample.center_y_m - sample.radius_m) / tile_span).floor() as i32;
        let max_ty = ((sample.center_y_m + sample.radius_m) / tile_span).floor() as i32;
        let op = BrushOperation::from_sample(&sample);
        let falloff = BrushFalloff::from_code(sample.falloff_code);
        let mut modified_count = 0;

        for ty in min_ty..=max_ty {
            for tx in min_tx..=max_tx {
                if !self.tiles.contains_key(&(tx, ty)) { continue; }
                let (tile_min_x, tile_min_y) = (tx as f32 * tile_span, ty as f32 * tile_span);
                let (rmin_x, rmax_x) = (sample.center_x_m - sample.radius_m - tile_min_x, sample.center_x_m + sample.radius_m - tile_min_x);
                let (rmin_y, rmax_y) = (sample.center_y_m - sample.radius_m - tile_min_y, sample.center_y_m + sample.radius_m - tile_min_y);
                if rmax_x < 0.0 || rmin_x > tile_span || rmax_y < 0.0 || rmin_y > tile_span { continue; }

                let start_lx = (rmin_x / self.cell_size_m).floor().max(0.0) as usize;
                let end_lx = (rmax_x / self.cell_size_m).ceil().min((self.tile_size - 1) as f32) as usize;
                let start_ly = (rmin_y / self.cell_size_m).floor().max(0.0) as usize;
                let end_ly = (rmax_y / self.cell_size_m).ceil().min((self.tile_size - 1) as f32) as usize;
                if start_lx > end_lx || start_ly > end_ly { continue; }

                let rad_sq = sample.radius_m * sample.radius_m;
                let has_affected = (start_ly..=end_ly).any(|ly| {
                    (start_lx..=end_lx).any(|lx| {
                        let (vx, vy) = (tile_min_x + lx as f32 * self.cell_size_m, tile_min_y + ly as f32 * self.cell_size_m);
                        (vx - sample.center_x_m).powi(2) + (vy - sample.center_y_m).powi(2) <= rad_sq
                    })
                });
                if !has_affected { continue; }

                self.active_baseline.entry((tx, ty)).or_insert_with(|| Arc::clone(&self.tiles[&(tx, ty)]));
                let tile_vec = Arc::make_mut(self.tiles.get_mut(&(tx, ty)).unwrap());

                for ly in start_ly..=end_ly {
                    for lx in start_lx..=end_lx {
                        let (vx, vy) = (tile_min_x + lx as f32 * self.cell_size_m, tile_min_y + ly as f32 * self.cell_size_m);
                        let dist = ((vx - sample.center_x_m).powi(2) + (vy - sample.center_y_m).powi(2)).sqrt();
                        if dist <= sample.radius_m {
                            let weight = falloff.compute_weight(dist, sample.radius_m);
                            apply_op_to_vertex(op, sample.strength, weight, self.tile_size, lx, ly, tile_vec);
                        }
                    }
                }
                modified_count += 1;
            }
        }
        Ok(modified_count)
    }

    pub fn commit_stroke(&mut self, label: &str) -> Result<u64, GeoStudioError> {
        if !self.is_stroke_active { return Err(GeoStudioError::NoActiveStroke); }
        let commit_id = self.next_commit_id;
        self.next_commit_id += 1;
        self.is_stroke_active = false;

        let mut modified_tiles = Vec::new();
        for (coord, before_tile) in self.active_baseline.drain() {
            if let Some(after_tile) = self.tiles.get(&coord) {
                if !Arc::ptr_eq(&before_tile, after_tile) {
                    modified_tiles.push(TileDeltaPatch { tile_coord: coord, before_tile, after_tile: Arc::clone(after_tile) });
                }
            }
        }
        modified_tiles.sort_by_key(|p| p.tile_coord);
        let stroke_samples = std::mem::take(&mut self.active_strokes);
        self.undo_stack.push(UndoCommitEntry { commit_id, label: label.to_string(), modified_tiles, stroke_samples });
        self.redo_stack.clear();

        let current_bytes = self.estimate_memory_bytes();
        if current_bytes > self.byte_budget {
            if let Some(failed_entry) = self.undo_stack.pop() {
                for patch in failed_entry.modified_tiles { self.tiles.insert(patch.tile_coord, patch.before_tile); }
            }
            return Err(GeoStudioError::ByteBudgetExceeded { current_bytes, budget_bytes: self.byte_budget });
        }
        Ok(commit_id)
    }

    pub fn undo(&mut self) -> Result<Option<u64>, GeoStudioError> {
        if self.is_stroke_active { return Err(GeoStudioError::StrokeAlreadyActive); }
        let Some(entry) = self.undo_stack.pop() else { return Ok(None); };
        for patch in &entry.modified_tiles { self.tiles.insert(patch.tile_coord, Arc::clone(&patch.before_tile)); }
        let commit_id = entry.commit_id;
        self.redo_stack.push(entry);
        Ok(Some(commit_id))
    }

    pub fn redo(&mut self) -> Result<Option<u64>, GeoStudioError> {
        if self.is_stroke_active { return Err(GeoStudioError::StrokeAlreadyActive); }
        let Some(entry) = self.redo_stack.pop() else { return Ok(None); };
        for patch in &entry.modified_tiles { self.tiles.insert(patch.tile_coord, Arc::clone(&patch.after_tile)); }
        let commit_id = entry.commit_id;
        self.undo_stack.push(entry);
        Ok(Some(commit_id))
    }

    pub fn replay_stroke_journal(&mut self, journal: &[StrokeSample], label: &str) -> Result<u64, GeoStudioError> {
        self.begin_stroke()?;
        for sample in journal { self.apply_stroke_sample(*sample)?; }
        self.commit_stroke(label)
    }

    pub fn sample_height(&self, tx: i32, ty: i32, lx: usize, ly: usize) -> Option<f32> {
        if lx >= self.tile_size || ly >= self.tile_size { return None; }
        self.tiles.get(&(tx, ty)).map(|t| t[ly * self.tile_size + lx])
    }

    pub fn tile_arc(&self, tx: i32, ty: i32) -> Option<&Arc<Vec<f32>>> {
        self.tiles.get(&(tx, ty))
    }

    fn estimate_memory_bytes(&self) -> usize {
        let mut unique_arcs = HashSet::new();
        for tile in self.tiles.values() { unique_arcs.insert(Arc::as_ptr(tile) as usize); }
        for entry in self.undo_stack.iter().chain(self.redo_stack.iter()) {
            for patch in &entry.modified_tiles {
                unique_arcs.insert(Arc::as_ptr(&patch.before_tile) as usize);
                unique_arcs.insert(Arc::as_ptr(&patch.after_tile) as usize);
            }
        }
        for tile in self.active_baseline.values() { unique_arcs.insert(Arc::as_ptr(tile) as usize); }
        let tile_bytes = self.tile_size * self.tile_size * core::mem::size_of::<f32>();
        let samples_count = self.active_strokes.len()
            + self.undo_stack.iter().map(|e| e.stroke_samples.len()).sum::<usize>()
            + self.redo_stack.iter().map(|e| e.stroke_samples.len()).sum::<usize>();
        unique_arcs.len() * tile_bytes + samples_count * core::mem::size_of::<StrokeSample>()
    }
}

fn apply_op_to_vertex(
    op: BrushOperation, strength: f32, weight: f32, tile_size: usize, lx: usize, ly: usize, tile_vec: &mut [f32],
) {
    let idx = ly * tile_size + lx;
    let h = tile_vec[idx];
    let eff = strength * weight;
    match op {
        BrushOperation::Raise => tile_vec[idx] = h + eff,
        BrushOperation::Lower => tile_vec[idx] = h - eff,
        BrushOperation::Flatten { target_height } => {
            tile_vec[idx] = h + (target_height - h) * eff.clamp(0.0, 1.0);
        }
        BrushOperation::Smooth => {
            let left = tile_vec[ly * tile_size + lx.saturating_sub(1)];
            let right = tile_vec[ly * tile_size + (lx + 1).min(tile_size - 1)];
            let down = tile_vec[ly.saturating_sub(1) * tile_size + lx];
            let up = tile_vec[(ly + 1).min(tile_size - 1) * tile_size + lx];
            tile_vec[idx] = h + ((left + right + down + up) * 0.25 - h) * eff.clamp(0.0, 1.0);
        }
        BrushOperation::Terrace { step_m } => {
            if step_m > 0.0 {
                tile_vec[idx] = h + ((h / step_m).round() * step_m - h) * eff.clamp(0.0, 1.0);
            }
        }
        BrushOperation::HydraulicDrop => {
            let left = tile_vec[ly * tile_size + lx.saturating_sub(1)];
            let right = tile_vec[ly * tile_size + (lx + 1).min(tile_size - 1)];
            let down = tile_vec[ly.saturating_sub(1) * tile_size + lx];
            let up = tile_vec[(ly + 1).min(tile_size - 1) * tile_size + lx];
            let diff = h - (left + right + down + up) * 0.25;
            tile_vec[idx] = h - diff * eff.clamp(0.0, 1.0) * 0.5;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_sample(id: u64, x: f32, y: f32, r: f32, s: f32, op: u8, falloff: u8) -> StrokeSample {
        StrokeSample {
            stroke_id: id, timestamp_ms: 100, center_x_m: x, center_y_m: y, radius_m: r, strength: s,
            target_height_m: 0.0, step_m: 0.0, op_code: op, falloff_code: falloff, layer_index: 0,
            flags: 0, seed: 42, pad: [0; 2],
        }
    }

    #[test]
    fn stroke_sample_is_64_bytes() {
        assert_eq!(core::mem::size_of::<StrokeSample>(), 64);
    }

    #[test]
    fn cow_untouched_tiles_zero_copy() {
        let mut grid = CowTerrainTileGrid::new(16, 1.0, 1_000_000).unwrap();
        grid.ensure_tile(0, 0, 5.0).unwrap();
        grid.ensure_tile(5, 5, 20.0).unwrap();

        let touched_before = Arc::clone(grid.tile_arc(0, 0).unwrap());
        let untouched_before = Arc::clone(grid.tile_arc(5, 5).unwrap());

        grid.begin_stroke().unwrap();
        let sample = make_sample(1, 8.0, 8.0, 4.0, 2.0, 0, 0);
        let modified = grid.apply_stroke_sample(sample).unwrap();
        assert_eq!(modified, 1);
        grid.commit_stroke("raise_tile_0_0").unwrap();

        let touched_after = grid.tile_arc(0, 0).unwrap();
        let untouched_after = grid.tile_arc(5, 5).unwrap();

        assert!(!Arc::ptr_eq(&touched_before, touched_after));
        assert!(Arc::ptr_eq(&untouched_before, untouched_after));
        assert_eq!(grid.sample_height(5, 5, 0, 0), Some(20.0));
        assert!(grid.sample_height(0, 0, 8, 8).unwrap() > 5.0);
    }

    #[test]
    fn exact_undo_redo_multi_tile_restoration() {
        let mut grid = CowTerrainTileGrid::new(16, 1.0, 1_000_000).unwrap();
        grid.ensure_tile(0, 0, 10.0).unwrap();
        grid.ensure_tile(1, 0, 10.0).unwrap();

        let t0_initial = Arc::clone(grid.tile_arc(0, 0).unwrap());
        let t1_initial = Arc::clone(grid.tile_arc(1, 0).unwrap());

        grid.begin_stroke().unwrap();
        let sample = make_sample(2, 16.0, 8.0, 6.0, 3.0, 0, 2);
        let modified = grid.apply_stroke_sample(sample).unwrap();
        assert_eq!(modified, 2);
        grid.commit_stroke("straddle_border").unwrap();

        let h0_mod = grid.sample_height(0, 0, 15, 8).unwrap();
        let h1_mod = grid.sample_height(1, 0, 1, 8).unwrap();
        assert!(h0_mod > 10.0);
        assert!(h1_mod > 10.0);

        let undone = grid.undo().unwrap();
        assert_eq!(undone, Some(1));
        assert!(Arc::ptr_eq(&t0_initial, grid.tile_arc(0, 0).unwrap()));
        assert!(Arc::ptr_eq(&t1_initial, grid.tile_arc(1, 0).unwrap()));
        assert_eq!(grid.sample_height(0, 0, 15, 8), Some(10.0));
        assert_eq!(grid.sample_height(1, 0, 1, 8), Some(10.0));

        let redone = grid.redo().unwrap();
        assert_eq!(redone, Some(1));
        assert_eq!(grid.sample_height(0, 0, 15, 8), Some(h0_mod));
        assert_eq!(grid.sample_height(1, 0, 1, 8), Some(h1_mod));
    }

    #[test]
    fn deterministic_replay_stroke_journal_and_byte_budget_fail_closed() {
        let mut grid1 = CowTerrainTileGrid::new(16, 1.0, 500_000).unwrap();
        let mut grid2 = CowTerrainTileGrid::new(16, 1.0, 500_000).unwrap();

        grid1.ensure_tile(0, 0, 5.0).unwrap();
        grid2.ensure_tile(0, 0, 5.0).unwrap();

        let journal = vec![
            make_sample(1, 8.0, 8.0, 5.0, 2.5, 0, 0),
            make_sample(1, 10.0, 8.0, 4.0, 0.5, 3, 1),
        ];

        grid1.replay_stroke_journal(&journal, "journal_replay").unwrap();
        grid2.replay_stroke_journal(&journal, "journal_replay").unwrap();

        let tile1 = grid1.tile_arc(0, 0).unwrap();
        let tile2 = grid2.tile_arc(0, 0).unwrap();
        assert_eq!(tile1.as_slice(), tile2.as_slice());

        let mut low_budget_grid = CowTerrainTileGrid::new(16, 1.0, 500).unwrap();
        let err = low_budget_grid.ensure_tile(0, 0, 1.0).unwrap_err();
        match err {
            GeoStudioError::ByteBudgetExceeded { budget_bytes, .. } => {
                assert_eq!(budget_bytes, 500);
            }
            other => panic!("Expected ByteBudgetExceeded, got {other:?}"),
        }
    }
}

//! Botanical developmental state, organ kind ontology, and persistent organ continuity.
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BotanicalDevelopmentalStage {
    Seedling,
    Sapling,
    Mature,
    Senescent,
    EpicormicRegrowth,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OrganKind {
    PrimaryRoot,
    LateralRoot,
    MycorrhizalRootTip,
    Lignotuber,
    TrunkMetamer,
    ScaffoldBranch,
    EpicormicShoot,
    Petiole,
    LeafBlade,
    InflorescenceBud,
    OperculumFlower,
    WoodyCapsuleFruit,
    BarkRhytidome,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PersistentOrganDescriptor {
    pub organ_id: u64,
    pub parent_organ_id: Option<u64>,
    pub kind: OrganKind,
    pub stage: BotanicalDevelopmentalStage,
    pub metamer_order: u16,
    pub physiological_age_days: f32,
    pub water_potential_mpa: f32,
    pub carbohydrate_reserve_ratio: f32,
    pub split_uv_seam_id: u32,
    pub q42_lineage_hash: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub enum OrganStateError {
    InvalidWaterPotential { organ_id: u64, value: f32 },
    InvalidCarbohydrateReserve { organ_id: u64, value: f32 },
    OrganNotFound(u64),
    DuplicateOrganId(u64),
    MissingEpicormicParent(u64),
    ParentNotFound(u64),
    InvalidEpicormicParentKind { parent_id: u64, kind: OrganKind },
    EpicormicCarbohydrateDeficit { parent_id: u64, reserve: f32, required: f32 },
    InvalidStageTransition { from: BotanicalDevelopmentalStage, to: BotanicalDevelopmentalStage },
    CyclicParentHierarchy { organ_id: u64 },
    SerializationFailure(String),
    DeserializationFailure(String),
}

impl fmt::Display for OrganStateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidWaterPotential { organ_id, value } => write!(f, "Organ {organ_id} water potential {value:.2} MPa outside [-12.0, 0.0]"),
            Self::InvalidCarbohydrateReserve { organ_id, value } => write!(f, "Organ {organ_id} carbohydrate reserve {value:.3} outside [0.0, 1.0]"),
            Self::OrganNotFound(id) => write!(f, "Organ {id} not found"),
            Self::DuplicateOrganId(id) => write!(f, "Duplicate organ {id}"),
            Self::MissingEpicormicParent(id) => write!(f, "Epicormic organ {id} requires parent"),
            Self::ParentNotFound(id) => write!(f, "Parent {id} not found"),
            Self::InvalidEpicormicParentKind { parent_id, kind } => write!(f, "Parent {parent_id} kind {kind:?} must be Lignotuber/TrunkMetamer"),
            Self::EpicormicCarbohydrateDeficit { parent_id, reserve, required } => write!(f, "Parent {parent_id} reserve {reserve:.3} < required {required:.3}"),
            Self::InvalidStageTransition { from, to } => write!(f, "Invalid transition {from:?} -> {to:?}"),
            Self::CyclicParentHierarchy { organ_id } => write!(f, "Cyclic hierarchy at {organ_id}"),
            Self::SerializationFailure(msg) => write!(f, "Serialization failed: {msg}"),
            Self::DeserializationFailure(msg) => write!(f, "Deserialization failed: {msg}"),
        }
    }
}
impl std::error::Error for OrganStateError {}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BotanicalOrganStateGraph {
    organs: BTreeMap<u64, PersistentOrganDescriptor>,
    #[serde(skip)]
    undo_stack: Vec<BTreeMap<u64, PersistentOrganDescriptor>>,
    #[serde(skip)]
    redo_stack: Vec<BTreeMap<u64, PersistentOrganDescriptor>>,
}

impl Default for BotanicalOrganStateGraph {
    fn default() -> Self { Self::new() }
}

impl BotanicalOrganStateGraph {
    pub fn new() -> Self {
        Self { organs: BTreeMap::new(), undo_stack: Vec::new(), redo_stack: Vec::new() }
    }
    pub fn organ_count(&self) -> usize { self.organs.len() }
    pub fn get_organ(&self, id: u64) -> Option<&PersistentOrganDescriptor> { self.organs.get(&id) }
    pub fn get_organ_mut(&mut self, id: u64) -> Option<&mut PersistentOrganDescriptor> { self.organs.get_mut(&id) }
    pub fn organs(&self) -> impl Iterator<Item = &PersistentOrganDescriptor> { self.organs.values() }

    pub fn push_checkpoint(&mut self) {
        self.undo_stack.push(self.organs.clone());
        self.redo_stack.clear();
    }
    pub fn undo(&mut self) -> bool {
        if let Some(prev) = self.undo_stack.pop() {
            let curr = std::mem::replace(&mut self.organs, prev);
            self.redo_stack.push(curr);
            true
        } else { false }
    }
    pub fn redo(&mut self) -> bool {
        if let Some(next) = self.redo_stack.pop() {
            let curr = std::mem::replace(&mut self.organs, next);
            self.undo_stack.push(curr);
            true
        } else { false }
    }

    fn validate_organ(organ: &PersistentOrganDescriptor, map: &BTreeMap<u64, PersistentOrganDescriptor>) -> Result<(), OrganStateError> {
        if !(-12.0..=0.0).contains(&organ.water_potential_mpa) || !organ.water_potential_mpa.is_finite() {
            return Err(OrganStateError::InvalidWaterPotential { organ_id: organ.organ_id, value: organ.water_potential_mpa });
        }
        if !(0.0..=1.0).contains(&organ.carbohydrate_reserve_ratio) || !organ.carbohydrate_reserve_ratio.is_finite() {
            return Err(OrganStateError::InvalidCarbohydrateReserve { organ_id: organ.organ_id, value: organ.carbohydrate_reserve_ratio });
        }
        if let Some(parent_id) = organ.parent_organ_id {
            if parent_id == organ.organ_id {
                return Err(OrganStateError::CyclicParentHierarchy { organ_id: organ.organ_id });
            }
            let (mut curr, mut steps) = (parent_id, 0);
            while let Some(parent) = map.get(&curr) {
                if parent.organ_id == organ.organ_id || steps > 512 {
                    return Err(OrganStateError::CyclicParentHierarchy { organ_id: organ.organ_id });
                }
                if let Some(next) = parent.parent_organ_id { curr = next; steps += 1; } else { break; }
            }
        }
        if organ.stage == BotanicalDevelopmentalStage::EpicormicRegrowth {
            let parent_id = organ.parent_organ_id.ok_or(OrganStateError::MissingEpicormicParent(organ.organ_id))?;
            let parent = map.get(&parent_id).ok_or(OrganStateError::ParentNotFound(parent_id))?;
            if !matches!(parent.kind, OrganKind::Lignotuber | OrganKind::TrunkMetamer) {
                return Err(OrganStateError::InvalidEpicormicParentKind { parent_id, kind: parent.kind });
            }
            if parent.carbohydrate_reserve_ratio < 0.08 {
                return Err(OrganStateError::EpicormicCarbohydrateDeficit { parent_id, reserve: parent.carbohydrate_reserve_ratio, required: 0.08 });
            }
        }
        Ok(())
    }

    pub fn insert_organ(&mut self, organ: PersistentOrganDescriptor) -> Result<(), OrganStateError> {
        if self.organs.contains_key(&organ.organ_id) {
            return Err(OrganStateError::DuplicateOrganId(organ.organ_id));
        }
        Self::validate_organ(&organ, &self.organs)?;
        self.organs.insert(organ.organ_id, organ);
        Ok(())
    }

    pub fn update_physiology(&mut self, organ_id: u64, water_potential_mpa: f32, carbohydrate_reserve_ratio: f32) -> Result<(), OrganStateError> {
        if !(-12.0..=0.0).contains(&water_potential_mpa) || !water_potential_mpa.is_finite() {
            return Err(OrganStateError::InvalidWaterPotential { organ_id, value: water_potential_mpa });
        }
        if !(0.0..=1.0).contains(&carbohydrate_reserve_ratio) || !carbohydrate_reserve_ratio.is_finite() {
            return Err(OrganStateError::InvalidCarbohydrateReserve { organ_id, value: carbohydrate_reserve_ratio });
        }
        let organ = self.organs.get_mut(&organ_id).ok_or(OrganStateError::OrganNotFound(organ_id))?;
        organ.water_potential_mpa = water_potential_mpa;
        organ.carbohydrate_reserve_ratio = carbohydrate_reserve_ratio;
        Ok(())
    }

    pub fn transition_stage(&mut self, organ_id: u64, new_stage: BotanicalDevelopmentalStage) -> Result<(), OrganStateError> {
        let organ = self.organs.get(&organ_id).ok_or(OrganStateError::OrganNotFound(organ_id))?;
        let current_stage = organ.stage;
        let valid = match (current_stage, new_stage) {
            (from, to) if from == to => true,
            (BotanicalDevelopmentalStage::Seedling, BotanicalDevelopmentalStage::Sapling)
            | (BotanicalDevelopmentalStage::Sapling, BotanicalDevelopmentalStage::Mature)
            | (BotanicalDevelopmentalStage::Mature, BotanicalDevelopmentalStage::Senescent)
            | (BotanicalDevelopmentalStage::Mature, BotanicalDevelopmentalStage::EpicormicRegrowth)
            | (BotanicalDevelopmentalStage::Senescent, BotanicalDevelopmentalStage::EpicormicRegrowth)
            | (BotanicalDevelopmentalStage::EpicormicRegrowth, BotanicalDevelopmentalStage::Mature) => true,
            _ => false,
        };
        if !valid {
            return Err(OrganStateError::InvalidStageTransition { from: current_stage, to: new_stage });
        }
        if new_stage == BotanicalDevelopmentalStage::EpicormicRegrowth {
            let parent_id = organ.parent_organ_id.ok_or(OrganStateError::MissingEpicormicParent(organ_id))?;
            let parent = self.organs.get(&parent_id).ok_or(OrganStateError::ParentNotFound(parent_id))?;
            if !matches!(parent.kind, OrganKind::Lignotuber | OrganKind::TrunkMetamer) {
                return Err(OrganStateError::InvalidEpicormicParentKind { parent_id, kind: parent.kind });
            }
            if parent.carbohydrate_reserve_ratio < 0.08 {
                return Err(OrganStateError::EpicormicCarbohydrateDeficit { parent_id, reserve: parent.carbohydrate_reserve_ratio, required: 0.08 });
            }
            if !(-12.0..=0.0).contains(&organ.water_potential_mpa) || !organ.water_potential_mpa.is_finite() {
                return Err(OrganStateError::InvalidWaterPotential { organ_id, value: organ.water_potential_mpa });
            }
        }
        self.organs.get_mut(&organ_id).unwrap().stage = new_stage;
        Ok(())
    }

    pub fn to_checkpoint_json(&self) -> Result<String, OrganStateError> {
        serde_json::to_string(self).map_err(|e| OrganStateError::SerializationFailure(e.to_string()))
    }

    pub fn from_checkpoint_json(json_str: &str) -> Result<Self, OrganStateError> {
        serde_json::from_str(json_str).map_err(|e| OrganStateError::DeserializationFailure(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_root(id: u64) -> PersistentOrganDescriptor {
        PersistentOrganDescriptor {
            organ_id: id, parent_organ_id: None, kind: OrganKind::PrimaryRoot,
            stage: BotanicalDevelopmentalStage::Seedling, metamer_order: 0,
            physiological_age_days: 14.0, water_potential_mpa: -0.5, carbohydrate_reserve_ratio: 0.65,
            split_uv_seam_id: 101, q42_lineage_hash: 0x42_1111_2222_3333,
        }
    }
    fn sample_trunk(id: u64, parent_id: u64) -> PersistentOrganDescriptor {
        PersistentOrganDescriptor {
            organ_id: id, parent_organ_id: Some(parent_id), kind: OrganKind::TrunkMetamer,
            stage: BotanicalDevelopmentalStage::Seedling, metamer_order: 1,
            physiological_age_days: 14.0, water_potential_mpa: -0.8, carbohydrate_reserve_ratio: 0.25,
            split_uv_seam_id: 102, q42_lineage_hash: 0x42_5555_6666_7777,
        }
    }

    #[test]
    fn test_developmental_stage_transitions() {
        let mut graph = BotanicalOrganStateGraph::new();
        graph.insert_organ(sample_root(1)).unwrap();
        graph.insert_organ(sample_trunk(2, 1)).unwrap();
        assert_eq!(graph.get_organ(2).unwrap().stage, BotanicalDevelopmentalStage::Seedling);
        graph.transition_stage(2, BotanicalDevelopmentalStage::Sapling).unwrap();
        graph.transition_stage(2, BotanicalDevelopmentalStage::Mature).unwrap();
        graph.transition_stage(2, BotanicalDevelopmentalStage::Senescent).unwrap();
        assert_eq!(graph.get_organ(2).unwrap().stage, BotanicalDevelopmentalStage::Senescent);

        let branch = PersistentOrganDescriptor {
            organ_id: 3, parent_organ_id: Some(2), kind: OrganKind::EpicormicShoot,
            stage: BotanicalDevelopmentalStage::Mature, metamer_order: 2,
            physiological_age_days: 200.0, water_potential_mpa: -1.2, carbohydrate_reserve_ratio: 0.15,
            split_uv_seam_id: 103, q42_lineage_hash: 0x42_9999_aaaa_bbbb,
        };
        graph.insert_organ(branch).unwrap();
        graph.transition_stage(3, BotanicalDevelopmentalStage::EpicormicRegrowth).unwrap();
        assert_eq!(graph.get_organ(3).unwrap().stage, BotanicalDevelopmentalStage::EpicormicRegrowth);
    }

    #[test]
    fn test_checkpoint_json_roundtrip_preserves_identifiers() {
        let mut graph = BotanicalOrganStateGraph::new();
        graph.insert_organ(sample_root(10)).unwrap();
        graph.insert_organ(sample_trunk(20, 10)).unwrap();
        let json = graph.to_checkpoint_json().unwrap();
        let restored = BotanicalOrganStateGraph::from_checkpoint_json(&json).unwrap();
        assert_eq!(restored.organ_count(), 2);
        let (o1, o2) = (restored.get_organ(10).unwrap(), restored.get_organ(20).unwrap());
        assert_eq!(o1.organ_id, 10);
        assert_eq!(o1.split_uv_seam_id, 101);
        assert_eq!(o1.q42_lineage_hash, 0x42_1111_2222_3333);
        assert_eq!(o2.organ_id, 20);
        assert_eq!(o2.parent_organ_id, Some(10));
        assert_eq!(o2.split_uv_seam_id, 102);
        assert_eq!(o2.q42_lineage_hash, 0x42_5555_6666_7777);
    }

    #[test]
    fn test_undo_redo_preserves_state() {
        let mut graph = BotanicalOrganStateGraph::new();
        graph.insert_organ(sample_root(1)).unwrap();
        graph.push_checkpoint();
        graph.transition_stage(1, BotanicalDevelopmentalStage::Sapling).unwrap();
        assert_eq!(graph.get_organ(1).unwrap().stage, BotanicalDevelopmentalStage::Sapling);
        assert!(graph.undo());
        assert_eq!(graph.get_organ(1).unwrap().stage, BotanicalDevelopmentalStage::Seedling);
        assert!(graph.redo());
        assert_eq!(graph.get_organ(1).unwrap().stage, BotanicalDevelopmentalStage::Sapling);
    }

    #[test]
    fn test_negative_water_potential_out_of_bounds() {
        let mut graph = BotanicalOrganStateGraph::new();
        let mut bad = sample_root(1);
        bad.water_potential_mpa = -13.5;
        assert_eq!(graph.insert_organ(bad), Err(OrganStateError::InvalidWaterPotential { organ_id: 1, value: -13.5 }));
        graph.insert_organ(sample_root(1)).unwrap();
        assert_eq!(graph.update_physiology(1, 0.5, 0.5), Err(OrganStateError::InvalidWaterPotential { organ_id: 1, value: 0.5 }));
    }

    #[test]
    fn test_negative_epicormic_regrowth_invariants() {
        let mut graph = BotanicalOrganStateGraph::new();
        let leaf = PersistentOrganDescriptor {
            organ_id: 5, parent_organ_id: None, kind: OrganKind::LeafBlade,
            stage: BotanicalDevelopmentalStage::Mature, metamer_order: 1,
            physiological_age_days: 30.0, water_potential_mpa: -1.0, carbohydrate_reserve_ratio: 0.05,
            split_uv_seam_id: 50, q42_lineage_hash: 0x42_1eaf,
        };
        graph.insert_organ(leaf).unwrap();

        let child1 = PersistentOrganDescriptor {
            organ_id: 6, parent_organ_id: Some(5), kind: OrganKind::EpicormicShoot,
            stage: BotanicalDevelopmentalStage::Mature, metamer_order: 2,
            physiological_age_days: 10.0, water_potential_mpa: -1.0, carbohydrate_reserve_ratio: 0.1,
            split_uv_seam_id: 51, q42_lineage_hash: 0x42_c1d1,
        };
        graph.insert_organ(child1).unwrap();
        assert_eq!(
            graph.transition_stage(6, BotanicalDevelopmentalStage::EpicormicRegrowth),
            Err(OrganStateError::InvalidEpicormicParentKind { parent_id: 5, kind: OrganKind::LeafBlade })
        );

        let trunk = PersistentOrganDescriptor {
            organ_id: 7, parent_organ_id: None, kind: OrganKind::TrunkMetamer,
            stage: BotanicalDevelopmentalStage::Mature, metamer_order: 1,
            physiological_age_days: 100.0, water_potential_mpa: -1.0, carbohydrate_reserve_ratio: 0.04,
            split_uv_seam_id: 70, q42_lineage_hash: 0x42_aa01,
        };
        graph.insert_organ(trunk).unwrap();

        let child2 = PersistentOrganDescriptor {
            organ_id: 8, parent_organ_id: Some(7), kind: OrganKind::EpicormicShoot,
            stage: BotanicalDevelopmentalStage::Mature, metamer_order: 2,
            physiological_age_days: 10.0, water_potential_mpa: -1.0, carbohydrate_reserve_ratio: 0.1,
            split_uv_seam_id: 80, q42_lineage_hash: 0x42_c1d2,
        };
        graph.insert_organ(child2).unwrap();
        assert_eq!(
            graph.transition_stage(8, BotanicalDevelopmentalStage::EpicormicRegrowth),
            Err(OrganStateError::EpicormicCarbohydrateDeficit { parent_id: 7, reserve: 0.04, required: 0.08 })
        );
    }

    #[test]
    fn test_negative_invalid_stage_transition() {
        let mut graph = BotanicalOrganStateGraph::new();
        let mut trunk = sample_trunk(1, 0);
        trunk.stage = BotanicalDevelopmentalStage::Mature;
        trunk.parent_organ_id = None;
        graph.insert_organ(trunk).unwrap();
        assert_eq!(
            graph.transition_stage(1, BotanicalDevelopmentalStage::Seedling),
            Err(OrganStateError::InvalidStageTransition {
                from: BotanicalDevelopmentalStage::Mature,
                to: BotanicalDevelopmentalStage::Seedling,
            })
        );
    }

    #[test]
    fn test_negative_cyclic_hierarchy() {
        let mut graph = BotanicalOrganStateGraph::new();
        let mut cyclic = sample_root(1);
        cyclic.parent_organ_id = Some(1);
        assert_eq!(graph.insert_organ(cyclic), Err(OrganStateError::CyclicParentHierarchy { organ_id: 1 }));
    }
}

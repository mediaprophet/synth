//! Canonical checked FIEL cases execute against Poet's exact native dependency.
pub mod container_10d {
    pub use qualia_core_db::container_10d::*;
}
use qualia_core_db::container_10d::field_section::*;
#[path = "../../../qualiaDB/crates/qualia-core-db/src/container_10d/field_section/checked/tests.rs"]
mod canonical_checked_cases;

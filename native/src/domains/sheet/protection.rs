//! Clean-room spreadsheet cell and range protection lock manager.

use super::{CellCoord, SheetError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CellRange {
    pub start: CellCoord,
    pub end: CellCoord,
}

impl CellRange {
    pub const fn new(start: CellCoord, end: CellCoord) -> Self {
        Self { start, end }
    }

    pub fn single_cell(coord: CellCoord) -> Self {
        Self { start: coord, end: coord }
    }

    pub fn contains(&self, coord: CellCoord) -> bool {
        let min_row = self.start.row.min(self.end.row);
        let max_row = self.start.row.max(self.end.row);
        let min_col = self.start.col.min(self.end.col);
        let max_col = self.start.col.max(self.end.col);

        coord.row >= min_row
            && coord.row <= max_row
            && coord.col >= min_col
            && coord.col <= max_col
    }

    pub fn intersects(&self, other: &CellRange) -> bool {
        let r1_min = self.start.row.min(self.end.row);
        let r1_max = self.start.row.max(self.end.row);
        let c1_min = self.start.col.min(self.end.col);
        let c1_max = self.start.col.max(self.end.col);

        let r2_min = other.start.row.min(other.end.row);
        let r2_max = other.start.row.max(other.end.row);
        let c2_min = other.start.col.min(other.end.col);
        let c2_max = other.start.col.max(other.end.col);

        r1_min <= r2_max && r1_max >= r2_min && c1_min <= c2_max && c1_max >= c2_min
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtectionRule {
    pub range: CellRange,
    pub locked: bool,
    pub authorized_actors: Vec<u64>,
}

impl ProtectionRule {
    pub fn new(range: CellRange, locked: bool, authorized_actors: Vec<u64>) -> Self {
        Self {
            range,
            locked,
            authorized_actors,
        }
    }

    pub fn allows_actor(&self, actor_did_hash: u64) -> bool {
        self.authorized_actors.contains(&actor_did_hash)
    }
}

#[derive(Debug, Clone, Default)]
pub struct SheetProtectionEngine {
    sheet_locked: bool,
    owner_did_hash: Option<u64>,
    rules: Vec<ProtectionRule>,
}

impl SheetProtectionEngine {
    pub fn new(owner_did_hash: Option<u64>) -> Self {
        Self {
            sheet_locked: false,
            owner_did_hash,
            rules: Vec::new(),
        }
    }

    pub fn set_sheet_locked(&mut self, locked: bool) {
        self.sheet_locked = locked;
    }

    pub fn is_sheet_locked(&self) -> bool {
        self.sheet_locked
    }

    pub fn add_rule(&mut self, rule: ProtectionRule) {
        self.rules.push(rule);
    }

    pub fn clear_rules(&mut self) {
        self.rules.clear();
    }

    /// Evaluates if an actor can edit a specific cell coordinate.
    /// Returns `Err(SheetError::ProtectedCellRefused)` if locked without authorization.
    pub fn can_edit_cell(&self, coord: CellCoord, actor_did_hash: u64) -> Result<(), SheetError> {
        // Sheet owner has universal authoring authority
        if let Some(owner) = self.owner_did_hash {
            if owner == actor_did_hash {
                return Ok(());
            }
        }

        // Check explicit rules in reverse (most recent rule takes precedence)
        for rule in self.rules.iter().rev() {
            if rule.range.contains(coord) {
                if !rule.locked {
                    return Ok(()); // Explicit unlocked exception
                }
                if rule.allows_actor(actor_did_hash) {
                    return Ok(());
                } else {
                    return Err(SheetError::ProtectedCellRefused);
                }
            }
        }

        // If sheet is globally locked and cell is not in an unlocked exception
        if self.sheet_locked {
            return Err(SheetError::ProtectedCellRefused);
        }

        Ok(())
    }

    /// Evaluates if an actor can edit every cell within a range.
    pub fn can_edit_range(&self, range: CellRange, actor_did_hash: u64) -> Result<(), SheetError> {
        let min_row = range.start.row.min(range.end.row);
        let max_row = range.start.row.max(range.end.row);
        let min_col = range.start.col.min(range.end.col);
        let max_col = range.start.col.max(range.end.col);

        for row in min_row..=max_row {
            for col in min_col..=max_col {
                self.can_edit_cell(CellCoord::new(row, col), actor_did_hash)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const OWNER_DID: u64 = 0xAAAA_1111;
    const ALICE_DID: u64 = 0xBBBB_2222;
    const BOB_DID: u64 = 0xCCCC_3333;

    #[test]
    fn test_unlocked_sheet_permits_edits() {
        let engine = SheetProtectionEngine::new(Some(OWNER_DID));
        assert_eq!(engine.can_edit_cell(CellCoord::new(5, 5), BOB_DID), Ok(()));
    }

    #[test]
    fn test_owner_always_permitted() {
        let mut engine = SheetProtectionEngine::new(Some(OWNER_DID));
        engine.set_sheet_locked(true);
        assert_eq!(engine.can_edit_cell(CellCoord::new(10, 10), OWNER_DID), Ok(()));
    }

    #[test]
    fn test_mandatory_negative_locked_cell_refused() {
        let mut engine = SheetProtectionEngine::new(Some(OWNER_DID));
        let locked_range = CellRange::new(CellCoord::new(1, 1), CellCoord::new(3, 3));
        engine.add_rule(ProtectionRule::new(locked_range, true, vec![ALICE_DID]));

        // Alice is authorized
        assert_eq!(engine.can_edit_cell(CellCoord::new(2, 2), ALICE_DID), Ok(()));

        // Bob is unauthorized -> MUST return ProtectedCellRefused
        assert_eq!(
            engine.can_edit_cell(CellCoord::new(2, 2), BOB_DID),
            Err(SheetError::ProtectedCellRefused)
        );

        // Outside locked range is permitted on an unlocked sheet
        assert_eq!(engine.can_edit_cell(CellCoord::new(4, 4), BOB_DID), Ok(()));
    }

    #[test]
    fn test_mandatory_negative_global_sheet_locked() {
        let mut engine = SheetProtectionEngine::new(Some(OWNER_DID));
        engine.set_sheet_locked(true);

        // Unauthorized actor refused everywhere
        assert_eq!(
            engine.can_edit_cell(CellCoord::new(0, 0), BOB_DID),
            Err(SheetError::ProtectedCellRefused)
        );

        // Add unlocked range exception
        let open_range = CellRange::new(CellCoord::new(0, 0), CellCoord::new(0, 5));
        engine.add_rule(ProtectionRule::new(open_range, false, vec![]));

        // Now cell in open_range is editable by Bob
        assert_eq!(engine.can_edit_cell(CellCoord::new(0, 2), BOB_DID), Ok(()));
        // Cell outside open_range is still refused
        assert_eq!(
            engine.can_edit_cell(CellCoord::new(1, 0), BOB_DID),
            Err(SheetError::ProtectedCellRefused)
        );
    }

    #[test]
    fn test_can_edit_range_negative() {
        let mut engine = SheetProtectionEngine::new(Some(OWNER_DID));
        let locked_range = CellRange::new(CellCoord::new(2, 2), CellCoord::new(2, 2));
        engine.add_rule(ProtectionRule::new(locked_range, true, vec![ALICE_DID]));

        let test_range = CellRange::new(CellCoord::new(1, 1), CellCoord::new(3, 3));
        // Alice can edit because she is authorized on the locked cell (and others are unlocked)
        assert_eq!(engine.can_edit_range(test_range, ALICE_DID), Ok(()));
        // Bob is refused because (2, 2) is locked
        assert_eq!(
            engine.can_edit_range(test_range, BOB_DID),
            Err(SheetError::ProtectedCellRefused)
        );
    }
}

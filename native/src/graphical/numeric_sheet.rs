//! Typed view of an existing mutable Vibe numeric grid; no new workbook format.
use crate::graphical::preview::SourceRef;
use vibe::Value;
#[derive(Clone, Debug)]
pub struct SheetView { pub source: SourceRef, pub execution_epoch: crate::graphical::controller::SessionEpoch, pub rows: Vec<Vec<Value>> }
#[derive(Clone, Debug)]
pub struct SheetEdit { pub row: usize, pub col: usize, pub value: f64, pub execution_epoch: crate::graphical::controller::SessionEpoch }
pub fn admit(source: SourceRef, execution_epoch: crate::graphical::controller::SessionEpoch, grid: &Value) -> Result<SheetView, String> {
 let Value::List(rows) = grid else { return Err("Sheet must be a numeric grid".into()); };
 if rows.is_empty() || rows.len() > 32 { return Err("Sheet row count must be 1..=32".into()); }
 let mut width = None; let mut result = Vec::with_capacity(rows.len());
 for row in rows {
  let Value::List(cells) = row else { return Err("Sheet row must be a list".into()); };
  if cells.is_empty() || cells.len() > 32 || width.is_some_and(|n| n != cells.len()) { return Err("Sheet must be rectangular with 1..=32 columns".into()); }
  width = Some(cells.len());
  for value in cells { match value { Value::I64(_) | Value::U64(_) => {}, Value::F64(n) if n.is_finite() => {}, _ => return Err("Sheet cells must be finite numbers".into()) } }
  result.push(cells.clone());
 }
 Ok(SheetView { source, execution_epoch, rows: result })
}
pub fn source(edit: &SheetEdit, view: &SheetView) -> Result<String, String> {
 if !edit.value.is_finite() { return Err("Sheet edit must be finite".into()); }
 if edit.row >= view.rows.len() || edit.col >= view.rows[edit.row].len() { return Err("Sheet edit is outside the existing grid".into()); }
 Ok(format!("requires [capability(\"capability.invoke\"),capability(\"Sheet.set_cell\")]; poet_sheet = capability.invoke(\"Sheet.set_cell\",{{grid:poet_sheet,row:{},col:{},value:{:?}}}); poet_sheet;",edit.row,edit.col,edit.value))
}

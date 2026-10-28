//! Caller-selected cold operation budget over existing canonical session values.
//! The cost model counts Value slots, record overhead and text before either
//! clone. These are budgeted structural bytes, not measured allocator/RSS bytes.
//! SessionLimits independently governs retained values; no source grants budget.
use vibe::Value;
#[cfg(test)]
const LEGACY_BYTES: usize = 64 * 1024 * 1024;

#[derive(Debug)]
struct SnapshotCost {
    bytes: usize,
    nodes: usize,
    text_bytes: usize,
}
fn cost(parts: Option<&Value>, base: Option<&Value>) -> Result<SnapshotCost, String> {
    let mut total = SnapshotCost {
        bytes: 0,
        nodes: 0,
        text_bytes: 0,
    };
    for value in [parts, base].into_iter().flatten() {
        // Retain the existing primitive/container profile and per-value
        // 1m-node / 1MiB-text / 64-frame bounds. This operation changes only the
        // independently selected combined byte allowance, with checked sums.
        super::super::parameter_safety::equal(value, value)?;
        enum Frame<'a> {
            Value(&'a Value),
            List(std::slice::Iter<'a, Value>),
            Record(std::collections::btree_map::Iter<'a, String, Value>),
        }
        let mut stack = vec![Frame::Value(value)];
        while let Some(frame) = stack.pop() {
            let (charge, text) = match frame {
                Frame::Value(value) => {
                    total.nodes = total
                        .nodes
                        .checked_add(1)
                        .ok_or("Source layer snapshot node count overflow")?;
                    let text = match value {
                        Value::String(s) => s.len(),
                        _ => 0,
                    };
                    match value {
                        Value::List(xs) => stack.push(Frame::List(xs.iter())),
                        Value::Record(xs) => stack.push(Frame::Record(xs.iter())),
                        _ => {}
                    }
                    (std::mem::size_of::<Value>().checked_add(text), text)
                }
                Frame::List(mut xs) => {
                    if let Some(value) = xs.next() {
                        stack.push(Frame::List(xs));
                        stack.push(Frame::Value(value));
                    }
                    (Some(0), 0)
                }
                Frame::Record(mut xs) => {
                    if let Some((key, value)) = xs.next() {
                        let charge = key.len().checked_add(
                            4 * std::mem::size_of::<usize>() + std::mem::size_of::<String>(),
                        );
                        stack.push(Frame::Record(xs));
                        stack.push(Frame::Value(value));
                        (charge, key.len())
                    } else {
                        (Some(0), 0)
                    }
                }
            };
            total.bytes = total
                .bytes
                .checked_add(charge.ok_or("Source layer snapshot byte charge overflow")?)
                .ok_or("Source layer snapshot byte sum overflow")?;
            total.text_bytes = total
                .text_bytes
                .checked_add(text)
                .ok_or("Source layer snapshot text count overflow")?;
        }
    }
    Ok(total)
}

/// Test-only compatibility profile: unchanged 64 MiB combined structural-byte ceiling.
#[cfg(test)]
pub(crate) fn validation_snapshot(
    parts: Option<&Value>,
    base: Option<&Value>,
) -> Result<(Option<Value>, Option<Value>), String> {
    validation_snapshot_with_budget(parts, base, LEGACY_BYTES)
}

/// Native owner passes its existing SessionLimits.retained_value_bytes. The
/// caller chooses this operation allowance, independently of source content.
/// Count the complete bounded input before cloning, so refusals report required
/// versus authorized structural bytes without allocating either output copy.
pub(crate) fn validation_snapshot_with_budget(
    parts: Option<&Value>,
    base: Option<&Value>,
    working_bytes: usize,
) -> Result<(Option<Value>, Option<Value>), String> {
    if working_bytes == 0 {
        return Err("Source layer snapshot requires a nonzero host-selected cold operation budget; admitted 0 bytes; accepted work preserved".into());
    }
    let cost = cost(parts, base)?;
    if cost.bytes > working_bytes {
        return Err(format!("Source layer snapshot requires {} budgeted structural bytes; host admits {} bytes; observed {} Value nodes and {} text bytes; accepted work preserved",cost.bytes,working_bytes,cost.nodes,cost.text_bytes));
    }
    Ok((parts.cloned(), base.cloned()))
}

#[cfg(test)]
#[path = "snapshot_tests.rs"]
mod tests;

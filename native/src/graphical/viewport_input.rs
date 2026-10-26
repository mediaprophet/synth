//! Fresh native client-rectangle mapping for the square retained viewport attachment.

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct ClientRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

fn square(rect: ClientRect, side: u32) -> Option<(f64, f64, f64)> {
    if side == 0
        || ![rect.x, rect.y, rect.width, rect.height]
            .into_iter()
            .all(f64::is_finite)
        || rect.width <= 0.0
        || rect.height <= 0.0
    {
        return None;
    }
    let extent = rect.width.min(rect.height);
    Some((
        rect.x + (rect.width - extent) * 0.5,
        rect.y + (rect.height - extent) * 0.5,
        extent / f64::from(side),
    ))
}

/// Map client coordinates through CSS `contain` letterboxing into attachment pixels.
/// The right and bottom edges are exclusive, matching backend pick bounds.
pub(super) fn client_to_attachment(
    client_x: f64,
    client_y: f64,
    rect: ClientRect,
    side: u32,
) -> Option<[f32; 2]> {
    if !client_x.is_finite() || !client_y.is_finite() {
        return None;
    }
    let (left, top, scale) = square(rect, side)?;
    let x = (client_x - left) / scale;
    let y = (client_y - top) / scale;
    if x < 0.0 || y < 0.0 || x >= f64::from(side) || y >= f64::from(side) {
        return None;
    }
    Some([x as f32, y as f32])
}

/// Map a client-space drag vector to attachment pixels using the fresh contained image scale.
pub(super) fn delta_to_attachment(
    dx: f64,
    dy: f64,
    rect: ClientRect,
    side: u32,
) -> Option<[f32; 2]> {
    if !dx.is_finite() || !dy.is_finite() {
        return None;
    }
    let (_, _, scale) = square(rect, side)?;
    let mapped = [dx / scale, dy / scale];
    mapped
        .iter()
        .all(|value| value.is_finite() && value.abs() <= 8192.0)
        .then(|| [mapped[0] as f32, mapped[1] as f32])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(x: f64, y: f64, width: f64, height: f64) -> ClientRect {
        ClientRect { x, y, width, height }
    }

    #[test]
    fn maps_square_coordinates_to_attachment() {
        assert_eq!(client_to_attachment(74.0, 91.0, rect(10.0, 27.0, 128.0, 128.0), 512), Some([256.0, 256.0]));
    }

    #[test]
    fn contain_letterbox_is_not_pickable() {
        let bounds = rect(0.0, 0.0, 200.0, 100.0);
        assert_eq!(client_to_attachment(50.0, 0.0, bounds, 512), Some([0.0, 0.0]));
        assert_eq!(client_to_attachment(49.9, 50.0, bounds, 512), None);
        assert_eq!(client_to_attachment(150.0, 75.0, bounds, 512), None);
    }

    #[test]
    fn rejects_nonfinite_zero_and_outside_inputs() {
        assert_eq!(client_to_attachment(f64::NAN, 1.0, rect(0.0, 0.0, 50.0, 50.0), 512), None);
        assert_eq!(client_to_attachment(1.0, 1.0, rect(0.0, 0.0, 0.0, 50.0), 512), None);
        assert_eq!(client_to_attachment(512.0, 20.0, rect(0.0, 0.0, 512.0, 512.0), 512), None);
    }

    #[test]
    fn drag_delta_uses_same_contain_scale_and_rejects_excess() {
        assert_eq!(delta_to_attachment(10.0, -20.0, rect(0.0, 0.0, 100.0, 200.0), 512), Some([51.2, -102.4]));
        assert_eq!(delta_to_attachment(f64::INFINITY, 0.0, rect(0.0, 0.0, 100.0, 100.0), 512), None);
        assert_eq!(delta_to_attachment(2000.0, 0.0, rect(0.0, 0.0, 100.0, 100.0), 512), None);
    }
}

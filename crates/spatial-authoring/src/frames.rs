//! Representation frames reuse the Core inverse-transpose instance record.
use crate::AuthoringError;
use poet_geometry::Vec3;
use qualia_core_db::{
    render::instance_culling::GpuInstanceRecord,
    specialized_libs::computational_geometry::authoring::Mat4,
};
pub(crate) fn unit(value: [f64; 3]) -> Result<[f32; 3], AuthoringError> {
    let v = Vec3::new(value[0], value[1], value[2]);
    let length = v.len();
    if !length.is_finite() || length <= 1e-30 {
        return Err(AuthoringError::Invalid(
            "degenerate/nonfinite authored normal or triangle",
        ));
    }
    let n = v.normalized();
    let out = [n.x as f32, n.y as f32, n.z as f32];
    if out.iter().any(|v| !v.is_finite()) {
        return Err(AuthoringError::Invalid("normal exceeds f32 domain"));
    }
    Ok(out)
}
pub(crate) fn record(world: &Mat4, id: u64) -> Result<GpuInstanceRecord, AuthoringError> {
    let matrix = std::array::from_fn(|c| std::array::from_fn(|r| world[c][r] as f32));
    if matrix.iter().flatten().any(|v| !v.is_finite()) {
        return Err(AuthoringError::Invalid(
            "world transform exceeds native f32 domain",
        ));
    }
    let record = GpuInstanceRecord::new(matrix, id);
    if record.orientation_sign != 1.0
        || record
            .normal_from_local
            .iter()
            .any(|c| c[..3].iter().any(|n| !n.is_finite()) || c[..3].iter().all(|n| *n == 0.0))
    {
        return Err(AuthoringError::Invalid(
            "singular, reflected or nonfinite world normal transform",
        ));
    }
    Ok(record)
}
pub(crate) fn normal(
    record: &GpuInstanceRecord,
    value: [f32; 3],
) -> Result<[f32; 3], AuthoringError> {
    if value.iter().any(|v| !v.is_finite()) {
        return Err(AuthoringError::Invalid("authored normal is nonfinite"));
    }
    unit(std::array::from_fn(|r| {
        (0..3)
            .map(|c| record.normal_from_local[c][r] as f64 * value[c] as f64)
            .sum()
    }))
}
pub(crate) fn face(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> Result<[f32; 3], AuthoringError> {
    let v = |p: [f32; 3]| Vec3::new(p[0] as f64, p[1] as f64, p[2] as f64);
    let n = (v(b) - v(a)).cross(v(c) - v(a));
    unit([n.x, n.y, n.z])
}

//! Native operator intent projection; not a resource format or authoritative store.
use super::checkpoints::{GeoRasterBrushRequest, ResourceInspectionRequest};
use poet_spatial_authoring::geo_studio::StrokeSample;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TerrainOperation { Raise, Lower, Flatten, Smooth, Terrace }
impl TerrainOperation {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "raise"=>Ok(Self::Raise),"lower"=>Ok(Self::Lower),"flatten"=>Ok(Self::Flatten),
            "smooth"=>Ok(Self::Smooth),"terrace"=>Ok(Self::Terrace),
            _=>Err("Unknown terrain brush operation".into()),
        }
    }
    fn code(self) -> u8 { match self {Self::Raise=>0,Self::Lower=>1,Self::Flatten=>2,Self::Smooth=>3,Self::Terrace=>4} }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TerrainFalloff { SmoothStep, Gaussian, Linear, Plateau }
impl TerrainFalloff {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value { "smooth_step"=>Ok(Self::SmoothStep),"gaussian"=>Ok(Self::Gaussian),"linear"=>Ok(Self::Linear),"plateau"=>Ok(Self::Plateau),_=>Err("Unknown terrain falloff".into()) }
    }
    fn code(self) -> u8 { match self {Self::SmoothStep=>0,Self::Gaussian=>1,Self::Linear=>2,Self::Plateau=>3} }
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TerrainStrokeDraft {
    pub operation: TerrainOperation,
    pub falloff: TerrainFalloff,
    pub center_m: [f64; 2],
    pub radius_m: f64,
    pub strength: f64,
    #[serde(default)]
    pub target_height_m: Option<f64>,
    #[serde(default)]
    pub step_m: Option<f64>,
}
/// The existing M64 owner projects edited heights to nearest integer millimetres,
/// with ties away from zero. Require authored metres to survive the operator's
/// f64-to-f32 boundary within that same quantization cell and within half a mm.
const MAX_METRE_NARROWING_ERROR_M: f64 = 0.0005;
const MAX_WEIGHT_NARROWING_ERROR: f64 = 1.0e-6;

fn finite_f32(value: f64) -> Result<f32, String> {
    let out = value as f32;
    if !value.is_finite() || !out.is_finite() { return Err("Terrain parameters require finite native f32 values".into()); }
    Ok(out)
}
fn project_metres_f32(value: f64, field: &str) -> Result<f32, String> {
    let out = finite_f32(value)?;
    let narrowed = f64::from(out);
    let error = (value - narrowed).abs();
    let source_mm = (value * 1000.0).round();
    let narrowed_mm = (narrowed * 1000.0).round();
    if !error.is_finite()
        || error > MAX_METRE_NARROWING_ERROR_M
        || !source_mm.is_finite()
        || !narrowed_mm.is_finite()
        || source_mm != narrowed_mm
    {
        return Err(format!(
            "Terrain {field} loses more than half a millimetre or changes the existing nearest-millimetre projection"
        ));
    }
    Ok(out)
}
fn project_weight_f32(value: f64) -> Result<f32, String> {
    let out = finite_f32(value)?;
    if (value - f64::from(out)).abs() > MAX_WEIGHT_NARROWING_ERROR {
        return Err("Terrain dimensionless strength exceeds the admitted f32 precision tolerance".into());
    }
    Ok(out)
}
impl TerrainStrokeDraft {
    fn sample(&self, id: u64) -> Result<StrokeSample, String> {
        let radius = project_metres_f32(self.radius_m, "radius")?;
        let strength = if matches!(self.operation,TerrainOperation::Raise|TerrainOperation::Lower) {
            project_metres_f32(self.strength, "raise/lower strength")?
        } else {
            project_weight_f32(self.strength)?
        };
        let center = [project_metres_f32(self.center_m[0], "centre X")?,project_metres_f32(self.center_m[1], "centre Y")?];
        if radius <= 0.0 || !(radius*radius).is_finite() || strength < 0.0
            || ![center[0]-radius,center[0]+radius,center[1]-radius,center[1]+radius].iter().all(|v|v.is_finite()) {
            return Err("Terrain radius must be positive and strength nonnegative within native bounds".into());
        }
        if !matches!(self.operation,TerrainOperation::Raise|TerrainOperation::Lower) && strength > 1.0 {
            return Err("Flatten, smooth and terrace strength must be in 0..=1".into());
        }
        let target = match (self.operation,self.target_height_m) {
            (TerrainOperation::Flatten,Some(v)) => project_metres_f32(v, "flatten target height")?,
            (TerrainOperation::Flatten,None) => return Err("Flatten requires an explicit absolute target height in metres".into()),
            (_,None) => 0.0,
            (_,Some(_)) => return Err("Target height applies only to flatten".into()),
        };
        let step = match (self.operation,self.step_m) {
            (TerrainOperation::Terrace,Some(v)) if v>0.0 => { let value=project_metres_f32(v, "terrace step")?; if value<=0.0 {return Err("Terrace step is below the native metre range".into());} value },
            (TerrainOperation::Terrace,_) => return Err("Terrace requires an explicit positive step in metres".into()),
            (_,None) => 0.0,
            (_,Some(_)) => return Err("Step applies only to terrace".into()),
        };
        Ok(StrokeSample {stroke_id:id,timestamp_ms:0,center_x_m:center[0],center_y_m:center[1],radius_m:radius,strength,
            target_height_m:target,step_m:step,op_code:self.operation.code(),falloff_code:self.falloff.code(),layer_index:0,flags:0,seed:0,pad:[0;2]})
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TerrainBrushDraft {
    pub resource: ResourceInspectionRequest,
    pub window_x: usize,
    pub window_y: usize,
    pub window_side: usize,
    pub strokes: Vec<TerrainStrokeDraft>,
}
impl TerrainBrushDraft {
    pub fn validate(&self) -> Result<(), String> {
        self.resource.validate_bounds()?;
        if !(1..=64).contains(&self.window_side) || !(1..=64).contains(&self.strokes.len()) {
            return Err("Select a square window of 1..=64 cells and 1..=64 strokes".into());
        }
        self.window_x.checked_add(self.window_side).ok_or("Terrain column extent overflow")?;
        self.window_y.checked_add(self.window_side).ok_or("Terrain row extent overflow")?;
        for stroke in &self.strokes { stroke.sample(1)?; }
        Ok(())
    }
    pub fn into_request(self) -> Result<GeoRasterBrushRequest, String> {
        self.validate()?;
        let mut samples=Vec::new();
        samples.try_reserve_exact(self.strokes.len()).map_err(|_|"Terrain sample allocation refused")?;
        for (index,stroke) in self.strokes.iter().enumerate() { samples.push(stroke.sample(index as u64+1)?); }
        let request=GeoRasterBrushRequest {resource:self.resource,window_x:self.window_x,window_y:self.window_y,window_side:self.window_side,samples};
        request.validate_bounds()?;
        Ok(request)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn draft() -> TerrainBrushDraft { TerrainBrushDraft {
        resource:ResourceInspectionRequest {filename:"terrain.hmc".into(),package_digest:[1;32],key:"height.m64".into(),resource_digest:[2;32]},
        window_x:2,window_y:3,window_side:4,strokes:vec![TerrainStrokeDraft {operation:TerrainOperation::Raise,falloff:TerrainFalloff::Linear,center_m:[1.0,2.0],radius_m:3.0,strength:0.25,target_height_m:None,step_m:None}],
    } }
    #[test] fn operator_projection_preserves_selection_and_sets_only_supported_sample_fields() {
        let value=draft(); let resource=value.resource.clone();
        let request=value.into_request().unwrap();
        assert_eq!(request.resource,resource);assert_eq!((request.window_x,request.window_y,request.window_side),(2,3,4));
        let s=request.samples[0];assert_eq!((s.op_code,s.falloff_code,s.stroke_id),(0,2,1));
        assert_eq!((s.center_x_m,s.center_y_m,s.radius_m,s.strength),(1.0,2.0,3.0,0.25));
        assert_eq!((s.layer_index,s.flags,s.pad),(0,0,[0;2]));
    }
    #[test] fn unknown_fields_and_operations_do_not_become_brush_codes() {
        let mut value=serde_json::to_value(draft()).unwrap();value["authority"]=serde_json::json!("owner");
        assert!(serde_json::from_value::<TerrainBrushDraft>(value).is_err());
        for rejected in ["erode_physical", "relax", "hydraulic_drop"] {
            let mut value=serde_json::to_value(draft()).unwrap();value["strokes"][0]["operation"]=serde_json::json!(rejected);
            assert!(serde_json::from_value::<TerrainBrushDraft>(value).is_err());
            assert!(TerrainOperation::parse(rejected).is_err());
        }
    }
    #[test] fn flatten_and_terrace_require_explicit_finite_parameters_without_ignored_extras() {
        let mut value=draft();value.strokes[0].operation=TerrainOperation::Flatten;
        assert!(value.validate().is_err());value.strokes[0].target_height_m=Some(123.456);assert!(value.validate().is_ok());
        value.strokes[0].step_m=Some(1.0);assert!(value.validate().is_err());
        value.strokes[0].operation=TerrainOperation::Terrace;value.strokes[0].target_height_m=None;assert!(value.validate().is_ok());
        value.strokes[0].step_m=Some(f64::NAN);assert!(value.validate().is_err());
    }
    #[test] fn unbounded_or_nonfinite_drafts_refuse_before_queue_admission() {
        let mut value=draft();value.strokes[0].center_m[0]=f64::NAN;assert!(value.validate().is_err());
        value=draft();value.window_x=usize::MAX;assert!(value.validate().is_err());
        value=draft();value.window_side=65;assert!(value.validate().is_err());
        value=draft();value.strokes=vec![value.strokes[0].clone();65];assert!(value.validate().is_err());
        value=draft();value.strokes[0].radius_m=f64::MAX;assert!(value.validate().is_err());
    }
    #[test] fn metre_projection_preserves_millimetre_owner_and_common_decimal_input() {
        assert!(project_metres_f32(0.1,"test value").is_ok());
        assert!(project_metres_f32(1.0004999,"test value").is_ok());
        // f32 narrows this to just below the owner's 1.0005 m half-mm boundary.
        assert!(project_metres_f32(1.00050001,"test value").is_err());

        let mut value=draft();value.strokes[0].center_m[0]=16384.0006;
        assert!(value.validate().is_err()); // more than half a mm is lost at this magnitude
        value=draft();value.strokes[0].strength=1.00050001;
        assert!(value.validate().is_err()); // raise strength is metres
        value=draft();value.strokes[0].operation=TerrainOperation::Flatten;
        value.strokes[0].strength=0.1;value.strokes[0].target_height_m=Some(1.00050001);
        assert!(value.validate().is_err()); // explicit target is metres too
        value=draft();value.strokes[0].operation=TerrainOperation::Terrace;
        value.strokes[0].strength=0.1;value.strokes[0].step_m=Some(1.00050001);
        assert!(value.validate().is_err()); // step is metres; terrace strength is a weight
        value=draft();value.strokes[0].operation=TerrainOperation::Smooth;
        value.strokes[0].strength=0.1;assert!(value.validate().is_ok());
    }
}

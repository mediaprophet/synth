// Synthetic public QualiaDB field/operator caller tests; not asset acceptance.
mod tests {
    use qualia_core_db::specialized_libs::computational_geometry::tapered_branch_field::*;
    use qualia_core_db::specialized_libs::computational_geometry::{Point3,sdf_capsule};
    fn seg(r0:f64,r1:f64)->TaperedFieldSegment { TaperedFieldSegment {a:Point3::new(0.0,0.0,0.0),b:Point3::new(0.0,0.0,2.0),radius_a:r0,radius_b:r1} }
    #[test]
    fn explicit_endpoint_radii_produce_declared_linear_profile() {
        let points=[Point3::new(0.35,0.0,0.0),Point3::new(0.35,0.0,1.0),Point3::new(0.35,0.0,2.0)];
        let mut out=[99.0;4];let n=tapered_branch_field_into(&points,&[seg(0.4,0.2)],0.0,1e-6,1e-9,3,1,3,&mut out).unwrap();
        assert_eq!(n,3);for (actual,expected) in out[..3].iter().zip([-0.05,0.05,0.15]) {assert!((actual-expected).abs()<1e-12);}
        assert_eq!(out[3],99.0);
    }
    #[test]
    fn constant_radius_matches_existing_capsule_and_blend_is_ordered() {
        let segment=seg(0.5,0.5);let points=[Point3::new(1.0,0.0,1.0),Point3::new(0.0,0.0,2.2),Point3::new(0.0,0.0,-0.2)];let mut out=[0.0;3];
        tapered_branch_field_into(&points,&[segment],0.0,1e-6,1e-9,3,1,3,&mut out).unwrap();
        for (p,value) in points.iter().zip(out) {assert!((value-sdf_capsule(*p,segment.a,segment.b,0.5)).abs()<1e-12);}
        let p=[Point3::new(1.0,0.0,1.0)];let mut forward=[0.0];let mut reverse=[0.0];
        let v=[seg(0.8,0.8),seg(0.9,0.9),seg(1.0,1.0)];let reversed=[v[2],v[1],v[0]];
        tapered_branch_field_into(&p,&v,0.5,1e-6,1e-9,1,3,3,&mut forward).unwrap();
        tapered_branch_field_into(&p,&reversed,0.5,1e-6,1e-9,1,3,3,&mut reverse).unwrap();
        assert!((forward[0]-reverse[0]).abs()>1e-6);
    }
    #[test]
    fn extreme_endpoints_and_distant_union_preserve_representable_values() {
        let mut out=[77.0];
        let long=TaperedFieldSegment {a:Point3::new(1e100,0.0,0.0),b:Point3::new(1.0,0.0,0.0),radius_a:1.0,radius_b:1.0};
        tapered_branch_field_into(&[long.b],&[long],0.0,1e-6,1e90,1,1,1,&mut out).unwrap(); assert_eq!(out,[-1.0]);
        let diagonal=TaperedFieldSegment {a:Point3::new(1e100,1e100,0.0),b:Point3::new(0.0,0.0,0.0),radius_a:0.8,radius_b:0.8};
        tapered_branch_field_into(&[Point3::new(1.0,0.0,0.0)],&[diagonal],0.0,1e-6,1e90,1,1,1,&mut out).unwrap(); assert!((out[0]-(0.5_f64.sqrt()-0.8)).abs()<1e-12);
        let reversed=TaperedFieldSegment {a:diagonal.b,b:diagonal.a,radius_a:0.8,radius_b:0.8};
        tapered_branch_field_into(&[Point3::new(1.0,0.0,0.0)],&[reversed],0.0,1e-6,1e90,1,1,1,&mut out).unwrap(); assert!((out[0]-(0.5_f64.sqrt()-0.8)).abs()<1e-12);
        let tiny=seg(1.0,1e-100);
        tapered_branch_field_into(&[tiny.b],&[tiny],0.0,1e-6,1e90,1,1,1,&mut out).unwrap(); assert_eq!(out,[-1e-100]);
        let local=seg(1.0,1.0);let distant=TaperedFieldSegment {a:Point3::new(1e100,0.0,0.0),b:Point3::new(1e100,0.0,2.0),radius_a:1.0,radius_b:1.0};
        let sample=[Point3::new(0.0,0.0,1.0)];
        for pair in [[local,distant],[distant,local]] {tapered_branch_field_into(&sample,&pair,1.0,1e-6,1e90,1,2,2,&mut out).unwrap();assert_eq!(out,[-1.0]);}
        let near_limit=TaperedFieldSegment {a:Point3::new(5e99,0.0,0.0),b:Point3::new(5e99,0.0,2.0),radius_a:1.0,radius_b:1.0};
        for pair in [[local,near_limit],[near_limit,local]] {tapered_branch_field_into(&sample,&pair,5.000000000000001e99,1e-6,1e90,1,2,2,&mut out).unwrap();assert!(out[0].is_finite() && out[0]< -1.0,"inside blend must not invert the sign: {}",out[0]);}
    }
    #[test]
    fn ill_conditioned_projection_refuses_with_caller_tolerance() {
        let segment=TaperedFieldSegment {a:Point3::new(0.0,0.0,0.0),b:Point3::new(4.0,4.0,0.0),radius_a:0.8,radius_b:0.8};
        let sample=[Point3::new(1e16,-1e16+2.0,0.0)];let mut out=[77.0];
        assert_eq!(tapered_branch_field_into(&sample,&[segment],0.0,1e-6,1e-8,1,1,1,&mut out),Err(TaperedFieldError::NumericPrecision));assert_eq!(out,[77.0]);
        assert_eq!(tapered_branch_field_into(&sample,&[segment],0.0,1e-6,0.0,1,1,1,&mut out),Err(TaperedFieldError::InvalidInput));assert_eq!(out,[77.0]);
    }
    #[test]
    fn subnormal_axes_refuse_before_writes() {
        let tiny=TaperedFieldSegment {a:Point3::new(0.0,0.0,0.0),b:Point3::new(f64::from_bits(1),0.0,0.0),radius_a:1.0,radius_b:1.0};
        let mut out=[77.0];
        assert_eq!(tapered_branch_field_into(&[tiny.a],&[tiny],0.0,f64::from_bits(1),1e-9,1,1,1,&mut out),Err(TaperedFieldError::DegenerateSegment));assert_eq!(out,[77.0]);
    }
    #[test]
    fn nonfinite_input_and_empty_inventories_refuse_without_writes() {
        let points=[Point3::new(0.0,0.0,0.0),Point3::new(f64::NAN,0.0,0.0)];let valid=seg(0.4,0.2);let mut out=[77.0;2];
        assert_eq!(tapered_branch_field_into(&points,&[valid],0.0,1e-6,1e-9,2,1,2,&mut out),Err(TaperedFieldError::InvalidInput));assert_eq!(out,[77.0;2]);
        assert_eq!(tapered_branch_field_into(&points[..0],&[valid],0.0,1e-6,1e-9,2,1,2,&mut out),Err(TaperedFieldError::EmptySamples));assert_eq!(out,[77.0;2]);
        assert_eq!(tapered_branch_field_into(&points[..1],&[],0.0,1e-6,1e-9,2,1,2,&mut out),Err(TaperedFieldError::EmptySegments));assert_eq!(out,[77.0;2]);
        for (blend,min_len,tolerance) in [(f64::INFINITY,1e-6,1e-9),(0.0,f64::NAN,1e-9),(0.0,1e-6,f64::NAN)] {
            assert_eq!(tapered_branch_field_into(&points[..1],&[valid],blend,min_len,tolerance,1,1,1,&mut out),Err(TaperedFieldError::InvalidInput));assert_eq!(out,[77.0;2]);
        }
    }
    #[test]
    fn bad_inputs_work_and_short_output_fail_before_any_write() {
        let points=[Point3::new(0.0,0.0,0.0),Point3::new(0.0,1.0,0.0)];let good=seg(0.4,0.2);
        let mut out=[77.0;2];let mut bad=good;bad.b=bad.a;
        assert_eq!(tapered_branch_field_into(&points,&[good,bad],0.1,1e-6,1e-9,2,2,4,&mut out),Err(TaperedFieldError::DegenerateSegment));assert_eq!(out,[77.0;2]);
        bad=good;bad.radius_b=f64::NAN;
        assert_eq!(tapered_branch_field_into(&points,&[good,bad],0.1,1e-6,1e-9,2,2,4,&mut out),Err(TaperedFieldError::InvalidInput));assert_eq!(out,[77.0;2]);
        assert_eq!(tapered_branch_field_into(&points,&[good],-1.0,1e-6,1e-9,2,1,2,&mut out),Err(TaperedFieldError::InvalidInput));assert_eq!(out,[77.0;2]);
        assert_eq!(tapered_branch_field_into(&points,&[good],0.0,1e-6,1e-9,2,1,1,&mut out),Err(TaperedFieldError::WorkLimitExceeded));assert_eq!(out,[77.0;2]);
        assert_eq!(tapered_branch_field_into(&points,&[good],0.0,1e-6,1e-9,2,1,2,&mut out[..1]),Err(TaperedFieldError::OutputTooSmall));assert_eq!(out,[77.0;2]);
        assert_eq!(tapered_field_work(usize::MAX,2,usize::MAX,usize::MAX,usize::MAX),Err(TaperedFieldError::SizeOverflow));
    }

    #[test]
    fn tapered_branch_field_extracts_one_closed_indexed_surface_for_synthetic_y() {
        use qualia_core_db::specialized_libs::computational_geometry::{HalfEdge,EdgeSlot};
        use qualia_core_db::specialized_libs::computational_geometry::indexed_tetra_surface::{tetra_surface_bounds,indexed_tetra_surface_into,TetraEdgeSlot};
        use qualia_core_db::specialized_libs::computational_geometry::surface_topology_qualification::{surface_qualification_bounds,qualify_surface_topology_into};
        let dims=[15usize,15,15];let origin=Point3::new(-3.57,-3.59,-3.53);let spacing=0.5;
        let center=Point3::new(0.0,0.0,0.0);let arms=[Point3::new(-1.4,0.3,1.2),Point3::new(1.3,0.1,1.3),Point3::new(0.0,-0.2,-1.7)];
        let segments=[TaperedFieldSegment{a:center,b:arms[0],radius_a:0.58,radius_b:0.42},TaperedFieldSegment{a:center,b:arms[1],radius_a:0.58,radius_b:0.36},TaperedFieldSegment{a:center,b:arms[2],radius_a:0.58,radius_b:0.48}];
        let mut samples=Vec::new();for z in 0..dims[2] {for y in 0..dims[1] {for x in 0..dims[0] {samples.push(Point3::new(origin.x+x as f64*spacing,origin.y+y as f64*spacing,origin.z+z as f64*spacing));}}}
        let mut grid=vec![0.0;samples.len()];let count=tapered_branch_field_into(&samples,&segments,0.2,1e-6,1e-9,samples.len(),segments.len(),samples.len()*segments.len(),&mut grid).unwrap();assert_eq!(count,samples.len()*segments.len());
        let b=tetra_surface_bounds(dims[0],dims[1],dims[2],100_000).unwrap();let mut vertices=vec![Point3::default();b.vertices];let mut triangles=vec![[0;3];b.triangles];let mut provenance=vec![[0usize;2];b.vertices];let mut slots=vec![TetraEdgeSlot::default();b.edge_slots];
        let (nv,nt)=indexed_tetra_surface_into(&grid,dims,origin,Point3::new(spacing,spacing,spacing),0.0,100_000,10_000_000,1e-28,&mut vertices,&mut triangles,&mut provenance,&mut slots).unwrap();assert!(nv>0 && nt>0);
        for end in arms {assert!(vertices[..nv].iter().any(|p| (p.x-end.x).powi(2)+(p.y-end.y).powi(2)+(p.z-end.z).powi(2)<1.0));}
        let q=surface_qualification_bounds(nv as u32,nt).unwrap();let mut he=vec![HalfEdge::default();q.half_edges];let mut edges=vec![EdgeSlot::default();q.edge_slots];let mut starts=vec![0;q.vertex_starts];let mut visited=vec![false;q.visited_half_edges];let mut labels=vec![0;q.face_labels];let mut queue=vec![0;q.face_queue];
        let receipt=qualify_surface_topology_into(nv as u32,&triangles[..nt],q.work,&mut he,&mut edges,&mut starts,&mut visited,&mut labels,&mut queue).unwrap();assert_eq!(receipt.component_count,1);
    }
}

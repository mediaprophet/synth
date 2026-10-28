//! Synthetic public QualiaDB caller check; no native UI, asset or GPU claim.
use qualia_core_db::specialized_libs::computational_geometry as cg;

#[test]
fn poet_calls_qualified_caller_buffered_hard_cap_sweep() {
    use cg::hard_cap_sweep::{hard_cap_sweep_counts, hard_cap_sweep_normals_into};
    let axis = [cg::Point3::new(0.0,0.0,0.0),cg::Point3::new(0.0,0.0,1.0),cg::Point3::new(0.2,0.0,2.0)];
    let section = (0..8).map(|i| { let a=std::f64::consts::TAU*i as f64/8.0; cg::Point2::new(a.cos(),a.sin()) }).collect::<Vec<_>>();
    let counts=hard_cap_sweep_counts(axis.len(),section.len()).unwrap();
    let zero=cg::Point3::new(0.0,0.0,0.0);
    let mut frames=vec![cg::AffineFrame3 { origin:zero,e0:zero,e1:zero,e2:zero };counts.frames];
    let mut positions=vec![zero;counts.vertices]; let mut indices=vec![[0u32;3];counts.triangles];
    let mut normals=vec![zero;counts.vertices]; let mut source_vertices=vec![0u32;counts.vertices];
    assert_eq!(hard_cap_sweep_normals_into(&axis,&section,None,&mut frames,&mut positions,&mut indices,&mut normals,&mut source_vertices).unwrap(),counts);
    assert_eq!(counts.vertices,42); assert_eq!(counts.triangles,48);
    for i in 0..8 {
        let bottom=26+i; let top=34+i;
        assert_eq!(source_vertices[bottom],i as u32);
        assert_eq!(source_vertices[top],(16+i) as u32);
        assert_eq!(positions[bottom],positions[i]); assert_eq!(positions[top],positions[16+i]);
        let dot=normals[bottom].x*normals[i].x+normals[bottom].y*normals[i].y+normals[bottom].z*normals[i].z;
        assert!(dot.abs()<0.5,"cap must remain visibly distinct from adjacent wall normal");
    }
}

fn p(x: f64, y: f64, z: f64) -> cg::Point3 { cg::Point3::new(x,y,z) }
fn frame() -> cg::AffineFrame3 { let z=p(0.,0.,0.); cg::AffineFrame3 { origin:z,e0:z,e1:z,e2:z } }
fn square() -> [cg::Point2;4] { [cg::Point2::new(1.,0.),cg::Point2::new(0.,1.),cg::Point2::new(-1.,0.),cg::Point2::new(0.,-1.)] }
fn dot(a: cg::Point3,b: cg::Point3) -> f64 { a.x*b.x+a.y*b.y+a.z*b.z }
fn minus(a: cg::Point3,b: cg::Point3) -> cg::Point3 { p(a.x-b.x,a.y-b.y,a.z-b.z) }

#[test]
fn public_sweep_straight_radial_normals_match_across_stations_and_split_caps() {
    use cg::hard_cap_sweep::{hard_cap_sweep_counts,hard_cap_sweep_normals_into};
    let axis=[p(0.,0.,0.),p(0.,0.,1.),p(0.,0.,2.)]; let section=square();
    let count=hard_cap_sweep_counts(axis.len(),section.len()).unwrap();
    let mut frames=vec![frame();count.frames]; let mut v=vec![p(0.,0.,0.);count.vertices];
    let mut t=vec![[0u32;3];count.triangles]; let mut n=v.clone(); let mut map=vec![0u32;count.vertices];
    hard_cap_sweep_normals_into(&axis,&section,None,&mut frames,&mut v,&mut t,&mut n,&mut map).unwrap();
    assert_eq!((count.vertices,count.triangles),(22,24));
    for i in 0..12 {
        let radial=minus(v[i],frames[i/4].origin);
        assert!(dot(n[i],radial)>0.999999, "normal must be outward radial at vertex {i}");
        assert!((dot(n[i],n[i])-1.).abs()<1e-12);
        let first=n[i%4]; assert!(dot(n[i],first)>0.999999999999, "same angular index must keep its wall normal across stations");
    }
    for i in 0..4 {
        let bottom=14+i; let top=18+i;
        assert_eq!((map[bottom],map[top]),(i as u32,(8+i) as u32));
        assert_eq!(v[bottom],v[i]); assert_eq!(v[top],v[8+i]);
        assert!(dot(n[bottom],frames[0].e2) < -0.999999);
        assert!(dot(n[top],frames[2].e2)>0.999999);
        assert!(dot(n[bottom],n[i]).abs()<1e-12);
        assert!(dot(n[top],n[8+i]).abs()<1e-12);
    }
    assert!(t[..16].iter().all(|tri|tri.iter().all(|i| *i<12)),"wall indices must not share cap rims");
    assert!(t[16..20].iter().all(|tri|tri[0]==12 && (14..18).contains(&tri[1]) && (14..18).contains(&tri[2])));
    assert!(t[20..24].iter().all(|tri|tri[0]==13 && (18..22).contains(&tri[1]) && (18..22).contains(&tri[2])));
}

#[test]
fn public_sweep_rejects_each_insufficient_caller_buffer_without_writing() {
    use cg::hard_cap_sweep::{hard_cap_sweep_counts,hard_cap_sweep_normals_into,HardCapSweepError};
    let axis=[p(0.,0.,0.),p(0.,0.,1.)]; let section=square();
    let count=hard_cap_sweep_counts(axis.len(),section.len()).unwrap();
    let sentinel=p(19.,19.,19.);
    for short in 0..5 {
        let mut frames=vec![frame();count.frames - if short==0 {1} else {0}];
        let mut v=vec![sentinel;count.vertices - if short==1 {1} else {0}];
        let mut t=vec![[77u32;3];count.triangles - if short==2 {1} else {0}];
        let mut n=vec![sentinel;count.vertices - if short==3 {1} else {0}];
        let mut map=vec![99u32;count.vertices - if short==4 {1} else {0}];
        assert!(matches!(hard_cap_sweep_normals_into(&axis,&section,None,&mut frames,&mut v,&mut t,&mut n,&mut map),Err(HardCapSweepError::OutputTooSmall { .. })), "buffer {short}");
        assert!(frames.iter().all(|x| *x==frame()));
        assert!(v.iter().all(|x| *x==sentinel)); assert!(n.iter().all(|x| *x==sentinel));
        assert!(t.iter().all(|x| *x==[77;3])); assert!(map.iter().all(|x| *x==99));
    }
}

#[test]
fn public_sweep_tapered_and_bent_wall_normals_are_finite_and_outward() {
    use cg::hard_cap_sweep::{hard_cap_sweep_counts,hard_cap_sweep_normals_into};
    let axis=[p(0.,0.,0.),p(0.,0.,1.),p(0.3,0.,2.)]; let section=square();
    let stations=[cg::ExtrusionStation { scale_x:1.,scale_y:1.,twist_rad:0. },cg::ExtrusionStation { scale_x:0.8,scale_y:0.8,twist_rad:0. },cg::ExtrusionStation { scale_x:0.6,scale_y:0.6,twist_rad:0. }];
    let count=hard_cap_sweep_counts(axis.len(),section.len()).unwrap();
    let mut frames=vec![frame();count.frames]; let mut v=vec![p(0.,0.,0.);count.vertices];
    let mut t=vec![[0u32;3];count.triangles]; let mut n=v.clone(); let mut map=vec![0u32;count.vertices];
    hard_cap_sweep_normals_into(&axis,&section,Some(&stations),&mut frames,&mut v,&mut t,&mut n,&mut map).unwrap();
    for i in 0..12 {
        let radial=minus(v[i],frames[i/4].origin);
        assert!(n[i].x.is_finite() && n[i].y.is_finite() && n[i].z.is_finite());
        assert!((dot(n[i],n[i])-1.).abs()<1e-12 && dot(n[i],radial)>0.0, "normal must be finite, unit and outward");
        if i>=4 { assert!(dot(n[i],n[i-4])>0.7, "gentle bend should not flip neighboring ring normals"); }
    }
}

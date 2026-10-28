// Public Core caller tests; synthetic fields only.
mod tests {
    use qualia_core_db::specialized_libs::computational_geometry::indexed_tetra_surface::*;
    use qualia_core_db::specialized_libs::computational_geometry::{Point3, sdf_capsule, sdf_smooth_union};
    fn point(x: usize, y: usize, z: usize, o: Point3, s: Point3) -> Point3 { Point3::new(o.x+x as f64*s.x,o.y+y as f64*s.y,o.z+z as f64*s.z) }
    use std::collections::{HashMap, VecDeque};

    fn output(grid: &[f64], dims: [usize;3], origin: Point3, spacing: Point3)
      -> Result<(Vec<Point3>,Vec<[u32;3]>),TetraSurfaceError> {
        let b=tetra_surface_bounds(dims[0],dims[1],dims[2],100_000)?;
        let mut vertices=vec![Point3::default();b.vertices];
        let mut triangles=vec![[0;3];b.triangles];
        let mut slots=vec![TetraEdgeSlot::default();b.edge_slots];
        let mut edge_sources=vec![[0usize;2];b.vertices];
        let (v,t)=indexed_tetra_surface_into(grid,dims,origin,spacing,0.0,100_000,10_000_000,1e-28,&mut vertices,&mut triangles,&mut edge_sources,&mut slots)?;
        vertices.truncate(v);triangles.truncate(t); Ok((vertices,triangles))
    }
    fn check_closed_connected(vertices:&[Point3], triangles:&[[u32;3]]) {
        assert!(!vertices.is_empty() && !triangles.is_empty());
        let mut edges:HashMap<(u32,u32),(usize,i32)>=HashMap::new();
        let mut adj=vec![Vec::new();vertices.len()];
        for &[a,b,c] in triangles {
            assert!(a!=b && b!=c && c!=a);
            for (u,v) in [(a,b),(b,c),(c,a)] {
                let key=(u.min(v),u.max(v)); let e=edges.entry(key).or_insert((0,0));
                e.0+=1; e.1+=if u<v {1} else {-1};
                adj[u as usize].push(v as usize);adj[v as usize].push(u as usize);
            }
        }
        assert!(edges.values().all(|&(count,orientation)| count==2 && orientation==0),"closed oriented edges required");
        let mut visited=vec![false;vertices.len()]; let mut queue=VecDeque::from([0usize]); visited[0]=true;
        while let Some(i)=queue.pop_front(){for &j in &adj[i]{if !visited[j]{visited[j]=true;queue.push_back(j);}}}
        assert!(visited.into_iter().all(|v|v),"one connected indexed surface required for fixture");
    }
    #[test]
    fn shared_edge_sphere_is_closed_connected_and_deterministic() {
        let dims=[13,13,13]; let origin=Point3::new(-3.07,-3.09,-3.03); let step=Point3::new(0.5,0.5,0.5);
        let mut grid=Vec::new();
        for z in 0..dims[2] { for y in 0..dims[1] { for x in 0..dims[0] {
            let p=point(x,y,z,origin,step);grid.push((p.x*p.x+p.y*p.y+p.z*p.z).sqrt()-1.1);
        } } }
        let a=output(&grid,dims,origin,step).unwrap(); let b=output(&grid,dims,origin,step).unwrap();
        assert_eq!(a,b);check_closed_connected(&a.0,&a.1);
    }
    #[test]
    fn blended_three_arm_constant_radius_field_is_indexed_and_closed_for_fixture() {
        let dims=[15,15,15]; let origin=Point3::new(-3.57,-3.59,-3.53);let step=Point3::new(0.5,0.5,0.5);
        let center=Point3::new(0.0,0.0,0.0); let arms=[Point3::new(-1.4,0.3,1.2),Point3::new(1.3,0.1,1.3),Point3::new(0.0,-0.2,-1.7)];
        let mut grid=Vec::new();
        for z in 0..dims[2] { for y in 0..dims[1] { for x in 0..dims[0] {
            let p=point(x,y,z,origin,step);let a=sdf_capsule(p,center,arms[0],0.52);let b=sdf_capsule(p,center,arms[1],0.43);let c=sdf_capsule(p,center,arms[2],0.62);
            grid.push(sdf_smooth_union(sdf_smooth_union(a,b,0.2),c,0.2));
        } } }
        let (vertices,triangles)=output(&grid,dims,origin,step).unwrap();check_closed_connected(&vertices,&triangles);
        for arm in arms { assert!(vertices.iter().any(|p| { let d=(p.x-arm.x).powi(2)+(p.y-arm.y).powi(2)+(p.z-arm.z).powi(2); d<1.0 }),"each synthetic arm must be represented"); }
    }
    #[test]
    fn checked_dimensions_and_short_buffers_refuse_without_writes() {
        assert_eq!(tetra_surface_bounds(usize::MAX,2,2,usize::MAX),Err(TetraSurfaceError::SizeOverflow));
        assert!(tetra_surface_bounds(3,3,3,7).is_err());
        let bounds=tetra_surface_bounds(3,3,3,8).unwrap(); let mut grid=vec![1.0;bounds.samples];grid[13]=-1.0;
        let sentinel=Point3::new(7.0,8.0,9.0);let mut v=vec![sentinel;bounds.vertices-1];let mut t=vec![[1,2,3];bounds.triangles];let mut e=vec![[4usize,5];bounds.vertices];let mut slots=vec![TetraEdgeSlot{a:1,b:2,vertex:3};bounds.edge_slots];
        assert_eq!(indexed_tetra_surface_into(&grid,[3,3,3],Point3::default(),Point3::new(1.0,1.0,1.0),0.0,8,10_000,0.0,&mut v,&mut t,&mut e,&mut slots),Err(TetraSurfaceError::InsufficientBuffer));
        assert!(v.iter().all(|x|*x==sentinel));assert!(t.iter().all(|x|*x==[1,2,3]));assert!(e.iter().all(|x|*x==[4,5]));assert!(slots.iter().all(|x|x.a==1 && x.b==2 && x.vertex==3));
    }
    #[test]
    fn public_extractor_refuses_empty_numerical_collapse_and_probe_exhaustion() {
        let dims=[3,3,3];let b=tetra_surface_bounds(3,3,3,8).unwrap();let origin=Point3::new(1.0,1.0,1.0);let spacing=Point3::new(1.0,1.0,1.0);
        let mut grid=vec![1.0;b.samples];let sentinel=Point3::new(9.0,9.0,9.0);let mut v=vec![sentinel;b.vertices];let mut t=vec![[1,2,3];b.triangles];let mut e=vec![[4usize,5];b.vertices];let mut slots=vec![TetraEdgeSlot{a:1,b:2,vertex:3};b.edge_slots];
        assert_eq!(indexed_tetra_surface_into(&grid,dims,origin,spacing,0.0,8,10_000,0.0,&mut v,&mut t,&mut e,&mut slots),Err(TetraSurfaceError::EmptySurface));
        grid[13]=-1e-30;
        assert_eq!(indexed_tetra_surface_into(&grid,dims,origin,spacing,0.0,8,10_000,0.0,&mut v,&mut t,&mut e,&mut slots),Err(TetraSurfaceError::InvalidGeometry));
        grid[13]=-1.0;
        assert_eq!(indexed_tetra_surface_into(&grid,dims,origin,spacing,0.0,8,0,0.0,&mut v,&mut t,&mut e,&mut slots),Err(TetraSurfaceError::WorkLimitExceeded));
        assert_eq!(indexed_tetra_surface_into(&grid,dims,origin,spacing,0.0,8,1,0.0,&mut v,&mut t,&mut e,&mut slots),Err(TetraSurfaceError::WorkLimitExceeded));
    }
    #[test]
    fn public_nonfinite_inputs_refuse_without_touching_caller_storage() {
        let bounds=tetra_surface_bounds(3,3,3,8).unwrap();let dims=[3,3,3];let mut grid=vec![1.0;bounds.samples];grid[13]=-1.0;
        let sentinel=Point3::new(7.0,8.0,9.0);let mut v=vec![sentinel;bounds.vertices];let mut t=vec![[1,2,3];bounds.triangles];let mut e=vec![[4usize,5];bounds.vertices];let mut slots=vec![TetraEdgeSlot{a:1,b:2,vertex:3};bounds.edge_slots];
        let one=Point3::new(1.0,1.0,1.0);let origin=Point3::default();
        for (o,sp,iso) in [(Point3::new(f64::NAN,0.0,0.0),one,0.0),(Point3::new(f64::INFINITY,0.0,0.0),one,0.0),(origin,Point3::new(0.0,1.0,1.0),0.0),(origin,Point3::new(-1.0,1.0,1.0),0.0),(origin,Point3::new(f64::INFINITY,1.0,1.0),0.0),(origin,one,f64::NAN),(origin,one,f64::INFINITY),(origin,one,f64::NEG_INFINITY)] {
            assert_eq!(indexed_tetra_surface_into(&grid,dims,o,sp,iso,8,10_000,0.0,&mut v,&mut t,&mut e,&mut slots),Err(TetraSurfaceError::InvalidGeometry));
            assert!(v.iter().all(|x|*x==sentinel));assert!(t.iter().all(|x|*x==[1,2,3]));assert!(e.iter().all(|x|*x==[4,5]));assert!(slots.iter().all(|x|x.a==1 && x.b==2 && x.vertex==3));
        }
        for bad in [f64::NAN,f64::INFINITY,f64::NEG_INFINITY] {
            grid[13]=bad;assert_eq!(indexed_tetra_surface_into(&grid,dims,origin,one,0.0,8,10_000,0.0,&mut v,&mut t,&mut e,&mut slots),Err(TetraSurfaceError::NonFiniteSample));
            assert!(v.iter().all(|x|*x==sentinel));assert!(t.iter().all(|x|*x==[1,2,3]));assert!(e.iter().all(|x|*x==[4,5]));assert!(slots.iter().all(|x|x.a==1 && x.b==2 && x.vertex==3));
        }
    }
    #[test]
    fn exact_zero_and_inside_boundary_refuse_before_touching_outputs() {
        let dims=[3,3,3]; let b=tetra_surface_bounds(3,3,3,8).unwrap(); let mut grid=vec![1.0;b.samples];
        let sentinel=Point3::new(7.0,8.0,9.0);let mut vertices=vec![sentinel;b.vertices];let mut tris=vec![[7,8,9];b.triangles];let mut slots=vec![TetraEdgeSlot{a:1,b:2,vertex:3};b.edge_slots]; let mut edge_sources=vec![[4usize,5];b.vertices];
        grid[13]=0.0;
        assert_eq!(indexed_tetra_surface_into(&grid,dims,Point3::default(),Point3::new(1.0,1.0,1.0),0.0,8,10_000,0.0,&mut vertices,&mut tris,&mut edge_sources,&mut slots),Err(TetraSurfaceError::SampleOnSurface));
        assert!(vertices.iter().all(|v|*v==sentinel));assert!(tris.iter().all(|t|*t==[7,8,9]));assert!(slots.iter().all(|s|s.a==1 && s.b==2 && s.vertex==3)); assert!(edge_sources.iter().all(|v|*v==[4,5]));
        grid[13]=-1.0;grid[0]=-1.0;
        assert_eq!(indexed_tetra_surface_into(&grid,dims,Point3::default(),Point3::new(1.0,1.0,1.0),0.0,8,10_000,0.0,&mut vertices,&mut tris,&mut edge_sources,&mut slots),Err(TetraSurfaceError::SurfaceTouchesBoundary));
        assert!(vertices.iter().all(|v|*v==sentinel));
    }
}

// Synthetic public QualiaDB caller diagnostics, not asset acceptance.
mod tests {
    use qualia_core_db::specialized_libs::computational_geometry::surface_topology_qualification::*;
    use qualia_core_db::specialized_libs::computational_geometry::{EdgeSlot, HalfEdge};

    fn qualify(vertex_count:u32,triangles:&[[u32;3]]) -> Result<SurfaceQualificationReceipt,SurfaceQualificationError> {
        let b=surface_qualification_bounds(vertex_count,triangles.len()).unwrap();
        let mut edges=vec![HalfEdge::default();b.half_edges];
        let mut slots=vec![EdgeSlot::default();b.edge_slots];
        let mut starts=vec![0;b.vertex_starts]; let mut visited=vec![false;b.visited_half_edges];
        let mut labels=vec![0;b.face_labels]; let mut queue=vec![0;b.face_queue];
        qualify_surface_topology_into(vertex_count,triangles,b.work,&mut edges,&mut slots,&mut starts,&mut visited,&mut labels,&mut queue)
    }
    const TET:[[u32;3];4]=[[0,2,1],[0,1,3],[1,2,3],[2,0,3]];
    const ICOS:[[u32;3];20]=[
        [0,11,5],[0,5,1],[0,1,7],[0,7,10],[0,10,11],
        [1,5,9],[5,11,4],[11,10,2],[10,7,6],[7,1,8],
        [3,9,4],[3,4,2],[3,2,6],[3,6,8],[3,8,9],
        [4,9,5],[2,4,11],[6,2,10],[8,6,7],[9,8,1],
    ];
    #[test]
    fn qualifies_closed_tetrahedron_and_icosahedron() {
        let a=qualify(4,&TET).unwrap();assert_eq!(a.component_count,1);assert_eq!(a.half_edge_count,12);
        let b=qualify(12,&ICOS).unwrap();assert_eq!(b.face_count,20);assert_eq!(b.component_count,1);
    }
    #[test]
    fn refuses_open_and_disconnected_closed_surfaces() {
        assert!(matches!(qualify(4,&TET[..3]),Err(SurfaceQualificationError::BoundaryEdges(_))));
        let mut pair=TET.to_vec(); pair.extend(TET.iter().map(|t|t.map(|v|v+4)));
        assert_eq!(qualify(8,&pair),Err(SurfaceQualificationError::Disconnected(2)));
        assert_eq!(qualify(5,&TET),Err(SurfaceQualificationError::UnusedVertex(4)));
    }
    #[test]
    fn refuses_closed_connected_edges_with_two_disjoint_vertex_fans() {
        // Icosahedron opposite vertices 0 and 3 share no edge/neighbour.
        // Identify them and compact the remaining indices: edges still pair
        // and faces remain connected, but vertex 0 has two distinct fans.
        let pinched:Vec<[u32;3]>=ICOS.iter().map(|t|t.map(|v|if v==3 {0} else if v>3 {v-1} else {v})).collect();
        assert_eq!(qualify(11,&pinched),Err(SurfaceQualificationError::InvalidVertexLink(0)));
    }
    #[test]
    fn exhausted_work_and_short_buffers_refuse_before_scratch_writes() {
        let b=surface_qualification_bounds(4,TET.len()).unwrap();
        let sentinel=HalfEdge{origin:9,twin:9,next:9,face:9};
        let mut edges=vec![sentinel;b.half_edges];let mut slots=vec![EdgeSlot{key:9,half_edge:9,_reserved:9};b.edge_slots];
        let mut starts=vec![9;b.vertex_starts];let mut visited=vec![true;b.visited_half_edges];let mut labels=vec![9;b.face_labels];let mut queue=vec![9;b.face_queue];
        assert_eq!(qualify_surface_topology_into(4,&TET,b.work-1,&mut edges,&mut slots,&mut starts,&mut visited,&mut labels,&mut queue),Err(SurfaceQualificationError::WorkLimitExceeded{required:b.work}));
        assert!(edges.iter().all(|v|*v==sentinel));assert!(slots.iter().all(|v|v.key==9 && v.half_edge==9 && v._reserved==9));
        assert!(starts.iter().all(|v|*v==9));assert!(visited.iter().all(|v|*v));assert!(labels.iter().all(|v|*v==9));assert!(queue.iter().all(|v|*v==9));
        assert_eq!(qualify_surface_topology_into(4,&TET,b.work,&mut edges[..b.half_edges-1],&mut slots,&mut starts,&mut visited,&mut labels,&mut queue),Err(SurfaceQualificationError::InsufficientScratch));
        assert!(edges.iter().all(|v|*v==sentinel));
    }
    #[test]
    fn checked_bounds_refuse_empty_and_oversized_inputs() {
        assert_eq!(surface_qualification_bounds(0,4),Err(SurfaceQualificationError::EmptyMesh));
        assert_eq!(surface_qualification_bounds(4,0),Err(SurfaceQualificationError::EmptyMesh));
        assert_eq!(surface_qualification_bounds(4,usize::MAX),Err(SurfaceQualificationError::SizeOverflow));
    }

    #[test]
    fn qualifies_indexed_tetra_three_arm_field() {
        use qualia_core_db::specialized_libs::computational_geometry::{Point3,sdf_capsule,sdf_smooth_union};
        use qualia_core_db::specialized_libs::computational_geometry::indexed_tetra_surface::{tetra_surface_bounds,indexed_tetra_surface_into,TetraEdgeSlot};
        let dims=[15usize,15,15];let origin=Point3::new(-3.57,-3.59,-3.53);let spacing=Point3::new(0.5,0.5,0.5);
        let center=Point3::new(0.0,0.0,0.0);let arms=[Point3::new(-1.4,0.3,1.2),Point3::new(1.3,0.1,1.3),Point3::new(0.0,-0.2,-1.7)];
        let mut grid=Vec::new();
        for z in 0..dims[2] { for y in 0..dims[1] { for x in 0..dims[0] {
            let p=Point3::new(origin.x+x as f64*spacing.x,origin.y+y as f64*spacing.y,origin.z+z as f64*spacing.z);
            let a=sdf_capsule(p,center,arms[0],0.52);let b=sdf_capsule(p,center,arms[1],0.43);let c=sdf_capsule(p,center,arms[2],0.62);
            grid.push(sdf_smooth_union(sdf_smooth_union(a,b,0.2),c,0.2));
        } } }
        let bound=tetra_surface_bounds(dims[0],dims[1],dims[2],100_000).unwrap();
        let mut v=vec![Point3::default();bound.vertices];let mut t=vec![[0;3];bound.triangles];let mut provenance=vec![[0usize;2];bound.vertices];let mut slots=vec![TetraEdgeSlot::default();bound.edge_slots];
        let (nv,nt)=indexed_tetra_surface_into(&grid,dims,origin,spacing,0.0,100_000,10_000_000,1e-28,&mut v,&mut t,&mut provenance,&mut slots).unwrap();
        assert!(nv>0 && nt>0);let receipt=qualify(nv as u32,&t[..nt]).unwrap();assert_eq!(receipt.component_count,1);
        let qb=surface_qualification_bounds(nv as u32,nt).unwrap();
        let mut he=vec![HalfEdge::default();qb.half_edges]; let mut es=vec![EdgeSlot::default();qb.edge_slots];
        let mut starts=vec![0;qb.vertex_starts];let mut visited=vec![false;qb.visited_half_edges];let mut labels=vec![0;qb.face_labels];let mut queue=vec![0;qb.face_queue];
        let mut remaining=2_000_000usize;
        let actual=qualify_surface_topology_budgeted_into(nv as u32,&t[..nt],&mut remaining,&mut he,&mut es,&mut starts,&mut visited,&mut labels,&mut queue).unwrap();
        assert_eq!(actual,receipt);assert!(remaining<2_000_000);assert!(qb.work>2_000_000);

    }
}

mod triangle_ordered_connectivity_tests {
    use qualia_core_db::specialized_libs::computational_geometry::{HalfEdge,EdgeSlot,INVALID_INDEX,build_triangle_half_edges,required_edge_slots,label_components,label_components_triangle_ordered,triangle_ordered_component_work,OrderedConnectivityError};
    const TET:[[u32;3];4]=[[0,2,1],[0,1,3],[1,2,3],[2,0,3]];
    fn fixture() -> Vec<HalfEdge> {
        let mut edges=vec![HalfEdge::default();12]; let mut slots=vec![EdgeSlot::default();required_edge_slots(4)];
        build_triangle_half_edges(4,&TET,&mut edges,&mut slots).unwrap();edges
    }
    #[test]
    fn ordered_bfs_matches_general_owner_for_connected_and_disconnected_meshes() {
        let mut tris=TET.to_vec();tris.extend(TET.iter().map(|t|t.map(|v|v+4)));
        let mut edges=vec![HalfEdge::default();tris.len()*3];let mut slots=vec![EdgeSlot::default();required_edge_slots(tris.len())];
        build_triangle_half_edges(8,&tris,&mut edges,&mut slots).unwrap();
        let mut a=vec![0;tris.len()];let mut aq=a.clone();let mut b=a.clone();let mut bq=a.clone();
        let general=label_components(tris.len() as u32,&edges,&mut a,&mut aq).unwrap();
        let ordered=label_components_triangle_ordered(tris.len() as u32,&edges,triangle_ordered_component_work(tris.len() as u32).unwrap(),&mut b,&mut bq).unwrap();
        assert_eq!(general,2);assert_eq!(ordered,general);assert_eq!(a,b);
    }
    #[test]
    fn malformed_layout_and_insufficient_work_refuse_without_scratch_mutation() {
        let mut edges=fixture();let mut labels=vec![77;4];let mut queue=vec![88;4];
        let work=triangle_ordered_component_work(4).unwrap();
        assert_eq!(label_components_triangle_ordered(4,&edges,work-1,&mut labels,&mut queue),Err(OrderedConnectivityError::WorkLimitExceeded{required:work}));
        assert_eq!(labels,vec![77;4]);assert_eq!(queue,vec![88;4]);
        edges[0].next=0;
        assert_eq!(label_components_triangle_ordered(4,&edges,work,&mut labels,&mut queue),Err(OrderedConnectivityError::InvalidLayout{index:0}));
        assert_eq!(labels,vec![77;4]);assert_eq!(queue,vec![88;4]);
        let mut edges=fixture();edges[0].twin=99;
        assert_eq!(label_components_triangle_ordered(4,&edges,work,&mut labels,&mut queue),Err(OrderedConnectivityError::InvalidLayout{index:0}));
        assert_eq!(labels,vec![77;4]);assert_eq!(queue,vec![88;4]);
        let mut edges=fixture();let twin=edges[0].twin as usize;edges[twin].origin=99;
        assert_eq!(label_components_triangle_ordered(4,&edges,work,&mut labels,&mut queue),Err(OrderedConnectivityError::InvalidLayout{index:0}));
        assert_eq!(labels,vec![77;4]);assert_eq!(queue,vec![88;4]);
        let edges=fixture();
        assert_eq!(label_components_triangle_ordered(4,&edges,work,&mut labels[..3],&mut queue),Err(OrderedConnectivityError::LabelBufferTooSmall{required:4}));
        assert_eq!(labels,vec![77;4]);assert_eq!(queue,vec![88;4]);
        assert_eq!(label_components_triangle_ordered(4,&edges,work,&mut labels,&mut queue[..3]),Err(OrderedConnectivityError::QueueBufferTooSmall{required:4}));
        assert_eq!(labels,vec![77;4]);assert_eq!(queue,vec![88;4]);
    }
    #[test]
    fn old_general_api_still_accepts_non_contiguous_order() {
        let edges=[HalfEdge{origin:0,twin:INVALID_INDEX,next:2,face:0},HalfEdge{origin:2,twin:INVALID_INDEX,next:0,face:0},HalfEdge{origin:1,twin:INVALID_INDEX,next:1,face:0}];
        let mut labels=[77];let mut queue=[88];
        assert_eq!(label_components(1,&edges,&mut labels,&mut queue).unwrap(),1);
        labels=[77];queue=[88];
        assert_eq!(label_components_triangle_ordered(1,&edges,triangle_ordered_component_work(1).unwrap(),&mut labels,&mut queue),Err(OrderedConnectivityError::InvalidLayout{index:0}));
        assert_eq!(labels,[77]);assert_eq!(queue,[88]);
    }
}

#[cfg(test)]
mod budgeted_topology_public_tests {
 use qualia_core_db::specialized_libs::computational_geometry::{build_triangle_half_edges_budgeted,required_edge_slots,EdgeSlot,HalfEdge,BudgetedTopologyError};
 use qualia_core_db::specialized_libs::computational_geometry::surface_topology_qualification::*;
 const TET:[[u32;3];4]=[[0,2,1],[0,1,3],[1,2,3],[2,0,3]];
 fn hash(key:u64)->usize {let mut x=key;x^=x>>30;x=x.wrapping_mul(0xbf58_476d_1ce4_e5b9);x^=x>>27;x=x.wrapping_mul(0x94d0_49bb_1331_11eb);(x^(x>>31)) as usize}
 #[test] fn adversarial_collisions_consume_budget_and_refuse() {
   let mask=required_edge_slots(16)-1;
   let bs:Vec<u32>=(1..10_000).filter(|i|hash(*i as u64)&mask==0).take(16).collect();assert_eq!(bs.len(),16);
   let tris:Vec<[u32;3]>=bs.iter().map(|&i|[0,i,i+10_000]).collect();
   let mut edges=vec![HalfEdge::default();48];let mut slots=vec![EdgeSlot::default();mask+1];
   let mut budget=96;assert_eq!(build_triangle_half_edges_budgeted(20_000,&tris,&mut edges,&mut slots,&mut budget),Err(BudgetedTopologyError::WorkLimitExceeded));assert_eq!(budget,0);
   let mut budget=100_000;let result=build_triangle_half_edges_budgeted(20_000,&tris,&mut edges,&mut slots,&mut budget).unwrap();assert_eq!(result.face_count,16);assert!(100_000-budget>96);
 }
 #[test] fn qualification_preflight_and_late_probe_exhaustion_fail_closed() {
  let b=surface_qualification_bounds(4,4).unwrap();let fixed=b.work-b.half_edges*2*b.edge_slots;let s=HalfEdge{origin:9,twin:9,next:9,face:9};
  let mut edges=vec![s;b.half_edges];let mut slots=vec![EdgeSlot{key:9,half_edge:9,_reserved:9};b.edge_slots];let mut starts=vec![9;b.vertex_starts];let mut vis=vec![true;b.visited_half_edges];let mut labels=vec![9;b.face_labels];let mut queue=vec![9;b.face_queue];
  let mut budget=fixed;assert_eq!(qualify_surface_topology_budgeted_into(4,&TET,&mut budget,&mut edges,&mut slots,&mut starts,&mut vis,&mut labels,&mut queue),Err(BudgetedSurfaceQualificationError::Qualification(SurfaceQualificationError::WorkLimitExceeded{required:fixed+1})));assert_eq!(budget,fixed);assert!(edges.iter().all(|e|*e==s));assert!(slots.iter().all(|e|e.key==9));
  let mut budget=fixed+1;assert_eq!(qualify_surface_topology_budgeted_into(4,&TET,&mut budget,&mut edges,&mut slots,&mut starts,&mut vis,&mut labels,&mut queue),Err(BudgetedSurfaceQualificationError::TopologyBuild(BudgetedTopologyError::WorkLimitExceeded)));assert_eq!(budget,0);
 }
}

// Include from a Poet graphical test module to exercise the public Core owner.
#[test]
fn qualiacore_merge_vertices_reuses_compacted_representatives() {
    use qualia_core_db::specialized_libs::computational_geometry::merge_vertices;
    // Input representatives arrive interleaved with duplicates. Before the
    // owner fix, the second B is compared against original slot 1 (A) and
    // becomes an incorrect third representative.
    let positions = [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 4.0, 1.0, 2.0, 4.0, 1.0, 2.0];
    let mut remap = [u32::MAX; 4];
    let mut merged = [0.0; 12];
    let count = merge_vertices(&positions, 4, 0.0, &mut remap, &mut merged).unwrap();
    assert_eq!(count, 2);
    assert_eq!(remap, [0, 0, 1, 1]);
    assert_eq!(&merged[..6], &[0.0, 0.0, 0.0, 4.0, 1.0, 2.0]);
}

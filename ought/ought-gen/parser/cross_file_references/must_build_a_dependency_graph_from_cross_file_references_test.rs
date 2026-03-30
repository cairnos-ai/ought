/// MUST build a dependency graph from cross-file references
#[test]
fn test_parser__cross_file_references__must_build_a_dependency_graph_from_cross_file_references() {
    use std::time::{SystemTime, UNIX_EPOCH};
    use ought_spec::SpecGraph;

    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let tmp = std::env::temp_dir().join(format!("ought_xref_graph_{nanos}"));
    std::fs::create_dir_all(&tmp).unwrap();

    // spec_b is the leaf — no requires of its own
    std::fs::write(
        tmp.join("spec_b.ought.md"),
        "# SpecB\n\n## Rules\n\n- **MUST** provide base data\n",
    )
    .unwrap();

    // spec_a depends on spec_b via a requires: link
    std::fs::write(
        tmp.join("spec_a.ought.md"),
        "# SpecA\n\nrequires: [SpecB](spec_b.ought.md)\n\n## Rules\n\n- **MUST** use SpecB data\n",
    )
    .unwrap();

    let graph =
        SpecGraph::from_roots(&[tmp.clone()]).expect("graph must build successfully with no cycles");

    assert_eq!(
        graph.specs().len(),
        2,
        "graph must contain all discovered spec files"
    );

    let order = graph.topological_order();
    assert_eq!(
        order.len(),
        2,
        "topological order must include every spec in the graph"
    );

    let pos_a = order
        .iter()
        .position(|s| s.name == "SpecA")
        .expect("SpecA must appear in topological order");
    let pos_b = order
        .iter()
        .position(|s| s.name == "SpecB")
        .expect("SpecB must appear in topological order");

    assert!(
        pos_b < pos_a,
        "dependency SpecB must appear before dependent SpecA in topological order \
         (got pos_b={pos_b}, pos_a={pos_a})"
    );

    let _ = std::fs::remove_dir_all(&tmp);
}
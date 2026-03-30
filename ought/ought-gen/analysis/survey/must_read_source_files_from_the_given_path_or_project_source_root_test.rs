```rust
/// read source files from the given path (or project source roots if no path given)
#[test]
fn must_read_source_files_from_the_given_path_or_project_source_root() {
    use std::fs;
    use std::path::PathBuf;

    use ought_analysis::survey::survey;
    use ought_gen::context::GenerationContext;
    use ought_gen::{GeneratedTest, Generator};
    use ought_spec::{Clause, SpecGraph};

    struct StubGenerator;
    impl Generator for StubGenerator {
        fn generate(
            &self,
            _clause: &Clause,
            _ctx: &GenerationContext,
        ) -> anyhow::Result<GeneratedTest> {
            unimplemented!("stub")
        }
    }

    // Use a unique temp directory to avoid cross-test interference.
    let base = std::env::temp_dir().join(format!(
        "ought_survey_path_test_{}",
        std::process::id()
    ));
    let src_dir = base.join("src");
    let alt_dir = base.join("other");
    let spec_dir = base.join("specs");
    fs::create_dir_all(&src_dir).unwrap();
    fs::create_dir_all(&alt_dir).unwrap();
    fs::create_dir_all(&spec_dir).unwrap();

    // Source file in the explicit path.
    fs::write(
        src_dir.join("lib.rs"),
        "pub fn multiply(a: i32, b: i32) -> i32 { a * b }\n",
    )
    .unwrap();

    // Source file outside the explicit path (should NOT be read when path is given).
    fs::write(
        alt_dir.join("other.rs"),
        "pub fn subtract(a: i32, b: i32) -> i32 { a - b }\n",
    )
    .unwrap();

    // Spec whose `source:` metadata points to src_dir (used as project source root
    // when no explicit path is supplied to survey).
    let spec_content = format!(
        "# Arithmetic\n\nsource: {src}\n\n## Operations\n\n- **MUST** handle overflow\n",
        src = src_dir.display()
    );
    fs::write(spec_dir.join("arithmetic.ought.md"), &spec_content).unwrap();

    let spec_graph =
        SpecGraph::from_roots(&[spec_dir.clone()]).expect("spec graph should parse");

    // --- Case 1: explicit path given ---
    // survey must read source files from the supplied path.
    let result = survey(&spec_graph, &[src_dir.clone()], &StubGenerator);
    assert!(
        result.is_ok(),
        "survey should succeed when given a valid explicit source path"
    );
    let survey_result = result.unwrap();
    // Every reported uncovered behavior must originate from the given path.
    for behavior in &survey_result.uncovered {
        assert!(
            behavior.file.starts_with(&src_dir),
            "uncovered behavior file {:?} must be within the supplied path {:?}",
            behavior.file,
            src_dir
        );
    }
    // The file in alt_dir must not appear (it was outside the given path).
    let alt_file = alt_dir.join("other.rs");
    assert!(
        !survey_result.uncovered.iter().any(|b| b.file == alt_file),
        "survey must not read source files outside the given path"
    );

    // --- Case 2: no explicit path given (empty slice) ---
    // survey must fall back to the project source roots (spec `source:` metadata).
    let result_default = survey(&spec_graph, &[], &StubGenerator);
    assert!(
        result_default.is_ok(),
        "survey should succeed when falling back to project source roots"
    );
    let survey_result_default = result_default.unwrap();
    // Project source roots include src_dir (via spec metadata), so behaviors from
    // src_dir may appear, but behaviors from alt_dir must not.
    assert!(
        !survey_result_default
            .uncovered
            .iter()
            .any(|b| b.file.starts_with(&alt_dir)),
        "survey must not read files outside project source roots when no path is given"
    );

    // Cleanup.
    let _ = fs::remove_dir_all(&base);
}
```
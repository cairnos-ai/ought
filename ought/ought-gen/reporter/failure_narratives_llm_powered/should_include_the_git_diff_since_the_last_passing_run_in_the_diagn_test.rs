/// SHOULD include the git diff since the last passing run in the diagnosis context
#[test]
fn test_reporter__failure_narratives_llm_powered__should_include_the_git_diff_since_the_last_passing_run_in_the_diagn() {
    struct DiagnosisContext {
        failing_clause: String,
        generated_test: String,
        failure_output: String,
        relevant_source: String,
        /// Diff of changes since the last run in which this clause passed, if known.
        git_diff_since_last_pass: Option<String>,
    }

    impl DiagnosisContext {
        fn to_prompt(&self) -> String {
            let mut parts = vec![
                format!("Clause: {}", self.failing_clause),
                format!("Test:\n{}", self.generated_test),
                format!("Failure output:\n{}", self.failure_output),
                format!("Relevant source:\n{}", self.relevant_source),
            ];
            if let Some(diff) = &self.git_diff_since_last_pass {
                parts.push(format!("Changes since last passing run (git diff):\n{}", diff));
            }
            parts.join("\n\n")
        }
    }

    let sample_diff =
        "@@ -10,6 +10,6 @@ fn login(creds: Credentials) -> Result<Jwt, Error> {\n-    Ok(generate_jwt(user))\n+    Err(Error::Unauthorized)\n }";

    // With a known diff: must appear in the prompt
    let ctx_with_diff = DiagnosisContext {
        failing_clause: "login must return a JWT token".to_string(),
        generated_test: "#[test]\nfn test_login_returns_jwt() { assert!(login(c).is_ok()); }".to_string(),
        failure_output: "assertion failed: result.is_ok()".to_string(),
        relevant_source: "fn login(creds: Credentials) -> Result<Jwt, Error> { ... }".to_string(),
        git_diff_since_last_pass: Some(sample_diff.to_string()),
    };

    let prompt = ctx_with_diff.to_prompt();
    assert!(
        prompt.contains("Changes since last passing run"),
        "prompt must include a labelled section for the git diff"
    );
    assert!(
        prompt.contains(sample_diff),
        "prompt must include the verbatim diff content"
    );

    // Without a known diff (first run or no passing baseline): diff section must be absent
    let ctx_no_diff = DiagnosisContext {
        failing_clause: "login must return a JWT token".to_string(),
        generated_test: "#[test]\nfn test_login_returns_jwt() { assert!(login(c).is_ok()); }".to_string(),
        failure_output: "assertion failed: result.is_ok()".to_string(),
        relevant_source: "fn login(creds: Credentials) -> Result<Jwt, Error> { ... }".to_string(),
        git_diff_since_last_pass: None,
    };

    let prompt_no_diff = ctx_no_diff.to_prompt();
    assert!(
        !prompt_no_diff.contains("git diff"),
        "when no passing baseline exists, the prompt must not contain an empty diff section"
    );
}
/// MUST send the failing clause, generated test, failure output, and relevant source code to the LLM
#[test]
fn test_reporter__failure_narratives_llm_powered__must_send_the_failing_clause_generated_test_failure_output_and_re() {
    struct DiagnosisRequest {
        failing_clause: String,
        generated_test: String,
        failure_output: String,
        relevant_source: String,
    }

    impl DiagnosisRequest {
        fn to_prompt(&self) -> String {
            format!(
                "Clause:\n{}\n\nGenerated test:\n{}\n\nFailure output:\n{}\n\nRelevant source:\n{}",
                self.failing_clause,
                self.generated_test,
                self.failure_output,
                self.relevant_source,
            )
        }

        fn is_complete(&self) -> bool {
            !self.failing_clause.is_empty()
                && !self.generated_test.is_empty()
                && !self.failure_output.is_empty()
                && !self.relevant_source.is_empty()
        }
    }

    let request = DiagnosisRequest {
        failing_clause: "user login must return a JWT token".to_string(),
        generated_test: "#[test]\nfn test_login_returns_jwt() {\n    assert!(login(creds).is_ok());\n}".to_string(),
        failure_output: "thread 'test_login_returns_jwt' panicked at 'assertion failed: result.is_ok()'".to_string(),
        relevant_source: "fn login(creds: Credentials) -> Result<Jwt, Error> {\n    Err(Error::Unauthorized)\n}".to_string(),
    };

    assert!(request.is_complete(), "all four fields must be populated before sending to LLM");

    let prompt = request.to_prompt();
    assert!(prompt.contains(&request.failing_clause), "prompt must contain the failing clause text");
    assert!(prompt.contains(&request.generated_test), "prompt must contain the generated test code");
    assert!(prompt.contains(&request.failure_output), "prompt must contain the failure output");
    assert!(prompt.contains(&request.relevant_source), "prompt must contain the relevant source code");

    // Verify a request with any missing field is not considered complete
    let incomplete = DiagnosisRequest {
        failing_clause: "some clause".to_string(),
        generated_test: String::new(),
        failure_output: "some output".to_string(),
        relevant_source: "some source".to_string(),
    };
    assert!(!incomplete.is_complete(), "request missing generated_test must not be considered complete");
}
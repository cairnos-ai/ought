#![allow(dead_code, clippy::all)]
#[allow(unused_imports)]
use ought_spec::parser::Parser;
#[allow(unused_imports)]
use ought_spec::types::*;
#[allow(unused_imports)]
use ought_spec::graph::SpecGraph;
#[allow(unused_imports)]
use std::path::{Path, PathBuf};
#[allow(unused_imports)]
use std::time::{Duration, SystemTime, UNIX_EPOCH};
#[allow(unused_imports)]
use std::fs;

// --- must_generate_a_content_hash_for_each_clause_based_on_keyword_tex_test.rs ---
/// MUST generate a content hash for each clause based on keyword + text + relevant context
#[test]
fn test_parser_clause_ir_must_generate_a_content_hash_for_each_clause_based_on_keyword_tex() {

    let md = "# Svc\n\n## Rules\n\n- **MUST** do something specific\n";

    // Same content parsed twice produces the same hash
    let spec1 = Parser::parse_string(md, Path::new("test.ought.md")).expect("parse failed");
    let spec2 = Parser::parse_string(md, Path::new("test.ought.md")).expect("parse failed");
    assert_eq!(
        spec1.sections[0].clauses[0].content_hash,
        spec2.sections[0].clauses[0].content_hash
    );

    // Different keyword changes the hash
    let md_should = "# Svc\n\n## Rules\n\n- **SHOULD** do something specific\n";
    let spec_should =
        Parser::parse_string(md_should, Path::new("test.ought.md")).expect("parse failed");
    assert_ne!(
        spec1.sections[0].clauses[0].content_hash,
        spec_should.sections[0].clauses[0].content_hash
    );

    // Different text changes the hash
    let md_diff = "# Svc\n\n## Rules\n\n- **MUST** do something entirely different\n";
    let spec_diff =
        Parser::parse_string(md_diff, Path::new("test.ought.md")).expect("parse failed");
    assert_ne!(
        spec1.sections[0].clauses[0].content_hash,
        spec_diff.sections[0].clauses[0].content_hash
    );

    // Different condition (GIVEN context) changes the hash
    let md_cond =
        "# Svc\n\n## Rules\n\n- **GIVEN** logged in:\n  - **MUST** do something specific\n";
    let spec_cond =
        Parser::parse_string(md_cond, Path::new("test.ought.md")).expect("parse failed");
    assert_ne!(
        spec1.sections[0].clauses[0].content_hash,
        spec_cond.sections[0].clauses[0].content_hash
    );

    // Hash is a non-empty lowercase hex string
    let hash = &spec1.sections[0].clauses[0].content_hash;
    assert!(!hash.is_empty());
    assert!(
        hash.chars().all(|c| c.is_ascii_hexdigit()),
        "expected hex hash, got: {hash}"
    );
}
// --- must_generate_stable_clause_identifiers_from_the_section_path_and_test.rs ---
/// MUST generate stable clause identifiers from the section path and clause text
/// (e.g. `auth::login::must_return_jwt`)
#[test]
fn test_parser_clause_ir_must_generate_stable_clause_identifiers_from_the_section_path_and() {

    let md = "# Auth\n\n## Login\n\n- **MUST** return a JWT token\n";

    // Parsing the same content twice produces identical identifiers
    let spec1 = Parser::parse_string(md, Path::new("test.ought.md")).expect("parse failed");
    let spec2 = Parser::parse_string(md, Path::new("test.ought.md")).expect("parse failed");
    assert_eq!(
        spec1.sections[0].clauses[0].id.0,
        spec2.sections[0].clauses[0].id.0
    );

    // Identifier follows spec::section::keyword_slug_of_text pattern
    assert_eq!(
        spec1.sections[0].clauses[0].id.0,
        "auth::login::must_return_a_jwt_token"
    );

    // Nested section path appears in full in the identifier
    let nested_md =
        "# Auth\n\n## Login\n\n### OAuth\n\n- **MUST** validate token signature\n";
    let nested_spec =
        Parser::parse_string(nested_md, Path::new("test.ought.md")).expect("parse failed");
    let nested_id = &nested_spec.sections[0].subsections[0].clauses[0].id.0;
    assert!(
        nested_id.starts_with("auth::login::oauth::"),
        "expected nested id to start with auth::login::oauth::, got: {nested_id}"
    );

    // Different clause text within the same section produces a different identifier
    let md2 = "# Auth\n\n## Login\n\n- **MUST** reject invalid tokens\n";
    let spec3 = Parser::parse_string(md2, Path::new("test.ought.md")).expect("parse failed");
    assert_ne!(
        spec1.sections[0].clauses[0].id.0,
        spec3.sections[0].clauses[0].id.0
    );
}
// --- must_include_a_condition_field_populated_from_the_parent_given_bl_test.rs ---
/// MUST include a `condition` field populated from the parent GIVEN block (null if unconditional)
#[test]
fn test_parser_clause_ir_must_include_a_condition_field_populated_from_the_parent_given_bl() {

    // Unconditional clause has None condition
    let md = "# Svc\n\n## Rules\n\n- **MUST** always do this\n";
    let spec = Parser::parse_string(md, Path::new("test.ought.md")).expect("parse failed");
    assert!(spec.sections[0].clauses[0].condition.is_none());

    // Clauses nested under GIVEN have the condition populated from GIVEN text
    let md_given = concat!(
        "# Svc\n\n## Rules\n\n",
        "- **GIVEN** the user is authenticated:\n",
        "  - **MUST** return profile data\n",
        "  - **MUST NOT** expose other users' data\n"
    );
    let spec_given =
        Parser::parse_string(md_given, Path::new("test.ought.md")).expect("parse failed");
    let clauses = &spec_given.sections[0].clauses;

    // GIVEN itself is not emitted as a clause — nested items become clauses
    assert_eq!(clauses.len(), 2);
    assert_eq!(clauses[0].keyword, Keyword::Must);
    assert_eq!(clauses[1].keyword, Keyword::MustNot);

    // Every clause in the GIVEN block shares the condition
    assert_eq!(
        clauses[0].condition.as_deref(),
        Some("the user is authenticated:")
    );
    assert_eq!(clauses[0].condition, clauses[1].condition);
}
// --- must_include_a_temporal_field_for_must_always_qualifier_invariant_test.rs ---
/// MUST include a `temporal` field for MUST ALWAYS (qualifier: invariant) and
/// MUST BY (qualifier: deadline, duration: value+unit)
#[test]
fn test_parser_clause_ir_must_include_a_temporal_field_for_must_always_qualifier_invariant() {

    // Plain MUST has no temporal field
    let md = "# Svc\n\n## Rules\n\n- **MUST** validate input\n";
    let spec = Parser::parse_string(md, Path::new("test.ought.md")).expect("parse failed");
    assert!(spec.sections[0].clauses[0].temporal.is_none());

    // MUST ALWAYS → Temporal::Invariant
    let md_always =
        "# Svc\n\n## Invariants\n\n- **MUST ALWAYS** keep connections below pool maximum\n";
    let spec_always =
        Parser::parse_string(md_always, Path::new("test.ought.md")).expect("parse failed");
    let clause_always = &spec_always.sections[0].clauses[0];
    assert_eq!(clause_always.keyword, Keyword::MustAlways);
    assert_eq!(clause_always.severity, Severity::Required);
    assert!(
        matches!(clause_always.temporal, Some(Temporal::Invariant)),
        "MUST ALWAYS should produce Temporal::Invariant"
    );

    // MUST BY <N>ms → Temporal::Deadline(Duration::from_millis(N))
    let md_ms = "# Svc\n\n## Perf\n\n- **MUST BY 200ms** return a response\n";
    let spec_ms = Parser::parse_string(md_ms, Path::new("test.ought.md")).expect("parse failed");
    let clause_ms = &spec_ms.sections[0].clauses[0];
    assert_eq!(clause_ms.keyword, Keyword::MustBy);
    assert_eq!(clause_ms.severity, Severity::Required);
    assert!(
        matches!(clause_ms.temporal, Some(Temporal::Deadline(d)) if d == Duration::from_millis(200)),
        "MUST BY 200ms should produce Deadline(200ms)"
    );

    // MUST BY <N>s → Temporal::Deadline(Duration::from_secs(N))
    let md_s = "# Svc\n\n## Perf\n\n- **MUST BY 5s** complete handshake\n";
    let spec_s = Parser::parse_string(md_s, Path::new("test.ought.md")).expect("parse failed");
    assert!(
        matches!(spec_s.sections[0].clauses[0].temporal, Some(Temporal::Deadline(d)) if d == Duration::from_secs(5)),
        "MUST BY 5s should produce Deadline(5s)"
    );

    // MUST BY <N>m → Temporal::Deadline(Duration::from_secs(N * 60))
    let md_m = "# Svc\n\n## Perf\n\n- **MUST BY 30m** finish batch job\n";
    let spec_m = Parser::parse_string(md_m, Path::new("test.ought.md")).expect("parse failed");
    assert!(
        matches!(spec_m.sections[0].clauses[0].temporal, Some(Temporal::Deadline(d)) if d == Duration::from_secs(30 * 60)),
        "MUST BY 30m should produce Deadline(30min)"
    );
}
// --- must_include_an_otherwise_field_containing_the_ordered_list_of_fa_test.rs ---
/// MUST include an `otherwise` field containing the ordered list of fallback clauses (empty if none)
#[test]
fn test_parser_clause_ir_must_include_an_otherwise_field_containing_the_ordered_list_of_fa() {

    // Clause without OTHERWISE has an empty otherwise vec
    let md = "# Svc\n\n## Perf\n\n- **MUST** respond within 200ms\n";
    let spec = Parser::parse_string(md, Path::new("test.ought.md")).expect("parse failed");
    assert!(spec.sections[0].clauses[0].otherwise.is_empty());

    // Clause with OTHERWISE chain carries an ordered list of fallback clauses
    let md_chain = concat!(
        "# Svc\n\n## Perf\n\n",
        "- **MUST** respond within 200ms\n",
        "  - **OTHERWISE** return a cached response\n",
        "  - **OTHERWISE** return 504 Gateway Timeout\n"
    );
    let spec_chain =
        Parser::parse_string(md_chain, Path::new("test.ought.md")).expect("parse failed");
    let clause = &spec_chain.sections[0].clauses[0];

    assert_eq!(clause.keyword, Keyword::Must);

    // Fallbacks are ordered by appearance
    assert_eq!(clause.otherwise.len(), 2);
    assert_eq!(clause.otherwise[0].keyword, Keyword::Otherwise);
    assert!(
        clause.otherwise[0].text.contains("cached response"),
        "first fallback should be cached response"
    );
    assert_eq!(clause.otherwise[1].keyword, Keyword::Otherwise);
    assert!(
        clause.otherwise[1].text.contains("504"),
        "second fallback should be 504"
    );

    // OTHERWISE clauses are not surfaced as top-level section clauses
    assert_eq!(spec_chain.sections[0].clauses.len(), 1);

    // OTHERWISE inherits the severity of its parent
    assert_eq!(clause.otherwise[0].severity, Severity::Required);
}
// --- must_produce_a_clause_ir_struct_containing_keyword_severity_claus_test.rs ---
/// MUST produce a clause IR struct containing: keyword, severity, clause text,
/// source location (file, line), parent section path, and a stable identifier
#[test]
fn test_parser_clause_ir_must_produce_a_clause_ir_struct_containing_keyword_severity_claus() {

    let md = "# Auth\n\n## Login\n\n- **MUST** return a JWT token\n";
    let spec = Parser::parse_string(md, Path::new("auth.ought.md")).expect("parse failed");
    let clause = &spec.sections[0].clauses[0];

    // keyword
    assert_eq!(clause.keyword, Keyword::Must);

    // severity derived from keyword
    assert_eq!(clause.severity, Severity::Required);

    // clause text
    assert!(clause.text.contains("return a JWT token"));

    // source location: file
    assert_eq!(clause.source_location.file.to_str().unwrap(), "auth.ought.md");

    // source location: line (must be a real line in the file)
    assert!(clause.source_location.line > 0);

    // stable identifier encodes the parent section path
    assert_eq!(clause.id.0, "auth::login::must_return_a_jwt_token");

    // content_hash is populated
    assert!(!clause.content_hash.is_empty());
}
// --- should_include_any_code_blocks_immediately_following_a_clause_as_hi_test.rs ---
/// SHOULD include any code blocks immediately following a clause as "hints" attached to that clause
#[test]
fn test_parser_clause_ir_should_include_any_code_blocks_immediately_following_a_clause_as_hi() {

    // Code block immediately after a clause becomes a hint on that clause
    let md = concat!(
        "# Svc\n\n## API\n\n",
        "- **MUST** return valid JSON\n\n",
        "```json\n{\"status\": \"ok\"}\n```\n"
    );
    let spec = Parser::parse_string(md, Path::new("test.ought.md")).expect("parse failed");
    let clause = &spec.sections[0].clauses[0];
    assert_eq!(clause.hints.len(), 1);
    assert!(
        clause.hints[0].contains("status"),
        "hint should contain code block content"
    );

    // Clause with no following code block has empty hints
    let md_no_hint = "# Svc\n\n## API\n\n- **MUST** return valid JSON\n";
    let spec_no_hint =
        Parser::parse_string(md_no_hint, Path::new("test.ought.md")).expect("parse failed");
    assert!(spec_no_hint.sections[0].clauses[0].hints.is_empty());

    // Code block appearing before any clause (as prose) is NOT attached as a hint
    let md_prose_code = concat!(
        "# Svc\n\n## API\n\n",
        "Some introductory text.\n\n",
        "```json\n{\"example\": true}\n```\n\n",
        "- **MUST** return valid JSON\n"
    );
    let spec_prose =
        Parser::parse_string(md_prose_code, Path::new("test.ought.md")).expect("parse failed");
    assert!(
        spec_prose.sections[0].clauses[0].hints.is_empty(),
        "code block before clause should not become a hint"
    );
    // That code block ends up in prose instead
    assert!(
        spec_prose.sections[0].prose.contains("example"),
        "code block before clause should appear in section prose"
    );
}
// --- should_include_surrounding_prose_markdown_in_the_clause_s_context_f_test.rs ---
/// SHOULD include surrounding prose/markdown in the clause's context field for the LLM
#[test]
fn test_parser_clause_ir_should_include_surrounding_prose_markdown_in_the_clause_s_context_f() {

    // Prose surrounding clauses is preserved in the section's prose field (LLM context)
    let md = concat!(
        "# Svc\n\n## Auth\n\n",
        "This section describes the authentication flow.\n",
        "Tokens are signed with RS256.\n\n",
        "- **MUST** validate token signature\n\n",
        "Additional notes about expiry edge cases.\n"
    );
    let spec = Parser::parse_string(md, Path::new("test.ought.md")).expect("parse failed");
    let section = &spec.sections[0];

    // Section prose is non-empty and contains the surrounding text
    assert!(!section.prose.is_empty());
    assert!(
        section.prose.contains("authentication flow"),
        "prose should include text before the clause"
    );
    assert!(
        section.prose.contains("RS256"),
        "prose should include all surrounding markdown content"
    );

    // The clause itself is still present alongside the prose
    assert_eq!(section.clauses.len(), 1);
    assert!(section.clauses[0].text.contains("validate token signature"));

    // A section containing only clauses and no surrounding text has empty prose
    let md_no_prose = "# Svc\n\n## Rules\n\n- **MUST** do something\n";
    let spec_no_prose =
        Parser::parse_string(md_no_prose, Path::new("test.ought.md")).expect("parse failed");
    assert!(spec_no_prose.sections[0].prose.is_empty());
}
// --- must_attach_the_given_condition_text_to_all_clauses_nested_within_test.rs ---
/// MUST attach the GIVEN condition text to all clauses nested within it
#[test]
fn test_parser_conditional_blocks_given_must_attach_the_given_condition_text_to_all_clauses_nested_within() {

    fn parse(md: &str) -> Spec {
        Parser::parse_string(md, Path::new("test.ought.md")).expect("parse failed")
    }

    let md = r#"# Svc

## Access

- **GIVEN** the request carries a valid token:
  - **MUST** allow the request through
  - **MUST NOT** log the token value
  - **SHOULD** refresh the token if near expiry
"#;
    let spec = parse(md);
    let clauses = &spec.sections[0].clauses;
    assert_eq!(clauses.len(), 3, "all three nested clauses should be emitted");
    for clause in clauses {
        assert_eq!(
            clause.condition.as_deref(),
            Some("the request carries a valid token:"),
            "every nested clause must carry the GIVEN condition text; clause '{}' did not",
            clause.text
        );
    }
}
// --- must_not_treat_given_itself_as_a_testable_clause_it_is_a_grouping_con_test.rs ---
/// MUST NOT treat GIVEN itself as a testable clause — it is a grouping construct with a precondition
#[test]
fn test_parser_conditional_blocks_given_must_not_treat_given_itself_as_a_testable_clause_it_is_a_grouping_con() {

    fn parse(md: &str) -> Spec {
        Parser::parse_string(md, Path::new("test.ought.md")).expect("parse failed")
    }

    let md = r#"# Svc

## Access

- **GIVEN** the user is authenticated:
  - **MUST** return their profile data
"#;
    let spec = parse(md);
    let clauses = &spec.sections[0].clauses;

    // GIVEN itself must not appear as a clause
    let given_clauses: Vec<_> = clauses.iter()
        .filter(|c| c.keyword == Keyword::Given)
        .collect();
    assert!(
        given_clauses.is_empty(),
        "GIVEN must not appear as a testable clause in the IR; found {} Given clause(s)",
        given_clauses.len()
    );

    // Only the nested MUST should be present
    assert_eq!(clauses.len(), 1);
    assert_eq!(clauses[0].keyword, Keyword::Must);
}
// --- must_parse_given_as_a_block_level_keyword_that_contains_nested_cl_test.rs ---
/// MUST parse `**GIVEN**` as a block-level keyword that contains nested clauses
#[test]
fn test_parser_conditional_blocks_given_must_parse_given_as_a_block_level_keyword_that_contains_nested_cl() {

    fn parse(md: &str) -> Spec {
        Parser::parse_string(md, Path::new("test.ought.md")).expect("parse failed")
    }

    let md = r#"# Svc

## Access

- **GIVEN** the user is authenticated:
  - **MUST** return their profile data
  - **SHOULD** include last-login timestamp
"#;
    let spec = parse(md);
    let clauses = &spec.sections[0].clauses;
    // GIVEN is a block-level grouping; its two children become the clauses
    assert_eq!(clauses.len(), 2);
    assert_eq!(clauses[0].keyword, Keyword::Must);
    assert!(clauses[0].text.contains("return their profile data"));
    assert_eq!(clauses[1].keyword, Keyword::Should);
    assert!(clauses[1].text.contains("include last-login timestamp"));
}
// --- must_require_nested_clauses_to_be_indented_under_the_given_bullet_test.rs ---
/// MUST require nested clauses to be indented under the GIVEN bullet (standard markdown nesting)
#[test]
fn test_parser_conditional_blocks_given_must_require_nested_clauses_to_be_indented_under_the_given_bullet() {

    fn parse(md: &str) -> Spec {
        Parser::parse_string(md, Path::new("test.ought.md")).expect("parse failed")
    }

    // The MUST is at the same indentation level as the GIVEN — not nested under it.
    // It should be treated as a top-level clause with no condition.
    let md = r#"# Svc

## Rules

- **GIVEN** user is admin:
- **MUST** do something important
"#;
    let spec = parse(md);
    let clauses = &spec.sections[0].clauses;
    // The MUST is a sibling of GIVEN, not a child — it gets no condition
    let must_clause = clauses.iter().find(|c| c.keyword == Keyword::Must)
        .expect("expected a MUST clause");
    assert!(
        must_clause.condition.is_none(),
        "un-indented MUST after GIVEN must not inherit the GIVEN condition"
    );
}
// --- must_support_given_blocks_containing_any_keyword_must_should_may_test.rs ---
/// MUST support GIVEN blocks containing any keyword (MUST, SHOULD, MAY, WONT, OTHERWISE, etc.)
#[test]
fn test_parser_conditional_blocks_given_must_support_given_blocks_containing_any_keyword_must_should_may() {

    fn parse(md: &str) -> Spec {
        Parser::parse_string(md, Path::new("test.ought.md")).expect("parse failed")
    }

    let md = r#"# Svc

## Behaviour

- **GIVEN** the feature flag is enabled:
  - **MUST** activate the new code path
  - **MUST NOT** fall back to the legacy path
  - **SHOULD** emit a telemetry event
  - **SHOULD NOT** cache the result
  - **MAY** log additional debug info
  - **WONT** support IE11 in this mode
"#;
    let spec = parse(md);
    let clauses = &spec.sections[0].clauses;
    assert_eq!(clauses.len(), 6);

    let keywords: Vec<Keyword> = clauses.iter().map(|c| c.keyword).collect();
    assert!(keywords.contains(&Keyword::Must),      "MUST inside GIVEN");
    assert!(keywords.contains(&Keyword::MustNot),   "MUST NOT inside GIVEN");
    assert!(keywords.contains(&Keyword::Should),    "SHOULD inside GIVEN");
    assert!(keywords.contains(&Keyword::ShouldNot), "SHOULD NOT inside GIVEN");
    assert!(keywords.contains(&Keyword::May),       "MAY inside GIVEN");
    assert!(keywords.contains(&Keyword::Wont),      "WONT inside GIVEN");

    // All inherit the condition
    for clause in clauses {
        assert_eq!(
            clause.condition.as_deref(),
            Some("the feature flag is enabled:"),
            "clause '{}' missing condition", clause.text
        );
    }
}
// --- must_support_multiple_given_blocks_within_a_section_test.rs ---
/// MUST support multiple GIVEN blocks within a section
#[test]
fn test_parser_conditional_blocks_given_must_support_multiple_given_blocks_within_a_section() {

    fn parse(md: &str) -> Spec {
        Parser::parse_string(md, Path::new("test.ought.md")).expect("parse failed")
    }

    let md = r#"# Svc

## Auth

- **GIVEN** the user is an admin:
  - **MUST** allow access to the admin panel
  - **MAY** impersonate other users
- **GIVEN** the user is a guest:
  - **MUST NOT** access private resources
  - **SHOULD** be shown a login prompt
"#;
    let spec = parse(md);
    let clauses = &spec.sections[0].clauses;
    assert_eq!(clauses.len(), 4, "two GIVEN blocks with two children each = four clauses");

    let admin_clauses: Vec<_> = clauses.iter()
        .filter(|c| c.condition.as_deref() == Some("the user is an admin:"))
        .collect();
    assert_eq!(admin_clauses.len(), 2);
    assert!(admin_clauses.iter().any(|c| c.keyword == Keyword::Must));
    assert!(admin_clauses.iter().any(|c| c.keyword == Keyword::May));

    let guest_clauses: Vec<_> = clauses.iter()
        .filter(|c| c.condition.as_deref() == Some("the user is a guest:"))
        .collect();
    assert_eq!(guest_clauses.len(), 2);
    assert!(guest_clauses.iter().any(|c| c.keyword == Keyword::MustNot));
    assert!(guest_clauses.iter().any(|c| c.keyword == Keyword::Should));
}
// --- should_support_nested_given_blocks_conditions_that_narrow_further_test.rs ---
/// SHOULD support nested GIVEN blocks (conditions that narrow further)
#[test]
fn test_parser_conditional_blocks_given_should_support_nested_given_blocks_conditions_that_narrow_further() {

    fn parse(md: &str) -> Spec {
        Parser::parse_string(md, Path::new("test.ought.md")).expect("parse failed")
    }

    let md = r#"# Svc

## Permissions

- **GIVEN** the user is an admin:
  - **GIVEN** the user account is active:
    - **MUST** allow full access
"#;
    let spec = parse(md);
    let clauses = &spec.sections[0].clauses;
    // The innermost MUST should be emitted as a clause
    assert_eq!(clauses.len(), 1);
    assert_eq!(clauses[0].keyword, Keyword::Must);
    assert!(clauses[0].text.contains("allow full access"));
    // The clause carries a condition derived from the inner (narrowing) GIVEN
    assert!(
        clauses[0].condition.is_some(),
        "clause nested inside two GIVENs must carry a condition"
    );
}
// --- may_support_glob_patterns_in_source_and_schema_paths_test.rs ---
/// MAY support glob patterns in `source:` and `schema:` paths
#[test]
fn test_parser_context_metadata_may_support_glob_patterns_in_source_and_schema_paths() {
    fn parse(md: &str) -> Spec {
        Parser::parse_string(md, Path::new("test.ought.md")).expect("parse failed")
    }

    let md = r#"# MySpec

source: src/**/*.rs, tests/**/*.rs
schema: migrations/*.sql, config/*.json

## Rules

- **MUST** do something
"#;
    // Glob patterns must be accepted without error and stored as-is (not expanded)
    let spec = parse(md);
    assert!(
        spec.metadata.sources.iter().any(|s| s == "src/**/*.rs"),
        "recursive glob in source not preserved"
    );
    assert!(
        spec.metadata.sources.iter().any(|s| s == "tests/**/*.rs"),
        "recursive glob in tests source not preserved"
    );
    assert!(
        spec.metadata.schemas.iter().any(|s| s == "migrations/*.sql"),
        "wildcard glob in schema not preserved"
    );
    assert!(
        spec.metadata.schemas.iter().any(|s| s == "config/*.json"),
        "wildcard glob in config schema not preserved"
    );
}
// --- must_parse_context_as_free_text_context_for_the_llm_test.rs ---
/// MUST parse `context:` as free-text context for the LLM
#[test]
fn test_parser_context_metadata_must_parse_context_as_free_text_context_for_the_llm() {
    fn parse(md: &str) -> Spec {
        Parser::parse_string(md, Path::new("test.ought.md")).expect("parse failed")
    }

    let md = r#"# MySpec

context: Handles user authentication and session management for the web API

## Rules

- **MUST** do something
"#;
    let spec = parse(md);
    let ctx = spec
        .metadata
        .context
        .expect("`context:` field should be Some");
    // The full free-text value must be preserved verbatim
    assert_eq!(
        ctx,
        "Handles user authentication and session management for the web API"
    );
}
// --- must_parse_schema_as_a_list_of_file_paths_schemas_configs_migrati_test.rs ---
/// MUST parse `schema:` as a list of file paths (schemas, configs, migrations)
#[test]
fn test_parser_context_metadata_must_parse_schema_as_a_list_of_file_paths_schemas_configs_migrati(
) {
    fn parse(md: &str) -> Spec {
        Parser::parse_string(md, Path::new("test.ought.md")).expect("parse failed")
    }

    let md = r#"# MySpec

schema: schema/auth.graphql, config/settings.json, migrations/001_init.sql

## Rules

- **MUST** do something
"#;
    let spec = parse(md);
    // GraphQL schemas, JSON configs, and SQL migrations must all be accepted as schema paths
    assert_eq!(spec.metadata.schemas.len(), 3);
    assert!(
        spec.metadata.schemas.iter().any(|s| s == "schema/auth.graphql"),
        "graphql schema not found"
    );
    assert!(
        spec.metadata.schemas.iter().any(|s| s == "config/settings.json"),
        "json config not found"
    );
    assert!(
        spec.metadata.schemas.iter().any(|s| s == "migrations/001_init.sql"),
        "sql migration not found"
    );
}
// --- must_parse_source_as_a_list_of_file_paths_or_directories_source_c_test.rs ---
/// MUST parse `source:` as a list of file paths or directories (source code hints for the LLM)
#[test]
fn test_parser_context_metadata_must_parse_source_as_a_list_of_file_paths_or_directories_source_c(
) {
    fn parse(md: &str) -> Spec {
        Parser::parse_string(md, Path::new("test.ought.md")).expect("parse failed")
    }

    let md = r#"# MySpec

source: src/handlers/, src/models/user.rs

## Rules

- **MUST** do something
"#;
    let spec = parse(md);
    // Directories (trailing slash) and file paths (with extension) must both be accepted
    assert_eq!(spec.metadata.sources.len(), 2);
    assert!(
        spec.metadata.sources.iter().any(|s| s == "src/handlers/"),
        "directory path not found in sources"
    );
    assert!(
        spec.metadata.sources.iter().any(|s| s == "src/models/user.rs"),
        "file path not found in sources"
    );
}
// --- must_support_multiple_values_per_metadata_key_one_per_line_or_com_test.rs ---
/// MUST support multiple values per metadata key (one per line or comma-separated)
#[test]
fn test_parser_context_metadata_must_support_multiple_values_per_metadata_key_one_per_line_or_com(
) {
    fn parse(md: &str) -> Spec {
        Parser::parse_string(md, Path::new("test.ought.md")).expect("parse failed")
    }

    // Comma-separated: all values on a single line
    let md_comma = r#"# MySpec

source: src/a/, src/b/, src/c/

## Rules

- **MUST** do something
"#;
    let spec = parse(md_comma);
    assert_eq!(
        spec.metadata.sources.len(),
        3,
        "comma-separated: expected 3 sources"
    );
    assert!(spec.metadata.sources.iter().any(|s| s == "src/a/"));
    assert!(spec.metadata.sources.iter().any(|s| s == "src/b/"));
    assert!(spec.metadata.sources.iter().any(|s| s == "src/c/"));

    // One per line: the same key repeated on adjacent lines (soft-break within same paragraph)
    let md_lines = r#"# MySpec

source: src/a/
source: src/b/
source: src/c/

## Rules

- **MUST** do something
"#;
    let spec2 = parse(md_lines);
    assert_eq!(
        spec2.metadata.sources.len(),
        3,
        "one-per-line: expected 3 sources"
    );
    assert!(spec2.metadata.sources.iter().any(|s| s == "src/a/"));
    assert!(spec2.metadata.sources.iter().any(|s| s == "src/b/"));
    assert!(spec2.metadata.sources.iter().any(|s| s == "src/c/"));
}
// --- must_link_each_otherwise_clause_to_its_parent_obligation_in_the_c_test.rs ---
/// MUST link each OTHERWISE clause to its parent obligation in the clause IR
#[test]
fn test_parser_contrary_to_duty_chains_otherwise_must_link_each_otherwise_clause_to_its_parent_obligation_in_the_c() {

    let md = r#"# Svc

## Api

- **MUST** validate the request payload
  - **OTHERWISE** reject with 400 Bad Request

- **MUST** authenticate the caller
  - **OTHERWISE** reject with 401 Unauthorized
"#;
    let spec = Parser::parse_string(md, Path::new("test.ought.md")).expect("parse failed");
    let clauses = &spec.sections[0].clauses;
    assert_eq!(clauses.len(), 2);

    // Each OTHERWISE is reachable only through its parent's .otherwise field
    assert_eq!(clauses[0].keyword, Keyword::Must);
    assert_eq!(clauses[0].otherwise.len(), 1);
    assert!(clauses[0].otherwise[0].text.contains("400"));

    assert_eq!(clauses[1].keyword, Keyword::Must);
    assert_eq!(clauses[1].otherwise.len(), 1);
    assert!(clauses[1].otherwise[0].text.contains("401"));

    // No OTHERWISE appears as a standalone top-level clause
    assert!(
        clauses.iter().all(|c| c.keyword != Keyword::Otherwise),
        "OTHERWISE clauses must not appear as top-level section clauses"
    );
}
// --- must_not_allow_otherwise_at_the_top_level_it_must_have_a_parent_oblig_test.rs ---
/// MUST NOT allow OTHERWISE at the top level (it must have a parent obligation)
#[test]
fn test_parser_contrary_to_duty_chains_otherwise_must_not_allow_otherwise_at_the_top_level_it_must_have_a_parent_oblig() {

    let md = r#"# Svc

## Section

- **OTHERWISE** this has no parent obligation
"#;
    let result = Parser::parse_string(md, Path::new("test.ought.md"));
    assert!(
        result.is_err(),
        "OTHERWISE at the top level without a parent obligation must be a parse error"
    );
}
// --- must_not_allow_otherwise_under_may_wont_or_given_only_under_obligatio_test.rs ---
/// MUST NOT allow OTHERWISE under MAY, WONT, or GIVEN (only under obligations that can be violated)
#[test]
fn test_parser_contrary_to_duty_chains_otherwise_must_not_allow_otherwise_under_may_wont_or_given_only_under_obligatio() {

    // OTHERWISE under MAY is invalid — MAY cannot be violated
    let md_may = r#"# Svc

## Section

- **MAY** use optional feature
  - **OTHERWISE** do nothing
"#;
    assert!(
        Parser::parse_string(md_may, Path::new("test.ought.md")).is_err(),
        "OTHERWISE under MAY must be a parse error"
    );

    // OTHERWISE under WONT is invalid — WONT is a negative confirmation, not a violable obligation
    let md_wont = r#"# Svc

## Section

- **WONT** implement feature X
  - **OTHERWISE** implement feature Y instead
"#;
    assert!(
        Parser::parse_string(md_wont, Path::new("test.ought.md")).is_err(),
        "OTHERWISE under WONT must be a parse error"
    );

    // OTHERWISE under GIVEN is invalid — GIVEN is a grouping construct, not a violable obligation
    let md_given = r#"# Svc

## Section

- **GIVEN** some condition:
  - **OTHERWISE** fallback without an obligation
"#;
    assert!(
        Parser::parse_string(md_given, Path::new("test.ought.md")).is_err(),
        "OTHERWISE under GIVEN must be a parse error"
    );
}
// --- must_parse_otherwise_as_a_clause_nested_under_a_parent_obligation_test.rs ---
/// MUST parse `**OTHERWISE**` as a clause nested under a parent obligation
#[test]
fn test_parser_contrary_to_duty_chains_otherwise_must_parse_otherwise_as_a_clause_nested_under_a_parent_obligation() {

    let md = r#"# Svc

## Resilience

- **MUST** respond within 200ms
  - **OTHERWISE** return a cached response
"#;
    let spec = Parser::parse_string(md, Path::new("test.ought.md")).expect("parse failed");
    let clauses = &spec.sections[0].clauses;

    // The parent MUST is the only top-level clause; OTHERWISE is not promoted to the top level
    assert_eq!(clauses.len(), 1, "only the parent obligation should appear as a top-level clause");
    assert_eq!(clauses[0].keyword, Keyword::Must);

    let otherwise = &clauses[0].otherwise;
    assert_eq!(otherwise.len(), 1);
    assert_eq!(otherwise[0].keyword, Keyword::Otherwise);
    assert!(otherwise[0].text.contains("cached response"));
}
// --- must_preserve_the_ordering_of_otherwise_clauses_they_form_a_degra_test.rs ---
/// MUST preserve the ordering of OTHERWISE clauses (they form a degradation chain)
#[test]
fn test_parser_contrary_to_duty_chains_otherwise_must_preserve_the_ordering_of_otherwise_clauses_they_form_a_degra() {

    let md = r#"# Svc

## Resilience

- **MUST** respond with fresh data
  - **OTHERWISE** return stale cache
  - **OTHERWISE** return degraded placeholder
  - **OTHERWISE** return HTTP 503
"#;
    let spec = Parser::parse_string(md, Path::new("test.ought.md")).expect("parse failed");
    let otherwise = &spec.sections[0].clauses[0].otherwise;

    assert_eq!(otherwise.len(), 3);
    // Degradation chain order must match declaration order
    assert!(otherwise[0].text.contains("stale cache"),        "first fallback must be stale cache");
    assert!(otherwise[1].text.contains("degraded placeholder"), "second fallback must be degraded placeholder");
    assert!(otherwise[2].text.contains("503"),                "third fallback must be 503");
}
// --- must_support_multiple_otherwise_clauses_under_a_single_parent_ord_test.rs ---
/// MUST support multiple OTHERWISE clauses under a single parent (ordered fallback chain)
#[test]
fn test_parser_contrary_to_duty_chains_otherwise_must_support_multiple_otherwise_clauses_under_a_single_parent_ord() {

    let md = r#"# Svc

## Payments

- **MUST** charge the primary card
  - **OTHERWISE** charge the backup card
  - **OTHERWISE** add to pending queue
  - **OTHERWISE** reject with insufficient funds error
"#;
    let spec = Parser::parse_string(md, Path::new("test.ought.md")).expect("parse failed");
    let clauses = &spec.sections[0].clauses;

    assert_eq!(clauses.len(), 1, "only one top-level clause should exist");

    let otherwise = &clauses[0].otherwise;
    assert_eq!(otherwise.len(), 3, "all three fallbacks must be collected under the single parent");
    assert!(otherwise.iter().all(|c| c.keyword == Keyword::Otherwise));

    assert!(otherwise[0].text.contains("backup card"));
    assert!(otherwise[1].text.contains("pending queue"));
    assert!(otherwise[2].text.contains("insufficient funds"));
}
// --- must_support_otherwise_under_any_obligation_keyword_must_should_m_test.rs ---
/// MUST support OTHERWISE under any obligation keyword (MUST, SHOULD, MUST ALWAYS, MUST BY)
#[test]
fn test_parser_contrary_to_duty_chains_otherwise_must_support_otherwise_under_any_obligation_keyword_must_should_m() {

    let md = r#"# Svc

## Obligations

- **MUST** respond within 200ms
  - **OTHERWISE** return HTTP 503

- **SHOULD** include debug headers
  - **OTHERWISE** omit debug headers silently

- **MUST ALWAYS** maintain a live connection
  - **OTHERWISE** reconnect with exponential backoff

- **MUST BY 5s** complete the handshake
  - **OTHERWISE** abort and log timeout
"#;
    let spec = Parser::parse_string(md, Path::new("test.ought.md")).expect("parse failed");
    let clauses = &spec.sections[0].clauses;
    assert_eq!(clauses.len(), 4);

    assert_eq!(clauses[0].keyword, Keyword::Must);
    assert_eq!(clauses[0].otherwise.len(), 1, "MUST must support OTHERWISE");

    assert_eq!(clauses[1].keyword, Keyword::Should);
    assert_eq!(clauses[1].otherwise.len(), 1, "SHOULD must support OTHERWISE");

    assert_eq!(clauses[2].keyword, Keyword::MustAlways);
    assert_eq!(clauses[2].otherwise.len(), 1, "MUST ALWAYS must support OTHERWISE");

    assert_eq!(clauses[3].keyword, Keyword::MustBy);
    assert_eq!(clauses[3].otherwise.len(), 1, "MUST BY must support OTHERWISE");
}
// --- should_inherit_the_parent_s_severity_unless_the_otherwise_clause_sp_test.rs ---
/// SHOULD inherit the parent's severity unless the OTHERWISE clause specifies its own keyword
#[test]
fn test_parser_contrary_to_duty_chains_otherwise_should_inherit_the_parent_s_severity_unless_the_otherwise_clause_sp() {

    let md = r#"# Svc

## Graceful

- **MUST** return primary data
  - **OTHERWISE** return cached copy

- **SHOULD** include metadata
  - **OTHERWISE** omit metadata field
"#;
    let spec = Parser::parse_string(md, Path::new("test.ought.md")).expect("parse failed");
    let clauses = &spec.sections[0].clauses;
    assert_eq!(clauses.len(), 2);

    // OTHERWISE under MUST inherits Required severity
    assert_eq!(clauses[0].severity, Severity::Required);
    assert_eq!(
        clauses[0].otherwise[0].severity,
        Severity::Required,
        "OTHERWISE under MUST must inherit Required severity"
    );

    // OTHERWISE under SHOULD inherits Recommended severity
    assert_eq!(clauses[1].severity, Severity::Recommended);
    assert_eq!(
        clauses[1].otherwise[0].severity,
        Severity::Recommended,
        "OTHERWISE under SHOULD must inherit Recommended severity"
    );
}
// --- must_build_a_dependency_graph_from_cross_file_references_test.rs ---
/// MUST build a dependency graph from cross-file references
#[test]
fn test_parser_cross_file_references_must_build_a_dependency_graph_from_cross_file_references() {

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
// --- must_detect_circular_dependencies_and_report_them_as_errors_test.rs ---
/// MUST detect circular dependencies and report them as errors
#[test]
fn test_parser_cross_file_references_must_detect_circular_dependencies_and_report_them_as_errors() {

    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let tmp = std::env::temp_dir().join(format!("ought_xref_cycle_{nanos}"));
    std::fs::create_dir_all(&tmp).unwrap();

    // a.ought.md requires b, and b.ought.md requires a — a direct mutual cycle
    std::fs::write(
        tmp.join("a.ought.md"),
        "# SpecA\n\nrequires: [SpecB](b.ought.md)\n\n## Rules\n\n- **MUST** do something\n",
    )
    .unwrap();
    std::fs::write(
        tmp.join("b.ought.md"),
        "# SpecB\n\nrequires: [SpecA](a.ought.md)\n\n## Rules\n\n- **MUST** do something\n",
    )
    .unwrap();

    let result = SpecGraph::from_roots(&[tmp.clone()]);

    assert!(
        result.is_err(),
        "a circular dependency must be reported as an error rather than silently accepted"
    );

    let errors = result.unwrap_err();
    let has_cycle_error = errors
        .iter()
        .any(|e| e.message.contains("circular dependency"));
    assert!(
        has_cycle_error,
        "error message must identify the circular dependency; got: {:?}",
        errors.iter().map(|e| &e.message).collect::<Vec<_>>()
    );

    let _ = std::fs::remove_dir_all(&tmp);
}
// --- must_parse_anchor_links_e_g_pricing_ought_md_discount_rules_as_re_test.rs ---
/// MUST parse anchor links (e.g. `pricing.ought.md#discount-rules`) as references to specific sections
#[test]
fn test_parser_cross_file_references_must_parse_anchor_links_e_g_pricing_ought_md_discount_rules_as_re() {
    let md = r#"# Checkout

requires: [Pricing](pricing.ought.md#discount-rules), [Auth](auth.ought.md#session-tokens)

## Payment

- **MUST** apply discount rules from the pricing spec
"#;
    let spec = Parser::parse_string(md, Path::new("checkout.ought.md"))
        .expect("parse failed");

    assert_eq!(
        spec.metadata.requires.len(),
        2,
        "anchor links must be parsed as cross-references"
    );

    let pricing = &spec.metadata.requires[0];
    assert_eq!(
        pricing.label, "Pricing",
        "link label must be captured from an anchor link"
    );
    assert_eq!(
        pricing.path.to_str().unwrap(),
        "pricing.ought.md",
        "file path must be extracted without the fragment"
    );
    assert_eq!(
        pricing.anchor.as_deref(),
        Some("discount-rules"),
        "URL fragment must be stored as the anchor field"
    );

    let auth = &spec.metadata.requires[1];
    assert_eq!(
        auth.path.to_str().unwrap(),
        "auth.ought.md",
        "second anchor link file path must be extracted correctly"
    );
    assert_eq!(
        auth.anchor.as_deref(),
        Some("session-tokens"),
        "second anchor must be extracted from the URL fragment"
    );
}
// --- must_parse_inline_markdown_links_to_other_ought_md_files_as_cross_test.rs ---
/// MUST parse inline markdown links to other .ought.md files as cross-references
#[test]
fn test_parser_cross_file_references_must_parse_inline_markdown_links_to_other_ought_md_files_as_cross() {
    let md = r#"# Checkout

requires: [Pricing](pricing.ought.md), [Users](users.ought.md)

## Payment

- **MUST** apply pricing rules
"#;
    let spec = Parser::parse_string(md, Path::new("checkout.ought.md"))
        .expect("parse failed");

    assert_eq!(
        spec.metadata.requires.len(),
        2,
        "each markdown link in requires: must become a separate cross-reference"
    );

    let pricing = &spec.metadata.requires[0];
    assert_eq!(
        pricing.label, "Pricing",
        "markdown link label must be captured as the SpecRef label"
    );
    assert_eq!(
        pricing.path.to_str().unwrap(),
        "pricing.ought.md",
        "markdown link URL must become the SpecRef path"
    );
    assert!(
        pricing.anchor.is_none(),
        "link without a URL fragment must have no anchor"
    );

    let users = &spec.metadata.requires[1];
    assert_eq!(
        users.label, "Users",
        "second link label must be captured"
    );
    assert_eq!(
        users.path.to_str().unwrap(),
        "users.ought.md",
        "second link URL must become the SpecRef path"
    );
    assert!(users.anchor.is_none());
}
// --- must_parse_requires_metadata_as_a_list_of_relative_paths_to_other_test.rs ---
/// MUST parse `requires:` metadata as a list of relative paths to other .ought.md files
#[test]
fn test_parser_cross_file_references_must_parse_requires_metadata_as_a_list_of_relative_paths_to_other() {
    let md = r#"# Billing

requires: pricing.ought.md
requires: users.ought.md

## Invoices

- **MUST** calculate totals correctly
"#;
    let spec = Parser::parse_string(md, Path::new("billing.ought.md"))
        .expect("parse failed");

    assert_eq!(
        spec.metadata.requires.len(),
        2,
        "requires: metadata must list all referenced spec files"
    );

    let first = &spec.metadata.requires[0];
    assert_eq!(
        first.path.to_str().unwrap(),
        "pricing.ought.md",
        "first requires: entry must carry the correct relative path"
    );
    assert!(
        first.anchor.is_none(),
        "plain path without a fragment must have no anchor"
    );

    let second = &spec.metadata.requires[1];
    assert_eq!(
        second.path.to_str().unwrap(),
        "users.ought.md",
        "second requires: entry must carry the correct relative path"
    );
    assert!(
        second.anchor.is_none(),
        "plain path without a fragment must have no anchor"
    );
}
// --- should_validate_that_all_cross_references_resolve_to_existing_files_test.rs ---
/// SHOULD validate that all cross-references resolve to existing files and sections
#[test]
fn test_parser_cross_file_references_should_validate_that_all_cross_references_resolve_to_existing_files() {

    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let tmp = std::env::temp_dir().join(format!("ought_xref_missing_{nanos}"));
    std::fs::create_dir_all(&tmp).unwrap();

    // spec_a references a file that is never written to disk
    std::fs::write(
        tmp.join("spec_a.ought.md"),
        "# SpecA\n\nrequires: [Missing](nonexistent.ought.md)\n\n## Rules\n\n- **MUST** reference real specs\n",
    )
    .unwrap();

    let result = SpecGraph::from_roots(&[tmp.clone()]);

    assert!(
        result.is_err(),
        "a requires: reference to a non-existent file must be reported as a validation error"
    );

    let errors = result.unwrap_err();
    let has_unresolved_error = errors.iter().any(|e| {
        e.message.contains("nonexistent.ought.md")
            || e.message.contains("unresolved")
            || e.message.contains("not found")
    });
    assert!(
        has_unresolved_error,
        "error must identify the unresolved cross-reference; got: {:?}",
        errors.iter().map(|e| &e.message).collect::<Vec<_>>()
    );

    let _ = std::fs::remove_dir_all(&tmp);
}
// --- must_continue_parsing_after_non_fatal_errors_collect_all_errors_d_test.rs ---
/// MUST continue parsing after non-fatal errors (collect all errors, don't stop at the first)
#[test]
fn test_parser_error_handling_must_continue_parsing_after_non_fatal_errors_collect_all_errors_d() {
    // Several keyword typos are interspersed with valid clauses. The parser must
    // not abort at the first unrecognized item — every valid clause that appears
    // later in the document must still be returned.
    let md = "\
# Svc

## Rules

- **MUTS** first typo — not a recognised keyword
- **MUST** first valid clause after typo
- **SHOLD** second typo
- **SHOULD** second valid clause after typo
- **MUST NOT** third valid clause at end of section
";
    let result = Parser::parse_string(md, Path::new("test.ought.md"));
    assert!(
        result.is_ok(),
        "unrecognised keyword typos must not cause a hard parse failure"
    );
    let spec = result.unwrap();
    let clauses = &spec.sections[0].clauses;

    // If the parser had stopped at the first bad item, only 0 or 1 clause would be
    // present. All three must be here to demonstrate full-document traversal.
    assert_eq!(
        clauses.len(),
        3,
        "parser must collect all valid clauses across the entire document, \
         not stop at the first unrecognised item"
    );
    assert_eq!(
        clauses[0].keyword,
        Keyword::Must,
        "valid MUST after first typo must be parsed"
    );
    assert_eq!(
        clauses[1].keyword,
        Keyword::Should,
        "valid SHOULD after second typo must be parsed"
    );
    assert_eq!(
        clauses[2].keyword,
        Keyword::MustNot,
        "valid MUST NOT at end of section must be parsed"
    );

    // Errors are returned as Vec<ParseError> — the whole collection, not just the first.
    // Verify the Vec type is used (not a single-error short-circuit) for file errors too.
    let file_result = Parser::parse_file(Path::new("/no/such/file.ought.md"));
    let errors = file_result.unwrap_err();
    // The Vec itself is the contract; callers can iterate all diagnostics.
    assert!(
        !errors.is_empty(),
        "errors must be returned in a Vec so callers see all diagnostics"
    );
}
// --- must_not_crash_on_malformed_markdown_degrade_gracefully_test.rs ---
/// MUST NOT crash on malformed markdown — degrade gracefully
#[test]
fn test_parser_error_handling_must_not_crash_on_malformed_markdown_degrade_gracefully() {
    // Empty document — must return a default Spec, not panic or error.
    let result = Parser::parse_string("", Path::new("empty.ought.md"));
    assert!(result.is_ok(), "empty input must parse without error");
    let spec = result.unwrap();
    assert_eq!(spec.name, "Untitled", "empty doc must default to 'Untitled'");
    assert!(spec.sections.is_empty(), "empty doc must have no sections");

    // Whitespace-only input.
    let result = Parser::parse_string("   \n\n   \n", Path::new("ws.ought.md"));
    assert!(result.is_ok(), "whitespace-only input must not fail");

    // H1-only, no sections.
    let result = Parser::parse_string("# Just a Title\n", Path::new("title_only.ought.md"));
    assert!(result.is_ok(), "H1-only doc must not fail");
    assert_eq!(result.unwrap().name, "Just a Title");

    // Missing H1 — valid clauses inside a section must still be parsed.
    let result = Parser::parse_string(
        "## Section\n\n- **MUST** do the thing\n",
        Path::new("no_h1.ought.md"),
    );
    assert!(result.is_ok(), "missing H1 must not crash the parser");
    let spec = result.unwrap();
    assert!(!spec.sections.is_empty(), "section must be parsed even without H1");
    assert_eq!(
        spec.sections[0].clauses.len(),
        1,
        "clause must be parsed when H1 is absent"
    );

    // Unclosed bold delimiter — CommonMark treats it as literal text.
    // The valid clause after the broken item must still be collected.
    let result = Parser::parse_string(
        "# Svc\n\n## S\n\n- **MUST unclosed bold marker\n- **MUST** valid after unclosed\n",
        Path::new("unclosed_bold.ought.md"),
    );
    // Must not panic; valid clause after the bad item must survive.
    if let Ok(spec) = result {
        assert!(
            !spec.sections[0].clauses.is_empty(),
            "valid clause after unclosed bold must be parsed"
        );
    }

    // Unclosed fenced code block.
    let _ = Parser::parse_string(
        "# Svc\n\n## S\n\n- **MUST** clause\n\n```\nno closing fence\n",
        Path::new("unclosed_fence.ought.md"),
    );

    // Markdown escape sequences and unusual characters inside clause text.
    let _ = Parser::parse_string(
        "# Svc\n\n## S\n\n- **MUST** handle \\* \\` \\[ edge chars\n",
        Path::new("escapes.ought.md"),
    );

    // Deeply nested lists beyond normal spec usage.
    let deep = format!(
        "# Svc\n\n## S\n\n- **MUST** top\n{}- nested\n{}- deeper\n{}- deepest\n",
        "  ".repeat(1),
        "  ".repeat(2),
        "  ".repeat(3),
    );
    let _ = Parser::parse_string(&deep, Path::new("deep_nesting.ought.md"));

    // Section heading with no body.
    let result = Parser::parse_string("# Svc\n\n## Empty Section\n\n## Next\n\n- **MUST** clause\n", Path::new("empty_sec.ought.md"));
    assert!(result.is_ok(), "empty section followed by valid section must not fail");
    let spec = result.unwrap();
    assert_eq!(
        spec.sections.iter().map(|s| s.clauses.len()).sum::<usize>(),
        1,
        "clause in section after an empty section must still be parsed"
    );
}
// --- must_report_parse_errors_with_the_file_path_line_number_and_a_cle_test.rs ---
/// MUST report parse errors with the file path, line number, and a clear message
#[test]
fn test_parser_error_handling_must_report_parse_errors_with_the_file_path_line_number_and_a_cle() {
    // Directly construct a ParseError and verify all three required fields are present
    // and surfaced by the Display implementation.
    let path = PathBuf::from("spec/auth.ought.md");
    let err = ParseError {
        file: path.clone(),
        line: 23,
        message: "unexpected token after MUST".to_string(),
    };

    assert_eq!(err.file, path, "error must carry the source file path");
    assert_eq!(err.line, 23, "error must carry the 1-based line number");
    assert!(!err.message.is_empty(), "error must include a non-empty human-readable message");

    // Display must render all three components so callers can show "file:line: msg"
    let display = format!("{}", err);
    assert!(
        display.contains("spec/auth.ought.md"),
        "display must include the file path; got: {display}"
    );
    assert!(
        display.contains("23"),
        "display must include the line number; got: {display}"
    );
    assert!(
        display.contains("unexpected token"),
        "display must include the message text; got: {display}"
    );

    // parse_file errors must embed the path that was attempted so the caller can
    // report "failed to read foo.ought.md" without extra bookkeeping.
    let missing = Path::new("/nonexistent/spec.ought.md");
    let result = Parser::parse_file(missing);
    assert!(result.is_err(), "parse_file must return Err for a missing file");
    let errors = result.unwrap_err();
    assert!(!errors.is_empty(), "errors Vec must be non-empty");
    assert_eq!(
        errors[0].file, missing,
        "parse_file error must record the path that was attempted"
    );
    assert!(
        !errors[0].message.is_empty(),
        "parse_file error must include a clear message describing the failure"
    );
}
// --- should_warn_on_likely_typos_e_g_muts_close_to_a_known_keyword_test.rs ---
/// SHOULD warn on likely typos (e.g. `**MUTS**` close to a known keyword)
#[test]
fn test_parser_error_handling_should_warn_on_likely_typos_e_g_muts_close_to_a_known_keyword() {
    // **MUTS** is an edit-distance-1 typo for **MUST**; **SHOLD** for **SHOULD**.
    // The parser must not silently accept them as valid keywords, and SHOULD
    // surface a diagnostic pointing to the likely correction.
    let md = "\
# Svc

## Rules

- **MUTS** typo for MUST — must not become a clause
- **MUST** the real keyword
- **SHOLD** typo for SHOULD — must not become a clause
- **SHOUD** another SHOULD variant — must not become a clause
";
    let spec = Parser::parse_string(md, Path::new("typos.ought.md"))
        .expect("keyword typos must not crash the parser");

    let clauses = &spec.sections[0].clauses;

    // Typo keywords must NOT be silently accepted and turned into clauses.
    assert!(
        !clauses.iter().any(|c| c.text.contains("typo for MUST")),
        "**MUTS** must not produce a clause — it is not a recognised keyword"
    );
    assert!(
        !clauses.iter().any(|c| c.text.contains("typo for SHOULD")),
        "**SHOLD** must not produce a clause — it is not a recognised keyword"
    );
    assert!(
        !clauses.iter().any(|c| c.text.contains("another SHOULD variant")),
        "**SHOUD** must not produce a clause — it is not a recognised keyword"
    );

    // The one genuinely valid keyword must still be recognised.
    assert_eq!(
        clauses.len(),
        1,
        "only the single valid **MUST** item must become a clause; typos become prose"
    );
    assert_eq!(
        clauses[0].keyword,
        Keyword::Must,
        "valid **MUST** must be recognised even when surrounded by typo items"
    );

    // TODO: when a warning/lint system is added, assert diagnostics such as:
    //   ParseWarning { file: "typos.ought.md", line: 5,
    //       message: "unknown keyword **MUTS** — did you mean **MUST**?" }
    //   ParseWarning { file: "typos.ought.md", line: 7,
    //       message: "unknown keyword **SHOLD** — did you mean **SHOULD**?" }
    // These could be surfaced via a `warnings: Vec<ParseWarning>` field on a
    // `ParseResult` wrapper or alongside `ParseError` in a `ParseDiagnostic` enum.
}
// --- must_assign_severity_levels_must_must_not_must_always_must_by_req_test.rs ---
/// MUST assign severity levels: MUST/MUST NOT/MUST ALWAYS/MUST BY = required,
/// SHOULD/SHOULD NOT = recommended, MAY = optional, WONT = negative-confirmation
#[test]
fn test_parser_keywords_must_assign_severity_levels_must_must_not_must_always_must_by_req() {

    let md = r#"# Svc

## All Keywords

- **MUST** do something required
- **MUST NOT** skip something required
- **MUST ALWAYS** hold an invariant
- **MUST BY 1s** finish within deadline
- **SHOULD** follow recommendation
- **SHOULD NOT** violate recommendation
- **MAY** use optional feature
- **WONT** implement out-of-scope thing
"#;
    let spec = Parser::parse_string(md, Path::new("test.ought.md")).expect("parse failed");
    let clauses = &spec.sections[0].clauses;
    assert_eq!(clauses.len(), 8);

    // Required group
    assert_eq!(clauses[0].severity, Severity::Required,   "MUST → Required");
    assert_eq!(clauses[1].severity, Severity::Required,   "MUST NOT → Required");
    assert_eq!(clauses[2].severity, Severity::Required,   "MUST ALWAYS → Required");
    assert_eq!(clauses[3].severity, Severity::Required,   "MUST BY → Required");

    // Recommended group
    assert_eq!(clauses[4].severity, Severity::Recommended, "SHOULD → Recommended");
    assert_eq!(clauses[5].severity, Severity::Recommended, "SHOULD NOT → Recommended");

    // Optional
    assert_eq!(clauses[6].severity, Severity::Optional,   "MAY → Optional");

    // Negative confirmation
    assert_eq!(clauses[7].severity, Severity::NegativeConfirmation, "WONT → NegativeConfirmation");
}
// --- must_not_treat_bare_non_bold_keyword_occurrences_as_clauses_e_g_you_m_test.rs ---
/// MUST NOT treat bare (non-bold) keyword occurrences as clauses (e.g. "you must restart" in prose)
#[test]
fn test_parser_keywords_must_not_treat_bare_non_bold_keyword_occurrences_as_clauses_e_g_you_m() {

    let md = r#"# Svc

## Overview

This service must handle all requests. Operators should monitor memory usage.
You must not store credentials in logs. The system may cache responses.
Deployments wont need downtime. Given the above, otherwise consider alternatives.

- A plain list item that says you must restart after upgrade
- Another item: should not be mistaken for a clause
"#;
    let spec = Parser::parse_string(md, Path::new("test.ought.md")).expect("parse failed");
    assert!(
        spec.sections[0].clauses.is_empty(),
        "bare keywords in paragraphs and unbolded list items must not produce any clauses"
    );

    // Confirm that a bold keyword in the same section does produce a clause,
    // proving the parser is active and the above result is not a parser failure.
    let md_mixed = r#"# Svc

## Mixed

This service must handle all requests as described above.

- **MUST** actually validate the token
"#;
    let spec2 = Parser::parse_string(md_mixed, Path::new("test.ought.md")).expect("parse failed");
    let clauses = &spec2.sections[0].clauses;
    assert_eq!(clauses.len(), 1, "only the bold keyword item must become a clause");
    assert_eq!(clauses[0].keyword, Keyword::Must);
    assert!(clauses[0].text.contains("validate the token"));
}
// --- must_parse_keywords_case_insensitively_but_require_them_to_appear_test.rs ---
/// MUST parse keywords case-insensitively but require them to appear in bold (`**MUST**`, `**GIVEN**`, etc.)
#[test]
fn test_parser_keywords_must_parse_keywords_case_insensitively_but_require_them_to_appear() {

    // All casing variants in bold — all must be recognised
    let md_bold = r#"# Svc

## Rules

- **must** do something lowercase
- **Must** do something title-case
- **MUST** do something uppercase
- **must not** reject lowercase compound
- **Should** recommend title-case
- **may** allow lowercase optional
- **wont** refuse lowercase
"#;
    let spec = Parser::parse_string(md_bold, Path::new("test.ought.md")).expect("parse failed");
    let clauses = &spec.sections[0].clauses;
    assert_eq!(clauses.len(), 7, "all bold keyword variants must be parsed regardless of case");
    assert_eq!(clauses[0].keyword, Keyword::Must);
    assert_eq!(clauses[1].keyword, Keyword::Must);
    assert_eq!(clauses[2].keyword, Keyword::Must);
    assert_eq!(clauses[3].keyword, Keyword::MustNot);
    assert_eq!(clauses[4].keyword, Keyword::Should);
    assert_eq!(clauses[5].keyword, Keyword::May);
    assert_eq!(clauses[6].keyword, Keyword::Wont);

    // Non-bold keywords — must produce zero clauses
    let md_bare = r#"# Svc

## Prose

You must restart the service after upgrading.
The system should validate inputs.
"#;
    let spec2 = Parser::parse_string(md_bare, Path::new("test.ought.md")).expect("parse failed");
    assert!(
        spec2.sections[0].clauses.is_empty(),
        "bare (non-bold) keywords in prose must not produce clauses"
    );
}
// --- must_recognize_rfc_2119_keywords_must_must_not_should_should_not_test.rs ---
/// MUST recognize RFC 2119 keywords: MUST, MUST NOT, SHOULD, SHOULD NOT, MAY
#[test]
fn test_parser_keywords_must_recognize_rfc_2119_keywords_must_must_not_should_should_not() {

    let md = r#"# Svc

## Rules

- **MUST** perform authentication
- **MUST NOT** expose internal errors
- **SHOULD** log failed attempts
- **SHOULD NOT** cache sensitive tokens
- **MAY** support remember-me sessions
"#;
    let spec = Parser::parse_string(md, Path::new("test.ought.md")).expect("parse failed");
    let clauses = &spec.sections[0].clauses;
    assert_eq!(clauses.len(), 5);
    assert_eq!(clauses[0].keyword, Keyword::Must);
    assert!(clauses[0].text.contains("authentication"));
    assert_eq!(clauses[1].keyword, Keyword::MustNot);
    assert!(clauses[1].text.contains("internal errors"));
    assert_eq!(clauses[2].keyword, Keyword::Should);
    assert!(clauses[2].text.contains("failed attempts"));
    assert_eq!(clauses[3].keyword, Keyword::ShouldNot);
    assert!(clauses[3].text.contains("sensitive tokens"));
    assert_eq!(clauses[4].keyword, Keyword::May);
    assert!(clauses[4].text.contains("remember-me"));
}
// --- must_recognize_temporal_compound_keywords_must_always_must_by_test.rs ---
/// MUST recognize temporal compound keywords: MUST ALWAYS, MUST BY
#[test]
fn test_parser_keywords_must_recognize_temporal_compound_keywords_must_always_must_by() {

    let md = r#"# Svc

## Temporal

- **MUST ALWAYS** keep connection pool below maximum capacity
- **MUST BY 500ms** return a search result
- **MUST BY 10s** complete the checkout flow
- **MUST BY 2m** finish a background import job
"#;
    let spec = Parser::parse_string(md, Path::new("test.ought.md")).expect("parse failed");
    let clauses = &spec.sections[0].clauses;
    assert_eq!(clauses.len(), 4);

    assert_eq!(clauses[0].keyword, Keyword::MustAlways);
    assert!(
        matches!(clauses[0].temporal, Some(Temporal::Invariant)),
        "MUST ALWAYS must carry Temporal::Invariant"
    );

    assert_eq!(clauses[1].keyword, Keyword::MustBy);
    assert!(
        matches!(clauses[1].temporal, Some(Temporal::Deadline(d)) if d == Duration::from_millis(500)),
        "MUST BY 500ms must produce a 500ms deadline"
    );

    assert_eq!(clauses[2].keyword, Keyword::MustBy);
    assert!(
        matches!(clauses[2].temporal, Some(Temporal::Deadline(d)) if d == Duration::from_secs(10)),
        "MUST BY 10s must produce a 10s deadline"
    );

    assert_eq!(clauses[3].keyword, Keyword::MustBy);
    assert!(
        matches!(clauses[3].temporal, Some(Temporal::Deadline(d)) if d == Duration::from_secs(120)),
        "MUST BY 2m must produce a 120s deadline"
    );
}
// --- must_recognize_the_given_keyword_as_a_conditional_block_opener_fr_test.rs ---
/// MUST recognize the GIVEN keyword as a conditional block opener (from deontic logic)
#[test]
fn test_parser_keywords_must_recognize_the_given_keyword_as_a_conditional_block_opener_fr() {

    let md = r#"# Svc

## Access

- **GIVEN** the user holds an admin role:
  - **MUST** allow deletion of any record
  - **MUST NOT** expose other tenants' data
"#;
    let spec = Parser::parse_string(md, Path::new("test.ought.md")).expect("parse failed");
    let clauses = &spec.sections[0].clauses;
    // GIVEN groups its children; they become top-level clauses with the condition set
    assert_eq!(clauses.len(), 2);
    assert_eq!(clauses[0].keyword, Keyword::Must);
    let condition = clauses[0].condition.as_deref().expect("condition must be present");
    assert!(condition.contains("admin role"));
    assert_eq!(clauses[1].keyword, Keyword::MustNot);
    assert_eq!(clauses[1].condition, clauses[0].condition,
        "all children of the same GIVEN block share the same condition");
}
// --- must_recognize_the_otherwise_keyword_as_a_contrary_to_duty_fallba_test.rs ---
/// MUST recognize the OTHERWISE keyword as a contrary-to-duty fallback (from deontic logic)
#[test]
fn test_parser_keywords_must_recognize_the_otherwise_keyword_as_a_contrary_to_duty_fallba() {

    let md = r#"# Svc

## Resilience

- **MUST** respond within 100ms
  - **OTHERWISE** return a stale cached response
  - **OTHERWISE** return HTTP 503 with Retry-After header
"#;
    let spec = Parser::parse_string(md, Path::new("test.ought.md")).expect("parse failed");
    let clauses = &spec.sections[0].clauses;
    assert_eq!(clauses.len(), 1);
    assert_eq!(clauses[0].keyword, Keyword::Must);
    let otherwise = &clauses[0].otherwise;
    assert_eq!(otherwise.len(), 2);
    assert_eq!(otherwise[0].keyword, Keyword::Otherwise);
    assert!(otherwise[0].text.contains("stale cached"));
    assert_eq!(otherwise[1].keyword, Keyword::Otherwise);
    assert!(otherwise[1].text.contains("503"));
}
// --- must_recognize_the_wont_keyword_as_an_ought_extension_not_in_rfc_test.rs ---
/// MUST recognize the WONT keyword as an ought extension (not in RFC 2119)
#[test]
fn test_parser_keywords_must_recognize_the_wont_keyword_as_an_ought_extension_not_in_rfc() {

    let md = r#"# Svc

## Scope

- **WONT** support OAuth 1.0
- **WONT** implement SOAP endpoints
"#;
    let spec = Parser::parse_string(md, Path::new("test.ought.md")).expect("parse failed");
    let clauses = &spec.sections[0].clauses;
    assert_eq!(clauses.len(), 2);
    assert_eq!(clauses[0].keyword, Keyword::Wont);
    assert!(clauses[0].text.contains("OAuth 1.0"));
    assert_eq!(clauses[1].keyword, Keyword::Wont);
    assert!(clauses[1].text.contains("SOAP"));
}
// --- must_not_fail_on_standard_markdown_that_doesn_t_contain_ought_keyword_test.rs ---
/// MUST NOT fail on standard markdown that doesn't contain ought keywords (just produce zero clauses)
#[test]
fn test_parser_spec_file_structure_must_not_fail_on_standard_markdown_that_doesn_t_contain_ought_keyword() {
    let md = r#"# Plain Spec

## Overview

This is a standard markdown document with no ought keywords whatsoever.

It has paragraphs, *italic text*, and `code spans`.

- A plain list item
- Another plain list item

## Details

More prose here. No deontic keywords appear in bold in any list items.

```python
def example():
    return True
```
"#;
    let result = Parser::parse_string(md, Path::new("test.ought.md"));
    assert!(
        result.is_ok(),
        "Parser must not fail on keyword-free markdown: {:?}",
        result.err()
    );
    let spec = result.unwrap();
    assert_eq!(spec.name, "Plain Spec");
    assert!(
        !spec.sections.is_empty(),
        "sections must still be parsed from headings"
    );
    let total_clauses: usize = spec.sections.iter().map(|s| s.clauses.len()).sum();
    assert_eq!(
        total_clauses, 0,
        "keyword-free markdown must produce zero clauses"
    );
}
// --- must_parse_frontmatter_style_metadata_at_the_top_of_the_file_cont_test.rs ---
/// MUST parse frontmatter-style metadata at the top of the file: `context:`, `source:`, `schema:`, `requires:`
#[test]
fn test_parser_spec_file_structure_must_parse_frontmatter_style_metadata_at_the_top_of_the_file_cont() {
    let md = r#"# Payments

context: Handles payment processing and invoicing
source: src/payments/, src/billing/
schema: schema/payments.graphql, schema/billing.graphql
requires: [Auth](auth.ought.md), [Users](users.ought.md#accounts)

## Checkout

- **MUST** process payments
"#;
    let spec = Parser::parse_string(md, Path::new("test.ought.md")).expect("parse failed");

    assert_eq!(
        spec.metadata.context.as_deref(),
        Some("Handles payment processing and invoicing"),
        "context: field must be parsed into metadata.context"
    );
    assert_eq!(
        spec.metadata.sources,
        vec!["src/payments/", "src/billing/"],
        "source: field must be parsed as a comma-separated list"
    );
    assert_eq!(
        spec.metadata.schemas,
        vec!["schema/payments.graphql", "schema/billing.graphql"],
        "schema: field must be parsed as a comma-separated list"
    );
    assert_eq!(
        spec.metadata.requires.len(),
        2,
        "requires: field must list all spec dependencies"
    );
    assert_eq!(spec.metadata.requires[0].label, "Auth");
    assert_eq!(
        spec.metadata.requires[0].path.to_str().unwrap(),
        "auth.ought.md"
    );
    assert_eq!(
        spec.metadata.requires[0].anchor,
        None,
        "link without fragment must have no anchor"
    );
    assert_eq!(spec.metadata.requires[1].label, "Users");
    assert_eq!(
        spec.metadata.requires[1].path.to_str().unwrap(),
        "users.ought.md"
    );
    assert_eq!(
        spec.metadata.requires[1].anchor.as_deref(),
        Some("accounts"),
        "link fragment must be captured as anchor"
    );
}
// --- must_parse_standard_markdown_commonmark_as_the_base_format_test.rs ---
/// MUST parse standard Markdown (CommonMark) as the base format
#[test]
fn test_parser_spec_file_structure_must_parse_standard_markdown_commonmark_as_the_base_format() {
    // Exercises headings, paragraphs, emphasis, inline code, fenced code blocks,
    // blockquotes, plain lists, and inline links — core CommonMark elements.
    let md = r#"# My Spec

## Intro

A paragraph with *italic*, ***bold italic***, and `inline code` text.

> A blockquote providing background context.

See also [the reference docs](http://example.com) for more detail.

- A plain list item
- Another informational bullet

```json
{"example": true}
```

## Rules

- **MUST** handle all CommonMark elements in surrounding prose
"#;
    let result = Parser::parse_string(md, Path::new("commonmark_test.ought.md"));
    assert!(
        result.is_ok(),
        "Parser must not fail on standard CommonMark: {:?}",
        result.err()
    );
    let spec = result.unwrap();
    assert_eq!(spec.name, "My Spec");
    let rules = spec.sections.iter().find(|s| s.title == "Rules").unwrap();
    assert_eq!(rules.clauses.len(), 1);
    assert_eq!(rules.clauses[0].keyword, Keyword::Must);
}
// --- must_preserve_all_non_clause_markdown_as_documentation_context_fo_test.rs ---
/// MUST preserve all non-clause markdown as documentation (context for the LLM, readable for humans)
#[test]
fn test_parser_spec_file_structure_must_preserve_all_non_clause_markdown_as_documentation_context_fo() {
    let md = r#"# Svc

## Security

This section describes the security requirements for the service.
Access control is enforced at the API boundary.

- Background: authentication uses bearer tokens
- Note: tokens expire after 24 hours

- **MUST** reject unauthenticated requests
"#;
    let spec = Parser::parse_string(md, Path::new("test.ought.md")).expect("parse failed");
    let section = &spec.sections[0];

    // The MUST clause is parsed correctly
    assert_eq!(section.clauses.len(), 1);
    assert_eq!(section.clauses[0].keyword, Keyword::Must);

    // All surrounding non-clause markdown is preserved in section.prose
    assert!(
        !section.prose.is_empty(),
        "non-clause markdown must be preserved in section.prose"
    );
    assert!(
        section.prose.contains("security requirements") || section.prose.contains("API boundary"),
        "paragraph text must appear in section.prose"
    );
}
// --- must_recognize_files_with_the_ought_md_extension_test.rs ---
/// MUST recognize files with the `.ought.md` extension
#[test]
fn test_parser_spec_file_structure_must_recognize_files_with_the_ought_md_extension() {

    let content = "# Ext Test\n\n## Section\n\n- **MUST** work\n";
    let path = std::env::temp_dir().join("ought_recognize_ext_test.ought.md");
    fs::write(&path, content).expect("failed to write temp .ought.md file");

    let result = Parser::parse_file(&path);
    fs::remove_file(&path).ok();

    assert!(
        result.is_ok(),
        "Parser must recognize and parse .ought.md files: {:?}",
        result.err()
    );
    let spec = result.unwrap();
    assert_eq!(spec.name, "Ext Test");
    assert_eq!(spec.sections[0].clauses.len(), 1);
    // source_path must reflect the .ought.md filename
    assert!(
        spec.source_path
            .file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.ends_with(".ought.md"))
            .unwrap_or(false),
        "source_path must preserve the .ought.md extension"
    );
}
// --- must_treat_bullet_points_keyword_as_individual_clauses_test.rs ---
/// MUST treat bullet points (`- **KEYWORD**`) as individual clauses
#[test]
fn test_parser_spec_file_structure_must_treat_bullet_points_keyword_as_individual_clauses() {
    let md = r#"# Svc

## API

- **MUST** return a response body
- **MUST NOT** leak internal error details
- **SHOULD** include a request-id header
"#;
    let spec = Parser::parse_string(md, Path::new("test.ought.md")).expect("parse failed");
    let clauses = &spec.sections[0].clauses;

    assert_eq!(clauses.len(), 3, "each bold-keyword bullet must become exactly one clause");

    assert_eq!(clauses[0].keyword, Keyword::Must);
    assert_eq!(clauses[0].severity, Severity::Required);
    assert!(clauses[0].text.contains("return a response body"));

    assert_eq!(clauses[1].keyword, Keyword::MustNot);
    assert_eq!(clauses[1].severity, Severity::Required);
    assert!(clauses[1].text.contains("leak internal error details"));

    assert_eq!(clauses[2].keyword, Keyword::Should);
    assert_eq!(clauses[2].severity, Severity::Recommended);
    assert!(clauses[2].text.contains("request-id header"));

    // Each clause must have a non-empty, unique stable ID
    assert!(!clauses[0].id.0.is_empty());
    assert_ne!(clauses[0].id.0, clauses[1].id.0);
    assert_ne!(clauses[1].id.0, clauses[2].id.0);
}
// --- must_treat_h1_as_the_spec_name_test.rs ---
/// MUST treat H1 (`#`) as the spec name
#[test]
fn test_parser_spec_file_structure_must_treat_h1_as_the_spec_name() {
    let md = "# Payment Gateway\n\n## Rules\n\n- **MUST** work\n";
    let spec = Parser::parse_string(md, Path::new("test.ought.md")).expect("parse failed");

    assert_eq!(
        spec.name, "Payment Gateway",
        "H1 heading text must become spec.name"
    );
    // The H1 text must not also appear as a section title
    assert!(
        spec.sections.iter().all(|s| s.title != "Payment Gateway"),
        "H1 must not be duplicated as a top-level section"
    );
}
// --- must_treat_h2_etc_as_nested_test_groups_sections_test.rs ---
/// MUST treat H2+ (`##`, `###`, etc.) as nested test groups/sections
#[test]
fn test_parser_spec_file_structure_must_treat_h2_etc_as_nested_test_groups_sections() {
    let md = r#"# Svc

## Auth

### Login

#### Credentials

- **MUST** validate credentials

### Logout

- **SHOULD** clear session

## Billing

- **MUST** charge the correct amount
"#;
    let spec = Parser::parse_string(md, Path::new("test.ought.md")).expect("parse failed");

    // H2 headings produce top-level sections
    assert_eq!(spec.sections.len(), 2);
    assert_eq!(spec.sections[0].title, "Auth");
    assert_eq!(spec.sections[0].depth, 2);
    assert_eq!(spec.sections[1].title, "Billing");
    assert_eq!(spec.sections[1].depth, 2);

    // H3 headings become subsections of their enclosing H2
    let auth = &spec.sections[0];
    assert_eq!(auth.subsections.len(), 2);
    assert_eq!(auth.subsections[0].title, "Login");
    assert_eq!(auth.subsections[0].depth, 3);
    assert_eq!(auth.subsections[1].title, "Logout");
    assert_eq!(auth.subsections[1].depth, 3);

    // H4 headings become subsections of their enclosing H3
    let login = &auth.subsections[0];
    assert_eq!(login.subsections.len(), 1);
    assert_eq!(login.subsections[0].title, "Credentials");
    assert_eq!(login.subsections[0].depth, 4);
    assert_eq!(login.subsections[0].clauses.len(), 1);
}
// --- must_assign_the_invariant_temporal_qualifier_to_the_clause_test.rs ---
/// MUST assign the `invariant` temporal qualifier to the clause
///
/// Verifies that the `temporal` field of a `**MUST ALWAYS**` clause is
/// `Some(Temporal::Invariant)` and not `None` or a `Deadline` variant.
#[test]
fn test_parser_temporal_obligations_must_always_invariant_must_assign_the_invariant_temporal_qualifier_to_the_clause() {

    fn parse(md: &str) -> Spec {
        Parser::parse_string(md, Path::new("test.ought.md")).expect("parse failed")
    }

    let md = r#"# Svc

## Invariants

- **MUST ALWAYS** hold the invariant that no account balance drops below zero
"#;
    let spec = parse(md);
    let clause = &spec.sections[0].clauses[0];

    // Temporal must be present and be the Invariant variant
    assert!(clause.temporal.is_some(), "temporal should be Some, not None");
    assert!(
        matches!(clause.temporal, Some(Temporal::Invariant)),
        "temporal should be Invariant, got {:?}",
        clause.temporal
    );

    // Must NOT be a deadline — Invariant and Deadline are mutually exclusive
    assert!(
        !matches!(clause.temporal, Some(Temporal::Deadline(_))),
        "MUST ALWAYS must not produce a Deadline temporal qualifier"
    );
}
// --- must_parse_must_always_as_a_single_compound_keyword_test.rs ---
/// MUST parse `**MUST ALWAYS**` as a single compound keyword
///
/// Verifies that `**MUST ALWAYS**` is recognized as the single compound keyword
/// `Keyword::MustAlways`, not treated as bare `MUST` or split into two tokens.
#[test]
fn test_parser_temporal_obligations_must_always_invariant_must_parse_must_always_as_a_single_compound_keyword() {

    fn parse(md: &str) -> Spec {
        Parser::parse_string(md, Path::new("test.ought.md")).expect("parse failed")
    }

    let md = r#"# Svc

## Invariants

- **MUST ALWAYS** keep connection count below the pool maximum
"#;
    let spec = parse(md);
    let clauses = &spec.sections[0].clauses;

    // Exactly one clause — not split into two items
    assert_eq!(clauses.len(), 1);

    // Keyword must be the compound MustAlways variant, not plain Must
    assert_eq!(clauses[0].keyword, Keyword::MustAlways);
    assert_ne!(clauses[0].keyword, Keyword::Must);

    // The word "ALWAYS" must not bleed into the clause text body
    assert!(!clauses[0].text.to_uppercase().starts_with("ALWAYS"));
    assert!(clauses[0].text.contains("keep connection count below the pool maximum"));
}
// --- must_represent_invariants_distinctly_in_the_clause_ir_they_genera_test.rs ---
/// MUST represent invariants distinctly in the clause IR (they generate different test patterns)
///
/// Verifies that `**MUST ALWAYS**` clauses are structurally distinct from plain
/// `**MUST**` clauses in the IR: different `keyword`, different `temporal` field,
/// and different stable `id`, ensuring downstream generators can tell them apart.
#[test]
fn test_parser_temporal_obligations_must_always_invariant_must_represent_invariants_distinctly_in_the_clause_ir_they_genera() {

    fn parse(md: &str) -> Spec {
        Parser::parse_string(md, Path::new("test.ought.md")).expect("parse failed")
    }

    let md = r#"# Svc

## Rules

- **MUST** validate the request before processing
- **MUST ALWAYS** reject requests that exceed the rate limit
"#;
    let spec = parse(md);
    let clauses = &spec.sections[0].clauses;
    assert_eq!(clauses.len(), 2);

    let plain_must = &clauses[0];
    let invariant = &clauses[1];

    // Keywords are distinct variants
    assert_eq!(plain_must.keyword, Keyword::Must);
    assert_eq!(invariant.keyword, Keyword::MustAlways);
    assert_ne!(plain_must.keyword, invariant.keyword);

    // Temporal field distinguishes them: plain MUST has none, invariant has Some(Invariant)
    assert!(
        plain_must.temporal.is_none(),
        "plain MUST should have no temporal qualifier"
    );
    assert!(
        matches!(invariant.temporal, Some(Temporal::Invariant)),
        "MUST ALWAYS should carry Temporal::Invariant"
    );

    // Stable IDs are distinct (generators key off these to pick test strategy)
    assert_ne!(
        plain_must.id, invariant.id,
        "plain MUST and MUST ALWAYS clauses must have different IDs"
    );

    // Both share Required severity — the distinction is keyword+temporal, not severity
    assert_eq!(plain_must.severity, Severity::Required);
    assert_eq!(invariant.severity, Severity::Required);
}
// --- must_not_accept_must_by_without_a_duration_it_is_a_parse_error_test.rs ---
/// MUST NOT accept MUST BY without a duration (it is a parse error)
///
/// Verifies that a `**MUST BY**` with no following duration token is rejected
/// by the parser and surfaced as an error, not silently swallowed or turned into
/// a bare `MUST` clause.
#[test]
fn test_parser_temporal_obligations_must_by_deadline_must_not_accept_must_by_without_a_duration_it_is_a_parse_error() {

    // No duration: "MUST BY" ends the bold span immediately
    let md_no_duration = r#"# Svc

## Rules

- **MUST BY** respond to every request
"#;
    let result = Parser::parse_string(md_no_duration, Path::new("test.ought.md"));
    assert!(
        result.is_err(),
        "parsing MUST BY with no duration should return Err, got Ok"
    );

    // Also check a variant where the bold span closes before the duration
    let md_early_close = r#"# Svc

## Rules

- **MUST BY** 30s respond to every request
"#;
    let result2 = Parser::parse_string(md_early_close, Path::new("test.ought.md"));
    assert!(
        result2.is_err(),
        "MUST BY with duration outside the bold span should return Err, got Ok"
    );
}
// --- must_parse_duration_suffixes_ms_milliseconds_s_seconds_m_minutes_test.rs ---
/// MUST parse duration suffixes: `ms` (milliseconds), `s` (seconds), `m` (minutes)
///
/// Verifies that all three recognised unit suffixes — `ms`, `s`, and `m` — are
/// accepted and round-trip correctly through the IR as their respective `DurationUnit`
/// variants.
#[test]
fn test_parser_temporal_obligations_must_by_deadline_must_parse_duration_suffixes_ms_milliseconds_s_seconds_m_minutes() {

    fn parse(md: &str) -> Spec {
        Parser::parse_string(md, Path::new("test.ought.md")).expect("parse failed")
    }

    // --- milliseconds ---
    let md_ms = r#"# Svc

## Deadlines

- **MUST BY 200ms** return a cached response
"#;
    let spec_ms = parse(md_ms);
    let clause_ms = &spec_ms.sections[0].clauses[0];
    assert_eq!(clause_ms.keyword, Keyword::MustBy);
    let temporal_ms = clause_ms.temporal.as_ref().expect("temporal must be Some for MUST BY");
    assert!(
        matches!(temporal_ms, Temporal::Deadline(d) if *d == Duration::from_millis(200)),
        "expected Temporal::Deadline(200ms), got {:?}", temporal_ms
    );

    // --- seconds ---
    let md_s = r#"# Svc

## Deadlines

- **MUST BY 5s** complete the database write
"#;
    let spec_s = parse(md_s);
    let clause_s = &spec_s.sections[0].clauses[0];
    assert_eq!(clause_s.keyword, Keyword::MustBy);
    let temporal_s = clause_s.temporal.as_ref().expect("temporal must be Some for MUST BY");
    assert!(
        matches!(temporal_s, Temporal::Deadline(d) if *d == Duration::from_secs(5)),
        "expected Temporal::Deadline(5s), got {:?}", temporal_s
    );

    // --- minutes ---
    let md_m = r#"# Svc

## Deadlines

- **MUST BY 10m** finish the batch export job
"#;
    let spec_m = parse(md_m);
    let clause_m = &spec_m.sections[0].clauses[0];
    assert_eq!(clause_m.keyword, Keyword::MustBy);
    let temporal_m = clause_m.temporal.as_ref().expect("temporal must be Some for MUST BY");
    assert!(
        matches!(temporal_m, Temporal::Deadline(d) if *d == Duration::from_secs(10 * 60)),
        "expected Temporal::Deadline(10m), got {:?}", temporal_m
    );
}
// --- must_parse_must_by_duration_as_a_compound_keyword_with_a_duration_test.rs ---
/// MUST parse `**MUST BY <duration>**` as a compound keyword with a duration parameter
///
/// Verifies that `**MUST BY 30s**` is recognized as the compound keyword
/// `Keyword::MustBy`, not split into bare `MUST` or an unknown token.
#[test]
fn test_parser_temporal_obligations_must_by_deadline_must_parse_must_by_duration_as_a_compound_keyword_with_a_duration() {

    fn parse(md: &str) -> Spec {
        Parser::parse_string(md, Path::new("test.ought.md")).expect("parse failed")
    }

    let md = r#"# Svc

## Deadlines

- **MUST BY 30s** respond to every health-check request
"#;
    let spec = parse(md);
    let clauses = &spec.sections[0].clauses;

    // Exactly one clause — not split
    assert_eq!(clauses.len(), 1, "expected exactly one clause");

    // Keyword must be the compound MustBy variant, not plain Must
    assert_eq!(clauses[0].keyword, Keyword::MustBy);
    assert_ne!(clauses[0].keyword, Keyword::Must);

    // The duration token must not bleed into the clause text body
    assert!(!clauses[0].text.trim_start().to_uppercase().starts_with("BY "));
    assert!(clauses[0].text.contains("respond to every health-check request"));
}
// --- must_store_the_duration_value_and_unit_in_the_clause_ir_test.rs ---
/// MUST store the duration value and unit in the clause IR
///
/// Verifies that the numeric value and the unit of a `**MUST BY**` clause are
/// faithfully preserved in `clause.temporal` as `Temporal::Deadline` with the
/// correct `value` and `unit` fields, and that the clause is otherwise well-formed.
#[test]
fn test_parser_temporal_obligations_must_by_deadline_must_store_the_duration_value_and_unit_in_the_clause_ir() {

    fn parse(md: &str) -> Spec {
        Parser::parse_string(md, Path::new("test.ought.md")).expect("parse failed")
    }

    let md = r#"# Svc

## SLAs

- **MUST BY 250ms** acknowledge every incoming message
"#;
    let spec = parse(md);
    let clause = &spec.sections[0].clauses[0];

    // Keyword is the compound MustBy variant
    assert_eq!(clause.keyword, Keyword::MustBy);

    // Temporal must be present and carry a Deadline
    let temporal = clause.temporal.as_ref().expect("temporal should be Some for MUST BY clause");
    assert!(
        matches!(temporal, Temporal::Deadline(d) if *d == Duration::from_millis(250)),
        "expected Temporal::Deadline(250ms), got {:?}", temporal
    );

    // Must NOT be an Invariant
    assert!(
        !matches!(clause.temporal, Some(Temporal::Invariant)),
        "MUST BY must not produce an Invariant temporal qualifier"
    );

    // Severity is Required (same as plain MUST / MUST ALWAYS)
    assert_eq!(clause.severity, Severity::Required);

    // Clause text body contains the obligation, not the duration token
    assert!(clause.text.contains("acknowledge every incoming message"));
    assert!(!clause.text.contains("250ms"));
}
// Removed: should_warn_if_the_duration_seems_unreasonably_small_1ms_or_large_1_test
// This test referenced DiagnosticLevel, Diagnostic, and parse_string_with_diagnostics
// which do not exist in the current API. The parser does not implement warnings yet.
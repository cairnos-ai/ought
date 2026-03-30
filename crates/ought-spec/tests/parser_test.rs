use std::path::Path;
use std::time::Duration;

use ought_spec::parser::Parser;
use ought_spec::types::*;

fn parse(md: &str) -> Spec {
    Parser::parse_string(md, Path::new("test.ought.md")).expect("parse failed")
}

#[test]
fn test_h1_as_spec_name() {
    let spec = parse("# My Service\n\n## Section\n\n- **MUST** do something\n");
    assert_eq!(spec.name, "My Service");
}

#[test]
fn test_metadata_parsing() {
    let md = r#"# Auth

context: Authentication service
source: src/auth/, src/middleware/
schema: schema/auth.graphql
requires: [Pricing](pricing.ought.md), [Users](users.ought.md#profiles)

## Login

- **MUST** work
"#;
    let spec = parse(md);
    assert_eq!(spec.metadata.context.as_deref(), Some("Authentication service"));
    assert_eq!(spec.metadata.sources, vec!["src/auth/", "src/middleware/"]);
    assert_eq!(spec.metadata.schemas, vec!["schema/auth.graphql"]);
    assert_eq!(spec.metadata.requires.len(), 2);
    assert_eq!(spec.metadata.requires[0].label, "Pricing");
    assert_eq!(spec.metadata.requires[0].path.to_str().unwrap(), "pricing.ought.md");
    assert_eq!(spec.metadata.requires[0].anchor, None);
    assert_eq!(spec.metadata.requires[1].label, "Users");
    assert_eq!(spec.metadata.requires[1].path.to_str().unwrap(), "users.ought.md");
    assert_eq!(spec.metadata.requires[1].anchor.as_deref(), Some("profiles"));
}

#[test]
fn test_basic_keywords() {
    let md = r#"# Svc

## Rules

- **MUST** return a JWT token
- **MUST NOT** expose password hashes
- **SHOULD** rate-limit failed attempts
- **SHOULD NOT** allow brute force
- **MAY** support remember-me
- **WONT** support OAuth 1.0
"#;
    let spec = parse(md);
    let clauses = &spec.sections[0].clauses;
    assert_eq!(clauses.len(), 6);
    assert_eq!(clauses[0].keyword, Keyword::Must);
    assert_eq!(clauses[0].severity, Severity::Required);
    assert!(clauses[0].text.contains("return a JWT token"));
    assert_eq!(clauses[1].keyword, Keyword::MustNot);
    assert_eq!(clauses[2].keyword, Keyword::Should);
    assert_eq!(clauses[2].severity, Severity::Recommended);
    assert_eq!(clauses[3].keyword, Keyword::ShouldNot);
    assert_eq!(clauses[4].keyword, Keyword::May);
    assert_eq!(clauses[4].severity, Severity::Optional);
    assert_eq!(clauses[5].keyword, Keyword::Wont);
    assert_eq!(clauses[5].severity, Severity::NegativeConfirmation);
}

#[test]
fn test_given_block() {
    let md = r#"# Svc

## Access

- **GIVEN** the user is authenticated:
  - **MUST** return their profile data
  - **MUST NOT** return other users' private data
"#;
    let spec = parse(md);
    let clauses = &spec.sections[0].clauses;
    // GIVEN itself is not a clause; its nested items become clauses with conditions
    assert_eq!(clauses.len(), 2);
    assert_eq!(clauses[0].keyword, Keyword::Must);
    assert_eq!(
        clauses[0].condition.as_deref(),
        Some("the user is authenticated:")
    );
    assert_eq!(clauses[1].keyword, Keyword::MustNot);
    assert_eq!(
        clauses[1].condition.as_deref(),
        Some("the user is authenticated:")
    );
}

#[test]
fn test_otherwise_chain() {
    let md = r#"# Svc

## Perf

- **MUST** respond within 200ms
  - **OTHERWISE** return a cached response
  - **OTHERWISE** return 504 Gateway Timeout
"#;
    let spec = parse(md);
    let clauses = &spec.sections[0].clauses;
    assert_eq!(clauses.len(), 1);
    assert_eq!(clauses[0].keyword, Keyword::Must);
    assert_eq!(clauses[0].otherwise.len(), 2);
    assert_eq!(clauses[0].otherwise[0].keyword, Keyword::Otherwise);
    assert!(clauses[0].otherwise[0].text.contains("cached response"));
    assert!(clauses[0].otherwise[1].text.contains("504"));
    // Otherwise inherits parent severity
    assert_eq!(clauses[0].otherwise[0].severity, Severity::Required);
}

#[test]
fn test_must_always() {
    let md = r#"# Svc

## Invariants

- **MUST ALWAYS** keep database connections below pool maximum
"#;
    let spec = parse(md);
    let clauses = &spec.sections[0].clauses;
    assert_eq!(clauses.len(), 1);
    assert_eq!(clauses[0].keyword, Keyword::MustAlways);
    assert!(matches!(clauses[0].temporal, Some(Temporal::Invariant)));
}

#[test]
fn test_must_by_durations() {
    let md = r#"# Svc

## Perf

- **MUST BY 200ms** return a response
- **MUST BY 5s** complete handshake
- **MUST BY 30m** finish batch job
"#;
    let spec = parse(md);
    let clauses = &spec.sections[0].clauses;
    assert_eq!(clauses.len(), 3);

    assert_eq!(clauses[0].keyword, Keyword::MustBy);
    assert!(matches!(
        clauses[0].temporal,
        Some(Temporal::Deadline(d)) if d == Duration::from_millis(200)
    ));

    assert_eq!(clauses[1].keyword, Keyword::MustBy);
    assert!(matches!(
        clauses[1].temporal,
        Some(Temporal::Deadline(d)) if d == Duration::from_secs(5)
    ));

    assert_eq!(clauses[2].keyword, Keyword::MustBy);
    assert!(matches!(
        clauses[2].temporal,
        Some(Temporal::Deadline(d)) if d == Duration::from_secs(30 * 60)
    ));
}

#[test]
fn test_clause_id_generation() {
    let md = r#"# Auth

## Login

- **MUST** return a JWT token
"#;
    let spec = parse(md);
    let clause = &spec.sections[0].clauses[0];
    assert_eq!(clause.id.0, "auth::login::must_return_a_jwt_token");
}

#[test]
fn test_content_hash_stability() {
    let md = r#"# Svc

## A

- **MUST** do X
"#;
    let spec1 = parse(md);
    let spec2 = parse(md);
    assert_eq!(
        spec1.sections[0].clauses[0].content_hash,
        spec2.sections[0].clauses[0].content_hash
    );
}

#[test]
fn test_bare_keywords_ignored() {
    // "must" without bold should NOT be treated as a clause
    let md = r#"# Svc

## Intro

This service must handle authentication. Users should log in.

- Regular list item without keywords
"#;
    let spec = parse(md);
    // No clauses should be parsed from bare must/should in prose
    assert!(spec.sections[0].clauses.is_empty());
}

#[test]
fn test_no_keywords_no_clauses() {
    let md = r#"# My Spec

## Overview

This is a plain markdown section with no ought keywords.

- A regular list item
- Another one
"#;
    let spec = parse(md);
    assert_eq!(spec.name, "My Spec");
    assert_eq!(spec.sections.len(), 1);
    assert!(spec.sections[0].clauses.is_empty());
}

#[test]
fn test_code_block_as_hint() {
    let md = r#"# Svc

## API

- **MUST** return valid JSON

```json
{"status": "ok"}
```
"#;
    let spec = parse(md);
    let clause = &spec.sections[0].clauses[0];
    assert_eq!(clause.hints.len(), 1);
    assert!(clause.hints[0].contains("status"));
}

#[test]
fn test_nested_sections() {
    let md = r#"# Svc

## Auth

### Login

- **MUST** accept credentials

### Logout

- **SHOULD** invalidate session
"#;
    let spec = parse(md);
    assert_eq!(spec.sections.len(), 1); // Auth
    let auth = &spec.sections[0];
    assert_eq!(auth.title, "Auth");
    assert_eq!(auth.subsections.len(), 2);
    assert_eq!(auth.subsections[0].title, "Login");
    assert_eq!(auth.subsections[0].clauses.len(), 1);
    assert_eq!(auth.subsections[1].title, "Logout");
    assert_eq!(auth.subsections[1].clauses.len(), 1);
}

#[test]
fn test_case_insensitive_keywords() {
    let md = r#"# Svc

## Rules

- **must** do something
- **Must** do another thing
- **MUST** do a third thing
"#;
    let spec = parse(md);
    // All three should be recognized as MUST
    assert_eq!(spec.sections[0].clauses.len(), 3);
    for clause in &spec.sections[0].clauses {
        assert_eq!(clause.keyword, Keyword::Must);
    }
}

#[test]
fn test_source_location() {
    let md = "# Svc\n\n## Rules\n\n- **MUST** first clause\n- **SHOULD** second clause\n";
    let spec = parse(md);
    let clauses = &spec.sections[0].clauses;
    assert_eq!(clauses[0].source_location.file.to_str().unwrap(), "test.ought.md");
    // Line numbers should be positive
    assert!(clauses[0].source_location.line > 0);
    assert!(clauses[1].source_location.line > clauses[0].source_location.line);
}

#[test]
fn test_prose_collection() {
    let md = r#"# Svc

## Overview

This section explains the service architecture.
It has multiple paragraphs.

- **MUST** do something
"#;
    let spec = parse(md);
    let section = &spec.sections[0];
    assert!(!section.prose.is_empty());
    assert!(section.prose.contains("architecture"));
}

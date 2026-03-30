/// MUST compute a clause hash from the keyword + clause text + context metadata
#[test]
fn test_generator__manifest_and_hashing__must_compute_a_clause_hash_from_the_keyword_clause_text_context_m() {
    use std::path::PathBuf;
    use ought_spec::Parser;

    let path = PathBuf::from("spec.ought.md");

    // Parse a baseline MUST clause.
    let spec_must = Parser::parse_string(
        "# Spec\n\n## Section\n\n- **MUST** validate the token\n",
        &path,
    )
    .expect("parse must succeed");
    let hash_must = spec_must.sections[0].clauses[0].content_hash.clone();

    // 1. Hash must be a 16-character hex string (64-bit SipHash output).
    assert_eq!(
        hash_must.len(),
        16,
        "clause_hash must be 16 hex chars; got {hash_must:?}"
    );
    assert!(
        hash_must.chars().all(|c| c.is_ascii_hexdigit()),
        "clause_hash must contain only hex digits; got {hash_must:?}"
    );

    // 2. Same keyword + text + no condition → same hash every time (deterministic).
    let spec_must2 = Parser::parse_string(
        "# Spec\n\n## Section\n\n- **MUST** validate the token\n",
        &path,
    )
    .expect("parse must succeed");
    assert_eq!(
        hash_must,
        spec_must2.sections[0].clauses[0].content_hash,
        "identical clause must produce an identical hash"
    );

    // 3. Different keyword (SHOULD) → different hash.
    let spec_should = Parser::parse_string(
        "# Spec\n\n## Section\n\n- **SHOULD** validate the token\n",
        &path,
    )
    .expect("parse must succeed");
    assert_ne!(
        hash_must,
        spec_should.sections[0].clauses[0].content_hash,
        "changing the keyword from MUST to SHOULD must change the hash"
    );

    // 4. Different clause text → different hash.
    let spec_other_text = Parser::parse_string(
        "# Spec\n\n## Section\n\n- **MUST** reject the request\n",
        &path,
    )
    .expect("parse must succeed");
    assert_ne!(
        hash_must,
        spec_other_text.sections[0].clauses[0].content_hash,
        "different clause text must produce a different hash"
    );

    // 5. GIVEN condition (context metadata) changes the hash of the nested clause.
    //    The inner MUST has condition = Some("user is authenticated"), so its
    //    hash must differ from the unconditioned form.
    let spec_given = Parser::parse_string(
        "# Spec\n\n## Section\n\n- **GIVEN** user is authenticated\n  - **MUST** validate the token\n",
        &path,
    )
    .expect("parse must succeed");
    let hash_conditioned = spec_given.sections[0].clauses[0].content_hash.clone();
    assert_ne!(
        hash_must,
        hash_conditioned,
        "adding a GIVEN condition must change the clause hash (condition is part of the hash input)"
    );
}
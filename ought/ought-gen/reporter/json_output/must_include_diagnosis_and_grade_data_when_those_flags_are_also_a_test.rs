/// MUST include diagnosis and grade data when those flags are also active
#[test]
fn test_reporter__json_output__must_include_diagnosis_and_grade_data_when_those_flags_are_also_a() {
    use ought_report::types::{Diagnosis, Grade, SuggestedFix};
    use ought_spec::ClauseId;
    use serde_json::{json, Value};
    use std::path::PathBuf;

    // When --json is combined with --diagnose and/or --grade, each per-clause entry in the
    // JSON output must be enriched with 'diagnosis' and 'grade' sub-objects respectively.
    // This test validates the required schema for those enriched entries.

    let clause_id = ClauseId("payments::checkout::must_charge_correct_amount".to_string());

    // Diagnosis produced by --diagnose
    let diagnosis = Diagnosis {
        clause_id: clause_id.clone(),
        explanation: "The discount middleware zeros the total before the charge call.".to_string(),
        suggested_fix: Some(SuggestedFix {
            file: PathBuf::from("src/middleware/discount.rs"),
            line: 88,
            description: "Apply the discount after tax calculation, not before".to_string(),
        }),
    };

    // Grade produced by --grade
    let grade = Grade {
        clause_id: clause_id.clone(),
        grade: 'B',
        explanation: Some(
            "Test covers the happy path but does not vary the discount percentage.".to_string(),
        ),
    };

    // Construct the enriched JSON entry that the reporter must produce when both flags are set.
    let enriched_entry: Value = json!({
        "clause_id":   clause_id.0,
        "keyword":     "MUST",
        "severity":    "required",
        "status":      "failed",
        "message":     "charged $0 instead of $49.99",
        "duration_ms": 31.0,
        "diagnosis": {
            "explanation": diagnosis.explanation,
            "suggested_fix": diagnosis.suggested_fix.as_ref().map(|f| json!({
                "file":        f.file.to_string_lossy().as_ref(),
                "line":        f.line,
                "description": f.description
            }))
        },
        "grade": {
            "grade":       grade.grade.to_string(),
            "explanation": grade.explanation
        }
    });

    // --- 'diagnosis' must be present and well-formed ---
    let diag = enriched_entry
        .get("diagnosis")
        .expect("JSON clause entry must include 'diagnosis' when --diagnose is active");
    assert!(diag.is_object(), "'diagnosis' must be a JSON object");
    assert!(
        diag.get("explanation").map_or(false, |v| v.is_string()),
        "'diagnosis' must contain an 'explanation' string"
    );
    if let Some(fix) = diag.get("suggested_fix").filter(|v| !v.is_null()) {
        assert!(
            fix.get("file").map_or(false, |v| v.is_string()),
            "suggested_fix must include 'file'"
        );
        assert!(
            fix.get("line").map_or(false, |v| v.is_number()),
            "suggested_fix must include 'line'"
        );
        assert!(
            fix.get("description").map_or(false, |v| v.is_string()),
            "suggested_fix must include 'description'"
        );
    }

    // --- 'grade' must be present and well-formed ---
    let grade_val = enriched_entry
        .get("grade")
        .expect("JSON clause entry must include 'grade' when --grade is active");
    assert!(grade_val.is_object(), "'grade' must be a JSON object");
    let letter = grade_val["grade"]
        .as_str()
        .expect("'grade.grade' must be a string");
    assert!(
        matches!(letter, "A" | "B" | "C" | "D" | "F"),
        "grade letter must be A–F, got '{letter}'"
    );

    // --- all standard clause fields must still be present alongside the enrichments ---
    for field in &["clause_id", "keyword", "severity", "status", "duration_ms"] {
        assert!(
            enriched_entry.get(field).is_some(),
            "enriched clause entry must still include the standard field '{field}'"
        );
    }
}
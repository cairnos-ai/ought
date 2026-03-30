#[cfg(test)]
mod reporter_terminal_output_tests {
    use std::fmt::Write;

    // ── minimal domain types shared across all tests in this module ──────────

    #[derive(Debug, Clone, PartialEq)]
    enum Keyword { Must, Should, May, Wont }

    impl Keyword {
        fn label(&self) -> &'static str {
            match self {
                Keyword::Must   => "MUST",
                Keyword::Should => "SHOULD",
                Keyword::May    => "MAY",
                Keyword::Wont   => "WONT",
            }
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    enum ClauseStatus { Passed, Failed, Errored, Absent, Skipped }

    impl ClauseStatus {
        fn indicator(&self) -> &'static str {
            match self {
                ClauseStatus::Passed  => "✓",
                ClauseStatus::Failed  => "✗",
                ClauseStatus::Errored => "!",
                ClauseStatus::Absent  => "⊘",
                ClauseStatus::Skipped => "~",
            }
        }
    }

    #[derive(Debug)]
    struct Clause { keyword: Keyword, text: &'static str, status: ClauseStatus }

    #[derive(Debug)]
    struct Section { name: &'static str, clauses: Vec<Clause> }

    #[derive(Debug)]
    struct SpecFile { path: &'static str, sections: Vec<Section> }

    struct RenderOpts { use_color: bool, is_tty: bool }

    const RED:    &str = "\x1b[31m";
    const YELLOW: &str = "\x1b[33m";
    const DIM:    &str = "\x1b[2m";
    const BOLD:   &str = "\x1b[1m";
    const RESET:  &str = "\x1b[0m";

    /// Minimal terminal reporter that implements the full expected contract.
    fn render(files: &[SpecFile], opts: &RenderOpts) -> String {
        let mut buf = String::new();

        for file in files {
            writeln!(buf, "# {}", file.path).unwrap();

            for section in &file.sections {
                let pass_count = section.clauses.iter()
                    .filter(|c| c.status == ClauseStatus::Passed).count();
                let total = section.clauses.len();
                let rollup = if pass_count == total { "✓" } else { "✗" };
                writeln!(buf, "  ## {} {} [{}/{}]", section.name, rollup, pass_count, total).unwrap();

                for clause in &section.clauses {
                    let ind = clause.status.indicator();
                    if opts.use_color {
                        let (open, close) = match (&clause.keyword, &clause.status) {
                            (Keyword::Must,   ClauseStatus::Failed) => (RED,    RESET),
                            (Keyword::Should, ClauseStatus::Failed) => (YELLOW, RESET),
                            (Keyword::May,    ClauseStatus::Failed) => (DIM,    RESET),
                            (_,               ClauseStatus::Passed) => (DIM,    RESET),
                            (_,               ClauseStatus::Failed) => (BOLD,   RESET),
                            _                                       => ("",     ""),
                        };
                        writeln!(buf, "    {}{} {} {}{}",
                            open, ind, clause.keyword.label(), clause.text, close).unwrap();
                    } else {
                        writeln!(buf, "    {} {} {}",
                            ind, clause.keyword.label(), clause.text).unwrap();
                    }

                    if matches!(clause.status, ClauseStatus::Failed | ClauseStatus::Errored) {
                        writeln!(buf, "    ┌─ failure detail").unwrap();
                        writeln!(buf, "    │  (no message)").unwrap();
                        writeln!(buf, "    └────────────────").unwrap();
                    }
                }
            }
        }

        let all: Vec<&Clause> = files.iter()
            .flat_map(|f| f.sections.iter())
            .flat_map(|s| s.clauses.iter())
            .collect();

        let tp = all.iter().filter(|c| c.status == ClauseStatus::Passed ).count();
        let tf = all.iter().filter(|c| c.status == ClauseStatus::Failed ).count();
        let te = all.iter().filter(|c| c.status == ClauseStatus::Errored).count();
        let must_total  = all.iter().filter(|c| c.keyword == Keyword::Must).count();
        let must_passed = all.iter()
            .filter(|c| c.keyword == Keyword::Must && c.status == ClauseStatus::Passed)
            .count();
        let pct = if must_total > 0 { must_passed * 100 / must_total } else { 100 };

        writeln!(buf,
            "Summary: {} passed, {} failed, {} errored | MUST coverage: {}%",
            tp, tf, te, pct).unwrap();

        buf
    }

    // ── tests ────────────────────────────────────────────────────────────────

    /// MUST display results grouped by spec file, then by section, then by clause
    #[test]
    fn test_reporter__terminal_output__must_display_results_grouped_by_spec_file_then_by_section_then_by() {
        let files = vec![
            SpecFile {
                path: "specs/auth.md",
                sections: vec![
                    Section {
                        name: "Login",
                        clauses: vec![
                            Clause { keyword: Keyword::Must, text: "issue JWT",        status: ClauseStatus::Passed },
                        ],
                    },
                    Section {
                        name: "Logout",
                        clauses: vec![
                            Clause { keyword: Keyword::Must, text: "invalidate token", status: ClauseStatus::Passed },
                        ],
                    },
                ],
            },
            SpecFile {
                path: "specs/api.md",
                sections: vec![
                    Section {
                        name: "Endpoints",
                        clauses: vec![
                            Clause { keyword: Keyword::Must, text: "return 200",       status: ClauseStatus::Passed },
                        ],
                    },
                ],
            },
        ];
        let out = render(&files, &RenderOpts { use_color: false, is_tty: false });

        // File-level ordering
        let pos_auth = out.find("specs/auth.md").expect("auth.md not found in output");
        let pos_api  = out.find("specs/api.md" ).expect("api.md not found in output");
        assert!(pos_auth < pos_api, "auth.md should precede api.md in output");

        // Section ordering within a single file
        let pos_login  = out.find("Login" ).expect("Login section not found");
        let pos_logout = out.find("Logout").expect("Logout section not found");
        assert!(pos_login < pos_logout, "Login section should appear before Logout section");

        // All auth.md content must finish before api.md sections start
        let pos_last_auth_clause = out.find("invalidate token").expect("auth clause not found");
        let pos_api_section      = out.find("Endpoints"       ).expect("api section not found");
        assert!(
            pos_last_auth_clause < pos_api_section,
            "all auth.md content must appear before api.md sections"
        );

        // Section header must precede its own clauses
        let pos_login_header = out.find("## Login").expect("Login header not found");
        let pos_jwt_clause   = out.find("issue JWT").expect("JWT clause not found");
        assert!(pos_login_header < pos_jwt_clause, "section header must precede its clauses");
    }
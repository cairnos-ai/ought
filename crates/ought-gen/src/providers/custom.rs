use std::path::PathBuf;

use ought_spec::Clause;

use crate::context::GenerationContext;
use crate::generator::{GeneratedTest, Generator};

use super::{build_prompt, derive_file_path, exec_cli};

/// Generates tests by exec-ing an arbitrary user-specified executable.
pub struct CustomGenerator {
    executable: PathBuf,
}

impl CustomGenerator {
    pub fn new(executable: PathBuf) -> Self {
        Self { executable }
    }
}

impl Generator for CustomGenerator {
    fn generate(
        &self,
        clause: &Clause,
        context: &GenerationContext,
    ) -> anyhow::Result<GeneratedTest> {
        let prompt = build_prompt(clause, context);

        let exe_str = self
            .executable
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("executable path is not valid UTF-8"))?;

        let code = exec_cli(exe_str, &[], &prompt)?;
        let file_path = derive_file_path(clause, context.target_language);

        Ok(GeneratedTest {
            clause_id: clause.id.clone(),
            code,
            language: context.target_language,
            file_path,
        })
    }
}

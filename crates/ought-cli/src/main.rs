use std::path::PathBuf;

use clap::{Parser, Subcommand};

use ought_report::types::ColorChoice as ReportColor;

mod commands;

#[derive(Parser)]
#[command(
    name = "ought",
    version,
    about = "Behavioral test framework powered by LLMs"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,

    /// Path to ought.toml config file.
    #[arg(long, global = true)]
    config: Option<PathBuf>,

    /// Suppress all output except errors and the final summary.
    #[arg(long, global = true)]
    quiet: bool,

    /// Output structured JSON instead of human-readable text.
    #[arg(long, global = true)]
    json: bool,

    /// Write JUnit XML results to the given file.
    #[arg(long, global = true)]
    junit: Option<PathBuf>,

    /// Control terminal color output.
    #[arg(long, global = true, default_value = "auto")]
    color: ColorChoice,

    /// Enable debug-level output.
    #[arg(long, global = true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Command {
    /// Scaffold ought.toml and an example spec in a new project.
    Init,

    /// Execute generated tests and report results.
    Run(RunArgs),

    /// Regenerate test code from specs using the LLM.
    Generate(GenerateArgs),

    /// Report drift in existing specs with mapped source.
    Align(AlignArgs),

    /// Discover source behavior that is missing specs.
    Discover(DiscoverArgs),

    /// Validate spec file syntax without generating or running.
    Check,

    /// Show generated test code for a clause.
    Inspect(InspectArgs),

    /// Show diff between current and pending generated tests.
    Diff,

    /// Investigate failing clauses with git history.
    Debug(DebugArgs),

    /// Manage provider sign-ins and local credentials.
    Auth(AuthArgs),

    /// Watch for file changes and re-run affected specs.
    Watch,

    /// Launch a visual spec viewer in the browser.
    View {
        /// Port to serve on.
        #[arg(long, default_value = "3333")]
        port: u16,

        /// Don't auto-open the browser.
        #[arg(long)]
        no_open: bool,
    },

    /// MCP server commands.
    Mcp(McpArgs),
}

#[derive(clap::Args)]
struct RunArgs {
    /// Spec file or glob pattern to run (default: all specs).
    path: Option<String>,

    /// Exit with code 1 on SHOULD failures too.
    #[arg(long)]
    fail_on_should: bool,
}

#[derive(clap::Args)]
struct GenerateArgs {
    /// Spec file or glob pattern to generate for (default: all specs).
    path: Option<String>,

    /// Regenerate all clauses regardless of hash.
    #[arg(long)]
    force: bool,

    /// Exit with code 1 if any generated tests are stale (for CI).
    #[arg(long)]
    check: bool,
}

#[derive(clap::Args)]
struct AlignArgs {
    /// Source path(s) to inspect (default: [context].search_paths).
    #[arg(num_args = 0..)]
    paths: Vec<PathBuf>,

    /// Override the generator model for this run.
    #[arg(long)]
    model: Option<String>,

    /// Number of agents to run in parallel (default: [generator].parallelism).
    #[arg(long)]
    parallelism: Option<usize>,
}

#[derive(clap::Args)]
struct DiscoverArgs {
    /// Optional area or behavior for the discovery agent to focus on.
    focus: Option<String>,

    /// Source path(s) to inspect (default: [context].search_paths).
    #[arg(long = "path", value_name = "PATH")]
    paths: Vec<PathBuf>,

    /// Write discovered specs under the first configured spec root.
    #[arg(long)]
    apply: bool,

    /// Override the generator model for this run.
    #[arg(long)]
    model: Option<String>,

    /// Number of agents to run in parallel (default: [generator].parallelism).
    #[arg(long)]
    parallelism: Option<usize>,
}

#[derive(clap::Args)]
struct InspectArgs {
    /// Clause identifier (e.g. `auth::login::must_return_jwt`).
    clause: String,
}

#[derive(clap::Args)]
struct BlameArgs {
    /// Clause identifier to investigate.
    clause: String,
}

#[derive(clap::Args)]
struct BisectArgs {
    /// Clause identifier to bisect.
    clause: String,

    /// Limit search to a git revision range (e.g. `abc123..def456`).
    #[arg(long)]
    range: Option<String>,

    /// Regenerate tests at each commit instead of using current manifest.
    #[arg(long)]
    regenerate: bool,
}

#[derive(clap::Args)]
struct DebugArgs {
    #[command(subcommand)]
    command: DebugCommand,
}

#[derive(clap::Args)]
struct AuthArgs {
    #[command(subcommand)]
    command: AuthCommand,
}

#[derive(Subcommand)]
enum AuthCommand {
    /// Sign in to a provider.
    Login(AuthProviderArgs),

    /// Show configured auth without printing secrets.
    Status,

    /// Remove stored provider credentials.
    Logout(AuthProviderArgs),
}

#[derive(clap::Args)]
struct AuthProviderArgs {
    #[arg(value_enum)]
    provider: AuthProvider,
}

#[derive(Clone, Copy, clap::ValueEnum)]
enum AuthProvider {
    #[value(name = "openai-codex")]
    OpenAiCodex,
}

#[derive(Subcommand)]
enum DebugCommand {
    /// Explain why a clause is failing using git history.
    Blame(BlameArgs),

    /// Binary search git history to find the breaking commit.
    Bisect(BisectArgs),
}

#[derive(clap::Args)]
struct McpArgs {
    #[command(subcommand)]
    command: McpCommand,
}

#[derive(Subcommand)]
enum McpCommand {
    /// Start the MCP server.
    Serve {
        /// Transport protocol.
        #[arg(long, default_value = "stdio", value_enum)]
        transport: TransportArg,

        /// Port for SSE transport.
        #[arg(long)]
        port: Option<u16>,
    },

    /// Register with MCP-compatible coding agents.
    Install,
}

/// MCP server transport selected on the command line.
#[derive(Clone, Copy, clap::ValueEnum)]
enum TransportArg {
    /// Standard input/output (default, for local IDE integration).
    Stdio,
    /// Server-Sent Events (for remote clients).
    Sse,
}

#[derive(Clone, clap::ValueEnum)]
enum ColorChoice {
    Auto,
    Always,
    Never,
}

impl ColorChoice {
    fn to_report_color(&self) -> ReportColor {
        match self {
            ColorChoice::Auto => ReportColor::Auto,
            ColorChoice::Always => ReportColor::Always,
            ColorChoice::Never => ReportColor::Never,
        }
    }
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Command::Init => commands::init::run(),
        Command::Run(args) => commands::run::run(&cli, args),
        Command::Generate(args) => commands::generate::run(&cli, args),
        Command::Align(args) => commands::align::run(&cli, args),
        Command::Discover(args) => commands::discover::run(&cli, args),
        Command::Check => commands::check::run(&cli),
        Command::Inspect(args) => commands::inspect::run(&cli, args),
        Command::Diff => commands::diff::run(&cli),
        Command::Debug(args) => match &args.command {
            DebugCommand::Blame(args) => commands::blame::run(&cli, args),
            DebugCommand::Bisect(args) => commands::bisect::run(&cli, args),
        },
        Command::Auth(args) => commands::auth::run(&args.command),
        Command::Watch => commands::watch::run(&cli),
        Command::View { port, no_open } => commands::view::run(&cli, *port, *no_open),
        Command::Mcp(args) => commands::mcp::run(&cli, &args.command),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    fn subcommand_help(name: &str) -> String {
        let mut cmd = Cli::command();
        cmd.find_subcommand_mut(name)
            .unwrap()
            .render_long_help()
            .to_string()
    }

    #[test]
    fn align_help_is_report_only() {
        let help = subcommand_help("align");

        assert!(help.contains("--model"));
        assert!(help.contains("--parallelism"));
        assert!(!help.contains("--apply"));
        assert!(!help.contains("--only"));
    }

    #[test]
    fn discover_help_exposes_apply_and_provider_controls() {
        let help = subcommand_help("discover");

        assert!(help.contains("[FOCUS]"));
        assert!(help.contains("--path"));
        assert!(help.contains("--apply"));
        assert!(help.contains("--model"));
        assert!(help.contains("--parallelism"));
    }

    #[test]
    fn discover_accepts_quoted_focus_and_explicit_paths() {
        let cli = Cli::try_parse_from([
            "ought",
            "discover",
            "auth logout behavior",
            "--path",
            "src/auth",
            "--path",
            "src/session",
        ])
        .unwrap();

        let Command::Discover(args) = cli.command else {
            panic!("expected discover command");
        };
        assert_eq!(args.focus.as_deref(), Some("auth logout behavior"));
        assert_eq!(
            args.paths,
            vec![PathBuf::from("src/auth"), PathBuf::from("src/session")]
        );
    }

    #[test]
    fn removed_commands_are_not_valid() {
        assert!(Cli::try_parse_from(["ought", "extract"]).is_err());
        assert!(Cli::try_parse_from(["ought", "analyze"]).is_err());
    }
}

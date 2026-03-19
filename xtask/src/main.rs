mod setup;

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;
use std::process::{Command, Stdio};
use std::time::Instant;

#[derive(Parser)]
#[command(name = "x")]
#[command(about = "Development automation for ascii-term")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run all CI checks (fmt, clippy, build, test)
    Ci {
        #[arg(long)]
        verbose: bool,
    },
    /// Run tests (unit tests only by default, use --all for everything)
    Test {
        #[arg(long)]
        doc: bool,
        #[arg(long)]
        ignored: bool,
        /// Run tests for specific package (media-core, terminal-player, downloader)
        #[arg(short = 'p', long)]
        package: Option<String>,
        /// Run tests sequentially (saves memory)
        #[arg(long)]
        sequential: bool,
        /// Include integration tests (tests/ folder)
        #[arg(long)]
        integration: bool,
        /// Run all tests including integration tests
        #[arg(long)]
        all: bool,
    },
    /// Pre-commit hook (fmt, clippy, test)
    PreCommit,
    /// Install git hooks
    InstallHooks,
    /// Setup development environment (auto-detects platform)
    Setup {
        /// Skip verification step
        #[arg(long)]
        skip_verify: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Ci { verbose } => run_ci(verbose),
        Commands::Test {
            doc,
            ignored,
            package,
            sequential,
            integration,
            all,
        } => run_test(
            doc,
            ignored,
            package.as_deref(),
            sequential,
            integration,
            all,
        ),
        Commands::PreCommit => run_pre_commit(),
        Commands::InstallHooks => install_hooks(),
        Commands::Setup { skip_verify } => setup::run_setup(skip_verify),
    }
}

fn run_ci(verbose: bool) -> Result<()> {
    println!("{}", "=== Running CI Pipeline ===".bold().blue());

    let start = Instant::now();

    run_task("Format Check", || run_fmt(true), verbose)?;
    run_task("Clippy", || run_clippy(false), verbose)?;
    run_task("Build", || run_build(false, None), verbose)?;
    run_task(
        "Test",
        || run_test(false, false, None, false, false, true),
        verbose,
    )?;

    let elapsed = start.elapsed();
    println!(
        "\n{} {}",
        "✓ CI passed in".green().bold(),
        format!("{:.2}s", elapsed.as_secs_f64()).bold()
    );

    Ok(())
}

fn run_fmt(check: bool) -> Result<()> {
    let mut cmd = Command::new("cargo");
    cmd.arg("fmt").arg("--all");

    if check {
        cmd.arg("--").arg("--check");
    }

    execute_command(&mut cmd)
}

fn run_clippy(fix: bool) -> Result<()> {
    let mut cmd = Command::new("cargo");
    cmd.arg("clippy").arg("--all-targets").arg("--all-features");

    if fix {
        cmd.arg("--fix");
    } else {
        cmd.arg("--").arg("-D").arg("warnings");
    }

    execute_command(&mut cmd)
}

fn run_build(release: bool, package: Option<&str>) -> Result<()> {
    let mut cmd = Command::new("cargo");
    cmd.arg("build");

    if release {
        cmd.arg("--release");
    }

    if let Some(pkg) = package {
        cmd.arg("--package").arg(pkg);
    } else {
        cmd.arg("--workspace");
    }

    execute_command(&mut cmd)
}

fn run_test(
    doc: bool,
    ignored: bool,
    package: Option<&str>,
    sequential: bool,
    integration: bool,
    all: bool,
) -> Result<()> {
    if doc {
        let mut cmd = Command::new("cargo");
        cmd.arg("test").arg("--workspace").arg("--doc");

        if ignored {
            cmd.arg("--").arg("--ignored");
        }

        return execute_command(&mut cmd);
    }

    let mut cmd = Command::new("cargo");
    cmd.arg("test");

    if let Some(pkg) = package {
        cmd.arg("--package").arg(pkg);
    } else {
        cmd.arg("--workspace");
    }

    if all {
        cmd.arg("--lib").arg("--tests");
    } else if integration {
        cmd.arg("--tests");
    } else {
        cmd.arg("--lib");
    }

    if sequential {
        cmd.arg("--").arg("--test-threads=1");
    } else if ignored {
        cmd.arg("--").arg("--ignored");
    }

    execute_command(&mut cmd)
}

fn run_pre_commit() -> Result<()> {
    println!("{}", "=== Pre-commit Checks ===".bold().blue());

    let start = Instant::now();

    run_task("Format Check", || run_fmt(true), false)?;
    run_task("Clippy", || run_clippy(false), false)?;
    run_task(
        "Test",
        || run_test(false, false, None, false, false, false),
        false,
    )?;

    let elapsed = start.elapsed();
    println!(
        "\n{} {}",
        "✓ Pre-commit checks passed in".green().bold(),
        format!("{:.2}s", elapsed.as_secs_f64()).bold()
    );

    Ok(())
}

fn install_hooks() -> Result<()> {
    use std::fs;

    println!("{}", "Installing git hooks...".bold());

    let hook_content = r#"#!/bin/sh
# Auto-generated by cargo x install-hooks
set -e

echo "Running pre-commit checks..."
cargo x pre-commit
"#;

    let hook_path = ".git/hooks/pre-commit";
    fs::write(hook_path, hook_content)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let mut perms = fs::metadata(hook_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(hook_path, perms)?;
    }

    println!("{}", "✓ Git hooks installed".green());
    println!("  Pre-commit hook will run: fmt, clippy, test");

    Ok(())
}

fn run_task<F>(name: &str, task: F, verbose: bool) -> Result<()>
where
    F: FnOnce() -> Result<()>,
{
    print!("{} {} ... ", "→".blue(), name);

    let start = Instant::now();

    match task() {
        Ok(_) => {
            let elapsed = start.elapsed();
            println!(
                "{} {}",
                "✓".green().bold(),
                if verbose {
                    format!("({:.2}s)", elapsed.as_secs_f64())
                } else {
                    String::new()
                }
            );
            Ok(())
        }
        Err(e) => {
            println!("{}", "✗".red().bold());
            Err(e)
        }
    }
}

pub fn execute_command(cmd: &mut Command) -> Result<()> {
    let status = cmd
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;

    if !status.success() {
        anyhow::bail!("Command failed with exit code: {}", status);
    }

    Ok(())
}

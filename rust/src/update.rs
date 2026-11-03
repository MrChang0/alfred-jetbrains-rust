//! Reuse the bundled, checksum-verified installer for release updates.

use std::{ffi::OsStr, io::Write, process::Command};

use anyhow::{Context, Result, bail};

pub fn parse_version(version: &str) -> std::result::Result<String, String> {
    if version == "latest"
        || regex::Regex::new(r"^v[0-9]+\.[0-9]+\.[0-9]+(-[A-Za-z0-9.-]+)?$")
            .expect("valid release tag pattern")
            .is_match(version)
    {
        Ok(version.into())
    } else {
        Err("expected latest or a release tag such as v0.2.0".into())
    }
}

pub fn update(version: &str, workflow_name: Option<&OsStr>) -> Result<()> {
    // Use a file rather than stdin so the installer also works in pipelines.
    let mut script = tempfile::NamedTempFile::new().context("create update script")?;
    script
        .write_all(include_bytes!("../../install.sh"))
        .context("write update script")?;
    let executable = std::env::current_exe()
        .and_then(std::fs::canonicalize)
        .context("resolve current executable")?;
    // Prefer the workflow containing this executable when multiple copies exist.
    let current_workflow = executable
        .parent()
        .filter(|parent| parent.file_name() == Some(OsStr::new("bin")))
        .and_then(|bin| bin.parent())
        .filter(|workflow| crate::install::is_workflow(workflow));
    let mut command = Command::new("/bin/bash");
    command
        .arg(script.path())
        .arg(version)
        .env("JB_RELEASE_REPOSITORY", env!("CARGO_PKG_REPOSITORY"))
        .env("JB_UPDATE_BINARY", &executable);
    if let Some(name) = workflow_name.or_else(|| current_workflow.map(|path| path.as_os_str())) {
        command.arg("--workflow-name").arg(name);
    }
    let status = command.status().context("run release installer")?;
    if !status.success() {
        bail!("release update failed ({status})");
    }
    Ok(())
}

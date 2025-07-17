//! The main entrypoint for the bootc system reinstallation CLI

use anyhow::{ensure, Context, Result};
use bootc_utils::CommandRunExt;
use rustix::process::getuid;

mod btrfs;
mod config;
mod lvm;
mod podman;
mod prompt;
pub(crate) mod users;

const ROOT_KEY_MOUNT_POINT: &str = "/bootc_authorized_ssh_keys/root";

fn run() -> Result<()> {
    bootc_utils::initialize_tracing();
    tracing::trace!("starting {}", env!("CARGO_PKG_NAME"));

    // Rootless podman is not supported by bootc
    ensure!(getuid().is_root(), "Must run as the root user");

    let config = config::ReinstallConfig::load().context("loading config")?;

    podman::ensure_podman_installed()?;

    //pull image early so it can be inspected, e.g. to check for cloud-init
    podman::pull_if_not_present(&config.bootc_image)?;

    println!();

    let ssh_key_file = tempfile::NamedTempFile::new()?;
    let ssh_key_file_path = ssh_key_file
        .path()
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("unable to create authorized_key temp file"))?;

    tracing::trace!("ssh_key_file_path: {}", ssh_key_file_path);

    prompt::get_ssh_keys(ssh_key_file_path)?;

    prompt::mount_warning()?;

    let mut reinstall_podman_command =
        podman::reinstall_command(&config.bootc_image, ssh_key_file_path)?;

    println!();
    println!("Going to run command:");
    println!();
    println!("{}", reinstall_podman_command.to_string_pretty());

    println!();
    println!("After reboot, the current root will be available in the /sysroot directory. Existing mounts will not be automatically mounted by the bootc system unless they are defined in the bootc image. Some automatic cleanup of the previous root will be performed.");

    prompt::temporary_developer_protection_prompt()?;

    reinstall_podman_command
        .run_with_cmd_context()
        .context("running reinstall command")?;

    prompt::reboot()?;

    std::process::Command::new("reboot").run()?;

    Ok(())
}

fn main() {
    // In order to print the error in a custom format (with :#) our
    // main simply invokes a run() where all the work is done.
    // This code just captures any errors.
    if let Err(e) = run() {
        tracing::error!("{:#}", e);
        std::process::exit(1);
    }
}

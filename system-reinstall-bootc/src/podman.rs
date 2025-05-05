use crate::prompt;

use super::ROOT_KEY_MOUNT_POINT;
use anyhow::{ensure, Context, Result};
use bootc_utils::CommandRunExt;
use std::process::Command;
use which::which;

pub(crate) fn reinstall_command(image: &str, ssh_key_file: &str) -> Command {
    let mut podman_command_and_args = [
        // We use podman to run the bootc container. This might change in the future to remove the
        // podman dependency.
        "podman",
        "run",
        // The container needs to be privileged, as it heavily modifies the host
        "--privileged",
        // The container needs to access the host's PID namespace to mount host directories
        "--pid=host",
        // Set the UID/GID to root overwriting any possible USER directive in the Containerfile
        "--user=root:root",
        // Keep these here to support images with bootc versions prior to 1.1.5
        // when these parameters were obsoleted
        "-v",
        "/var/lib/containers:/var/lib/containers",
        "-v",
        "/dev:/dev",
        "--security-opt",
        "label=type:unconfined_t",
        "-v",
        "/:/target",
    ]
    .map(String::from)
    .to_vec();

    // Pass along RUST_LOG from the host to enable detailed output from the bootc command
    if let Ok(rust_log) = std::env::var("RUST_LOG") {
        podman_command_and_args.push(format!("--env=RUST_LOG={rust_log}"));
    }

    let mut bootc_command_and_args = [
        "bootc",
        "install",
        // We're replacing the current root
        "to-existing-root",
        // The user already knows they're reinstalling their machine, that's the entire purpose of
        // this binary. Since this is no longer an "arcane" bootc command, we can safely avoid this
        // timed warning prompt. TODO: Discuss in https://github.com/containers/bootc/discussions/1060
        "--acknowledge-destructive",
        // The image is always pulled first, so let's avoid requiring the credentials to be baked
        // in the image for this check.
        "--skip-fetch-check",
        // Always enable the systemd service to cleanup the previous install after booting into the
        // bootc system for the first time
        "--cleanup",
    ]
    .map(String::from)
    .to_vec();

    podman_command_and_args.push("-v".to_string());
    podman_command_and_args.push(format!("{ssh_key_file}:{ROOT_KEY_MOUNT_POINT}"));

    bootc_command_and_args.push("--root-ssh-authorized-keys".to_string());
    bootc_command_and_args.push(ROOT_KEY_MOUNT_POINT.to_string());

    let all_args = [
        podman_command_and_args,
        vec![image.to_string()],
        bootc_command_and_args,
    ]
    .concat();

    let mut command = Command::new(&all_args[0]);
    command.args(&all_args[1..]);

    command
}

pub(crate) fn pull_image_command(image: &str) -> Command {
    let mut command = Command::new("podman");
    command.args(["pull", image]);
    command
}

/// Path to the podman installation script. Can be influenced by the build
/// SYSTEM_REINSTALL_BOOTC_INSTALL_PODMAN_PATH parameter to override. Defaults
/// to /usr/lib/system-reinstall-bootc/install-podman
const fn podman_install_script_path() -> &'static str {
    if let Some(path) = option_env!("SYSTEM_REINSTALL_BOOTC_INSTALL_PODMAN_PATH") {
        path
    } else {
        "/usr/lib/system-reinstall-bootc/install-podman"
    }
}

pub(crate) fn ensure_podman_installed() -> Result<()> {
    if which("podman").is_ok() {
        return Ok(());
    }

    prompt::ask_yes_no("Podman was not found on this system. It's required in order to install a bootc image. Do you want to install it now?", true)?;

    ensure!(
        which(podman_install_script_path()).is_ok(),
        "Podman installation script {} not found, cannot automatically install podman. Please install it manually and try again.",
        podman_install_script_path()
    );

    Command::new(podman_install_script_path())
        .run_with_cmd_context()
        .context("installing podman")?;

    // Make sure the installation was actually successful
    ensure!(
        which("podman").is_ok(),
        "podman still doesn't seem to be available, despite the installation. Please install it manually and try again."
    );

    Ok(())
}

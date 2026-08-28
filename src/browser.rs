use std::process::Command;

use crate::error::{GitowError, Result};

pub fn open_url(url: &str, print_only: bool) -> Result<()> {
    if print_only {
        println!("{url}");
        return Ok(());
    }

    if let Some(browser) = std::env::var_os("BROWSER") {
        let browser = browser.to_string_lossy().trim().to_string();
        if browser.is_empty() {
            return open_with_system_default(url);
        }
        if browser == "echo" {
            println!("{url}");
            return Ok(());
        }

        return spawn_browser_command(&browser, url);
    }

    open_with_system_default(url)
}

pub fn open_urls(urls: &[String], print_only: bool) -> Result<()> {
    for url in urls {
        open_url(url, print_only)?;
    }

    Ok(())
}

fn open_with_system_default(url: &str) -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        run_browser_command("cmd", &["/C", "start", "", url], "cmd /C start")
    }

    #[cfg(target_os = "macos")]
    {
        run_browser_command("open", &[url], "open")
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        if is_wsl() {
            return run_browser_command(
                "/mnt/c/Windows/System32/WindowsPowerShell/v1.0/powershell.exe",
                &["-NoProfile", "Start", url],
                "powershell.exe -NoProfile Start",
            );
        }

        run_browser_command("xdg-open", &[url], "xdg-open")
    }
}

fn spawn_browser_command(browser: &str, url: &str) -> Result<()> {
    if cfg!(target_os = "windows") && browser.eq_ignore_ascii_case("start") {
        run_browser_command("cmd", &["/C", "start", "", url], "cmd /C start")
    } else {
        run_browser_command(browser, &[url], browser)
    }
}

fn run_browser_command(executable: &str, args: &[&str], command: &str) -> Result<()> {
    let status = Command::new(executable)
        .args(args)
        .status()
        .map_err(|source| GitowError::BrowserCommand {
            command: command.to_string(),
            source,
        })?;

    if status.success() {
        return Ok(());
    }

    Err(GitowError::BrowserCommandFailed {
        command: command.to_string(),
        status,
    })
}

#[cfg(all(unix, not(target_os = "macos")))]
fn is_wsl() -> bool {
    std::fs::read_to_string("/proc/sys/kernel/osrelease")
        .or_else(|_| std::fs::read_to_string("/proc/version"))
        .map(|content| content.to_ascii_lowercase().contains("microsoft"))
        .unwrap_or(false)
        || std::env::var_os("WSL_DISTRO_NAME").is_some()
}

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
        let status = Command::new("cmd")
            .args(["/C", "start", "", url])
            .status()
            .map_err(|source| GitowError::BrowserCommand {
                command: "cmd /C start".to_string(),
                source,
            })?;

        if status.success() {
            return Ok(());
        }

        Err(GitowError::BrowserCommandFailed {
            command: "cmd /C start".to_string(),
            status,
        })
    }

    #[cfg(target_os = "macos")]
    {
        let status = Command::new("open").arg(url).status().map_err(|source| {
            GitowError::BrowserCommand {
                command: "open".to_string(),
                source,
            }
        })?;

        if status.success() {
            return Ok(());
        }

        return Err(GitowError::BrowserCommandFailed {
            command: "open".to_string(),
            status,
        });
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        if is_wsl() {
            let status =
                Command::new("/mnt/c/Windows/System32/WindowsPowerShell/v1.0/powershell.exe")
                    .args(["-NoProfile", "Start", url])
                    .status()
                    .map_err(|source| GitowError::BrowserCommand {
                        command: "powershell.exe -NoProfile Start".to_string(),
                        source,
                    })?;

            if status.success() {
                return Ok(());
            }

            return Err(GitowError::BrowserCommandFailed {
                command: "powershell.exe -NoProfile Start".to_string(),
                status,
            });
        }

        let status = Command::new("xdg-open")
            .arg(url)
            .status()
            .map_err(|source| GitowError::BrowserCommand {
                command: "xdg-open".to_string(),
                source,
            })?;

        if status.success() {
            return Ok(());
        }

        Err(GitowError::BrowserCommandFailed {
            command: "xdg-open".to_string(),
            status,
        })
    }
}

fn spawn_browser_command(browser: &str, url: &str) -> Result<()> {
    let status = if cfg!(target_os = "windows") && browser.eq_ignore_ascii_case("start") {
        Command::new("cmd")
            .args(["/C", "start", "", url])
            .status()
            .map_err(|source| GitowError::BrowserCommand {
                command: "cmd /C start".to_string(),
                source,
            })?
    } else {
        Command::new(browser)
            .arg(url)
            .status()
            .map_err(|source| GitowError::BrowserCommand {
                command: browser.to_string(),
                source,
            })?
    };

    if status.success() {
        return Ok(());
    }

    Err(GitowError::BrowserCommandFailed {
        command: browser.to_string(),
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

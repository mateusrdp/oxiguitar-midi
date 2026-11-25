// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Mateus Rocha de Paula and the OxiGuitar contributors

//! Opening the manual's page on better sounds in the system's browser,
//! only when the user clicks the Sound Sets window's link. The plugin never connects anywhere itself:
//! the browser does, as a program of its own, as OxiGuitar's own links do.

use std::process::{Command, Stdio};

/// The system's command that opens an address in the default browser.
fn opener(url: &str) -> Command {
    #[cfg(target_os = "windows")]
    {
        // url.dll takes the address as it is, & and all.
        let mut c = Command::new("rundll32");
        c.args(["url.dll,FileProtocolHandler", url]);
        c
    }
    #[cfg(target_os = "macos")]
    {
        let mut c = Command::new("open");
        c.arg(url);
        c
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        let mut c = Command::new("xdg-open");
        c.arg(url);
        c
    }
}

/// Opens `url` in the browser. Only `https://` addresses are opened, so
/// nothing else (a file, a program) can ever be started this way.
pub(crate) fn open(url: &str) -> Result<(), String> {
    if !url.starts_with("https://") || url.chars().any(|c| c.is_whitespace() || c.is_control()) {
        return Err(format!("{url} is not a web address"));
    }
    let mut child = opener(url)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("the browser couldn't be started ({e}); the address is {url}"))?;
    // Wait for the opener (it hands the address over and ends) on a thread
    // of its own, so nothing waits and no zombie is left behind.
    std::thread::spawn(move || {
        let _ = child.wait();
    });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_web_addresses_are_opened() {
        assert!(open("file:///etc/passwd").is_err());
        assert!(open("/usr/bin/true").is_err());
        assert!(open("http://example.org").is_err());
        assert!(open("https://example.org/ --x").is_err());
        assert!(crate::BETTER_SOUNDS.starts_with("https://"));
    }
}

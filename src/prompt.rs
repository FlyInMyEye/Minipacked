use crate::constants::{DEFAULT_PASSWORD, PASSWORD_MODE_DEFAULT, PASSWORD_MODE_PROMPTED};
use anyhow::{Result, bail};
use std::io::{IsTerminal, Write, stdin, stdout};

pub(crate) fn prompt_line(prompt: &str) -> Result<String> {
    Ok(prompt_raw_line(prompt)?.trim().to_string())
}

pub(crate) fn prompt_password_pair() -> Result<(String, u8)> {
    let first = prompt_password("Enter password (optional): ", true)?;
    if first.is_empty() {
        return Ok((DEFAULT_PASSWORD.to_string(), PASSWORD_MODE_DEFAULT));
    }

    let second = prompt_password("Repeat password: ", false)?;
    if first != second {
        bail!("passwords do not match");
    }

    Ok((first, PASSWORD_MODE_PROMPTED))
}

pub(crate) fn prompt_password(prompt: &str, allow_empty: bool) -> Result<String> {
    let password = if stdin().is_terminal() {
        rpassword::prompt_password(prompt)?
    } else {
        prompt_raw_line(prompt)?
    };

    if password.is_empty() && !allow_empty {
        bail!("password cannot be empty");
    }

    Ok(password)
}

fn prompt_raw_line(prompt: &str) -> Result<String> {
    print!("{prompt}");
    stdout().flush()?;
    let mut buf = String::new();
    stdin().read_line(&mut buf)?;
    while buf.ends_with(['\n', '\r']) {
        buf.pop();
    }
    Ok(buf)
}

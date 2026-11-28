//! Start Pusthakam in the system tray when the user signs in to Windows.
//!
//! The installer turns this on by default (`installer-hooks.nsh`). The entry lives under
//! HKEY_CURRENT_USER, so staff can change it in Settings without administrator rights.
#[cfg(not(windows))]
use crate::error::AppError;
use crate::error::AppResult;
use serde::Serialize;

/// Command-line flag used by the startup entry: open hidden in the tray, not as a window.
pub const BACKGROUND_ARG: &str = "--background";

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StartupSetting {
    /// False on systems where Pusthakam cannot register itself to start at sign-in.
    pub supported: bool,
    pub enabled: bool,
}

pub fn started_in_background() -> bool {
    std::env::args().any(|arg| arg == BACKGROUND_ARG)
}

#[cfg(windows)]
mod registry {
    use windows_sys::Win32::Foundation::{ERROR_FILE_NOT_FOUND, ERROR_SUCCESS};
    use windows_sys::Win32::System::Registry::{
        RegDeleteKeyValueW, RegGetValueW, RegSetKeyValueW, HKEY_CURRENT_USER, REG_SZ,
        RRF_RT_REG_BINARY, RRF_RT_REG_SZ,
    };

    pub const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
    /// Task Manager's Startup tab records "disabled" here without removing the Run value.
    pub const APPROVED_KEY: &str =
        r"Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run";
    pub const VALUE: &str = "Pusthakam";

    fn wide(text: &str) -> Vec<u16> {
        text.encode_utf16().chain(Some(0)).collect()
    }

    fn io_error(code: u32) -> std::io::Error {
        std::io::Error::from_raw_os_error(code as i32)
    }

    pub fn read_string(key: &str, value: &str) -> std::io::Result<Option<String>> {
        let (key, value) = (wide(key), wide(value));
        let mut size = 0u32;
        // SAFETY: both names are NUL-terminated UTF-16; a null buffer asks only for the size.
        let status = unsafe {
            RegGetValueW(
                HKEY_CURRENT_USER,
                key.as_ptr(),
                value.as_ptr(),
                RRF_RT_REG_SZ,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                &mut size,
            )
        };
        if status == ERROR_FILE_NOT_FOUND {
            return Ok(None);
        }
        if status != ERROR_SUCCESS {
            return Err(io_error(status));
        }
        let mut buffer = vec![0u16; (size as usize).div_ceil(2).max(1)];
        // SAFETY: `buffer` holds `size` bytes, as reported by the call above.
        let status = unsafe {
            RegGetValueW(
                HKEY_CURRENT_USER,
                key.as_ptr(),
                value.as_ptr(),
                RRF_RT_REG_SZ,
                std::ptr::null_mut(),
                buffer.as_mut_ptr().cast(),
                &mut size,
            )
        };
        if status != ERROR_SUCCESS {
            return Err(io_error(status));
        }
        let length = buffer
            .iter()
            .position(|&unit| unit == 0)
            .unwrap_or(buffer.len());
        Ok(Some(String::from_utf16_lossy(&buffer[..length])))
    }

    pub fn read_binary_first_byte(key: &str, value: &str) -> std::io::Result<Option<u8>> {
        let (key, value) = (wide(key), wide(value));
        let mut buffer = [0u8; 16];
        let mut size = buffer.len() as u32;
        // SAFETY: names are NUL-terminated; `buffer` holds `size` bytes.
        let status = unsafe {
            RegGetValueW(
                HKEY_CURRENT_USER,
                key.as_ptr(),
                value.as_ptr(),
                RRF_RT_REG_BINARY,
                std::ptr::null_mut(),
                buffer.as_mut_ptr().cast(),
                &mut size,
            )
        };
        match status {
            ERROR_SUCCESS if size > 0 => Ok(Some(buffer[0])),
            ERROR_SUCCESS | ERROR_FILE_NOT_FOUND => Ok(None),
            code => Err(io_error(code)),
        }
    }

    pub fn write_string(key: &str, value: &str, data: &str) -> std::io::Result<()> {
        let (key, value, data) = (wide(key), wide(value), wide(data));
        // SAFETY: all strings are NUL-terminated UTF-16; the size includes the terminator.
        let status = unsafe {
            RegSetKeyValueW(
                HKEY_CURRENT_USER,
                key.as_ptr(),
                value.as_ptr(),
                REG_SZ,
                data.as_ptr().cast(),
                (data.len() * 2) as u32,
            )
        };
        if status == ERROR_SUCCESS {
            Ok(())
        } else {
            Err(io_error(status))
        }
    }

    pub fn delete(key: &str, value: &str) -> std::io::Result<()> {
        let (key, value) = (wide(key), wide(value));
        // SAFETY: both names are NUL-terminated UTF-16.
        let status = unsafe { RegDeleteKeyValueW(HKEY_CURRENT_USER, key.as_ptr(), value.as_ptr()) };
        match status {
            ERROR_SUCCESS | ERROR_FILE_NOT_FOUND => Ok(()),
            code => Err(io_error(code)),
        }
    }
}

#[cfg(windows)]
pub fn get() -> AppResult<StartupSetting> {
    let registered = registry::read_string(registry::RUN_KEY, registry::VALUE)?.is_some();
    // Odd first bytes (01, 03) mean the user disabled the entry in Task Manager.
    let disabled = registry::read_binary_first_byte(registry::APPROVED_KEY, registry::VALUE)?
        .is_some_and(|flag| flag & 1 == 1);
    Ok(StartupSetting {
        supported: true,
        enabled: registered && !disabled,
    })
}

#[cfg(windows)]
pub fn set(enabled: bool) -> AppResult<StartupSetting> {
    if enabled {
        let exe = std::env::current_exe()?;
        let command = format!("\"{}\" {BACKGROUND_ARG}", exe.display());
        registry::write_string(registry::RUN_KEY, registry::VALUE, &command)?;
        registry::delete(registry::APPROVED_KEY, registry::VALUE)?;
    } else {
        registry::delete(registry::RUN_KEY, registry::VALUE)?;
    }
    get()
}

#[cfg(not(windows))]
pub fn get() -> AppResult<StartupSetting> {
    Ok(StartupSetting {
        supported: false,
        enabled: false,
    })
}

#[cfg(not(windows))]
pub fn set(_enabled: bool) -> AppResult<StartupSetting> {
    Err(AppError::Validation(
        "Starting with the computer is available on Windows only.".into(),
    ))
}

//! Native "choose a folder" dialog for the daily backup folder (Windows only).
use crate::error::{AppError, AppResult};
use std::path::PathBuf;

/// Shows the Windows folder dialog owned by `owner` (a window handle). `None` means cancelled.
/// Blocks until the user answers, so call it from a blocking task, not the UI thread.
#[cfg(windows)]
pub fn choose(owner: usize, title: &str) -> AppResult<Option<PathBuf>> {
    use std::os::windows::ffi::OsStringExt;
    use windows_sys::Win32::System::Com::{
        CoInitializeEx, CoTaskMemFree, CoUninitialize, COINIT_APARTMENTTHREADED,
    };
    use windows_sys::Win32::UI::Shell::{
        SHBrowseForFolderW, SHGetPathFromIDListEx, BIF_NEWDIALOGSTYLE, BIF_RETURNONLYFSDIRS,
        BROWSEINFOW, GPFIDL_DEFAULT,
    };

    let title: Vec<u16> = title.encode_utf16().chain(Some(0)).collect();
    let mut display = vec![0u16; 260];
    let info = BROWSEINFOW {
        hwndOwner: owner as _,
        pszDisplayName: display.as_mut_ptr(),
        lpszTitle: title.as_ptr(),
        ulFlags: BIF_RETURNONLYFSDIRS | BIF_NEWDIALOGSTYLE,
        ..Default::default()
    };
    // SAFETY: the new-style dialog needs a single-threaded COM apartment on this thread; every
    // successful initialisation (S_OK or S_FALSE) is balanced by CoUninitialize below.
    let initialised =
        unsafe { CoInitializeEx(std::ptr::null(), COINIT_APARTMENTTHREADED as u32) } >= 0;
    // SAFETY: `info` and the buffers it points to outlive this synchronous call.
    let pidl = unsafe { SHBrowseForFolderW(&info) };
    let result = if pidl.is_null() {
        Ok(None)
    } else {
        let mut path = vec![0u16; 32768];
        // SAFETY: `pidl` came from SHBrowseForFolderW; `path` holds `path.len()` characters.
        let ok = unsafe {
            SHGetPathFromIDListEx(pidl, path.as_mut_ptr(), path.len() as u32, GPFIDL_DEFAULT)
        };
        // SAFETY: the shell allocated `pidl` with the COM task allocator; it is not used again.
        unsafe { CoTaskMemFree(pidl as *const _) };
        if ok == 0 {
            Err(AppError::Validation(
                "That location is not a folder on this computer. Choose a drive or folder.".into(),
            ))
        } else {
            let length = path
                .iter()
                .position(|&unit| unit == 0)
                .unwrap_or(path.len());
            Ok(Some(PathBuf::from(std::ffi::OsString::from_wide(
                &path[..length],
            ))))
        }
    };
    if initialised {
        // SAFETY: balances the successful CoInitializeEx above on the same thread.
        unsafe { CoUninitialize() };
    }
    result
}

#[cfg(not(windows))]
pub fn choose(_owner: usize, _title: &str) -> AppResult<Option<PathBuf>> {
    Err(AppError::Validation(
        "Type the full path of the backup folder instead.".into(),
    ))
}

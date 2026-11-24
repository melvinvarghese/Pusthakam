//! Save bytes only to a path selected by the native dialog, never a frontend-supplied path.
use crate::error::{AppError, AppResult};
use base64::Engine;
use serde::Serialize;
use std::{
    io::Write,
    path::{Path, PathBuf},
};

const MAX_BYTES: usize = 50 * 1024 * 1024;
pub struct Payload {
    pub file_name: String,
    pub extension: String,
    pub bytes: Vec<u8>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveResult {
    pub native: bool,
    pub path: Option<String>,
}

pub fn prepare(file_name: &str, encoded: &str) -> AppResult<Payload> {
    let extension = match file_name {
        "pusthakam-catalogue.json" => "json",
        "pusthakam-catalogue.csv" => "csv",
        "pusthakam-catalogue.xlsx" => "xlsx",
        "pusthakam-catalogue.mrc" => "mrc",
        "pusthakam-catalogue.xml" => "xml",
        _ => {
            return Err(AppError::Validation(
                "Choose a prepared catalogue export.".into(),
            ))
        }
    };
    prepare_payload(file_name, extension, encoded)
}

pub fn prepare_report(file_name: &str, encoded: &str) -> AppResult<Payload> {
    if file_name.len() > 180
        || file_name.starts_with('.')
        || !file_name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "-_.".contains(c))
    {
        return Err(AppError::Validation(
            "Choose a valid report filename.".into(),
        ));
    }
    let extension = if file_name.ends_with(".pdf") {
        "pdf"
    } else if file_name.ends_with(".xlsx") {
        "xlsx"
    } else {
        return Err(AppError::Validation("Choose a PDF or XLSX report.".into()));
    };
    prepare_payload(file_name, extension, encoded)
}

fn prepare_payload(file_name: &str, extension: &str, encoded: &str) -> AppResult<Payload> {
    if encoded.len() > MAX_BYTES.div_ceil(3) * 4 {
        return Err(AppError::Validation("Export is too large to save.".into()));
    }
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .map_err(|_| {
            AppError::Validation("The prepared export could not be read. Prepare it again.".into())
        })?;
    if bytes.is_empty() || bytes.len() > MAX_BYTES {
        return Err(AppError::Validation("Export is empty or too large.".into()));
    }
    Ok(Payload {
        file_name: file_name.into(),
        extension: extension.into(),
        bytes,
    })
}

pub fn write_chosen(
    path: Option<PathBuf>,
    payload: &Payload,
    private_directory: &Path,
) -> AppResult<Option<String>> {
    let Some(path) = path else {
        return Ok(None);
    };
    if !path.is_absolute()
        || path
            .extension()
            .and_then(|v| v.to_str())
            .map_or(true, |ext| !ext.eq_ignore_ascii_case(&payload.extension))
    {
        return Err(AppError::Validation(format!(
            "Choose a filename ending in .{}.",
            payload.extension
        )));
    }
    let parent = path
        .parent()
        .ok_or_else(|| AppError::Validation("Choose a destination folder.".into()))?
        .canonicalize()?;
    let private = private_directory.canonicalize()?;
    if parent.starts_with(&private) || path.canonicalize().is_ok_and(|p| p.starts_with(&private)) {
        return Err(AppError::Validation(
            "Choose a folder outside Pusthakam's private data directory.".into(),
        ));
    }
    // Publish a complete file atomically; a failed write leaves an existing destination intact.
    let mut temporary = tempfile::NamedTempFile::new_in(&parent)?;
    temporary.write_all(&payload.bytes)?;
    temporary.as_file().sync_all()?;
    temporary
        .persist(&path)
        .map_err(|e| AppError::Io(e.error))?;
    Ok(Some(path.to_string_lossy().into_owned()))
}

#[cfg(windows)]
pub fn choose(payload: &Payload, owner: usize) -> AppResult<Option<PathBuf>> {
    use std::os::windows::ffi::OsStringExt;
    use windows_sys::Win32::UI::Controls::Dialogs::*;
    let wide = |value: &str| value.encode_utf16().chain(Some(0)).collect::<Vec<_>>();
    let mut buffer = vec![0u16; 32768];
    let initial = wide(&payload.file_name);
    buffer[..initial.len()].copy_from_slice(&initial);
    let filter = wide(&format!("Export file (*.{0})\0*.{0}\0", payload.extension));
    let extension = wide(&payload.extension);
    let title = wide(if payload.file_name.starts_with("pusthakam-catalogue.") {
        "Save catalogue export as"
    } else {
        "Save report as"
    });
    let mut dialog = OPENFILENAMEW {
        lStructSize: std::mem::size_of::<OPENFILENAMEW>() as u32,
        hwndOwner: owner as _,
        lpstrFile: buffer.as_mut_ptr(),
        nMaxFile: buffer.len() as u32,
        lpstrFilter: filter.as_ptr(),
        nFilterIndex: 1,
        lpstrDefExt: extension.as_ptr(),
        lpstrTitle: title.as_ptr(),
        Flags: OFN_EXPLORER
            | OFN_OVERWRITEPROMPT
            | OFN_PATHMUSTEXIST
            | OFN_NOCHANGEDIR
            | OFN_DONTADDTORECENT,
        ..Default::default()
    };
    // All UTF-16 buffers stay alive for the synchronous OS call; nMaxFile bounds output.
    let accepted = unsafe { GetSaveFileNameW(&mut dialog) };
    if accepted == 0 {
        let error = unsafe { CommDlgExtendedError() };
        if error == 0 {
            return Ok(None);
        }
        return Err(AppError::Validation(
            "The Save As dialog could not open. Try saving again.".into(),
        ));
    }
    let len = buffer
        .iter()
        .position(|v| *v == 0)
        .ok_or_else(|| AppError::Validation("The selected path is too long.".into()))?;
    Ok(Some(PathBuf::from(std::ffi::OsString::from_wide(
        &buffer[..len],
    ))))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn saves_exact_bytes_to_selected_unicode_path_and_replaces_complete_file() {
        let dir = tempfile::tempdir().unwrap();
        let private = dir.path().join("private");
        std::fs::create_dir(&private).unwrap();
        let path = dir.path().join("chosen-\u{e9}.xlsx");
        std::fs::write(&path, b"previous").unwrap();
        let payload = prepare("pusthakam-catalogue.xlsx", "AAEC/w==").unwrap();
        assert_eq!(
            write_chosen(Some(path.clone()), &payload, &private).unwrap(),
            Some(path.to_string_lossy().into_owned())
        );
        assert_eq!(std::fs::read(path).unwrap(), [0, 1, 2, 255]);
    }
    #[test]
    fn cancel_writes_nothing_and_bad_paths_preserve_existing_data() {
        let dir = tempfile::tempdir().unwrap();
        let private = dir.path().join("private");
        std::fs::create_dir(&private).unwrap();
        let payload = prepare("pusthakam-catalogue.json", "W10=").unwrap();
        assert_eq!(write_chosen(None, &payload, &private).unwrap(), None);
        let protected = private.join("library.json");
        std::fs::write(&protected, b"keep").unwrap();
        assert!(write_chosen(Some(protected.clone()), &payload, &private).is_err());
        assert_eq!(std::fs::read(protected).unwrap(), b"keep");
        assert!(write_chosen(Some(dir.path().join("wrong.db")), &payload, &private).is_err());
        assert!(write_chosen(
            Some(dir.path().join("missing").join("file.json")),
            &payload,
            &private
        )
        .is_err());
        assert!(prepare("../library.db", "W10=").is_err());
        assert!(prepare("pusthakam-catalogue.json", "invalid").is_err());
    }
}

/// Shows the owned save dialog and rechecks permission before publishing bytes.
pub async fn save<R: tauri::Runtime>(
    window: tauri::WebviewWindow<R>,
    state: tauri::State<'_, crate::state::AppState>,
    payload: Payload,
    permission: &str,
) -> AppResult<SaveResult> {
    use crate::services::export_save;
    #[cfg(windows)]
    {
        let owner = window
            .hwnd()
            .map_err(|_| {
                crate::error::AppError::Validation(
                    "Could not open the save dialog for this window.".into(),
                )
            })?
            .0 as usize;
        let (path, payload) = tauri::async_runtime::spawn_blocking(move || {
            export_save::choose(&payload, owner).map(|path| (path, payload))
        })
        .await
        .map_err(|_| crate::error::AppError::Internal("Save dialog task failed.".into()))??;
        state.require(permission)?;
        let private = state.paths.data_dir.clone();
        let path = tauri::async_runtime::spawn_blocking(move || {
            export_save::write_chosen(path, &payload, &private)
        })
        .await
        .map_err(|_| crate::error::AppError::Internal("Save file task failed.".into()))??;
        Ok(export_save::SaveResult { native: true, path })
    }
    #[cfg(not(windows))]
    {
        let _ = (window, payload, state, permission);
        Ok(export_save::SaveResult {
            native: false,
            path: None,
        })
    }
}

#[cfg(test)]
mod report_tests {
    use super::*;
    #[test]
    fn reports_use_pdf_or_xlsx_and_keep_destination_safety() {
        let root = tempfile::tempdir().unwrap();
        let private = root.path().join("private");
        std::fs::create_dir(&private).unwrap();
        let pdf = prepare_report("circulation-2026-11-24.pdf", "JVBERi0xLjQK").unwrap();
        assert_eq!(pdf.extension, "pdf");
        assert_eq!(write_chosen(None, &pdf, &private).unwrap(), None);
        let path = root.path().join("report.pdf");
        write_chosen(Some(path.clone()), &pdf, &private).unwrap();
        assert_eq!(std::fs::read(path).unwrap(), b"%PDF-1.4\n");
        assert!(write_chosen(Some(private.join("report.pdf")), &pdf, &private).is_err());
        for name in [
            "../report.pdf",
            "C:\\report.pdf",
            ".pdf",
            "report.exe",
            "report.csv",
        ] {
            assert!(prepare_report(name, "JVBERi0xLjQK").is_err());
        }
        assert!(prepare_report("report.xlsx", "UEsD").is_ok());
    }
}

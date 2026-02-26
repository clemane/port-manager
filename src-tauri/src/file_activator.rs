use std::fs;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;

use crate::auth;

/// Write `content` to the file at `path` with permissions `0600`.
///
/// Expands `~` in the path. Creates parent directories as needed.
pub fn activate_file(path: &str, content: &[u8]) -> Result<(), String> {
    let expanded = auth::expand_path(path);

    if let Some(parent) = std::path::Path::new(&expanded).parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directory {}: {e}", parent.display()))?;
    }

    let mut file =
        fs::File::create(&expanded).map_err(|e| format!("Failed to create {expanded}: {e}"))?;

    file.write_all(content)
        .map_err(|e| format!("Failed to write {expanded}: {e}"))?;

    let perms = fs::Permissions::from_mode(0o600);
    fs::set_permissions(&expanded, perms)
        .map_err(|e| format!("Failed to set permissions on {expanded}: {e}"))?;

    Ok(())
}

/// Best-effort secure deletion: overwrite with zeros, then remove.
///
/// Never returns an error -- failures are silently ignored.
pub fn secure_delete(path: &str) {
    let expanded = auth::expand_path(path);

    // Try to overwrite with zeros
    if let Ok(metadata) = fs::metadata(&expanded) {
        let len = metadata.len() as usize;
        if let Ok(mut file) = fs::OpenOptions::new().write(true).open(&expanded) {
            let zeros = vec![0u8; len];
            let _ = file.write_all(&zeros);
            let _ = file.flush();
        }
    }

    // Remove the file
    let _ = fs::remove_file(&expanded);
}

/// Securely delete a list of files. Each entry is `(id, file_path)`.
pub fn deactivate_all(paths: &[(String, String)]) {
    for (_id, file_path) in paths {
        secure_delete(file_path);
    }
}

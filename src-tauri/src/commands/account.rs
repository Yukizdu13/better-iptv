//! Compte universel (#20), étape 1 : export / import d'un fichier de compte chiffré.
//!
//! Mêmes commandes sous Linux et Windows. Le dérivation Argon2id est coûteuse : tout le travail
//! se fait dans `spawn_blocking`. La phrase secrète n'est jamais journalisée ni stockée.

use crate::error::AppError;
use crate::state::AppState;
use crate::sync_domain::{crypto, merge, snapshot};
use log::info;
use rusqlite::Connection;
use serde::Serialize;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager, State};

/// Taille maximale acceptée à l'import (un compte réel fait quelques Ko).
const MAX_IMPORT_BYTES: u64 = 64 * 1024 * 1024;
const FILE_EXTENSION: &str = "biptv";

#[derive(Debug, Serialize)]
pub struct ExportReport {
    pub path: String,
    pub profiles: usize,
    pub favorites: usize,
    pub series_progress: usize,
    pub presets: usize,
    pub settings: usize,
}

/// Chemin proposé par défaut : `<Documents>/better-iptv-compte.biptv` (Linux et Windows).
#[tauri::command]
pub async fn default_account_file_path(app: AppHandle) -> Result<String, AppError> {
    let dir = app
        .path()
        .document_dir()
        .or_else(|_| app.path().home_dir())
        .map_err(|e| AppError::Config(format!("Dossier Documents introuvable : {}", e)))?;
    Ok(dir
        .join(format!("better-iptv-compte.{}", FILE_EXTENSION))
        .to_string_lossy()
        .into_owned())
}

#[tauri::command]
pub async fn export_account(
    state: State<'_, AppState>,
    path: String,
    passphrase: String,
) -> Result<ExportReport, AppError> {
    crypto::validate_passphrase(&passphrase).map_err(|e| AppError::InvalidInput(e.to_string()))?;
    let target = validated_path(&path)?;
    let pool = state.pool.clone();

    tokio::task::spawn_blocking(move || -> Result<ExportReport, AppError> {
        let conn = pool.get()?;
        do_export(&conn, &target, &passphrase, &crypto::KdfParams::DEFAULT)
    })
    .await
    .map_err(|e| AppError::Config(format!("Tâche d'export interrompue : {}", e)))?
}

#[tauri::command]
pub async fn import_account(
    state: State<'_, AppState>,
    path: String,
    passphrase: String,
) -> Result<merge::MergeReport, AppError> {
    let source = validated_path(&path)?;
    let pool = state.pool.clone();

    tokio::task::spawn_blocking(move || -> Result<merge::MergeReport, AppError> {
        let conn = pool.get()?;
        do_import(&conn, &source, &passphrase)
    })
    .await
    .map_err(|e| AppError::Config(format!("Tâche d'import interrompue : {}", e)))?
}

/// Construit le snapshot, le chiffre et l'écrit dans `target`.
fn do_export(
    conn: &Connection,
    target: &Path,
    passphrase: &str,
    params: &crypto::KdfParams,
) -> Result<ExportReport, AppError> {
    let snap = snapshot::build_snapshot(conn)?;
    let report = ExportReport {
        path: target.to_string_lossy().into_owned(),
        profiles: snap.profiles.len(),
        favorites: snap.favorites.iter().filter(|f| f.is_favorite).count(),
        series_progress: snap.series_progress.len(),
        presets: snap.presets.len(),
        settings: snap.settings.len(),
    };
    let json = serde_json::to_vec(&snap)?;
    let encrypted = crypto::encrypt(&json, passphrase, params)
        .map_err(|e| AppError::Config(e.to_string()))?;
    write_private_file(target, &encrypted)?;
    info!(
        "Compte exporté : {} profil(s), {} favori(s), {} progression(s), {} preset(s)",
        report.profiles, report.favorites, report.series_progress, report.presets
    );
    Ok(report)
}

/// Lit, déchiffre et fusionne le fichier `source` dans la base locale.
fn do_import(
    conn: &Connection,
    source: &Path,
    passphrase: &str,
) -> Result<merge::MergeReport, AppError> {
    let meta = std::fs::metadata(source)?;
    if meta.len() > MAX_IMPORT_BYTES {
        return Err(AppError::InvalidInput("Fichier de compte trop volumineux".into()));
    }
    let data = std::fs::read(source)?;
    let json = crypto::decrypt(&data, passphrase)
        .map_err(|e| AppError::InvalidInput(e.to_string()))?;
    let snap: snapshot::Snapshot = serde_json::from_slice(&json)?;
    if snap.version != snapshot::SNAPSHOT_VERSION {
        return Err(AppError::InvalidInput(format!(
            "Version de compte non prise en charge : {}",
            snap.version
        )));
    }
    let report = merge::merge_snapshot(conn, &snap)?;
    // Un appareil vide n'a pas encore de profil actif
    crate::db::schema::ensure_active_profile(conn)?;
    info!(
        "Compte importé : +{} profil(s), {} mis à jour, {} supprimé(s), {} en attente",
        report.profiles_added, report.profiles_updated, report.profiles_deleted, report.pending
    );
    Ok(report)
}

/// Refuse un chemin vide ou relatif (le dossier courant d'une appli de bureau est imprévisible).
fn validated_path(path: &str) -> Result<PathBuf, AppError> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err(AppError::InvalidInput("Chemin du fichier vide".into()));
    }
    let p = PathBuf::from(trimmed);
    if !p.is_absolute() {
        return Err(AppError::InvalidInput(
            "Le chemin du fichier doit être absolu".into(),
        ));
    }
    Ok(p)
}

/// Écrit via un fichier temporaire puis renomme (pas de fichier à moitié écrit), en `0600` sous Unix.
fn write_private_file(target: &Path, data: &[u8]) -> Result<(), AppError> {
    let tmp = target.with_extension(format!("{}.tmp", FILE_EXTENSION));
    #[cfg(unix)]
    {
        use std::io::Write;
        use std::os::unix::fs::OpenOptionsExt;
        let mut f = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(&tmp)?;
        f.write_all(data)?;
        f.sync_all()?;
    }
    #[cfg(not(unix))]
    {
        std::fs::write(&tmp, data)?;
    }
    std::fs::rename(&tmp, target)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_empty_and_relative_paths() {
        assert!(validated_path("   ").is_err());
        assert!(validated_path("compte.biptv").is_err());
        #[cfg(unix)]
        assert!(validated_path("/tmp/compte.biptv").is_ok());
        #[cfg(windows)]
        assert!(validated_path(r"C:\Users\x\compte.biptv").is_ok());
    }


    #[test]
    fn export_then_import_through_a_real_file() {
        use crate::db::{models::Playlist, mutations, queries, schema::init_schema};
        let dir = std::env::temp_dir().join(format!("biptv-e2e-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("compte.biptv");

        let a = Connection::open_in_memory().unwrap();
        init_schema(&a).unwrap();
        mutations::create_playlist(
            &a,
            &Playlist {
                id: None,
                name: "brad".into(),
                url: Some("http://exemple.test".into()),
                file_path: None,
                last_updated: None,
                auto_refresh: false,
                xtream_username: Some("u".into()),
                xtream_password: Some("p".into()),
                created_at: None,
            },
        )
        .unwrap();
        mutations::set_setting(&a, "theme", "dark").unwrap();

        let pass = "phrase de test suffisamment longue";
        let rep = do_export(&a, &file, pass, &crypto::KdfParams::TEST).unwrap();
        assert_eq!((rep.profiles, rep.settings), (1, 1));
        // Le fichier ne contient ni le nom du profil ni les identifiants en clair
        let raw = std::fs::read(&file).unwrap();
        let text = String::from_utf8_lossy(&raw);
        assert!(!text.contains("brad") && !text.contains("exemple.test"));

        let b = Connection::open_in_memory().unwrap();
        init_schema(&b).unwrap();
        assert!(do_import(&b, &file, "mauvaise phrase de test longue").is_err());
        assert_eq!(queries::get_playlists(&b).unwrap().len(), 0, "échec = rien d'écrit");
        let rep = do_import(&b, &file, pass).unwrap();
        assert_eq!(rep.profiles_added, 1);
        let playlists = queries::get_playlists(&b).unwrap();
        assert_eq!(playlists[0].name, "brad");
        assert_eq!(queries::get_setting(&b, "active_profile_id").unwrap().as_deref(), Some("1"));
        assert_eq!(queries::get_setting(&b, "theme").unwrap().as_deref(), Some("dark"));

        // Fichier qui n'est pas un compte
        std::fs::write(dir.join("autre.biptv"), b"pas un compte").unwrap();
        assert!(do_import(&b, &dir.join("autre.biptv"), pass).is_err());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn written_file_is_private_and_atomic() {
        use std::os::unix::fs::PermissionsExt;
        let dir = std::env::temp_dir().join(format!("biptv-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let target = dir.join("c.biptv");
        write_private_file(&target, b"abc").unwrap();
        assert_eq!(std::fs::read(&target).unwrap(), b"abc");
        assert_eq!(std::fs::metadata(&target).unwrap().permissions().mode() & 0o777, 0o600);
        assert!(!target.with_extension("biptv.tmp").exists());
        std::fs::remove_dir_all(&dir).unwrap();
    }
}

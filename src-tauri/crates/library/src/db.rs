//! SQLite access for the media library.

use crate::models::{MediaItem, MediaType, Playlist};
use crate::Result;
use rusqlite::{params, Connection};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

const MIGRATION_SQL: &str = include_str!("../migrations/001_init.sql");

/// Open library database handle.
pub struct LibraryDb {
    conn: Connection,
}

impl LibraryDb {
    /// Open (or create) the library database at `path` and run migrations.
    ///
    /// # Errors
    /// Returns [`LibraryError::Db`] on SQLite failures.
    pub fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(path)?;
        conn.execute_batch(
            "PRAGMA foreign_keys = ON;
             PRAGMA journal_mode = WAL;
             PRAGMA busy_timeout = 5000;",
        )?;
        conn.execute_batch(MIGRATION_SQL)?;
        let count: i64 =
            conn.query_row("SELECT COUNT(*) FROM schema_version", [], |row| row.get(0))?;
        if count == 0 {
            conn.execute("INSERT INTO schema_version (version) VALUES (1)", [])?;
        }
        migrate_scan_root_kind(&conn)?;
        Ok(Self { conn })
    }

    /// Open an in-memory database (tests).
    ///
    /// # Errors
    /// Returns [`LibraryError::Db`] on SQLite failures.
    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch(
            "PRAGMA foreign_keys = ON;
             PRAGMA busy_timeout = 5000;",
        )?;
        conn.execute_batch(MIGRATION_SQL)?;
        conn.execute("INSERT INTO schema_version (version) VALUES (1)", [])?;
        migrate_scan_root_kind(&conn)?;
        Ok(Self { conn })
    }

    /// Begin a write transaction (for batched scans).
    ///
    /// # Errors
    /// Returns [`LibraryError::Db`].
    pub fn begin_write(&self) -> Result<()> {
        self.conn.execute_batch("BEGIN IMMEDIATE")?;
        Ok(())
    }

    /// Commit the current transaction.
    ///
    /// # Errors
    /// Returns [`LibraryError::Db`].
    pub fn commit(&self) -> Result<()> {
        self.conn.execute_batch("COMMIT")?;
        Ok(())
    }

    /// Roll back the current transaction.
    ///
    /// # Errors
    /// Returns [`LibraryError::Db`].
    pub fn rollback(&self) -> Result<()> {
        self.conn.execute_batch("ROLLBACK")?;
        Ok(())
    }

    /// Add a scan root folder tagged for Videos (`video`) or Music (`audio`).
    ///
    /// # Errors
    /// Returns [`LibraryError::Db`] on conflict/SQLite errors.
    pub fn add_scan_root(&self, path: &str, kind: &str) -> Result<()> {
        let now = now_secs();
        let kind = normalize_root_kind(kind);
        // Re-adding the same path upgrades/keeps the section it was opened from.
        self.conn.execute(
            "INSERT INTO scan_roots (path, added_at, kind) VALUES (?1, ?2, ?3)
             ON CONFLICT(path) DO UPDATE SET kind=excluded.kind, added_at=excluded.added_at",
            params![path, now, kind],
        )?;
        Ok(())
    }

    /// List scan roots, optionally filtered by section kind (`video` / `audio`).
    ///
    /// Roots with kind `both` (legacy) appear in every section.
    ///
    /// # Errors
    /// Returns [`LibraryError::Db`].
    pub fn list_scan_roots(&self, kind: Option<&str>) -> Result<Vec<String>> {
        Ok(self
            .list_scan_roots_with_kind(kind)?
            .into_iter()
            .map(|(path, _)| path)
            .collect())
    }

    /// List scan roots as `(path, kind)`.
    ///
    /// # Errors
    /// Returns [`LibraryError::Db`].
    pub fn list_scan_roots_with_kind(&self, kind: Option<&str>) -> Result<Vec<(String, String)>> {
        let mut out = Vec::new();
        if let Some(k) = kind {
            let k = normalize_root_kind(k);
            let mut stmt = self.conn.prepare(
                "SELECT path, kind FROM scan_roots
                 WHERE kind = ?1 OR kind = 'both'
                 ORDER BY id",
            )?;
            let rows = stmt.query_map(params![k], |row| Ok((row.get(0)?, row.get(1)?)))?;
            for row in rows {
                out.push(row?);
            }
        } else {
            let mut stmt = self
                .conn
                .prepare("SELECT path, kind FROM scan_roots ORDER BY id")?;
            let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;
            for row in rows {
                out.push(row?);
            }
        }
        Ok(out)
    }

    /// Persist probed duration for a media row.
    ///
    /// # Errors
    /// Returns [`LibraryError::Db`].
    pub fn set_duration_ms(&self, media_id: i64, duration_ms: i64) -> Result<()> {
        self.conn.execute(
            "UPDATE media_items SET duration_ms = ?1 WHERE id = ?2",
            params![duration_ms, media_id],
        )?;
        Ok(())
    }

    /// Recently played media filtered by type (`video` / `audio`), newest first.
    ///
    /// # Errors
    /// Returns [`LibraryError::Db`].
    pub fn list_recent_by_type(&self, media_type: MediaType, limit: i64) -> Result<Vec<MediaItem>> {
        let mut stmt = self.conn.prepare(
            "SELECT m.id, m.path, m.title, m.duration_ms, m.media_type, m.mtime, m.artwork_path
             FROM recent r
             JOIN media_items m ON m.id = r.media_id
             WHERE m.media_type = ?1
             ORDER BY r.played_at DESC
             LIMIT ?2",
        )?;
        let rows = stmt.query_map(params![media_type.as_str(), limit], map_media_row)?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    }

    /// Upsert a media item by path. Returns the row id.
    ///
    /// # Errors
    /// Returns [`LibraryError::Db`].
    pub fn upsert_media(
        &self,
        path: &str,
        title: &str,
        media_type: MediaType,
        mtime: Option<i64>,
        duration_ms: Option<i64>,
    ) -> Result<i64> {
        let now = now_secs();
        self.conn.execute(
            "INSERT INTO media_items (path, title, duration_ms, media_type, mtime, added_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(path) DO UPDATE SET
               title=excluded.title,
               duration_ms=excluded.duration_ms,
               media_type=excluded.media_type,
               mtime=excluded.mtime",
            params![path, title, duration_ms, media_type.as_str(), mtime, now],
        )?;
        let id: i64 = self.conn.query_row(
            "SELECT id FROM media_items WHERE path = ?1",
            params![path],
            |row| row.get(0),
        )?;
        Ok(id)
    }

    /// Persist a generated/sidecar artwork path for a media row.
    ///
    /// # Errors
    /// Returns [`LibraryError::Db`].
    pub fn set_artwork_path(&self, media_id: i64, artwork_path: &str) -> Result<()> {
        if artwork_path.is_empty() {
            self.conn.execute(
                "UPDATE media_items SET artwork_path = NULL WHERE id = ?1",
                params![media_id],
            )?;
        } else {
            self.conn.execute(
                "UPDATE media_items SET artwork_path = ?1 WHERE id = ?2",
                params![artwork_path, media_id],
            )?;
        }
        Ok(())
    }

    /// List media rows that still need artwork.
    ///
    /// # Errors
    /// Returns [`LibraryError::Db`].
    pub fn list_media_missing_artwork(&self, limit: usize) -> Result<Vec<MediaItem>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, path, title, duration_ms, media_type, mtime, artwork_path
             FROM media_items
             WHERE artwork_path IS NULL OR artwork_path = ''
             ORDER BY id DESC
             LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit as i64], map_media_row)?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    }

    /// Fetch a single media row by id.
    ///
    /// # Errors
    /// Returns [`LibraryError::Db`].
    pub fn get_media_by_id(&self, id: i64) -> Result<Option<MediaItem>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, path, title, duration_ms, media_type, mtime, artwork_path
             FROM media_items WHERE id = ?1",
        )?;
        let mut rows = stmt.query_map(params![id], map_media_row)?;
        Ok(rows.next().transpose()?)
    }

    /// Delete media rows whose files no longer exist under the given scan roots.
    ///
    /// # Errors
    /// Returns [`LibraryError::Db`].
    pub fn prune_missing_media(&self, roots: &[String]) -> Result<usize> {
        let items = self.list_media(None)?;
        let mut removed = 0usize;
        for item in items {
            let path = Path::new(&item.path);
            if !path.is_file() {
                let under_root =
                    roots.is_empty() || roots.iter().any(|r| path.starts_with(Path::new(r)));
                if under_root {
                    self.conn
                        .execute("DELETE FROM media_items WHERE id = ?1", params![item.id])?;
                    removed += 1;
                }
            }
        }
        Ok(removed)
    }

    /// List media, optionally filtered by type.
    ///
    /// # Errors
    /// Returns [`LibraryError::Db`].
    pub fn list_media(&self, media_type: Option<MediaType>) -> Result<Vec<MediaItem>> {
        self.list_media_limited(media_type, None)
    }

    /// List media with an optional row limit (for home / previews).
    ///
    /// # Errors
    /// Returns [`LibraryError::Db`].
    pub fn list_media_limited(
        &self,
        media_type: Option<MediaType>,
        limit: Option<i64>,
    ) -> Result<Vec<MediaItem>> {
        let mut out = Vec::new();
        if let Some(mt) = media_type {
            if let Some(lim) = limit {
                let mut stmt = self.conn.prepare(
                    "SELECT id, path, title, duration_ms, media_type, mtime, artwork_path
                     FROM media_items WHERE media_type = ?1 ORDER BY title COLLATE NOCASE LIMIT ?2",
                )?;
                let rows = stmt.query_map(params![mt.as_str(), lim], map_media_row)?;
                for row in rows {
                    out.push(row?);
                }
            } else {
                let mut stmt = self.conn.prepare(
                    "SELECT id, path, title, duration_ms, media_type, mtime, artwork_path
                     FROM media_items WHERE media_type = ?1 ORDER BY title COLLATE NOCASE",
                )?;
                let rows = stmt.query_map(params![mt.as_str()], map_media_row)?;
                for row in rows {
                    out.push(row?);
                }
            }
        } else if let Some(lim) = limit {
            let mut stmt = self.conn.prepare(
                "SELECT id, path, title, duration_ms, media_type, mtime, artwork_path
                 FROM media_items ORDER BY title COLLATE NOCASE LIMIT ?1",
            )?;
            let rows = stmt.query_map(params![lim], map_media_row)?;
            for row in rows {
                out.push(row?);
            }
        } else {
            let mut stmt = self.conn.prepare(
                "SELECT id, path, title, duration_ms, media_type, mtime, artwork_path
                 FROM media_items ORDER BY title COLLATE NOCASE",
            )?;
            let rows = stmt.query_map([], map_media_row)?;
            for row in rows {
                out.push(row?);
            }
        }
        Ok(out)
    }

    /// Create a playlist.
    ///
    /// # Errors
    /// Returns [`LibraryError::Db`].
    pub fn create_playlist(&self, name: &str) -> Result<i64> {
        let now = now_secs();
        self.conn.execute(
            "INSERT INTO playlists (name, created_at) VALUES (?1, ?2)",
            params![name, now],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// List playlists.
    ///
    /// # Errors
    /// Returns [`LibraryError::Db`].
    pub fn list_playlists(&self) -> Result<Vec<Playlist>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name FROM playlists ORDER BY name COLLATE NOCASE")?;
        let rows = stmt.query_map([], |row| {
            Ok(Playlist {
                id: row.get(0)?,
                name: row.get(1)?,
            })
        })?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    }

    /// Record a recent play.
    ///
    /// # Errors
    /// Returns [`LibraryError::Db`].
    pub fn touch_recent(&self, media_id: i64) -> Result<()> {
        let now = now_secs();
        self.conn.execute(
            "INSERT INTO recent (media_id, played_at) VALUES (?1, ?2)
             ON CONFLICT(media_id) DO UPDATE SET played_at=excluded.played_at",
            params![media_id, now],
        )?;
        Ok(())
    }

    /// Clear the recently played list.
    ///
    /// # Errors
    /// Returns [`LibraryError::Db`].
    pub fn clear_recent(&self) -> Result<()> {
        self.conn.execute("DELETE FROM recent", [])?;
        Ok(())
    }

    /// List recently played media (newest first).
    ///
    /// # Errors
    /// Returns [`LibraryError::Db`].
    pub fn list_recent(&self, limit: i64) -> Result<Vec<MediaItem>> {
        let mut stmt = self.conn.prepare(
            "SELECT m.id, m.path, m.title, m.duration_ms, m.media_type, m.mtime, m.artwork_path
             FROM recent r
             JOIN media_items m ON m.id = r.media_id
             ORDER BY r.played_at DESC
             LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit], map_media_row)?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    }

    /// List media belonging to a playlist.
    ///
    /// # Errors
    /// Returns [`LibraryError::Db`].
    pub fn list_playlist_media(&self, playlist_id: i64) -> Result<Vec<MediaItem>> {
        let mut stmt = self.conn.prepare(
            "SELECT m.id, m.path, m.title, m.duration_ms, m.media_type, m.mtime, m.artwork_path
             FROM playlist_items pi
             JOIN media_items m ON m.id = pi.media_id
             WHERE pi.playlist_id = ?1
             ORDER BY pi.position ASC, m.title COLLATE NOCASE",
        )?;
        let rows = stmt.query_map(params![playlist_id], map_media_row)?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    }

    /// Append a media item to a playlist.
    ///
    /// # Errors
    /// Returns [`LibraryError::Db`].
    pub fn add_to_playlist(&self, playlist_id: i64, media_id: i64) -> Result<()> {
        let pos: i64 = self
            .conn
            .query_row(
                "SELECT COALESCE(MAX(position), -1) + 1 FROM playlist_items WHERE playlist_id = ?1",
                params![playlist_id],
                |row| row.get(0),
            )
            .unwrap_or(0);
        self.conn.execute(
            "INSERT OR IGNORE INTO playlist_items (playlist_id, media_id, position)
             VALUES (?1, ?2, ?3)",
            params![playlist_id, media_id, pos],
        )?;
        Ok(())
    }

    /// Rename a playlist.
    ///
    /// # Errors
    /// Returns [`LibraryError::Db`].
    pub fn rename_playlist(&self, playlist_id: i64, name: &str) -> Result<()> {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            return Ok(());
        }
        self.conn.execute(
            "UPDATE playlists SET name = ?1 WHERE id = ?2",
            params![trimmed, playlist_id],
        )?;
        Ok(())
    }

    /// Delete a playlist and its items.
    ///
    /// # Errors
    /// Returns [`LibraryError::Db`].
    pub fn delete_playlist(&self, playlist_id: i64) -> Result<()> {
        self.conn.execute(
            "DELETE FROM playlist_items WHERE playlist_id = ?1",
            params![playlist_id],
        )?;
        self.conn
            .execute("DELETE FROM playlists WHERE id = ?1", params![playlist_id])?;
        Ok(())
    }

    /// Duplicate a playlist (new name + copied items). Returns the new id.
    ///
    /// # Errors
    /// Returns [`LibraryError::Db`].
    pub fn duplicate_playlist(&self, playlist_id: i64) -> Result<i64> {
        let name: String = self.conn.query_row(
            "SELECT name FROM playlists WHERE id = ?1",
            params![playlist_id],
            |row| row.get(0),
        )?;
        let new_name = format!("{name} (copy)");
        let new_id = self.create_playlist(&new_name)?;
        let items = self.list_playlist_media(playlist_id)?;
        for item in items {
            self.add_to_playlist(new_id, item.id)?;
        }
        Ok(new_id)
    }
}

fn map_media_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<MediaItem> {
    let media_type: String = row.get(4)?;
    Ok(MediaItem {
        id: row.get(0)?,
        path: row.get(1)?,
        title: row.get(2)?,
        duration_ms: row.get(3)?,
        media_type: MediaType::parse(&media_type),
        mtime: row.get(5)?,
        artwork_path: row.get(6)?,
    })
}

fn normalize_root_kind(kind: &str) -> &'static str {
    match kind {
        "audio" | "music" => "audio",
        "video" | "videos" => "video",
        _ => "both",
    }
}

fn migrate_scan_root_kind(conn: &Connection) -> Result<()> {
    let has_kind: bool = conn
        .prepare("PRAGMA table_info(scan_roots)")?
        .query_map([], |row| {
            let name: String = row.get(1)?;
            Ok(name)
        })?
        .filter_map(std::result::Result::ok)
        .any(|n| n == "kind");
    if !has_kind {
        conn.execute(
            "ALTER TABLE scan_roots ADD COLUMN kind TEXT NOT NULL DEFAULT 'both'",
            [],
        )?;
    }
    Ok(())
}

fn now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upsert_media_should_return_stable_id_on_conflict() {
        let db = LibraryDb::open_in_memory().expect("db");
        let id1 = db
            .upsert_media("/a.mp4", "A", MediaType::Video, Some(1), None)
            .expect("insert");
        let id2 = db
            .upsert_media("/a.mp4", "A2", MediaType::Video, Some(2), None)
            .expect("update");
        assert_eq!(id1, id2);
        let items = db.list_media(None).expect("list");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].title, "A2");
    }

    #[test]
    fn create_playlist_should_appear_in_list() {
        let db = LibraryDb::open_in_memory().expect("db");
        let id = db.create_playlist("Cool Songs").expect("create");
        assert!(id > 0);
        let lists = db.list_playlists().expect("list");
        assert_eq!(lists.len(), 1);
        assert_eq!(lists[0].name, "Cool Songs");
    }
}

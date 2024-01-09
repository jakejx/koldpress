use anyhow::{Context, Result};
use rusqlite::{Connection, OpenFlags};
use std::path::PathBuf;

pub struct Book {
    pub title: String,
    pub author: String,
    pub content_id: String,
}

pub struct KoboLibrary {
    db: Connection,
}

impl KoboLibrary {
    pub fn new(db_path: PathBuf) -> Result<Self> {
        let db = Connection::open_with_flags(
            db_path.clone(),
            OpenFlags::SQLITE_OPEN_READ_ONLY
                | OpenFlags::SQLITE_OPEN_NO_MUTEX
                | OpenFlags::SQLITE_OPEN_URI,
        )
        .with_context(|| format!("could not read database: {}", db_path.display()))?;
        Ok(Self { db })
    }

    pub fn get_books(&self) -> Result<Vec<Book>> {
        let mut stmt = self.db.prepare("SELECT Title, Attribution, ContentID FROM content WHERE BookTitle is NULL AND MimeType IN ('application/x-kobo-epub+zip', 'application/epub+zip')")?;
        let books = stmt
            .query_map([], |row| {
                // TODO: do some sanity checks on the data here
                Ok(Book {
                    title: row.get(0)?,
                    author: row.get(1)?,
                    content_id: row.get(2)?,
                })
            })?
            .collect::<core::result::Result<Vec<_>, _>>();
        Ok(books?)
    }
}

use super::db;
use anyhow::{Context, Result};
use rusqlite::{Connection, OpenFlags, OptionalExtension};
use sea_query::{Expr, SqliteQueryBuilder};
use sea_query_rusqlite::RusqliteBinder;
use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug, Serialize)]
pub struct Book {
    pub title: String,
    pub author: String,
    pub content_id: String,
}

#[derive(Debug)]
pub struct Library {
    db: Connection,
}

#[derive(Debug, Serialize)]
pub struct Bookmark {
    pub content_id: String,
    /// The chapter title
    pub title: Option<String>,
    pub text: String,
}

impl Library {
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
        let (sql, values) = db::books_query().build_rusqlite(SqliteQueryBuilder);
        println!("{}", sql); // TODO: turn this into a debug
        let mut stmt = self.db.prepare(sql.as_str())?;
        let books = stmt
            .query_map(&*values.as_params(), |row| Book::try_from(row))?
            .collect::<core::result::Result<Vec<_>, _>>()?;
        Ok(books)
    }

    pub fn get_book(&self, content_id: String) -> Result<Option<Book>> {
        let (sql, values) = db::books_query()
            .and_where(Expr::col(db::Content::ContentId).eq(content_id))
            .limit(1)
            .build_rusqlite(SqliteQueryBuilder);
        println!("{}", sql); // TODO: debug logging
        let mut stmt = self.db.prepare(sql.as_str())?;
        let book = stmt
            .query_row(&*values.as_params(), |row| Book::try_from(row))
            .optional()?;
        Ok(book)
    }

    pub fn get_bookmarks(&self) -> Result<Vec<Bookmark>> {
        let (sql, values) = db::bookmarks_query().build_rusqlite(SqliteQueryBuilder);
        println!("{}", sql); // TODO: debug logging
        println!("{:?}", values); // TODO: debug logging
        let mut stmt = self.db.prepare(sql.as_str())?;
        let bookmarks = stmt
            .query_map(&*values.as_params(), |row| Bookmark::try_from(row))?
            .collect::<core::result::Result<Vec<_>, _>>()?;
        Ok(bookmarks)
    }
}

use anyhow::{Context, Result};
use rusqlite::{Connection, OpenFlags, OptionalExtension, Row};
use sea_query::{Expr, Iden, Query, SelectStatement, SqliteQueryBuilder};
use sea_query_rusqlite::RusqliteBinder;

use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug, Serialize)]
pub struct Book {
    pub title: String,
    pub author: String,
    pub content_id: String,
}

impl TryFrom<&Row<'_>> for Book {
    type Error = rusqlite::Error;

    fn try_from(row: &Row) -> core::result::Result<Self, Self::Error> {
        Ok(Self {
            title: row.get("Title")?,
            author: row.get("Attribution")?,
            content_id: row.get("ContentID")?,
        })
    }
}

pub struct KoboLibrary {
    db: Connection,
}

#[derive(Iden)]
enum Content {
    Table,
    #[iden = "Title"]
    Title,
    #[iden = "BookTitle"]
    BookTitle,
    #[iden = "Attribution"]
    Attribution,
    #[iden = "ContentID"]
    ContentId,
    #[iden = "MimeType"]
    MimeType,
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
        let (sql, values) = KoboLibrary::books_query().build_rusqlite(SqliteQueryBuilder);
        println!("{}", sql); // TODO: turn this into a debug
        let mut stmt = self.db.prepare(sql.as_str())?;
        let books = stmt
            .query_map(&*values.as_params(), |row| Book::try_from(row))?
            .collect::<core::result::Result<Vec<_>, _>>()?;
        Ok(books)
    }

    pub fn get_book(&self, content_id: String) -> Result<Option<Book>> {
        let (sql, values) = KoboLibrary::books_query()
            .and_where(Expr::col(Content::ContentId).eq(content_id))
            .limit(1)
            .build_rusqlite(SqliteQueryBuilder);
        println!("{}", sql); // TODO: debug logging
        let mut stmt = self.db.prepare(sql.as_str())?;
        let book = stmt
            .query_row(&*values.as_params(), |row| Book::try_from(row))
            .optional()?;
        Ok(book)
    }

    // TODO: extract to some data layer?
    fn books_query() -> SelectStatement {
        Query::select()
            .columns([Content::Title, Content::Attribution, Content::ContentId])
            .from(Content::Table)
            .and_where(Expr::col(Content::BookTitle).is_null())
            .and_where(
                Expr::col(Content::MimeType)
                    .is_in(["application/x-kobo-epub+zip", "application/epub+zip"]),
            )
            .to_owned()
    }
}

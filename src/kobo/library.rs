use super::db;
use anyhow::{Context, Result};
use rusqlite::{Connection, OpenFlags, OptionalExtension};
use sea_query::{Expr, SqliteQueryBuilder};
use sea_query_rusqlite::RusqliteBinder;
use serde::Serialize;
use std::{fmt::Display, path::PathBuf};
use tracing::info;

#[derive(Debug, Serialize)]
pub struct Book {
    pub title: String,
    pub author: String,
    pub content_id: String,
}

impl Display for Book {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} - {}", self.title, self.author)
    }
}

#[derive(Debug)]
pub struct Library {
    db: Connection,
}

#[derive(Debug, Serialize)]
pub struct Bookmark {
    pub content_id: String,
    /// The chapter title
    pub chapter_title: Option<String>,
    pub text: String,
}

#[derive(Debug, Serialize)]
pub struct Chapter {
    title: String,
    bookmarks: Vec<Bookmark>,
    children: Vec<Chapter>,
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
        info!(query = sql, "Retrieving books");
        let mut stmt = self.db.prepare(sql.as_str())?;
        let books = stmt
            .query_map(&*values.as_params(), |row| Book::try_from(row))?
            .collect::<core::result::Result<Vec<_>, _>>()?;
        Ok(books)
    }

    pub fn get_book(&self, content_id: String) -> Result<Option<Book>> {
        let (sql, values) = db::books_query()
            .and_where(Expr::col(db::Content::ContentId).eq(&content_id))
            .limit(1)
            .build_rusqlite(SqliteQueryBuilder);
        info!(query = sql, content_id, "Retrieving book");
        let mut stmt = self.db.prepare(sql.as_str())?;
        let book = stmt
            .query_row(&*values.as_params(), |row| Book::try_from(row))
            .optional()?;
        Ok(book)
    }

    pub fn get_bookmarks(&self) -> Result<Vec<Bookmark>> {
        let sql = db::bookmarks_query();
        info!(query = sql, "Retrieving bookmarks");
        let mut stmt = self.db.prepare(sql.as_str())?;
        let bookmarks = stmt
            .query_map([], |row| Bookmark::try_from(row))?
            .collect::<core::result::Result<Vec<_>, _>>()?;
        Ok(bookmarks)
    }

    // TODO: for now we return a flat list, eventually we can use the epub
    // information to create a tree based on the TOC instead.
    pub fn get_bookmarks_for_book(&self, book: &Book) -> Result<Vec<Chapter>> {
        // TODO: extract out the logic for retrieving bookmarks from grouping them by chapter
        let sql = db::bookmarks_for_book_query();
        info!(
            query = sql,
            book = book.title,
            "Retrieving bookmarks for book"
        );
        let mut stmt = self.db.prepare(sql.as_str())?;
        let bookmarks = stmt
            .query_map([book.content_id.as_str()], |row| Bookmark::try_from(row))?
            .collect::<core::result::Result<Vec<_>, _>>()?;
        info!("Extracted {} bookmarks", bookmarks.len());
        let mut chapters: Vec<Chapter> = Vec::new();
        let mut current_chapter: Option<Chapter> = None;
        for bookmark in bookmarks {
            match current_chapter {
                None => {
                    current_chapter.replace(Chapter {
                        title: bookmark.chapter_title.clone().unwrap_or("".to_string()),
                        bookmarks: vec![],
                        children: vec![],
                    });
                }
                Some(ref chapter)
                    if chapter.title
                        != bookmark.chapter_title.clone().unwrap_or("".to_string()) =>
                {
                    let next_chapter = Chapter {
                        title: bookmark.chapter_title.clone().unwrap_or("".to_string()),
                        bookmarks: vec![],
                        children: vec![],
                    };
                    let previous_chapter = current_chapter.replace(next_chapter);
                    chapters.push(previous_chapter.expect("Chapter cannot be None"));
                }
                _ => (),
            };

            if let Some(chapter) = current_chapter.as_mut() {
                chapter.bookmarks.push(bookmark);
            }
        }

        if let Some(chapter) = current_chapter {
            chapters.push(chapter);
        }

        Ok(chapters)
    }
}

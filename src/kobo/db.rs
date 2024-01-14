use rusqlite::Row;
use sea_query::{Expr, Iden, Query, SelectStatement, WithClause};

impl TryFrom<&Row<'_>> for super::library::Book {
    type Error = rusqlite::Error;

    fn try_from(row: &Row) -> core::result::Result<Self, Self::Error> {
        Ok(Self {
            title: row.get(Content::Title.to_string().as_str())?,
            author: row.get("Attribution")?,
            content_id: row.get("ContentID")?,
        })
    }
}

impl TryFrom<&Row<'_>> for super::library::Bookmark {
    type Error = rusqlite::Error;

    fn try_from(row: &Row) -> core::result::Result<Self, Self::Error> {
        Ok(Self {
            content_id: row.get(Bookmark::ContentId.to_string().as_str())?,
            chapter_title: row.get("ChapterTitle")?,
            text: row.get(Bookmark::Text.to_string().as_str())?,
        })
    }
}

#[derive(Iden, Copy, Clone)]
pub(crate) enum Content {
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
    #[iden = "VolumeIndex"]
    VolumeIndex,
    #[iden = "ChapterIDBookmarked"]
    ChapterIDBookmarked,
}

#[derive(Iden, Copy, Clone)]
#[iden = "Bookmark"]
pub(crate) enum Bookmark {
    Table,
    #[iden = "ContentID"]
    ContentId,
    #[iden = "Text"]
    Text,
    #[iden = "Hidden"]
    Hidden,
    #[iden = "VolumeID"]
    VolumeId,
    #[iden = "ChapterProgress"]
    ChapterProgress,
}

pub fn books_query() -> SelectStatement {
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

/// SQL query for retrieving bookmarks and their chapter title.
/// In Kobo's DB, Content with ContentType 899 are the "chapter titles". But the
/// ContentID for these chapter titles are not the ContentIDs used by the
/// Bookmark. Usually the chapter title's ContentID is the bookmark's ContentID
/// with some suffix (where the suffix refers to some id in the epub HTML).
///
/// The chapter titles (content type = 899) are JOINed with the bookmark
/// content's (content type = 9) using a LIKE and the ContentID for both the
/// chapter title and the bookmark content are kept for later.
/// But there are cases where the chapter does not match to any chapter title.
/// The chapter title from an "earlier" chapter, identified by the VolumeIndex,
/// is used.
///
/// The Bookmarks are JOINed with the resolved chapter titles using either the
/// chapter title ContentID or the bookmark content's ContentID. From
/// observation, the Bookmark's ContentID can refer to either, so both are
/// tried.
const BOOKMARKS_QUERY: &str = r#"
WITH chapters AS (
	SELECT
		ContentID,
		Title,
		VolumeIndex
	FROM
		Content
	WHERE
		ContentType = 899
),
partial AS (
	SELECT
		c1.BookTitle,
		c1.ContentID,
		chapters.ContentID AS ChapterContentID,
		c1.VolumeIndex,
		chapters.Title AS ChapterTitle
	FROM
		Content AS c1
	LEFT JOIN chapters ON chapters.ContentID LIKE c1.ContentID || '%'
WHERE
	ContentType = 9
),
-- Fill missing chapter titles based on volume index
chapter_titles AS (
	SELECT
        VolumeIndex,
		ContentID,
		ChapterContentID,
		COALESCE(ChapterTitle,
			(
				SELECT
					ChapterTitle FROM partial p2
				WHERE
					ChapterTitle IS NOT NULL
					AND p2.BookTitle = p1.BookTitle
					AND p2.VolumeIndex < p1.VolumeIndex
				ORDER BY
					BookTitle,
					VolumeIndex DESC
				LIMIT 1)) AS ChapterTitle
	FROM
		partial p1
)
-- JOIN with the bookmarks and try both ContentIDs
SELECT
	b.ContentID
    , b.Text
    , ct.ChapterTitle
FROM
	Bookmark b
LEFT JOIN
    chapter_titles ct ON (ct.ChapterContentID LIKE b.ContentID || '%' OR ct.ContentID = b.ContentID)
WHERE
	Hidden = 'false'
	AND Text IS NOT NULL
	AND Text <> ''
ORDER BY
    ct.VolumeIndex ASC
    , b.ChapterProgress ASC
"#;

// TODO: remove this duplicate query. Rust does not allow using the const string as a literal for formatting.
const BOOKMARKS_QUERY_FOR_BOOK: &str = r#"
WITH chapters AS (
	SELECT
		ContentID,
		Title,
		VolumeIndex
	FROM
		Content
	WHERE
		ContentType = 899
),
partial AS (
	SELECT
		c1.BookTitle,
		c1.ContentID,
		chapters.ContentID AS ChapterContentID,
		c1.VolumeIndex,
		chapters.Title AS ChapterTitle
	FROM
		Content AS c1
	LEFT JOIN chapters ON chapters.ContentID LIKE c1.ContentID || '%'
WHERE
	ContentType = 9
),
-- Fill missing chapter titles based on volume index
chapter_titles AS (
	SELECT
        VolumeIndex,
		ContentID,
		ChapterContentID,
		COALESCE(ChapterTitle,
			(
				SELECT
					ChapterTitle FROM partial p2
				WHERE
					ChapterTitle IS NOT NULL
					AND p2.BookTitle = p1.BookTitle
					AND p2.VolumeIndex < p1.VolumeIndex
				ORDER BY
					BookTitle,
					VolumeIndex DESC
				LIMIT 1)) AS ChapterTitle
	FROM
		partial p1
)
-- JOIN with the bookmarks and try both ContentIDs
SELECT
	b.ContentID
    , b.Text
    , ct.ChapterTitle
FROM
	Bookmark b
LEFT JOIN
    chapter_titles ct ON (ct.ChapterContentID LIKE b.ContentID || '%' OR ct.ContentID = b.ContentID)
WHERE
	Hidden = 'false'
	AND Text IS NOT NULL
	AND Text <> ''
    AND VolumeID = ?1 -- the only difference with the previous query
ORDER BY
    ct.VolumeIndex ASC
    , b.ChapterProgress ASC
"#;

pub fn bookmarks_query() -> String {
    BOOKMARKS_QUERY.to_string()
}

pub fn bookmarks_for_book_query() -> String {
    BOOKMARKS_QUERY_FOR_BOOK.to_string()
}

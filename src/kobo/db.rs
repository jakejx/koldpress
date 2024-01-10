use rusqlite::Row;
use sea_query::{Alias, Expr, Func, Iden, Order, OrderedStatement, Query, SelectStatement};

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
            title: row.get("Title")?,
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

pub fn bookmarks_query() -> SelectStatement {
    Query::select()
        .column((Bookmark::Table, Bookmark::ContentId))
        .expr_as(Func::coalesce([
            Expr::col(Content::Title).into(),
            Expr::cust("(SELECT Title FROM content WHERE content.ContentID > Bookmark.ContentID AND content.ChapterIDBookmarked IS NOT NULL ORDER BY content.ContentID ASC LIMIT 1)"),
        ]), Alias::new("Title"))
        .column(Bookmark::Text)
        .from(Bookmark::Table).left_join(Content::Table, Expr::col((Bookmark::Table, Bookmark::ContentId)).equals((Content::Table, Content::ChapterIDBookmarked)))
        .and_where(Expr::col(Bookmark::Text).is_not_null())
        .and_where(Expr::col(Bookmark::Hidden).eq("false"))
        .order_by(Bookmark::VolumeId, Order::Desc)
        .order_by(Content::VolumeIndex, Order::Asc).to_owned()
}

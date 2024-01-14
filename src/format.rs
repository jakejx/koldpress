use anyhow::{Context, Result};
use std::{io::Write};
use tera::Tera;

use crate::kobo::library::Chapter;

pub fn json<W: Write>(w: &mut W, bookmarks: &Vec<Chapter>) -> Result<()> {
    Ok(write!(w, "{}", serde_json::to_string_pretty(bookmarks)?)?)
}

const MARKDOWN_TEMPLATE: &str = "markdown";
const DEFAULT_MD_TEMPLATE: &str = include_str!("../default.md.tera");

pub fn markdown<W: Write>(w: &mut W, chapters: &Vec<Chapter>) -> Result<()> {
    let mut tera = Tera::default();
    tera.add_raw_template(MARKDOWN_TEMPLATE, DEFAULT_MD_TEMPLATE)?;
    let mut context = tera::Context::new();
    context.insert("chapters", chapters);
    tera
        .render_to(MARKDOWN_TEMPLATE, &context, w)
        .context("Failed to render template")
}

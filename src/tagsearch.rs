use crate::{
    tagsdb::Entry,
    tagsdb::{TagsDb, Txt},
    utils::*,
};
use percent_encoding::{percent_encode, NON_ALPHANUMERIC};
use std::{fmt, sync::Arc};

#[derive(Clone)]
pub struct TagSearcher {
    vim_db: Arc<TagsDb>,
    neovim_db: Arc<TagsDb>,
    custom_db: Option<Arc<TagsDb>>,
}

#[derive(Clone, Copy, Debug)]
pub enum Flavor {
    Vim,
    NeoVim,
    Custom,
}

impl fmt::Display for Flavor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Flavor::Vim => f.write_str("Vim"),
            Flavor::NeoVim => f.write_str("NeoVim"),
            Flavor::Custom => f.write_str("custom"),
        }
    }
}

impl Flavor {
    pub const fn url_base(self) -> &'static str {
        match self {
            Flavor::Vim => "https://vimhelp.org",
            Flavor::NeoVim => "https://neovim.io/doc/user",
            Flavor::Custom => "",
        }
    }

    pub fn format_url(self, entry: &Entry) -> String {
        match self {
            Flavor::Vim => {
                format!(
                    "{}/{}.txt.html#{}",
                    self.url_base(),
                    entry.filename,
                    percent_encode(entry.topic.as_bytes(), NON_ALPHANUMERIC)
                )
            }
            Flavor::NeoVim => {
                format!(
                    "{}/{}.html#{}",
                    self.url_base(),
                    entry.filename,
                    percent_encode(entry.topic.as_bytes(), NON_ALPHANUMERIC)
                )
            }
            Flavor::Custom => entry.filename.to_string(),
        }
    }
}

impl TagSearcher {
    pub fn from_env() -> Result<Self, Flavor> {
        let vim_db = TagsDb::from_env("VIM_DB_PATH", Txt::Trim)
            .map(Arc::new)
            .ok_or(Flavor::Vim)?;
        let neovim_db = TagsDb::from_env("NEOVIM_DB_PATH", Txt::Trim)
            .map(Arc::new)
            .ok_or(Flavor::NeoVim)?;
        let custom_db = TagsDb::from_env("CUSTOM_DB_PATH", Txt::Keep).map(Arc::new);

        Ok(Self {
            vim_db,
            neovim_db,
            custom_db,
        })
    }

    pub fn search_by_topic(&self, topic: &str) -> impl Iterator<Item = (Entry, Flavor)> {
        use std::{convert::identity, iter::once};

        once(self.vim_db.find(topic).map(|entry| (entry, Flavor::Vim)))
            .chain(once(
                self.neovim_db
                    .find(topic)
                    .map(|entry| (entry, Flavor::NeoVim)),
            ))
            .chain(once(self.custom_db.as_ref().and_then(|db| {
                db.find(topic).map(|entry| (entry, Flavor::Custom))
            })))
            .filter_map(identity)
    }

    fn find_entries_by_text<'a>(
        &'a self,
        text: &'a str,
    ) -> impl Iterator<Item = (Entry, Flavor)> + 'a {
        HELP_REGEX
            .captures_iter(text)
            .filter_map(move |cap| self.search_by_topic(&cap[1]).next())
    }

    pub fn find_by_text(&self, text: &str) -> Option<String> {
        Some(format_message(self.find_entries_by_text(text))).filter(|s| !s.is_empty())
    }
}

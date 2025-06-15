use crate::{
    tagsdb::Entry,
    tagsdb::{TagsDb, Txt},
    utils::HELP_REGEX,
};
use percent_encoding::{percent_encode, NON_ALPHANUMERIC};
use std::fmt;

#[expect(clippy::struct_field_names, reason = "no, wtf is this lint")]
pub struct TagSearcher {
    vim_db: TagsDb,
    neovim_db: TagsDb,
    custom_db: Option<TagsDb>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Flavor {
    Vim,
    NeoVim,
    Custom,
}

impl fmt::Display for Flavor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let tag = match self {
            Flavor::Vim => "Vim",
            Flavor::NeoVim => "NeoVim",
            Flavor::Custom => "Custom",
        };
        f.write_str(tag)
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
                let filename = if entry.filename == "index" {
                    "vimindex"
                } else {
                    &entry.filename
                };
                format!(
                    "{}/{}.html#{}",
                    self.url_base(),
                    filename,
                    percent_encode(entry.topic.as_bytes(), NON_ALPHANUMERIC)
                )
            }
            Flavor::Custom => entry.filename.to_string(),
        }
    }
}

impl TagSearcher {
    pub fn from_env() -> Result<Self, Flavor> {
        let vim_db = TagsDb::from_env("VIM_DB_PATH", Txt::Trim).ok_or(Flavor::Vim)?;
        let neovim_db = TagsDb::from_env("NEOVIM_DB_PATH", Txt::Trim).ok_or(Flavor::NeoVim)?;
        let custom_db = TagsDb::from_env("CUSTOM_DB_PATH", Txt::Keep);

        Ok(Self {
            vim_db,
            neovim_db,
            custom_db,
        })
    }

    pub fn search_by_topic(&self, topic: &str) -> impl Iterator<Item = (Entry, Flavor)> {
        std::iter::once(self.vim_db.find(topic).map(|entry| (entry, Flavor::Vim)))
            .chain([self
                .neovim_db
                .find(topic)
                .map(|entry| (entry, Flavor::NeoVim))])
            .chain([self
                .custom_db
                .as_ref()
                .and_then(|db| db.find(topic).map(|entry| (entry, Flavor::Custom)))])
            .flatten()
    }

    pub fn search_by_text<'a>(
        &'a self,
        text: &'a str,
        preferred_flavor: Flavor,
    ) -> impl Iterator<Item = (Entry, Flavor)> + 'a {
        HELP_REGEX.captures_iter(text).filter_map(move |captures| {
            let topic = &captures[1];
            // false < true
            self.search_by_topic(topic)
                .min_by_key(|(_entry, flavor)| flavor != &preferred_flavor)
        })
    }
}

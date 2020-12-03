use crate::{tagsdb::TagsDb, utils::*};
use std::sync::Arc;

#[derive(Clone)]
pub struct TagSearch {
    vim_db: Arc<TagsDb>,
    neovim_db: Arc<TagsDb>,
}

#[derive(Clone, Copy, Debug)]
pub enum Flavor {
    Vim,
    NeoVim,
}

impl TagSearch {
    pub fn from_env() -> Self {
        let vim_db = TagsDb::from_env("VIM_DB_PATH")
            .map(Arc::new)
            .expect("Failed to parse Vim db");
        let neovim_db = TagsDb::from_env("NEOVIM_DB_PATH")
            .map(Arc::new)
            .expect("Failed to parse Neovim db");
        Self { vim_db, neovim_db }
    }

    pub fn find_by_text(&self, text: &str) -> Option<String> {
        HELP_REGEX
            .captures(text)
            .map(|cap| {
                let topic = &cap[1];
                if let Some(entry) = self.vim_db.find(topic) {
                    Some((entry, Flavor::Vim))
                } else if let Some(entry) = self.neovim_db.find(topic) {
                    Some((entry, Flavor::NeoVim))
                } else {
                    None
                }
            })
            .map(format_message)
            .and_then(|ans| if ans.is_empty() { None } else { Some(ans) })
    }
}

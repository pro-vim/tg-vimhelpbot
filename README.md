# vim-help-bot

## Usage:
- Put your bot token to `TELOXIDE_TOKEN` env var.
- Put path to Vim tags db to `VIM_DB_PATH` env var.
- Put path to NeoVim tags db to `NEOVIM_DB_PATH` env var.
- Run bot.

Use `:h topic` or `:help topic` anywhere in message and bot will send help for `topic`. Its matching algorithm is a lot less sophisticated than Vim's, but will work for most cases. Several uses of `:h(elp) topic` will get you several links. Vim link is given if possible.

If message consists only of one or several help requests, bot will try to delete this message after replying.


## License
This code is available under either MIT or Apache 2.0 license, at your choice.

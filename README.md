# vim-help-bot

## Usage:
- Put your bot token to `TELOXIDE_TOKEN` env var.
- Put path to Vim tags db to `VIM_DB_PATH` env var.
- Put path to NeoVim tags db to `NEOVIM_DB_PATH` env var.
- Run bot.

Use `:h topic` or `:help topic` anywhere in message and bot will send help for `topic`. Its matching algorithm is a lot less sophisticated than Vim's, but will work for most cases. Several uses of `:h(elp) topic` will get you several links. Vim link is given if possible, unless you’re in a group chat and its name contains `neovim` or `nvim` (case-insensitive), then Neovim link is preferred.

If message consists only of one or several help requests, bot will try to delete this message after replying.

### I use Nix btw

You can just do

```shell
$ TELOXIDE_TOKEN=... nix run github:pro-vim/tg-vimhelpbot
```

then, tag files are packaged with the flake.

If you want to actually deploy it, a NixOS module is also provided:

```nix
{
  services.tg-vimhelpbot = {
    enable = true;
    # should contain something like `TELOXIDE_TOKEN='...'`
    envFile = "/path/to/file/with/token.env";
  };
}
```

## License
This code is available under either MIT or Apache 2.0 license, at your choice.

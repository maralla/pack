pack
====

Package manager for vim8.

![demo](http://i.imgur.com/mhkRXPZ.gif)

Install
-------

Currently only macOS and Linux are supported and neovim is not supported.

To install the already compiled binary, go to the [releases](https://github.com/maralla/pack/releases)
page and download the proper compressed binary package:

```bash
$ v=v0.1.0
$ os=x86_64-unknown-linux-gnu
$ wget https://github.com/maralla/pack/releases/download/$v/pack-$v-$os.tar.gz
$ tar -vxf pack-$v-$os.tar.gz
```

Usage
-----

All tasks should be done through `pack` command. `pack` will create a file named
*packfile* under `$VIM_CONFIG_PATH/.pack/` and all plugins are tracked in the file.
Plugin config files are stored under `$VIM_CONFIG_PATH/.pack/`. The config files
will be concatenated and stored under `$VIM_CONFIG_PATH/plugin/_pack.vim` automatically.
These files are all managed by `pack`. Never change the files manually.

By default, if `$VIM_CONFIG_PATH` is not set, `pack` will create and install all files under `~/.vim`(default vim packagepath).
If using custom location by setting `$VIM_CONFIG_PATH` variable, you need to add the following at the top of your `.vimrc`:

```
set packpath+=$VIM_CONFIG_PATH
```

#### `pack` command

```bash
# Show general usage
$ pack -h
```

#### Install plugins

```bash
$ pack help install

# install plugins
# pack install <github_user/github_repo>
$ pack install maralla/completor.vim
$ pack install maralla/completor.vim maralla/completor-neosnippet

# install all plugins
$ pack install

# install optional plugin
$ pack install altercation/vim-colors-solarized -o

# install to a specific category
$ pack install pangloss/vim-javascript -c lang

# install a plugin for types
$ pack install maralla/rope.vim --for python
$ pack install mattn/emmet-vim --for html,jinja,xml

# install a plugin loaded for a command
$ pack install gregsexton/gitv --on Gitv

# install a plugin and build after installed
$ pack install Shougo/vimproc.vim --build 'make'
```

#### Config a plugin

```bash
$ pack config maralla/completor.vim
# This command will open an editor, enter vim scripts as the config for the plugin
# For example:
#
#   let g:completor_css_omni_trigger = '([\w-]+|@[\w-]*|[\w-]+:\s*[\w-]*)$'
```

#### List installed plugins

```bash
$ pack list
```

#### Uninstall plugins

Simple uninstall a plugin will not remove plugin config file. To remove a plugin
config file use `pack uninstall <plugin> -a` or `pack config <plugin> -d`.

```bash
$ pack uninstall maralla/completor.vim
$ pack uninstall maralla/completor.vim maralla/completor-neosnippet
```

#### Update plugins

```bash
$ pack update
$ pack update maralla/completor.vim
$ pack update maralla/completor.vim maralla/completor-neosnippet
```

Misc
----

#### Shell completions

For bash, move `contrib/pack.bash` to `$XDG_CONFIG_HOME/bash_completion` or `/etc/bash_completion.d/`.

For fish, move `contrib/pack.fish` to `$HOME/.config/fish/completions/`.

For zsh, move `contrib/_pack` to one of your `$fpath` directories.

License
-------

Distributed under the terms of the [MIT](LICENSE) license.

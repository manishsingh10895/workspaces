# Workspaces CLI

Simple CLI to manage workspaces locally

Add workspaces and the directories inside it to an sqlite database

## Usage

![Usage](./images/ws-usage.png)

### Add current directory as workspace

`ws add -p .`

### List all workspaces

`ws list`

### Add a directory to a workspace

`ws dir --workspace [name] add -p [path]`

### Open a workspace

`ws open -w [name]`

### Change editor in which workspace should open

`ws editor -n [editor]`

> default for editor is vscode, currently doesn't support screen based text-editors like
> `nvim`, `vim`, configured editor should be able to be opened in a new window, `neovide` will work

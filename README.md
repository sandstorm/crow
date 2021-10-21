# crow - cli command memorizer

## What is crow?
**crow** (command row) is a cli tool to help you memorize other cli commands by saving them with a unique description. Whenever you can't remember a certain command you can then use **crow** to fuzzy search commands by their description.

## TODO MVP

### persistence
* all data is being persisted inside a single json-file inside `$HOME/.config/crow/crow_db.json`

### general usage

* running `crow` opens the fuzzy search, <cr> puts the command to the prompt
* everytime we fuzzy search arrow keys as well as <c-J> and <c-K> allow to move between results
* the command description is shown inside a separate pane next to the result list

### commands
* `add <command>` - adds command to json file and prompts the user to enter a description
* `addlast` - adds the last command found inside the zsh/bash-history and prompts the user to enter a description
* `remove` - opens the fuzzy search and allows the user to type command/description words, typing <cr> will prompt the user for deletion
* `edit` - opens the fuzzy search and allows the user to type command/description words, typing <cr> will ask the user to edit 1) command 2) description 3) cancel
* `help <command>` - opens help for crow or a specific subcommand

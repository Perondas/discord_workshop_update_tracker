# Discord Workshop Update Tracker Bot

This is a simple bot that will periodically check if a steam workshop item has been updated. If it has, it will send a message to a discord channel.

## Commands
* /add - Add a item to the list of items to check for updates
* /add_collection - Add a collection to the list of items to check for updates
* /add_multiple - Add multiple items to the list of items to check for updates

* /remove - Remove a item from the list of items to check for updates
* /remove_all - Removes all items from the list of items to check for updates

* /register_channel - Register a channel to send update messages to
* /set_schedule - Set the interval for checking for updates
* /list - List all the items that are being checked for updates
* /help - Show a list of commands
* /info - Show information about the bot
* /restart - Restart the tracking job for your server
* /edit_note - Edit the note for a specific item

All the commands are purely slash commands.
They can also all be configured to be used by a specific role only.

## Features: 
* Keep track of workshop updates
* Add notes to items, you will be reminded of them when they update
* Total permission control over all commands

## Permissions
The bot requires the following permissions:
* Send Messages

## Setup
Rename the .evn.example file to .env and fill in the values. The bot will not work without this file.
Run docker compose up to start the bot.

## TODO
- [✓] Add a command to manually check for updates
- [✓] Add a command to add an entire collection
- [✓] Add a command to add notes
- [?] Git integration

### Links:
You can find the bot here: https://discord.com/api/oauth2/authorize?client_id=1062792290797629590&permissions=2048&scope=bot
# Discord Workshop Update Tracker Bot

This is a simple bot that will periodically check if a steam workshop item has been updated. If it has, it will send a message to a discord channel.

## Commands
* /item_add - Add a item to the list of items to check for updates
* /item_remove - Remove a item from the list of items to check for updates
* /register_channel - Register a channel to send update messages to
* /set_schedule - Set the interval for checking for updates
* /item_batch_add - Add a list of items to the list of items to check for updates
* /list_items - List all the items that are being checked for updates
* /help - Show a list of commands
* /info - Show information about the bot
* /restart - Restart the tracking job for your server

All the commands are purely slash commands, so you can use them by typing /item
They can also all be configured to be used by a specific role only.

## Permissions
The bot requires the following permissions:
* Send Messages

## Setup
Rename the .evn.example file to .env and fill in the values. The bot will not work without this file.
Run docker compose up to start the bot.

## TODO
- [ ] Add a command to manually check for updates
- [ ] Add a command to add an entire collection

### Links:
You can find the bot here: https://discord.com/api/oauth2/authorize?client_id=1062792290797629590&permissions=2048&scope=bot
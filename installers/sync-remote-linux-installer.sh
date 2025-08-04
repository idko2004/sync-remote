#!/bin/bash
install()
{
	set -e

	echo -e "\033[0;34mDownloading sync-remote v0.0.1\033[0m"
	curl https://github.com/idko2004/sync-remote/releases/download/v0.0.1/sync-remote-linux-v001 -o ~/.cache/sync-remote

	echo -e "\033[0;34mInstalling...\033[0m"
	mv ~/.cache/sync-remote ~/.local/bin/sync-remote

	echo -e "\033[0;34mCreating shorcut...\033[0m"
	touch ~/.cache/sync-remote.desktop

	#Write the file line by line
	echo "[Desktop Entry]" > ~/.cache/sync-remote.desktop
	echo "Type=Application" >> ~/.cache/sync-remote.desktop
	echo "Terminal=true" >> ~/.cache/sync-remote.desktop
	echo "Categories=Utilities;" >> ~/.cache/sync-remote.desktop
	echo "Name=Sync-Remote" >> ~/.cache/sync-remote.desktop
	echo "Icon=folder-html" >> ~/.cache/sync-remote.desktop
	echo "Exec=~/.local/bin/sync-remote" >> ~/.cache/sync-remote.desktop

	mv ~/.cache/sync-remote.desktop ~/.local/share/applications

	echo -e "\033[0;34mInstalled!\033[0m"
}

install
#!/bin/bash
install()
{
	set -e

	PROGRAM_PATH=$(realpath ~/.local/bin/sync-remote)
	TEMP_PATH=$(realpath ~/.cache/sync-remote)

	echo -e "\033[0;34mDownloading sync-remote v0.0.1\033[0m"
	wget https://github.com/idko2004/sync-remote/releases/download/v0.0.1/sync-remote-linux-portable-v001 -O $TEMP_PATH --show-progress --quiet

	echo -e "\033[0;34mChecking file hash\033[0m"
	echo "175e70cc4a51d4eb6374ec5efe57a7e2172acbfa187f6d1237211ed40e21aea9 $TEMP_PATH" | sha256sum -c

	echo -e "\033[0;34mInstalling...\033[0m"
	chmod +x ~/.cache/sync-remote
	echo "Moving to $PROGRAM_PATH"
	mv ~/.cache/sync-remote $PROGRAM_PATH

	echo -e "\033[0;34mCreating shorcut...\033[0m"
	touch $TEMP_PATH.desktop

	#Write the file line by line
	echo "[Desktop Entry]" > ~/.cache/sync-remote.desktop
	echo "Type=Application" >> ~/.cache/sync-remote.desktop
	echo "Terminal=true" >> ~/.cache/sync-remote.desktop
	echo "Categories=Utilities;" >> ~/.cache/sync-remote.desktop
	echo "Name=Sync Remote" >> ~/.cache/sync-remote.desktop
	echo "Icon=folder-html" >> ~/.cache/sync-remote.desktop
	echo "Exec=$PROGRAM_PATH --wait-to-exit" >> ~/.cache/sync-remote.desktop

	mv ~/.cache/sync-remote.desktop ~/.local/share/applications

	echo -e "\033[0;34mInstalled!\033[0m"
}

install
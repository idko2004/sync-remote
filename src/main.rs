mod sync;
mod config;
mod tui;

use crate::tui::TuiResult;

fn main()
{
	let sync_locations = match config::read_config()
	{
		Some(value) => value,
		None => return,
	};

	let mut remote_names: Vec<String> = Vec::with_capacity(sync_locations.len() + 1);
	for location in &sync_locations
	{
		remote_names.push(location.name.clone());
	}
	remote_names.push(String::from("(Add new remote)"));

	match tui::start_tui_blocking(&remote_names)
	{
		TuiResult::SyncRemote(remote_selected) =>
		{
			if remote_selected == remote_names.len() - 1
			{
				println!("TODO: Add new remote ui");
				return;
			}

			let selected_sync_location = match sync_locations.get(remote_selected)
			{
				Some(value) => value,
				None =>
				{
					println!("[ERROR] Failed to get access to the sync location at the index {remote_selected}");
					return;
				}
			};

			sync::start_sync_blocking(selected_sync_location);
		},
		TuiResult::CreateRemote(new_remote_details) =>
		{
			println!("todo! actually create the remote")
		}
	};


}

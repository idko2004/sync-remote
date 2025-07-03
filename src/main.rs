use crate::sync::start_sync_blocking;

mod sync;
mod config;
mod tui;


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

	let selected = match tui::start_tui_blocking(&remote_names)
	{
		Some(value) => value,
		None => return,
	};

	if selected == remote_names.len() - 1
	{
		println!("TODO: Add new remote ui");
		return;
	}

	let selected_sync_location = match sync_locations.get(selected)
	{
		Some(value) => value,
		None =>
		{
			println!("[ERROR] Failed to get access to the sync location at the index {selected}");
			return;
		}
	};

	start_sync_blocking(selected_sync_location);

}

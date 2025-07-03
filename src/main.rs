mod sync;
mod config;
mod tui;


fn main()
{
	let sync_locations = match config::read_config()
	{
		Some(value) => value,
		None =>
		{
			return;
		}
	};

	let mut remote_names: Vec<String> = Vec::with_capacity(sync_locations.len() + 1);
	for location in sync_locations
	{
		remote_names.push(location.name.clone());
	}
	remote_names.push(String::from("(Add new remote)"));

	tui::start_tui_blocking(remote_names);
}

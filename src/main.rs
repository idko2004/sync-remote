mod sync;
mod config;
mod tui;
mod args;

use crate::tui::{NewRemoteDetails, TuiResult};
use crate::config::SyncLocation;

use crossterm::execute;
use crossterm::{queue, style::{Color, Print, SetForegroundColor, SetAttribute, Attribute}};
use std::io::Write;
use html_escape::encode_unquoted_attribute_to_string;

fn main()
{
	let args = args::check_arguments();

	loop
	{
		let sync_locations = match config::get_config()
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

				if args.wait_to_exit
				{
					wait_to_exit();
				}
				break;
			},
			TuiResult::CreateRemote(new_remote_details) =>
			{
				let result = create_new_remote(new_remote_details, &sync_locations);
				if args.wait_to_exit
				{
					wait_to_exit();
				}
				if !result
				{
					break;
				}
			}
		};
	}
}

fn create_new_remote(new_remote_details: NewRemoteDetails, sync_locations: &Vec<SyncLocation>) -> bool
{
	//Get new remote values
	let name = match new_remote_details.name
	{
		Some(value) => value,
		None =>
		{
			print_error_to_save_remote("that somehow doesn't have a name");
			println!("[ERROR] New remote name is invalid!");
			return false;
		}
	};
	let name_encoded = encodify_name(&name);
	let remote = match new_remote_details.remote_url
	{
		Some(value) => value,
		None =>
		{
			print_error_to_save_remote(&name);
			println!("[ERROR] New remote url is invalid!");
			return false;
		}
	};
	let remote_path = match new_remote_details.remote_path
	{
		Some(value) =>
		{
			//In tui.rs, when setting the remote path, in the ui there is a / in the place where you type, indicating the root, however this is not set on the string, so we have to set it now, also we have to check if another / wasn't typed accidentally
			let mut value = match value.starts_with('/')
			{
				true => value,
				false => format!("/{value}")
			};
			//In sync.rs is expected for paths to not end with /, so if a / is present at the end we simply remove it
			if value.ends_with('/') || value.ends_with('\\')
			{
				value.truncate(value.len() -1);
			}
			value
		},
		None =>
		{
			print_error_to_save_remote(&name);
			println!("[ERROR] New remote path is invalid!");
			return false;
		}
	};
	let local_path = match new_remote_details.local_path
	{
		Some(value) =>
		{
			//In sync.rs is expected for paths to not end with /, so if a / is present at the end we simply remove it
			let mut value = value.clone();
			if value.ends_with('/') || value.ends_with('\\')
			{
				value.truncate(value.len() -1);
			}

			//Check poorly if it's an absolute path
			match std::env::consts::OS
			{
				"windows" =>
				{
					//Check for the letter drive
					if !value.starts_with(&['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z'])
					{
						print_error_to_save_remote(&name);
						println!("[ERROR] Local path should be absolute (should include drive letter)");
						return false;
					}

					//Check for the :\ in C:\
					let slice = &value[1..];
					if !slice.starts_with(":\\")
					{
						print_error_to_save_remote(&name);
						println!("[ERROR] Local path should be absolute (should include drive letter)");
						return false;
					}
				},
				_ => //Assume everything else works like linux
				{
					if !value.starts_with('/')
					{
						print_error_to_save_remote(&name);
						println!("[ERROR] Local path should be absolute");
						return false;
					}
				}
			}
			value
		},
		None =>
		{
			print_error_to_save_remote(&name);
			println!("[ERROR] New remote local path is invalid!");
			return false;
		}
	};
	let remote_username = match new_remote_details.remote_username
	{
		Some(value) => value,
		None => String::from("anonymous"),
	};
	let remote_password = match new_remote_details.remote_password
	{
		Some(value) => value,
		None => String::from("anon@localhost"),
	};
	let advanced_backups = match new_remote_details.advanced_backups
	{
		Some(value) => value,
		None => true,
	};

	//Create the remote
	let sync_location = SyncLocation
	{
		name: name.clone(),
		name_encoded: name_encoded,
		remote: remote,
		remote_path: remote_path,
		local_path: local_path,
		remote_username: remote_username,
		remote_password: remote_password,
		advanced_backups: advanced_backups,
	};

	//Chech if there isn't another remote with the name name or codified name
	if !check_if_remote_is_unique(&sync_location, sync_locations)
	{
		print_error_to_save_remote(&name);
		println!("[ERROR] Another remote with this name alredy exists.\n\n[WARN] If the names of the remotes are similar, but not exactly the same, the name of their backup folders might be conflicting. Try changing it a little bit.");
		return false;
	}

	//Save new remote
	match config::add_new_remote(&sync_location)
	{
		true =>
		{
			let mut stdout = std::io::stdout();
			let _ = queue!(stdout, SetAttribute(Attribute::Bold));
			let _ = queue!(stdout, SetForegroundColor(Color::Green));
			let _ = queue!(stdout, Print(format!("Remote {} saved!\n", &name)));
			let _ = queue!(stdout, SetForegroundColor(Color::Reset));
			let _ = queue!(stdout, SetAttribute(Attribute::Reset));
			let _ = stdout.flush();
			
			true
		},
		false =>
		{
			print_error_to_save_remote(&name);
			false
		}
	}
}

fn check_if_remote_is_unique(new_sync_location: &SyncLocation, sync_locations: &Vec<SyncLocation>) -> bool
{
	for sync_location in sync_locations
	{
		if sync_location.name == new_sync_location.name
		{
			return false;
		}
		if sync_location.name_encoded == new_sync_location.name_encoded
		{
			return false;
		}
	}
	true
}

fn print_error_to_save_remote(name: &str)
{
	let mut stdout = std::io::stdout();
	let _ = queue!(stdout, SetAttribute(Attribute::Bold));
	let _ = queue!(stdout, SetForegroundColor(Color::Red));
	let _ = queue!(stdout, Print(format!("Failed to save remote {}!\n", &name)));
	let _ = queue!(stdout, SetForegroundColor(Color::Reset));
	let _ = queue!(stdout, SetAttribute(Attribute::Reset));
	let _ = stdout.flush();
}

fn encodify_name(name: &String) -> String
{
	let mut result = String::new();
	let _ = encode_unquoted_attribute_to_string(name, &mut result);
	let result = result.replace("&#32;", "_");
	let result = result.replace("&#95;", "_");
	result
}

fn wait_to_exit()
{
	let mut stdout = std::io::stdout();
	let _ = execute!(stdout, Print("\nPress any key to exit...\n"));

	loop
	{
		match crossterm::event::read()
		{
			Ok(event) =>
			{
				match event.as_key_event()
				{
					Some(_) =>
					{
						break;
					}
					None => ()
				}
			},
			Err(_) => ()
		}
	}
}

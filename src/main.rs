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
				create_new_remote(new_remote_details);
				if args.wait_to_exit
				{
					wait_to_exit();
				}
			}
		};
	}
}

fn encodify_name(name: &String) -> String
{
	let mut result = String::new();
	let _ = encode_unquoted_attribute_to_string(name, &mut result);
	let result = result.replace("&#32;", "_");
	let result = result.replace("&#95;", "_");
	result
}

fn create_new_remote(new_remote_details: NewRemoteDetails)
{
	let name = match new_remote_details.name
	{
		Some(value) => value,
		None =>
		{
			println!("[ERROR] New remote name is invalid!");
			return;
		}
	};
	let remote = match new_remote_details.remote_url
	{
		Some(value) => value,
		None =>
		{
			println!("[ERROR] New remote url is invalid!");
			return;
		}
	};
	let remote_path = match new_remote_details.remote_path
	{
		Some(value) => format!("/{value}"),
		None =>
		{
			println!("[ERROR] New remote path is invalid!");
			return;
		}
	};
	let local_path = match new_remote_details.local_path
	{
		Some(value) => value,
		None =>
		{
			println!("[ERROR] New remote local path is invalid!");
			return;
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

	let sync_location = SyncLocation
	{
		name: name.clone(),
		name_encoded: encodify_name(&name),
		remote: remote,
		remote_path: remote_path,
		local_path: local_path,
		remote_username: remote_username,
		remote_password: remote_password,
		advanced_backups: advanced_backups,
	};

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
		},
		false =>
		{
			let mut stdout = std::io::stdout();
			let _ = queue!(stdout, SetAttribute(Attribute::Bold));
			let _ = queue!(stdout, SetForegroundColor(Color::Red));
			let _ = queue!(stdout, Print(format!("Failed to save remote {}!\n", &name)));
			let _ = queue!(stdout, SetForegroundColor(Color::Reset));
			let _ = queue!(stdout, SetAttribute(Attribute::Reset));
			let _ = stdout.flush();
		}
	}
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

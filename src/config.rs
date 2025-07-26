use std::{fs, env};
use serde_derive::Serialize;

#[derive(Clone, Serialize)]
pub struct SyncLocation
{
	pub remote: String,
	pub name: String,
	pub name_encoded: String,
	pub remote_path: String,
	pub local_path: String,
	pub remote_username: String,
	pub remote_password: String,
	pub advanced_backups: bool,
}

pub fn get_program_folder() -> String
{
	let default = if cfg!(debug_assertions)
	{
		String::from("sync-remote-debug")
	}
	else
	{
		String::from("sync-remote")
	};

	match env::consts::OS
	{
		"linux" =>
		{
			match env::var("USER")
			{
				Ok(user) =>
				{
					if !user.is_empty()
					{
						format!("/home/{user}/.local/share/idko2004.github.io/{default}")
					}
					else
					{
						default
					}
				},
				Err(error) =>
				{
					println!("[ERROR] USER environment variable is not set, using current directory as data folder ({error})");
					default
				}
			}
			
		},
		_ => default,
	}
}

fn get_config_location() -> String
{
	format!("{}/config.json", get_program_folder())
}

fn read_config_file_as_json() -> Option<serde_json::Value>
{
	let config_file = match fs::read_to_string(get_config_location())
	{
		Ok(value) => value,
		Err(_) =>
		{
			println!("[INFO] Looks like there isn't a config file saved, creating a default one...");
			match save_default_config()
			{
				Some(value) => value,
				None =>
				{
					println!("[ERROR] Failed to save default config file!!");
					return None;
				}
			}
		}
	};

	let json: serde_json::Value = match serde_json::from_str(&config_file.as_str())
	{
		Ok(value) => value,
		Err(error) =>
		{
			println!("[ERROR] Failed to parse config as json, {}  (Please fix this, config file is located at \"{}\")", error, get_config_location());
			return None;
		}
	};

	Some(json)
}

pub fn get_config() -> Option<Vec<SyncLocation>>
{
	let json = match read_config_file_as_json()
	{
		Some(value) => value,
		None => return None,
	};

	let root = match json.as_array()
	{
		Some(value) => value,
		None =>
		{
			println!("[ERROR] JSON config should be an array! (Please fix this, config file is located at \"{}\")", get_config_location());
			return None;
		}
	};

	let mut sync_locations: Vec<SyncLocation> = Vec::with_capacity(root.len());

	for item in root
	{
		match item.as_object()
		{
			Some(obj) =>
			{
				let name = match obj.get("name")
				{
					Some(value) =>
					{
						match value.as_str()
						{
							Some(value) => value,
							None =>
							{
								println!("[ERROR] Config error: field name should be a string! (Please fix this, config file is located at \"{}\")", get_config_location());
								continue;
							}
						}
					},
					None =>
					{
						println!("[ERROR] Config error: remote doesn't have a name field (string). (Please fix this, config file is located at \"{}\")", get_config_location());
						continue;
					}
				};
				let remote = match obj.get("remote")
				{
					Some(value) =>
					{
						match value.as_str()
						{
							Some(value) => value,
							None =>
							{
								println!("[ERROR] Config error: Remote with name \"{name}\" has an invalid field! - remote should be a string! (Please fix this, config file is located at \"{}\")", get_config_location());
								continue;
							}
						}
					},
					None =>
					{
						println!("[ERROR] Config error: remote with name \"{name}\" doesn't have a \"remote\" field (string) (Please fix this, config file is located at \"{}\")", get_config_location());
						continue;
					}
				};
				let name_encoded = match obj.get("name_encoded")
				{
					Some(value) =>
					{
						match value.as_str()
						{
							Some(value) => value,
							None =>
							{
								println!("[ERROR] Config error: Remote with name \"{name}\" has an invalid field! - name_encoded should be a string! (Please fix this, config file is located at \"{}\")", get_config_location());
								continue;
							}
						}
					},
					None =>
					{
						println!("[ERROR] Config error: Remote with name \"{name}\" doesn't have a name_encoded field (string). (Please fix this, config file is located at \"{}\")", get_config_location());
						continue;
					}
				};
				let remote_path = match obj.get("remote_path")
				{
					Some(value) =>
					{
						match value.as_str()
						{
							Some(value) => value,
							None =>
							{
								println!("[ERROR] Config error: Remote with name \"{name}\" has an invalid field! - remote_path should be a string! (Please fix this, config file is located at \"{}\")", get_config_location());
								continue;
							}
						}
					},
					None =>
					{
						println!("[ERROR] Config error: Remote with name \"{name}\" doesn't have a remote_path field (string). (Please fix this, config file is located at \"{}\")", get_config_location());
						continue;
					}
				};
				let local_path = match obj.get("local_path")
				{
					Some(value) =>
					{
						match value.as_str()
						{
							Some(value) => value,
							None =>
							{
								println!("[ERROR] Config error: Remote with name \"{name}\" has an invalid field! - local_path should be a string! (Please fix this, config file is located at \"{}\")", get_config_location());
								continue;
							}
						}
					},
					None =>
					{
						println!("[ERROR] Config error: Remote with name \"{name}\" doesn't have a local_path field (string). (Please fix this, config file is located at \"{}\")", get_config_location());
						continue;
					}
				};
				let remote_username = match obj.get("remote_username")
				{
					Some(value) =>
					{
						match value.as_str()
						{
							Some(value) => value,
							None =>
							{
								println!("[ERROR] Config error: Remote with name \"{name}\" has an invalid field! - remote_username should be a string! (Please fix this, config file is located at \"{}\")", get_config_location());
								continue;
							}
						}
					},
					None =>
					{
						println!("[ERROR] Config error: Remote with name \"{name}\" doesn't have a remote_username field (string). (Please fix this, config file is located at \"{}\")", get_config_location());
						continue;
					}
				};
				let remote_password = match obj.get("remote_password")
				{
					Some(value) =>
					{
						match value.as_str()
						{
							Some(value) => value,
							None =>
							{
								println!("[ERROR] Config error: Remote with name \"{name}\" has an invalid field! - remote_password should be a string! (Please fix this, config file is located at \"{}\")", get_config_location());
								continue;
							}
						}
					},
					None =>
					{
						println!("[ERROR] Remote with name \"{name}\" doesn't have a remote_password field (string). (Please fix this, config file is located at \"{}\")", get_config_location());
						continue;
					}
				};
				let advanced_backups = match obj.get("advanced_backups")
				{
					Some(value) =>
					{
						match value.as_bool()
						{
							Some(value) => value,
							None =>
							{
								println!("[ERROR] Config error: Remote with name \"{name}\" has an invalid field! - advanced_backups should be a boolean! (Please fix this, config file is located at \"{}\")", get_config_location());
								continue;
							}
						}
					},
					None =>
					{
						println!("[ERROR] Remote with name \"{name}\" doesn't have an advanced_backups field (boolean). (Please fix this, config file is located at \"{}\")", get_config_location());
						continue;
					}
				};

				sync_locations.push
				(
					SyncLocation
					{
						remote: String::from(remote),
						name: String::from(name),
						name_encoded: String::from(name_encoded),
						remote_path: String::from(remote_path),
						local_path: String::from(local_path),
						remote_username: String::from(remote_username),
						remote_password: String::from(remote_password),
						advanced_backups: advanced_backups,
					}
				);
			},
			None =>
			{
				println!("[ERROR] Config error: Remotes in config file should be objects! (Please fix this, config file is located at \"{}\")", get_config_location());
			}
		}
	}

	Some(sync_locations)
}

fn save_default_config() -> Option<String>
{
	let default_config_contents = "[]"; //Un array vacío en json
	match fs::create_dir_all(get_program_folder())
	{
		Ok(_) =>
		{
			match fs::write(get_config_location(), default_config_contents)
			{
				Ok(_) => Some(String::from(default_config_contents)), //Devolver un vector vacío porque no hay ningún remote por defecto.
				Err(error) =>
				{
					println!("[ERROR] Failed to save config file!! {error}");
					None
				}
			}
		},
		Err(error) =>
		{
			println!("[ERROR] Failed to create program folder!! {error}");
			None
		}
	}
}

pub fn add_new_remote(remote: &SyncLocation) -> bool
{
	let mut config = match read_config_file_as_json()
	{
		Some(value) => value,
		None =>
		{
			println!("[ERROR] Failed to save new remote!");
			return false;
		}
	};

	let root = match config.as_array_mut()
	{
		Some(value) => value,
		None =>
		{
			println!("[ERROR] current config is not an array somehow!");
			return false;
		}
	};

	let new_remote = match serde_json::to_value(remote)
	{
		Ok(value) => value,
		Err(error) =>
		{
			println!("[ERROR] Failed to serialize remote to json! ({error})");
			return false;
		}
	};

	root.push(new_remote);

	let new_root_json = match serde_json::to_value(root)
	{
		Ok(value) => value,
		Err(error) =>
		{
			println!("[ERROR] Failed to serialize root array back to json! ({error})");
			return false;
		}
	};

	let json_string = new_root_json.to_string();

	match fs::write(get_config_location(), json_string)
	{
		Ok(_) => true,
		Err(error) =>
		{
			println!("[ERROR] Failed to write config file! ({error})");
			false
		}
	}
}

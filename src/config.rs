use std::fs;

#[derive(Clone)]
pub struct SyncLocation
{
	pub remote: String,
	pub name: String,
	pub remote_path: String,
	pub local_path: String,
	pub remote_username: String,
	pub remote_password: String,
}

fn get_config_location() -> String
{
	String::from("config.json")
}

pub fn read_config() -> Option<Vec<SyncLocation>>
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
			println!("[ERROR] Failed to parse config as json, {}", error);
			return None;
		}
	};

	let root = match json.as_array()
	{
		Some(value) => value,
		None =>
		{
			println!("[ERROR] JSON config should be an array!");
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
				let remote = match obj.get("remote")
				{
					Some(value) =>
					{
						match value.as_str()
						{
							Some(value) => value,
							None =>
							{
								println!("[ERROR] Some elements of the config file are invalid! - remote should be a string!");
								continue;
							}
						}
					},
					None =>
					{
						println!("[ERROR] Some elements of the config file are invalid! - Failed to obtain remote value");
						continue;
					}
				};
				let name = match obj.get("name")
				{
					Some(value) =>
					{
						match value.as_str()
						{
							Some(value) => value,
							None =>
							{
								println!("[ERROR] Some elements of the config file are invalid! - name should be a string!");
								continue;
							}
						}
					},
					None =>
					{
						println!("[ERROR] Some elements of the config file are invalid! - Failed to obtain name value");
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
								println!("[ERROR] Some elements of the config file are invalid! - remote_path should be a string!");
								continue;
							}
						}
					},
					None =>
					{
						println!("[ERROR] Some elements of the config file are invalid! - Failed to obtain remote_path value");
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
								println!("[ERROR] Some elements of the config file are invalid! - local_path should be a string!");
								continue;
							}
						}
					},
					None =>
					{
						println!("[ERROR] Some elements of the config file are invalid! - Failed to obtain remote value");
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
								println!("[ERROR] Some elements of the config file are invalid! - remote_username should be a string!");
								continue;
							}
						}
					},
					None =>
					{
						println!("[ERROR] Some elements of the config file are invalid! - Failed to obtain remote_username value");
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
								println!("[ERROR] Some elements of the config file are invalid! - remote_password should be a string!");
								continue;
							}
						}
					},
					None =>
					{
						println!("[ERROR] Some elements of the config file are invalid! - Failed to obtain remote_password value");
						continue;
					}
				};

				sync_locations.push
				(
					SyncLocation
					{
						remote: String::from(remote),
						name: String::from(name),
						remote_path: String::from(remote_path),
						local_path: String::from(local_path),
						remote_username: String::from(remote_username),
						remote_password: String::from(remote_password),
					}
				);
			},
			None =>
			{
				println!("[ERROR] Some elements of the config file are invalid!");
			}
		}
	}

	Some(sync_locations)
}

fn save_default_config() -> Option<String>
{
	let default_config_contents = "[]"; //Un array vacío en json
	match fs::write(get_config_location(), default_config_contents)
	{
		Ok(_) => (),
		Err(error) =>
		{
			println!("[ERROR] Failed to save config file!! {error}");
			return None;
		}
	}
	Some(String::from(default_config_contents)) //Devolver un vector vacío porque no hay ningún remote por defecto.
}

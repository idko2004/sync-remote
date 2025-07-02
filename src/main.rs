use std::str::FromStr;
use chrono::{DateTime, Utc};
use suppaftp::{FtpStream, list};

struct File
{
	_directory: String,
	fullpath: String,
	date_modified: DateTime<Utc>,
	_ftp_file: Option<list::File>,
}

fn main()
{
	println!("Hello, world!");

	let mut ftp_stream = match FtpStream::connect("192.168.0.201:8888")
	{
		Ok(value) => value,
		Err(error) =>
		{
			println!("Error al conectarse al servidor, {}", error);
			return;
		}
	};

	match ftp_stream.login("j6", "patata")
	{
		Ok(_) =>
		{
			println!("Sesión iniciada!");
			()
		}
		Err(error) =>
		{
			println!("Error al iniciar sesión, {}", error);
			return;
		}
	}
	/*
	let directory = match ftp_stream.pwd()
	{
		Ok(value) => value,
		Err(error) =>
		{
			println!("Error al obtener directorio actual, {}", error);
			return;
		}
	};
	println!("Directorio actual: {directory}");
	*/

	let directory = String::from("/");
	let all_files = get_all_files_recursive_from(&directory, &mut ftp_stream);

	println!("All files:");
	for file in all_files
	{
		println!("{} {}",file.date_modified.to_string(), file.fullpath);
	}

}

fn get_all_files_recursive_from(directory: &String, ftp_stream: &mut FtpStream) -> Vec<File>
{
	let mut current_directory: String = directory.clone();
	let mut tree: Vec<File> = Vec::new();
	
	let mut directories: Vec<String> = Vec::new();
	let mut i: usize = 0;
	
	loop
	{
		match list_directory(&current_directory, ftp_stream)
		{
			Some(files_vector) =>
			{
				if files_vector.is_empty()
				{
					println!("Directory is empty: {}", current_directory);
					()
				}

				for ftp_file in files_vector
				{
					let fullpath = if current_directory == "/"
					{
						format!("{}{}", current_directory, ftp_file.name())
					}
					else
					{
						format!("{}/{}", current_directory, ftp_file.name())
					};

					if ftp_file.is_directory()
					{
						directories.push(fullpath);
					}
					else if ftp_file.is_file()
					{
						let date_modified: DateTime<Utc> = ftp_file.modified().into();
						tree.push
						(
							File
							{
								_directory: current_directory.clone(),
								fullpath: fullpath,
								date_modified: date_modified,
								_ftp_file: Some(ftp_file.clone()),
							}
						);
					}
				}
			},
			None =>
			{
				println!("Failed to list directory {}", current_directory);
			}
		}

		current_directory = match directories.get(i)
		{
			Some(value) => value.clone(),
			None =>
			{
				println!("Directories list is empty, no more directories to explore");
				break;
			}
		};	
		i += 1;
	}

	tree
}

fn list_directory(directory: &String, ftp_stream: &mut FtpStream) -> Option<Vec<list::File>>
{
	let directory_listing = match ftp_stream.list(Some(directory.as_str()))
	{
		Ok(value) => value,
		Err(error) =>
		{
			println!("No se pudo listar el contenido del directorio {directory}, {error}");
			return None;
		}
	};

	let mut directory_contents: Vec<list::File> = Vec::with_capacity(directory_listing.len());
	
	for item in directory_listing
	{
		match list::File::from_str(item.as_str())
		{
			Ok(value) =>
			{
				directory_contents.push(value);
			},
			Err(error) =>
			{
				println!("Error al interpretar el directorio actual, {}", error);
				continue;
			}
		}
	}

	Some(directory_contents)
}

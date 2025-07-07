use std::{fs, str::FromStr, io::{self, Stdout, stdout, Write}, time::Duration};
use chrono::{DateTime, Timelike, Utc};
use suppaftp::{FtpStream, list};
use crossterm::
{
	cursor::
	{
		MoveTo,
		Hide,
		Show,
	},
	event::
	{
		Event,
		KeyModifiers,
		KeyCode,
		read
	},
	execute,
	queue,
	style::
	{
		Color,
		Print,
		SetBackgroundColor,
		SetForegroundColor,
		SetAttribute,
		Attribute,
	},
	terminal::
	{
		disable_raw_mode,
		enable_raw_mode,
		size,
		Clear,
		ClearType,
		EnterAlternateScreen,
		LeaveAlternateScreen
	}
};


use crate::config::SyncLocation;


#[derive(Clone, Debug)]
struct File
{
	directory: String,
	fullpath: String,
	relative_path: String,
	date_modified: DateTime<Utc>,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum SyncVeredict
{
	UploadToRemote,
	DownloadToLocal,
	DoNothing,
	NotDecidedYet,
}

#[derive(Clone, Debug)]
struct LinkedFile
{
	relative_path: String,
	relative_directory: String,
	local_file: Option<File>,
	remote_file: Option<File>,
	sync_veredict: SyncVeredict,
}

pub fn start_sync_blocking(sync_location: &SyncLocation)
{
	println!("\nRemote: {}", sync_location.name);
	println!("Host: {}\n", sync_location.remote);
	println!("* Connecting...");
	let ftp_stream = match FtpStream::connect(&sync_location.remote)
	{
		Ok(value) => value,
		Err(error) =>
		{
			println!("[ERROR] Failed to connect to server, {}", error);
			return;
		}
	};

	let mut ftp_stream = ftp_stream.active_mode(Duration::from_secs(120));

	println!("* Logging in...");
	match ftp_stream.login(&sync_location.remote_username, &sync_location.remote_password)
	{
		Ok(_) =>
		{
			//println!("¡Sesión iniciada!");
			()
		}
		Err(error) =>
		{
			println!("[ERROR] Failed to log in, {}", error);
			return;
		}
	}

	println!("* Listing remote files...");
	let all_remote_files = get_all_remote_files_recursive_from(&sync_location.remote_path, &mut ftp_stream);

	println!("* Listing local files...");
	let all_local_files = get_all_local_files_recursive_from(&sync_location.local_path);

	println!("* Linking files...");
	let all_files_linked = link_all_files(&all_remote_files, &all_local_files, sync_location);

	println!("* Thinking how to sync...");
	let all_files_linked = set_sync_veredicts(all_files_linked);
	
	println!("* Syncing...");
	sync_files(&all_files_linked, sync_location, &mut ftp_stream);

	/*
	for a in all_files_linked
	{
		println!("{:?}", a);
	}
	*/
/*
	println!("\n\nAll linked files:");
	for file in &all_files_linked
	{
		if file.local_file.is_some() && file.remote_file.is_some()
		{
			println!("{:?}\n{}\nremote:\t{}\nlocal:\t{}\n",file.sync_veredict, file.relative_path, file.remote_file.clone().unwrap().date_modified, file.local_file.clone().unwrap().date_modified);
		}
	}

	println!("\n\nFiles only on remote:");
	for file in &all_files_linked
	{
		if file.local_file.is_none() && file.remote_file.is_some()
		{
			println!("{:?}\n{}\nremote:\t{}\nlocal:\t{}\n",file.sync_veredict,file.relative_path, file.remote_file.clone().unwrap().date_modified, "(None)");
		}
	}

	println!("\n\nFiles only on local:");
	for file in &all_files_linked
	{
		if file.local_file.is_some() && file.remote_file.is_none()
		{
			println!("{:?}\n{}\nremote:\t{}\nlocal:\t{}\n",file.sync_veredict,file.relative_path, "(None)", file.local_file.clone().unwrap().date_modified);
		}
	}
*/
}

fn get_all_remote_files_recursive_from(directory: &String, ftp_stream: &mut FtpStream) -> Vec<File>
{
	let mut current_directory: String = directory.clone();
	let mut tree: Vec<File> = Vec::new();
	
	let mut directories: Vec<String> = Vec::new();
	let mut i: usize = 0;
	
	loop
	{
		match list_remote_directory(&current_directory, ftp_stream)
		{
			Some(files_vector) =>
			{
				if files_vector.is_empty()
				{
					//println!("Directory is empty: {}", current_directory);
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
						let relative_path = fullpath.clone().replacen(directory, "", 1).to_string();
						let relative_path = if relative_path.starts_with("/")
						{
							relative_path
						}
						else
						{
							format!("/{relative_path}")
						};

						tree.push
						(
							File
							{
								directory: current_directory.clone(),
								fullpath: fullpath.clone(),
								relative_path: relative_path,
								date_modified: date_modified,
								//handler: FileHandler::FtpFile(Some(ftp_file.clone())),
							}
						);
					}
				}
			},
			None =>
			{
				println!("[ERROR] Failed to list remote directory {}", current_directory);
			}
		}

		current_directory = match directories.get(i)
		{
			Some(value) => value.clone(),
			None => break,
		};	
		i += 1;
	}

	tree
}

fn list_remote_directory(directory: &String, ftp_stream: &mut FtpStream) -> Option<Vec<list::File>>
{
	let directory_listing = match ftp_stream.list(Some(directory.as_str()))
	{
		Ok(value) => value,
		Err(error) =>
		{
			println!("[ERROR] Failed to list content of directory {directory}, {error}");
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
				println!("[ERROR] Failed to parse remote directory, {}", error);
				continue;
			}
		}
	}

	Some(directory_contents)
}

fn get_all_local_files_recursive_from(directory: &String) -> Vec<File>
{
	let mut current_directory = directory.clone();
	let mut tree: Vec<File> = Vec::new();
	
	let mut directories: Vec<String> = Vec::new();
	let mut i: usize = 0;

	loop
	{
		match list_local_directory(&current_directory) //Obtener un iterador que contiene todos los elementos dentro de un directorio
		{
			Some(value) =>
			{
				for item in value //Iterar por cada elemento ("archivo") en un directorio
				{
					match item
					{
						Ok(dir_entry) =>
						{
							let fullpath = match dir_entry.file_name().into_string() //Obtener la ruta completa
							{
								Ok(value) =>
								{
									if current_directory == "/"
									{
										format!("{}{}", current_directory, value)
									}
									else
									{
										format!("{}/{}", current_directory, value)
									}
								},
								Err(_) =>
								{
									println!("[ERROR] Failed to get file name of something in local directory");
									continue;
								}
							};

							match dir_entry.file_type()
							{
								Ok(value) =>
								{
									//Si es un directorio añadir a la lista de directorios a explorar para explorar en otra iteración.
									if value.is_dir()
									{
										directories.push(fullpath);
									}
									else if value.is_file()
									{
										//Conseguir la fecha modificada
										let date_modified = match dir_entry.metadata()
										{
											Ok(metadata) =>
											{
												match metadata.modified()
												{
													Ok(modified) =>
													{
														let date_modified: DateTime<Utc> = modified.into();
														let date_modified = match date_modified.with_second(0)
														{
															Some(value) =>
															{
																match value.with_nanosecond(0)
																{
																	Some(value) => value,
																	None =>
																	{
																		println!("[ERROR] Failed to trim local file nanoseconds");
																		continue;
																	}
																}
															},
															None =>
															{
																println!("[ERROR] Failed to trim local file seconds");
																continue;
															}
														};
														//println!("{}", date_modified.time().to_string());
														date_modified
													},
													Err(error) =>
													{
														println!("[ERROR] Failed to get last modified time of file in local directory \"{}\", {}", fullpath, error);
														continue;
													}
												}
											},
											Err(error) =>
											{
												println!("[ERROR] Failed to get metadata of file in local directory \"{}\", {}", fullpath, error);
												continue;
											}
										};

										let relative_path = fullpath.clone().replacen(directory, "", 1).to_string();
										let relative_path = if relative_path.starts_with("/")
										{
											relative_path
										}
										else
										{
											format!("/{relative_path}")
										};

										//Añadir al árbol de archivos que en realidad no es un árbol y es una lista nomás.
										tree.push
										(
											File
											{
												directory: current_directory.clone(),
												fullpath: fullpath.clone(),
												relative_path: relative_path,
												date_modified: date_modified,
												//handler: FileHandler::LocalFile,
											}
										);
									}
								},
								Err(error) =>
								{
									println!("[ERROR] Failed to get file type of something in local directory, {}", error);
								}
							}
						},
						Err(error) =>
						{
							println!("[ERROR] Failed to access something in local directory, {}", error);
						}
					}
				}
			},
			None =>
			{
				println!("[ERROR] Failed to list local directory {}", current_directory);
			}
		}

		//Cambiar al siguiente directorio en la lista de directorios
		current_directory = match directories.get(i)
		{
			Some(value) => value.clone(),
			None => break,
		};	
		i += 1;
	}

	tree
}

fn list_local_directory(directory: &String) -> Option<fs::ReadDir>
{
	match fs::read_dir(directory)
	{
		Ok(value) => Some(value),
		Err(_) => None,
	}
}

fn link_all_files(all_remote_files: &Vec<File>, all_local_files: &Vec<File>, sync_location: &SyncLocation) -> Vec<LinkedFile>
{
	let mut all_linked_files = Vec::with_capacity(all_remote_files.len());

	//Linkear todos los que tienen equivalentes en ambos lados
	for remote_file in all_remote_files
	{
		for local_file in all_local_files
		{
			if remote_file.relative_path == local_file.relative_path
			{
				all_linked_files.push
				(
					LinkedFile
					{
						relative_path: remote_file.relative_path.clone(),
						relative_directory: get_relative_directory(remote_file, &sync_location.remote_path),
						local_file: Some(local_file.clone()),
						remote_file: Some(remote_file.clone()),
						sync_veredict: SyncVeredict::NotDecidedYet,
					}
				)
			}
		}
	}

	//Encontrar archivos que estén en el remote pero no estén en local
	for remote_file in all_remote_files
	{
		let mut found = false;
		for linked_file in &all_linked_files
		{
			let linked_remote_file = match &linked_file.remote_file
			{
				Some(value) => value,
				None => continue,
			};

			if remote_file.fullpath == linked_remote_file.fullpath
			{
				found = true;
				break;
			}
		}

		if !found
		{
			all_linked_files.push
			(
				LinkedFile
				{
					relative_path: remote_file.relative_path.clone(),
					relative_directory: get_relative_directory(remote_file, &sync_location.remote_path),
					local_file: None,
					remote_file: Some(remote_file.clone()),
					sync_veredict: SyncVeredict::DownloadToLocal,
				}
			)
		}
	}

	//Encontrar archivos que estén en local pero no estén en remote
	for local_file in all_local_files
	{
		let mut found = false;
		for linked_file in &all_linked_files
		{
			let linked_remote_file = match &linked_file.local_file
			{
				Some(value) => value,
				None => continue,
			};

			if local_file.fullpath == linked_remote_file.fullpath
			{
				found = true;
				break;
			}
		}

		if !found
		{
			all_linked_files.push
			(
				LinkedFile
				{
					relative_path: local_file.relative_path.clone(),
					relative_directory: get_relative_directory(local_file, &sync_location.local_path),
					local_file: Some(local_file.clone()),
					remote_file: None,
					sync_veredict: SyncVeredict::UploadToRemote,
				}
			)
		}
	}

	all_linked_files
}

fn set_sync_veredicts(all_linked_files: Vec<LinkedFile>) -> Vec<LinkedFile>
{
	let mut new_linked_files_list: Vec<LinkedFile> = Vec::with_capacity(all_linked_files.len());
	
	for linked_file in &all_linked_files
	{
		let mut linked_file = linked_file.clone();

		if linked_file.sync_veredict != SyncVeredict::NotDecidedYet
		{
			new_linked_files_list.push(linked_file);
			continue;
		}

		if linked_file.local_file.is_none() && linked_file.remote_file.is_some()
		{
			linked_file.sync_veredict = SyncVeredict::DownloadToLocal;
		}
		else if linked_file.local_file.is_some() && linked_file.remote_file.is_none()
		{
			linked_file.sync_veredict = SyncVeredict::UploadToRemote;
		}
		else if linked_file.local_file.is_some() && linked_file.remote_file.is_some()
		{
			let local_date_modified = match &linked_file.local_file
			{
				Some(file) => file.date_modified,
				None =>
				{
					println!("[ERROR] somehow local file is none after comprobing that it's not??");
					continue;
				}
			};
			let remote_date_modified = match &linked_file.remote_file
			{
				Some(file) => file.date_modified,
				None =>
				{
					println!("[ERROR] somehow remote file is none after comprobing that it's not??");
					continue;
				}
			};

			//println!("l: {}, r: {}, {}", local_date_modified.timestamp(), remote_date_modified.timestamp(), linked_file.relative_path);

			if local_date_modified.timestamp() == remote_date_modified.timestamp()
			{
				linked_file.sync_veredict = SyncVeredict::DoNothing;
			}
			else if local_date_modified.timestamp() < remote_date_modified.timestamp()
			{
				linked_file.sync_veredict = SyncVeredict::DownloadToLocal;
			}
			else
			{
				linked_file.sync_veredict = SyncVeredict::UploadToRemote;
			}
		}
		else
		{
			println!("[ERROR] No idea what to do with this file: {}", linked_file.relative_path);
		}

		new_linked_files_list.push(linked_file);
	}

	new_linked_files_list
}

//Debe asegurarse de que ambas rutas provengan del mismo lugar.
//Si es que file es de un remote, entonces sync_location_path debería venir de SyncLocation::remote_path,
//Del mismo modo, si file es local, entonces sync_location_path debería venir de SyncLocation::local_path.
fn get_relative_directory(file: &File, sync_location_path: &String) -> String
{
	let relative_directory = file.directory.replacen(sync_location_path, "",1).to_string();
	
	if relative_directory.starts_with("/")
	{
		relative_directory
	}
	else
	{
		format!("/{relative_directory}")
	}
}

fn sync_files(all_linked_files: &Vec<LinkedFile>, sync_location: &SyncLocation, ftp_stream: &mut FtpStream)
{
	do_nothing(all_linked_files);
	upload_to_remote(all_linked_files, sync_location, ftp_stream);
	download_to_local(all_linked_files, sync_location, ftp_stream);

}

fn upload_to_remote(all_linked_files: &Vec<LinkedFile>, sync_location: &SyncLocation, ftp_stream: &mut FtpStream)
{
	let mut stdout = io::stdout();

	for linked_file in all_linked_files
	{
		if linked_file.sync_veredict == SyncVeredict::UploadToRemote
		{
			{ //Imprimir bonito
				let _ = queue!(stdout, SetAttribute(Attribute::Bold));
				let _ = queue!(stdout, SetForegroundColor(Color::Green));
				let _ = queue!(stdout, Print("\n->"));
				let _ = queue!(stdout, SetForegroundColor(Color::Reset));
				let _ = queue!(stdout, Print(" Uploading: "));
				let _ = queue!(stdout, SetAttribute(Attribute::Reset));
				let _ = queue!(stdout, Print(format!("{}", linked_file.relative_path)));
				let _ = stdout.flush();
			}

			let remote_directory = format!("{}{}", sync_location.remote_path, linked_file.relative_directory);
			let remote_fullpath = format!("{}{}", sync_location.remote_path, linked_file.relative_path);
			//println!("{remote_directory}");

			//Comprobar que el directorio existe
			let directory_exists: bool = match ftp_stream.list(Some(remote_directory.as_str()))
			{
				Ok(_) => true,
				Err(_) => false,
			};

			//Crear directorio si no existe
			if !directory_exists
			{
				match ftp_stream.mkdir(remote_directory.clone())
				{
					Ok(_) => (),
					Err(error) =>
					{
						let _ = queue!(stdout, SetForegroundColor(Color::Red));
						let _ = queue!(stdout, Print(" (failed!) \n[ERROR] "));
						let _ = queue!(stdout, SetForegroundColor(Color::Reset));
						let _ = queue!(stdout, Print(format!("Failed to create remote directory ({}), {}", remote_directory.clone(), error)));
						let _ = stdout.flush();
						continue;
					}
				}
			}

			//Cargar archivo local
			let local_file = match &linked_file.local_file
			{
				Some(value) => value,
				None =>
				{
					let _ = queue!(stdout, SetForegroundColor(Color::Red));
					let _ = queue!(stdout, Print(" (failed!) \n[ERROR] "));
					let _ = queue!(stdout, SetForegroundColor(Color::Reset));
					let _ = queue!(stdout, Print(format!("Failed to access internal local file handler! ({})", &remote_directory)));
					let _ = stdout.flush();
					continue;
				}
			};

			let mut local_file_handler = match fs::File::open(local_file.fullpath.clone())
			{
				Ok(value) => value,
				Err(error) =>
				{
					let _ = queue!(stdout, SetForegroundColor(Color::Red));
					let _ = queue!(stdout, Print(" (failed!) \n[ERROR] "));
					let _ = queue!(stdout, SetForegroundColor(Color::Reset));
					let _ = queue!(stdout, Print(format!("Failed to open local file!! ({}) {}", &remote_directory, error)));
					let _ = stdout.flush();
					continue;
				}
			};

			//Subir archivo a remote
			match ftp_stream.put_file(remote_fullpath, &mut local_file_handler)
			{
				Ok(_) => (),
				Err(error) =>
				{
					let _ = queue!(stdout, SetForegroundColor(Color::Red));
					let _ = queue!(stdout, Print(" (failed!) \n[ERROR] "));
					let _ = queue!(stdout, SetForegroundColor(Color::Reset));
					let _ = queue!(stdout, Print(format!("Failed to upload file to remote ({}), {}", &remote_directory, error)));
					let _ = stdout.flush();
					continue;
				}
			}

			{ //Imprimir bonito
				let _ = queue!(stdout, SetForegroundColor(Color::Green));
				let _ = queue!(stdout, Print(" (done!)"));
				let _ = queue!(stdout, SetForegroundColor(Color::Reset));
				let _ = stdout.flush();
			}
		}
	}
}

fn download_to_local(all_linked_files: &Vec<LinkedFile>, sync_location: &SyncLocation, ftp_stream: &mut FtpStream)
{
	let mut stdout = io::stdout();

	for linked_file in all_linked_files
	{
		if linked_file.sync_veredict == SyncVeredict::DownloadToLocal
		{
			{ //Imprimir bonito
				let _ = queue!(stdout, SetAttribute(Attribute::Bold));
				let _ = queue!(stdout, SetForegroundColor(Color::Blue));
				let _ = queue!(stdout, Print("\n<-"));
				let _ = queue!(stdout, SetForegroundColor(Color::Reset));
				let _ = queue!(stdout, Print(" Downloading: "));
				let _ = queue!(stdout, SetAttribute(Attribute::Reset));
				let _ = queue!(stdout, Print(format!("{}", linked_file.relative_path)));
				let _ = stdout.flush();
			}

			let local_directory = format!("{}{}", sync_location.local_path, linked_file.relative_directory);
			let local_fullpath = format!("{}{}", sync_location.local_path, linked_file.relative_path);
			//println!("{remote_directory}");

			//Comprobar que el directorio existe
			let directory_exists: bool = match list_local_directory(&local_directory)
			{
				Some(_) => true,
				None => false,
			};

			//Crear directorio si no existe
			if !directory_exists
			{
				match fs::create_dir(local_directory.as_str())
				{
					Ok(_) => (),
					Err(error) =>
					{
						let _ = queue!(stdout, SetForegroundColor(Color::Red));
						let _ = queue!(stdout, Print(" (failed!) \n[ERROR] "));
						let _ = queue!(stdout, SetForegroundColor(Color::Reset));
						let _ = queue!(stdout, Print(format!("Failed to create directory ({}), {}", &local_directory, error)));
						let _ = stdout.flush();
						continue;
					}
				}
			}

			//Preparar archivo local
			let mut local_file_handler = match fs::OpenOptions::new().read(true).write(true).create(true).open(local_fullpath)
			{
				Ok(value) => value,
				Err(error) =>
				{
					let _ = queue!(stdout, SetForegroundColor(Color::Red));
					let _ = queue!(stdout, Print(" (failed!) \n[ERROR] "));
					let _ = queue!(stdout, SetForegroundColor(Color::Reset));
					let _ = queue!(stdout, Print(format!("Failed to create local file or open existing local file as writable ({}), {}", &linked_file.relative_path, error)));
					let _ = stdout.flush();
					continue;
				}
			};

			//Cargar archivo remote
			let remote_file = match &linked_file.remote_file
			{
				Some(value) => value,
				None =>
				{
					let _ = queue!(stdout, SetForegroundColor(Color::Red));
					let _ = queue!(stdout, Print(" (failed!) \n[ERROR] "));
					let _ = queue!(stdout, SetForegroundColor(Color::Reset));
					let _ = queue!(stdout, Print(format!("Failed to access internal remote file handler! ({})", &linked_file.relative_path)));
					let _ = stdout.flush();
					continue;
				}
			};
			
			let mut remote_file_handler = match ftp_stream.retr_as_stream(&remote_file.fullpath)
			{
				Ok(value) => value,
				Err(error) =>
				{
					let _ = queue!(stdout, SetForegroundColor(Color::Red));
					let _ = queue!(stdout, Print(" (failed!) \n[ERROR] "));
					let _ = queue!(stdout, SetForegroundColor(Color::Reset));
					let _ = queue!(stdout, Print(format!("Unable to retrieve remote file ({}), {}", &linked_file.relative_path, error)));
					let _ = stdout.flush();
					continue;
				}
			};

			//Descargar archivo local
			match io::copy(&mut remote_file_handler, &mut local_file_handler)
			{
				Ok(_) =>
				{
					match ftp_stream.finalize_retr_stream(remote_file_handler)
					{
						Ok(_) => (),
						Err(error) =>
						{
							let _ = queue!(stdout, SetForegroundColor(Color::Red));
							let _ = queue!(stdout, Print(" (failed!) \n[ERROR] "));
							let _ = queue!(stdout, SetForegroundColor(Color::Reset));
							let _ = queue!(stdout, Print(format!("Failed to finalize remote stream ({}), {}", &linked_file.relative_path, error)));
							let _ = stdout.flush();
							continue;
						}
					}
				},
				Err(error) =>
				{
					let _ = queue!(stdout, SetForegroundColor(Color::Red));
					let _ = queue!(stdout, Print(" (failed!) \n[ERROR] "));
					let _ = queue!(stdout, SetForegroundColor(Color::Reset));
					let _ = queue!(stdout, Print(format!("Failed to download file ({}), {}", &linked_file.relative_path, error)));
					let _ = stdout.flush();
					continue;
				}
			}

			{ //Imprimir bonito
				let _ = queue!(stdout, SetForegroundColor(Color::Green));
				let _ = queue!(stdout, Print(" (done!)"));
				let _ = queue!(stdout, SetForegroundColor(Color::Reset));
				let _ = stdout.flush();
			}
		}
	}
}

fn do_nothing(all_linked_files: &Vec<LinkedFile>)
{
	let mut stdout = io::stdout();

	for linked_file in all_linked_files
	{
		if linked_file.sync_veredict == SyncVeredict::DoNothing
		{
			//Imprimir bonito
			let _ = queue!(stdout, SetAttribute(Attribute::Bold));
			let _ = queue!(stdout, SetForegroundColor(Color::Yellow));
			let _ = queue!(stdout, Print("\n--"));
			let _ = queue!(stdout, SetForegroundColor(Color::Reset));
			let _ = queue!(stdout, Print("\n Ignoring: "));
			let _ = queue!(stdout, SetAttribute(Attribute::Reset));
			let _ = queue!(stdout, Print(format!("{}", linked_file.relative_path)));
			let _ = queue!(stdout, SetForegroundColor(Color::Green));
			let _ = queue!(stdout, Print(" (done!)"));
			let _ = queue!(stdout, SetForegroundColor(Color::Reset));
		}
		else if linked_file.sync_veredict == SyncVeredict::NotDecidedYet
		{
			let _ = queue!(stdout, SetAttribute(Attribute::Bold));
			let _ = queue!(stdout, SetForegroundColor(Color::Red));
			let _ = queue!(stdout, Print("\n!!"));
			let _ = queue!(stdout, SetForegroundColor(Color::Reset));
			let _ = queue!(stdout, Print("\n FAILED TO DETERMINE WHAT TO DO WITH THIS, WILL BE IGNORED: "));
			let _ = queue!(stdout, SetAttribute(Attribute::Reset));
			let _ = queue!(stdout, Print(format!("{}", linked_file.relative_path)));
		}
	}
	let _ = stdout.flush();
	std::thread::sleep(Duration::from_millis(300));
}

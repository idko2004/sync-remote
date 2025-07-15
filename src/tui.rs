use std::
{
	io::
	{
		stdout,
		Stdout,
		Write,
	}
};
use crossterm::
{
	cursor::
	{
		MoveTo,
		MoveToNextLine,
		MoveToColumn,
		Hide,
		Show,
		EnableBlinking,
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
		is_raw_mode_enabled,
		disable_raw_mode,
		enable_raw_mode,
		size,
		Clear,
		ClearType,
		EnterAlternateScreen,
		LeaveAlternateScreen
	}
};

#[derive(Clone)]
enum TuiState
{
	MainMenu(usize), //usize saves the current remote selected
	AddRemote(AddRemoteTuiStep), //AddRemoteUiStep saves the current step of the process.

	RemoteSelected(usize), //This returns the usize to main and exits the ui loop.
	AddRemoteDone,
}

#[derive(Clone, Copy, PartialEq)]
enum AddRemoteTuiStep
{
	SettingName,
	SettingRemoteUrl,
	SettingRemotePath,
	SettingLocalPath,
	AskingIfNeedsLogin,
	SettingRemoteUsername,
	SettingRemotePassword,
	BasicSummary,
}

enum UserInput
{
	Exit,
	Select,
	MoveDown,
	MoveUp,
	Char(char),
	Backspace,
	Ignore,
}

struct RedrawOptions
{
	box_title: String,
	selectable_options: Option<Vec<String>>,
	draw_options_at_coordinates: (u16, u16), //row, column
	selected_option: usize,
}

#[derive(Clone)]
pub struct NewRemoteDetails
{
	pub name: Option<String>,
	pub remote_url: Option<String>,
	pub remote_path: Option<String>,
	pub local_path: Option<String>,
	pub remote_username: Option<String>,
	pub remote_password: Option<String>,
}

impl NewRemoteDetails
{
	fn new() -> Self
	{
		Self
		{
			name: None,
			remote_url: None,
			remote_path: None,
			local_path: None,
			remote_username: None,
			remote_password: None,
		}
	}
}

pub enum TuiResult
{
	SyncRemote(usize),
	CreateRemote(NewRemoteDetails),
}

pub fn start_tui_blocking(selectable_options: &Vec<String>) -> TuiResult
{
	let _ = execute!(stdout(), EnterAlternateScreen);
	let _ = execute!(stdout(), Hide);

	let mut ui_state = TuiState::MainMenu(0);
	let mut new_remote_details = NewRemoteDetails::new(); //This is only to keep track of UiState::AddRemote state.

	//render_main_menu(&ui_state, selectable_options);
	
	loop
	{
		match ui_state
		{
			TuiState::MainMenu(_) =>
			{
				render_main_menu(&ui_state, selectable_options);
				ui_state = logic_main_menu(&mut ui_state, selectable_options);
			},
			TuiState::RemoteSelected(value) =>
			{
				reset_terminal();
				return TuiResult::SyncRemote(value);
			},
			TuiState::AddRemote(_) =>
			{
				//render_add_remote_menu(&ui_state); //this should be called by logic_add_remote only once per screen
				ui_state = logic_add_remote_menu(&mut ui_state, &mut new_remote_details);
			},
			TuiState::AddRemoteDone =>
			{
				reset_terminal();
				return TuiResult::CreateRemote(new_remote_details);
			}
		}

		std::thread::sleep(std::time::Duration::from_millis(50));
	}
}

fn logic_main_menu(ui_state: &TuiState, selectable_options: &Vec<String>) -> TuiState
{
	match is_raw_mode_enabled()
	{
		Ok(raw_mode_enabled) =>
		{
			if !raw_mode_enabled
			{
				match enable_raw_mode()
				{
					Ok(_) => (),
					Err(error) =>
					{
						panic_gracefully(format!("[ERROR] Failed to activate terminal raw mode!! {}", error).as_str());
						std::process::exit(1);
					}
				}
			}
		},
		Err(error) =>
		{
			panic_gracefully(format!("[ERROR] Failed to determine if terminal is in raw mode!! {}", error).as_str());
			std::process::exit(1);
		}
	}

	let mut index_selected_option = match ui_state
	{
		TuiState::MainMenu(value) => value.clone(),
		_ =>
		{
			panic_gracefully("[ERROR] logic_main_menu should not be called if current menu is not MainMenu!!");
			std::process::exit(1);
		}
	};

	match read_input_raw_mode(false)
	{
		UserInput::MoveDown =>
		{
			index_selected_option += 1;
			if index_selected_option >= selectable_options.len()
			{
				index_selected_option = selectable_options.len() - 1;
			}
		},
		UserInput::MoveUp =>
		{
			if index_selected_option > 0
			{
				index_selected_option -= 1;
			}
		},
		UserInput::Select =>
		{
			if index_selected_option == selectable_options.len() - 1 //"(Add remote)" siempre es la última opción
			{
				return TuiState::AddRemote(AddRemoteTuiStep::SettingName);
			}
			else
			{
				return TuiState::RemoteSelected(index_selected_option);
			}
		},
		UserInput::Exit =>
		{
			kill_program();
		},
		_ => (),
	}

	TuiState::MainMenu(index_selected_option)
}

fn render_main_menu(ui_state: &TuiState, selectable_options: &Vec<String>)
{
	let selected_option = match ui_state
	{
		TuiState::MainMenu(value) => value.clone(),
		_ =>
		{
			panic_gracefully("[ERROR] render_main_menu should not be called if current menu is not MainMenu!!");
			return;
		}
	};

	let redraw_options = RedrawOptions
	{
		box_title: String::from(" Select a remote to sync "),
		selectable_options: Some(selectable_options.clone()),
		draw_options_at_coordinates: (0, 0),
		selected_option: selected_option,
	};

	redraw(&redraw_options);
}

fn logic_add_remote_menu(ui_state: &TuiState, new_remote_details: &mut NewRemoteDetails) -> TuiState
{
	match ui_state
	{
		TuiState::AddRemote(step) =>
		{
			match step
			{
				//Agrupar pantallas con comportamiento similiar
				AddRemoteTuiStep::SettingName |
				AddRemoteTuiStep::SettingRemoteUrl |
				AddRemoteTuiStep::SettingRemotePath |
				AddRemoteTuiStep::SettingLocalPath |
				AddRemoteTuiStep::SettingRemoteUsername |
				AddRemoteTuiStep::SettingRemotePassword =>
				{
					let _ = execute!(stdout(), Show);
					let mut full_string = String::new();

					loop
					{
						render_add_remote_menu(ui_state, &full_string, 0, None);
						match read_input_raw_mode(true)
						{
							UserInput::Char(input_char) =>
							{
								full_string.push(input_char);
							},
							UserInput::Backspace =>
							{
								full_string.pop();
							},
							UserInput::Select =>
							{
								full_string = full_string.trim().to_string();
								if !full_string.is_empty()
								{
									break;
								}
								//If setting a remote path, input can be empty because there's a "/" visible in the ui, this also means that a / should be added to the start of the string when saving.
								if *step == AddRemoteTuiStep::SettingRemotePath
								{
									break;
								}
							},
							UserInput::Exit =>
							{
								kill_program();
							},
							_ => (),
						}
						std::thread::sleep(std::time::Duration::from_millis(50));
					}

					//Comportamiento que varía en pantallas parecidas.
					match step
					{
						AddRemoteTuiStep::SettingName =>
						{
							new_remote_details.name = Some(full_string);
							TuiState::AddRemote(AddRemoteTuiStep::SettingRemoteUrl)
						},
						AddRemoteTuiStep::SettingRemoteUrl =>
						{
							new_remote_details.remote_url = Some(full_string);
							TuiState::AddRemote(AddRemoteTuiStep::SettingRemotePath)
						},
						AddRemoteTuiStep::SettingRemotePath =>
						{
							new_remote_details.remote_path = Some(full_string);
							TuiState::AddRemote(AddRemoteTuiStep::SettingLocalPath)
						},
						AddRemoteTuiStep::SettingLocalPath =>
						{
							new_remote_details.local_path = Some(full_string);
							TuiState::AddRemote(AddRemoteTuiStep::AskingIfNeedsLogin)
						},
						AddRemoteTuiStep::SettingRemoteUsername =>
						{
							new_remote_details.remote_username = Some(full_string);
							TuiState::AddRemote(AddRemoteTuiStep::SettingRemotePassword)
						},
						AddRemoteTuiStep::SettingRemotePassword =>
						{
							new_remote_details.remote_password = Some(full_string);
							TuiState::AddRemote(AddRemoteTuiStep::BasicSummary)
						},
						_ => ui_state.clone()
					}
				},
				AddRemoteTuiStep::AskingIfNeedsLogin =>
				{
					let _ = execute!(stdout(), Hide);
					let mut index_selected_option = 0;
					
					loop
					{
						render_add_remote_menu(ui_state, "", index_selected_option, None);
						match read_input_raw_mode(false)
						{
							UserInput::MoveDown =>
							{
								index_selected_option += 1;
								if index_selected_option >= 2 //Número total de opciones
								{
									index_selected_option = 1;
								}
							},
							UserInput::MoveUp =>
							{
								if index_selected_option > 0
								{
									index_selected_option -= 1;
								}
							},
							UserInput::Select =>
							{
								break;
							},
							UserInput::Exit =>
							{
								kill_program();
							},
							_ => (),
						}
					}

					if index_selected_option == 0
					{
						TuiState::AddRemote(AddRemoteTuiStep::SettingRemoteUsername)
					}
					else
					{
						TuiState::AddRemote(AddRemoteTuiStep::BasicSummary)
					}
				},
				AddRemoteTuiStep::BasicSummary =>
				{
					let _ = execute!(stdout(), Hide);
					let mut index_selected_option = 0;
					
					loop
					{
						render_add_remote_menu(ui_state, "", index_selected_option, Some(&new_remote_details));
						match read_input_raw_mode(false)
						{
							UserInput::MoveDown =>
							{
								index_selected_option += 1;
								if index_selected_option >= 3 //Número total de opciones
								{
									index_selected_option = 2;
								}
							},
							UserInput::MoveUp =>
							{
								if index_selected_option > 0
								{
									index_selected_option -= 1;
								}
							},
							UserInput::Select =>
							{
								break;
							},
							UserInput::Exit =>
							{
								kill_program();
							},
							_ => (),
						}
					}

					match index_selected_option
					{
						0 => TuiState::AddRemoteDone,
						1 =>
						{
							panic_gracefully("todo! advanced settings");
							std::process::exit(1);
						},
						2 =>
						{
							kill_program();
							ui_state.clone()
						},
						_ =>
						{
							panic_gracefully("[ERROR] Invalid item selected on list");
							std::process::exit(1);
						}
					}
				}
			}
		},
		_ =>
		{
			panic_gracefully("[ERROR] logic_add_remote_menu should not be called if current menu is not AddRemote!!");
			std::process::exit(1);
		}
	}
}

fn render_add_remote_menu(ui_state: &TuiState, current_string: &str, selected_option: usize, new_remote_details: Option<&NewRemoteDetails>)
{
	let mut stdout = stdout();
	let _ = queue!(stdout, EnableBlinking);

	match ui_state
	{
		TuiState::AddRemote(step) =>
		{
			match step
			{
				AddRemoteTuiStep::SettingName =>
				{
					redraw
					(
						&RedrawOptions
						{
							box_title: String::from(" Add a remote "),
							selectable_options: None,
							draw_options_at_coordinates: (0, 0),
							selected_option: 0,
						}
					);
					let _ = queue!(stdout, MoveTo(3, 2));
					let _ = queue!(stdout, SetAttribute(Attribute::Bold));
					let _ = queue!(stdout, Print("Name:"));
					let _ = queue!(stdout, SetAttribute(Attribute::Reset));
					let _ = queue!(stdout, SetBackgroundColor(Color::DarkBlue));
					let _ = queue!(stdout, SetForegroundColor(Color::White));
					//let _ = queue!(stdout, MoveTo(4, 3));
					let _ = queue!(stdout, MoveToNextLine(1));
					let _ = queue!(stdout, MoveToColumn(4));
					let _ = queue!(stdout, Print("What name or alias would you like to give to the remote?"));
					//let _ = queue!(stdout, MoveTo(3, 5));
					let _ = queue!(stdout, MoveToNextLine(2));
					let _ = queue!(stdout, MoveToColumn(3));
					let _ = queue!(stdout, SetForegroundColor(Color::Cyan));
					let prompt = format!(">> {} ", current_string);
					let _ = queue!(stdout, Print(&prompt));
					let _ = queue!(stdout, SetForegroundColor(Color::White));
					let _ = queue!(stdout, MoveToColumn(0));
					let _ = queue!(stdout, MoveToColumn(3 + (prompt.len() as u16) - 1));
					//let _ = queue!(stdout, MoveTo(3 + (prompt.len() as u16) - 1, 5));
					let _ = stdout.flush();
				},
				AddRemoteTuiStep::SettingRemoteUrl =>
				{
					redraw
					(
						&RedrawOptions
						{
							box_title: String::from(" Add a remote "),
							selectable_options: None,
							draw_options_at_coordinates: (0, 0),
							selected_option: 0,
						}
					);
					let _ = queue!(stdout, MoveTo(3, 2));
					let _ = queue!(stdout, SetAttribute(Attribute::Bold));
					let _ = queue!(stdout, Print("Remote Host:"));
					let _ = queue!(stdout, SetAttribute(Attribute::Reset));
					let _ = queue!(stdout, SetBackgroundColor(Color::DarkBlue));
					let _ = queue!(stdout, SetForegroundColor(Color::White));
					//let _ = queue!(stdout, MoveTo(4, 3));
					let _ = queue!(stdout, MoveToNextLine(1));
					let _ = queue!(stdout, MoveToColumn(4));
					let _ = queue!(stdout, Print("Enter the host of your FTP remote."));
					//let _ = queue!(stdout, MoveTo(4, 4));
					let _ = queue!(stdout, MoveToNextLine(1));
					let _ = queue!(stdout, MoveToColumn(4));
					let _ = queue!(stdout, Print("(For example: "));
					let _ = queue!(stdout, SetAttribute(Attribute::Italic));
					let _ = queue!(stdout, Print("ftp.myserver.com"));
					let _ = queue!(stdout, SetAttribute(Attribute::Reset));
					let _ = queue!(stdout, SetBackgroundColor(Color::DarkBlue));
					let _ = queue!(stdout, SetForegroundColor(Color::White));
					let _ = queue!(stdout, Print(" or "));
					let _ = queue!(stdout, SetAttribute(Attribute::Italic));
					let _ = queue!(stdout, Print("192.168.0.200:8021"));
					let _ = queue!(stdout, SetAttribute(Attribute::Reset));
					let _ = queue!(stdout, SetBackgroundColor(Color::DarkBlue));
					let _ = queue!(stdout, SetForegroundColor(Color::White));
					let _ = queue!(stdout, Print(")"));
					//let _ = queue!(stdout, MoveTo(3, 6));
					let _ = queue!(stdout, MoveToNextLine(2));
					let _ = queue!(stdout, MoveToColumn(3));
					let _ = queue!(stdout, SetForegroundColor(Color::Cyan));
					let prompt = format!(">> {} ", current_string);
					let _ = queue!(stdout, Print(&prompt));
					let _ = queue!(stdout, SetForegroundColor(Color::White));
					//let _ = queue!(stdout, MoveTo(3 + (prompt.len() as u16) - 1, 6));
					let _ = queue!(stdout, MoveToColumn(0));
					let _ = queue!(stdout, MoveToColumn(3 + (prompt.len() as u16) - 1));
					let _ = stdout.flush();
				},
				AddRemoteTuiStep::SettingRemotePath =>
				{
					redraw
					(
						&RedrawOptions
						{
							box_title: String::from(" Add a remote "),
							selectable_options: None,
							draw_options_at_coordinates: (0, 0),
							selected_option: 0,
						}
					);
					let _ = queue!(stdout, MoveTo(3, 2));
					let _ = queue!(stdout, SetAttribute(Attribute::Bold));
					let _ = queue!(stdout, Print("Remote directory:"));
					let _ = queue!(stdout, SetAttribute(Attribute::Reset));
					let _ = queue!(stdout, SetBackgroundColor(Color::DarkBlue));
					let _ = queue!(stdout, SetForegroundColor(Color::White));
					//let _ = queue!(stdout, MoveTo(4, 3));
					let _ = queue!(stdout, MoveToNextLine(1));
					let _ = queue!(stdout, MoveToColumn(4));
					let _ = queue!(stdout, Print("Which directory or folder of the remote would you like to sync?"));
					//let _ = queue!(stdout, MoveTo(4, 4));
					let _ = queue!(stdout, MoveToNextLine(1));
					let _ = queue!(stdout, MoveToColumn(4));
					let _ = queue!(stdout, Print("(For example: "));
					let _ = queue!(stdout, SetAttribute(Attribute::Italic));
					let _ = queue!(stdout, Print("/Movies"));
					let _ = queue!(stdout, SetAttribute(Attribute::Reset));
					let _ = queue!(stdout, SetBackgroundColor(Color::DarkBlue));
					let _ = queue!(stdout, SetForegroundColor(Color::White));
					let _ = queue!(stdout, Print(")"));
					//let _ = queue!(stdout, MoveTo(3, 6));
					let _ = queue!(stdout, MoveToNextLine(2));
					let _ = queue!(stdout, MoveToColumn(4));
					let _ = queue!(stdout, SetForegroundColor(Color::Cyan));
					let prompt = format!(">> /{} ", current_string);
					let _ = queue!(stdout, Print(&prompt));
					let _ = queue!(stdout, SetForegroundColor(Color::White));
					//let _ = queue!(stdout, MoveTo(3 + (prompt.len() as u16) - 1, 6));
					let _ = queue!(stdout, MoveToColumn(0));
					let _ = queue!(stdout, MoveToColumn(3 + (prompt.len() as u16)));
					let _ = stdout.flush();
				},
				AddRemoteTuiStep::SettingLocalPath =>
				{
					redraw
					(
						&RedrawOptions
						{
							box_title: String::from(" Add a remote "),
							selectable_options: None,
							draw_options_at_coordinates: (0, 0),
							selected_option: 0,
						}
					);
					let _ = queue!(stdout, MoveTo(3, 2));
					let _ = queue!(stdout, SetAttribute(Attribute::Bold));
					let _ = queue!(stdout, Print("Local directory:"));
					let _ = queue!(stdout, SetAttribute(Attribute::Reset));
					let _ = queue!(stdout, SetBackgroundColor(Color::DarkBlue));
					let _ = queue!(stdout, SetForegroundColor(Color::White));
					//let _ = queue!(stdout, MoveTo(4, 3));
					let _ = queue!(stdout, MoveToNextLine(1));
					let _ = queue!(stdout, MoveToColumn(4));
					let _ = queue!(stdout, Print("With which local directory or folder would you like to sync?"));
					//let _ = queue!(stdout, MoveTo(4, 4));
					let _ = queue!(stdout, MoveToNextLine(1));
					let _ = queue!(stdout, MoveToColumn(4));
					let _ = queue!(stdout, Print("(For example: "));
					let _ = queue!(stdout, SetAttribute(Attribute::Italic));
					let _ = queue!(stdout, Print("/home/user/Movies"));
					let _ = queue!(stdout, SetAttribute(Attribute::Reset));
					let _ = queue!(stdout, SetBackgroundColor(Color::DarkBlue));
					let _ = queue!(stdout, SetForegroundColor(Color::White));
					let _ = queue!(stdout, Print(" or "));
					let _ = queue!(stdout, SetAttribute(Attribute::Italic));
					let _ = queue!(stdout, Print("C:\\Users\\user\\Documents"));
					let _ = queue!(stdout, SetAttribute(Attribute::Reset));
					let _ = queue!(stdout, SetBackgroundColor(Color::DarkBlue));
					let _ = queue!(stdout, SetForegroundColor(Color::White));
					let _ = queue!(stdout, Print(")"));
					//let _ = queue!(stdout, MoveTo(3, 6));
					let _ = queue!(stdout, MoveToNextLine(2));
					let _ = queue!(stdout, MoveToColumn(4));
					let _ = queue!(stdout, SetForegroundColor(Color::Cyan));
					let prompt = format!(">> {} ", current_string);
					let _ = queue!(stdout, Print(&prompt));
					let _ = queue!(stdout, SetForegroundColor(Color::White));
					//let _ = queue!(stdout, MoveTo(3 + (prompt.len() as u16) - 1, 6));
					let _ = queue!(stdout, MoveToColumn(0));
					let _ = queue!(stdout, MoveToColumn(3 + (prompt.len() as u16)));
					let _ = stdout.flush();
				},
				AddRemoteTuiStep::AskingIfNeedsLogin =>
				{
					let selectable_options: Vec<String> = vec!
					[
						String::from("With username and password"),
						String::from("Anonimous login"),
					];

					redraw
					(
						&RedrawOptions
						{
							box_title: String::from(" Add a remote "),
							selectable_options: Some(selectable_options),
							draw_options_at_coordinates: (2, 0),
							selected_option: selected_option,
						}
					);

					let _ = queue!(stdout, MoveTo(3, 2));
					let _ = queue!(stdout, SetAttribute(Attribute::Bold));
					let _ = queue!(stdout, Print("How to login:"));
					let _ = queue!(stdout, SetAttribute(Attribute::Reset));
					let _ = queue!(stdout, SetBackgroundColor(Color::DarkBlue));
					let _ = queue!(stdout, SetForegroundColor(Color::White));
					let _ = stdout.flush();
				},
				AddRemoteTuiStep::SettingRemoteUsername =>
				{
					redraw
					(
						&RedrawOptions
						{
							box_title: String::from(" Add a remote "),
							selectable_options: None,
							draw_options_at_coordinates: (0, 0),
							selected_option: 0,
						}
					);
					let _ = queue!(stdout, MoveTo(3, 2));
					let _ = queue!(stdout, SetAttribute(Attribute::Bold));
					let _ = queue!(stdout, Print("Username:"));
					//let _ = queue!(stdout, MoveTo(3, 4));
					let _ = queue!(stdout, MoveToNextLine(2));
					let _ = queue!(stdout, MoveToColumn(4));
					let _ = queue!(stdout, SetForegroundColor(Color::Cyan));
					let prompt = format!(">> {} ", current_string);
					let _ = queue!(stdout, Print(&prompt));
					let _ = queue!(stdout, SetForegroundColor(Color::White));
					//let _ = queue!(stdout, MoveTo(3 + (prompt.len() as u16) - 1, 4));
					let _ = queue!(stdout, MoveToColumn(0));
					let _ = queue!(stdout, MoveToColumn(3 + (prompt.len() as u16)));
					let _ = stdout.flush();
				},
				AddRemoteTuiStep::SettingRemotePassword =>
				{
					redraw
					(
						&RedrawOptions
						{
							box_title: String::from(" Add a remote "),
							selectable_options: None,
							draw_options_at_coordinates: (0, 0),
							selected_option: 0,
						}
					);
					let _ = queue!(stdout, MoveTo(3, 2));
					let _ = queue!(stdout, SetAttribute(Attribute::Bold));
					let _ = queue!(stdout, Print("Password:"));
					let _ = queue!(stdout, SetAttribute(Attribute::Reset));
					let _ = queue!(stdout, SetBackgroundColor(Color::Yellow));
					let _ = queue!(stdout, SetForegroundColor(Color::Black));
					//let _ = queue!(stdout, MoveTo(4, 3));
					let _ = queue!(stdout, MoveToNextLine(1));
					let _ = queue!(stdout, MoveToColumn(4));
					let _ = queue!(stdout, Print("Keep in mind that passwords are stored insecurely as plain text!!"));
					//let _ = queue!(stdout, MoveTo(3, 5));
					let _ = queue!(stdout, MoveToNextLine(2));
					let _ = queue!(stdout, MoveToColumn(4));
					let _ = queue!(stdout, SetBackgroundColor(Color::DarkBlue));
					let _ = queue!(stdout, SetForegroundColor(Color::Cyan));
					let mut password_chars = String::with_capacity(current_string.len());
					for _ in 0..current_string.len()
					{
						password_chars.push('*');
					}
					let prompt = format!(">> {} ", password_chars);
					let _ = queue!(stdout, Print(&prompt));
					let _ = queue!(stdout, SetForegroundColor(Color::White));
					//let _ = queue!(stdout, MoveTo(3 + (prompt.len() as u16) - 1, 5));
					let _ = queue!(stdout, MoveToColumn(0));
					let _ = queue!(stdout, MoveToColumn(3 + (prompt.len() as u16)));
					let _ = stdout.flush();
				},
				AddRemoteTuiStep::BasicSummary =>
				{
					let new_remote_details = match new_remote_details
					{
						Some(value) => value,
						None =>
						{
							panic_gracefully("Failed to render summary!");
							std::process::exit(1);
						}
					};

					let name = match &new_remote_details.name
					{
						Some(value) => value.as_str(),
						None => "[None]"
					};
					let remote_url = match &new_remote_details.remote_url
					{
						Some(value) => value.as_str(),
						None => "[None]"
					};
					let remote_path = match &new_remote_details.remote_path
					{
						Some(value) => value.as_str(),
						None => "[None]"
					};
					let local_path = match &new_remote_details.local_path
					{
						Some(value) => value.as_str(),
						None => "[None]"
					};
					let remote_username = match &new_remote_details.remote_username
					{
						Some(value) => value.as_str(),
						None => "[None]"
					};
					let remote_password = match &new_remote_details.remote_password
					{
						Some(value) => 
						{
							let mut password_chars = String::with_capacity(value.len());
							for _ in 0..value.len()
							{
								password_chars.push('*');
							}
							password_chars
						}
						None => String::from("[None]")
					};

					let selectable_options: Vec<String> = vec!
					[
						String::from("Save basic settings"),
						String::from("Advanced settings"),
						String::from("Discard and exit"),
					];

					redraw
					(
						&RedrawOptions
						{
							box_title: String::from(" Add a remote "),
							selectable_options: Some(selectable_options),
							draw_options_at_coordinates: (12, 0),
							selected_option: selected_option,
						}
					);

					let _ = queue!(stdout, MoveTo(3, 2));
					let _ = queue!(stdout, SetAttribute(Attribute::Bold));
					let _ = queue!(stdout, Print("Basic settings are done!"));
					let _ = queue!(stdout, SetAttribute(Attribute::Reset));
					let _ = queue!(stdout, SetBackgroundColor(Color::DarkBlue));
					let _ = queue!(stdout, SetForegroundColor(Color::White));
					//let _ = queue!(stdout, MoveTo(4, 3));
					let _ = queue!(stdout, MoveToNextLine(1));
					let _ = queue!(stdout, MoveToColumn(4));
					let _ = queue!(stdout, Print("Are these settings ok?"));
					//let _ = queue!(stdout, MoveTo(4, 5));
					let _ = queue!(stdout, MoveToNextLine(2));
					let _ = queue!(stdout, MoveToColumn(4));
					let _ = queue!(stdout, SetAttribute(Attribute::Bold));
					let _ = queue!(stdout, Print("Name: "));
					let _ = queue!(stdout, SetAttribute(Attribute::Reset));
					let _ = queue!(stdout, SetBackgroundColor(Color::DarkBlue));
					let _ = queue!(stdout, SetForegroundColor(Color::White));
					let _ = queue!(stdout, Print(name));
					//let _ = queue!(stdout, MoveTo(4, 6));
					let _ = queue!(stdout, MoveToNextLine(1));
					let _ = queue!(stdout, MoveToColumn(4));
					let _ = queue!(stdout, SetAttribute(Attribute::Bold));
					let _ = queue!(stdout, Print("Remote: "));
					let _ = queue!(stdout, SetAttribute(Attribute::Reset));
					let _ = queue!(stdout, SetBackgroundColor(Color::DarkBlue));
					let _ = queue!(stdout, SetForegroundColor(Color::White));
					let _ = queue!(stdout, Print(remote_url));
					//let _ = queue!(stdout, MoveTo(4, 7));
					let _ = queue!(stdout, MoveToNextLine(1));
					let _ = queue!(stdout, MoveToColumn(4));
					let _ = queue!(stdout, SetAttribute(Attribute::Bold));
					let _ = queue!(stdout, Print("Remote path: "));
					let _ = queue!(stdout, SetAttribute(Attribute::Reset));
					let _ = queue!(stdout, SetBackgroundColor(Color::DarkBlue));
					let _ = queue!(stdout, SetForegroundColor(Color::White));
					let _ = queue!(stdout, Print(remote_path));
					//let _ = queue!(stdout, MoveTo(4, 8));
					let _ = queue!(stdout, MoveToNextLine(1));
					let _ = queue!(stdout, MoveToColumn(4));
					let _ = queue!(stdout, SetAttribute(Attribute::Bold));
					let _ = queue!(stdout, Print("Local path: "));
					let _ = queue!(stdout, SetAttribute(Attribute::Reset));
					let _ = queue!(stdout, SetBackgroundColor(Color::DarkBlue));
					let _ = queue!(stdout, SetForegroundColor(Color::White));
					let _ = queue!(stdout, Print(local_path));
					//let _ = queue!(stdout, MoveTo(4, 9));
					let _ = queue!(stdout, MoveToNextLine(1));
					let _ = queue!(stdout, MoveToColumn(4));
					let _ = queue!(stdout, SetAttribute(Attribute::Bold));
					let _ = queue!(stdout, Print("Username: "));
					let _ = queue!(stdout, SetAttribute(Attribute::Reset));
					let _ = queue!(stdout, SetBackgroundColor(Color::DarkBlue));
					let _ = queue!(stdout, SetForegroundColor(Color::White));
					let _ = queue!(stdout, Print(remote_username));
					//let _ = queue!(stdout, MoveTo(4, 10));
					let _ = queue!(stdout, MoveToNextLine(1));
					let _ = queue!(stdout, MoveToColumn(4));
					let _ = queue!(stdout, SetAttribute(Attribute::Bold));
					let _ = queue!(stdout, Print("Password: "));
					let _ = queue!(stdout, SetAttribute(Attribute::Reset));
					let _ = queue!(stdout, SetBackgroundColor(Color::DarkBlue));
					let _ = queue!(stdout, SetForegroundColor(Color::White));
					let _ = queue!(stdout, Print(remote_password.as_str()));
					//let _ = queue!(stdout, MoveTo(4, 12));
					let _ = queue!(stdout, MoveToNextLine(2));
					let _ = queue!(stdout, MoveToColumn(4));
					let _ = queue!(stdout, Print("Save these settings or continue to more advanced configurations?"));
					//let _ = queue!(stdout, MoveTo(4, 14));
					let _ = queue!(stdout, MoveToNextLine(2));
					let _ = queue!(stdout, MoveToColumn(3));
					let _ = stdout.flush();
				},
				/*
				_ =>
				{
					panic_gracefully("[ERROR] idk how to render this");
					std::process::exit(1);
				}
				*/
			}
		},
		_ =>
		{
			panic_gracefully("[ERROR] render_add_remote_menu should not be called if current menu is not AddRemote!!");
			std::process::exit(1);
		}
	}
}

fn redraw(redraw_options: &RedrawOptions)
{
	let mut stdout = stdout();

	let _ = queue!(stdout, SetBackgroundColor(Color::DarkBlue));
	let _ = queue!(stdout, SetForegroundColor(Color::White));
	let _ = queue!(stdout, Clear(ClearType::All));
	let _ = queue!(stdout, Clear(ClearType::Purge));
	let _ = queue!(stdout, MoveTo(0, 1));

	let (width, height) = match size()
	{
		Ok(value) => value,
		Err(error) =>
		{
			let _ = queue!(stdout, Print(format!("Failed to get terminal size! {error}").as_str()));
			let _ = stdout.flush();
			return;
		}
	};

	stdout = draw_box(&redraw_options.box_title, width, height, stdout);
	stdout = draw_options(redraw_options, stdout);
	let _ = queue!(stdout, MoveTo(0, height));
	let _ = stdout.flush();
}


fn draw_box(title: &String, width: u16, height: u16, mut stdout: Stdout) -> Stdout
{
	//Draw top horizontal line
	let mut line =  String::with_capacity(width as usize);
	line.push(' ');
	//line.push('\u{256D}');
	line.push('\u{2554}');

	let mut title_drawn = false;
	
	let mut i: u16 = 3;
	loop
	{
		if !title_drawn && i > 4
		{
			line.push_str("\u{2563}");
			line.push_str(title.as_str());
			line.push_str("\u{2560}");
			i += title.len() as u16 + 2;
			title_drawn = true;
		}
		else
		{
			//line.push('\u{2500}');
			line.push('\u{2550}');
			i += 1;
		}

		if i >= width  - 1
		{
			break;
		}
	}
	//line.push('\u{256E}');
	line.push('\u{2557}');
	line.push('\n');

	let _ = queue!(stdout, Print(line.as_str()));

	//Draw side borders
	let mut i: u16 = 2;
	loop
	{
		let _ = queue!(stdout, MoveTo(1, i));
		//let _ = queue!(stdout, Print("\u{2502}"));
		let _ = queue!(stdout, Print("\u{2551}"));
		let _ = queue!(stdout, MoveTo(width - 2, i));
		let _ = queue!(stdout, Print("\u{2551}"));

		i += 1;

		if i > height - 2
		{
			break;
		}
	}

	//Drwa bottom border
	let mut line =  String::with_capacity(width as usize);
	line.push(' ');
	line.push(' ');
	//line.push('\u{2570}');
	line.push('\u{255A}');
	
	let mut i: u16 = 3;
	loop
	{
		//line.push('\u{2500}');
		line.push('\u{2550}');
		i += 1;

		if i >= width - 1
		{
			break;
		}
	}
	//line.push('\u{256F}');
	line.push('\u{255D}');
	line.push('\n');
	let _ = queue!(stdout, Print(line.as_str()));

	stdout
}

fn draw_options(redraw_options: &RedrawOptions, mut stdout: Stdout) -> Stdout
{
	match &redraw_options.selectable_options
	{
		Some(selectable_options) =>
		{
			let mut row: u16 = 2 + redraw_options.draw_options_at_coordinates.0;
			let column: u16 = 3 + redraw_options.draw_options_at_coordinates.1;
			let mut i = 0;

			for item in selectable_options
			{
				let _ = queue!(stdout, MoveTo(column, row));
				if redraw_options.selected_option != i
				{
					let _ = queue!(stdout, Print(format!(" -  {item}" )));
				}
				else
				{
					let _ = queue!(stdout, SetBackgroundColor(Color::Cyan));
					let _ = queue!(stdout, SetForegroundColor(Color::Black));
					let _ = queue!(stdout, SetAttribute(Attribute::Bold));
					let _ = queue!(stdout, Print(format!(" -> {item} ")));
					let _ = queue!(stdout, SetAttribute(Attribute::Reset));
					let _ = queue!(stdout, SetForegroundColor(Color::White));
					let _ = queue!(stdout, SetBackgroundColor(Color::DarkBlue));
				}
				
				i += 1;
				row += 1;
			}
		},
		None => (),
	}

	stdout
}

fn process_input_raw_mode(event: &Event, return_chars: bool) -> UserInput
{
	let key_event = match event.as_key_event()
	{
		Some(value) => value,
		None =>
		{
			return UserInput::Ignore;
		}
	};

	match key_event.code
	{
		KeyCode::Down => return UserInput::MoveDown,
		KeyCode::Up => return UserInput::MoveUp,
		KeyCode::Enter => return UserInput::Select,
		KeyCode::Backspace => return UserInput::Backspace,
		KeyCode::Esc => return UserInput::Exit,
		_ => (),
	}

	if key_event.modifiers == KeyModifiers::CONTROL
	{
		if key_event.code == KeyCode::Char('c')
		|| key_event.code == KeyCode::Char('q')
		{
			return UserInput::Exit;
		}
	}

	if return_chars
	{
		match key_event.code.as_char()
		{
			Some(value) => return UserInput::Char(value),
			None => (),
		}
	}

	UserInput::Ignore
}

fn read_input_raw_mode(return_chars: bool) -> UserInput
{
	let input = match read()
	{
		Ok(value) => value,
		Err(error) =>
		{
			panic_gracefully(format!("[ERROR] Failed to read input!! {error}").as_str());
			std::process::exit(1);
		}
	};

	process_input_raw_mode(&input, return_chars)
}

fn reset_terminal()
{
	let mut stdout = stdout();
	let _ = queue!(stdout, SetBackgroundColor(Color::Reset));
	let _ = queue!(stdout, SetForegroundColor(Color::Reset));
	let _ = queue!(stdout, Clear(ClearType::All));
	let _ = queue!(stdout, Clear(ClearType::Purge));
	let _ = queue!(stdout, Show);
	let _ = queue!(stdout, LeaveAlternateScreen);
	let _ = stdout.flush();
	let _ = disable_raw_mode();
}

fn kill_program()
{
	reset_terminal();
	let _ = execute!(stdout(), Print("\nGoodbye!\n"));
	std::process::exit(0);
}

fn panic_gracefully(message: &str)
{
	reset_terminal();
	panic!("{}", message);
}

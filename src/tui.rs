use std::
{
	io::
	{
		stdout,
		Stdout,
		Write,
		stdin,
		Stdin,
	}
};
use crossterm::
{
	cursor::
	{
		MoveTo,
		Hide,
		Show,
		EnableBlinking,
		DisableBlinking,
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

#[derive(Clone, Copy)]
enum UiState
{
	MainMenu(usize), //usize saves the current remote selected
	AddRemote(AddRemoteUiStep), //AddRemoteUiStep saves the current step of the process.
	RemoteSelected(usize), //This returns the usize to main and exits the ui loop.
}

#[derive(Clone, Copy, PartialEq)]
enum AddRemoteUiStep
{
	SettingName,
	SettingRemoteUrl,
	SettingRemotePath,
	SettingLocalPath,
	AskingIfNeedsLogin,
	SettingRemoteUsername,
	SettingRemotePassword,
	Summary,
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

struct NewRemoteDetails
{
	name: Option<String>,
	remote_url: Option<String>,
	remote_path: Option<String>,
	local_path: Option<String>,
	remote_username: Option<String>,
	remote_password: Option<String>,
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

pub fn start_tui_blocking(selectable_options: &Vec<String>) -> Option<usize>
{
	let _ = execute!(stdout(), EnterAlternateScreen);
	let _ = execute!(stdout(), Hide);

	let mut ui_state = UiState::MainMenu(0);
	let mut new_remote_details = NewRemoteDetails::new(); //This is only to keep track of UiState::AddRemote state.
	let index_selected_option;

	//render_main_menu(&ui_state, selectable_options);
	
	loop
	{
		match ui_state
		{
			UiState::MainMenu(_) =>
			{
				render_main_menu(&ui_state, selectable_options);
				ui_state = logic_main_menu(&mut ui_state, selectable_options);
			},
			UiState::AddRemote(_) =>
			{
				//render_add_remote_menu(&ui_state); //this should be called by logic_add_remote only once per screen
				ui_state = logic_add_remote_menu(&mut ui_state, &mut new_remote_details);
			},
			UiState::RemoteSelected(value) =>
			{
				index_selected_option = value;
				break;
			}
		}

		std::thread::sleep(std::time::Duration::from_millis(50));
	}

	let _ = execute!(stdout(), SetBackgroundColor(Color::Reset));
	let _ = execute!(stdout(), SetForegroundColor(Color::Reset));
	let _ = execute!(stdout(), Clear(ClearType::All));
	let _ = execute!(stdout(), Clear(ClearType::Purge));
	let _ = execute!(stdout(), Show);
	let _ = execute!(stdout(), LeaveAlternateScreen);
	let _ = disable_raw_mode();
	
	Some(index_selected_option)
}

fn logic_main_menu(ui_state: &UiState, selectable_options: &Vec<String>) -> UiState
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
		UiState::MainMenu(value) => value.clone(),
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
				return UiState::AddRemote(AddRemoteUiStep::SettingName);
			}
			else
			{
				return UiState::RemoteSelected(index_selected_option);
			}
		},
		UserInput::Exit =>
		{
			kill_program();
		},
		_ => (),
	}

	UiState::MainMenu(index_selected_option)
}

fn render_main_menu(ui_state: &UiState, selectable_options: &Vec<String>)
{
	let selected_option = match ui_state
	{
		UiState::MainMenu(value) => value.clone(),
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

fn logic_add_remote_menu(ui_state: &UiState, new_remote_details: &mut NewRemoteDetails) -> UiState
{
	match ui_state
	{
		UiState::AddRemote(step) =>
		{
			match step
			{
				//Agrupar pantallas con comportamiento similiar
				AddRemoteUiStep::SettingName |
				AddRemoteUiStep::SettingRemoteUrl |
				AddRemoteUiStep::SettingRemotePath |
				AddRemoteUiStep::SettingLocalPath |
				AddRemoteUiStep::SettingRemoteUsername |
				AddRemoteUiStep::SettingRemotePassword =>
				{
					let _ = execute!(stdout(), Show);
					let mut full_string = String::new();

					loop
					{
						render_add_remote_menu(ui_state, &full_string, 0);
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
								if *step == AddRemoteUiStep::SettingRemotePath
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
						AddRemoteUiStep::SettingName =>
						{
							new_remote_details.name = Some(full_string);
							UiState::AddRemote(AddRemoteUiStep::SettingRemoteUrl)
						},
						AddRemoteUiStep::SettingRemoteUrl =>
						{
							new_remote_details.remote_url = Some(full_string);
							UiState::AddRemote(AddRemoteUiStep::SettingRemotePath)
						},
						AddRemoteUiStep::SettingRemotePath =>
						{
							new_remote_details.remote_path = Some(full_string);
							UiState::AddRemote(AddRemoteUiStep::SettingLocalPath)
						},
						AddRemoteUiStep::SettingLocalPath =>
						{
							new_remote_details.local_path = Some(full_string);
							UiState::AddRemote(AddRemoteUiStep::AskingIfNeedsLogin)
						},
						AddRemoteUiStep::SettingRemoteUsername =>
						{
							new_remote_details.remote_username = Some(full_string);
							UiState::AddRemote(AddRemoteUiStep::SettingRemotePassword)
						},
						AddRemoteUiStep::SettingRemotePassword =>
						{
							new_remote_details.remote_password = Some(full_string);
							UiState::AddRemote(AddRemoteUiStep::Summary)
						},
						_ => ui_state.clone()
					}
				},
				AddRemoteUiStep::AskingIfNeedsLogin =>
				{
					let _ = execute!(stdout(), Hide);
					let mut index_selected_option = 0;
					
					loop
					{
						render_add_remote_menu(ui_state, "", index_selected_option);
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
						UiState::AddRemote(AddRemoteUiStep::SettingRemoteUsername)
					}
					else
					{
						UiState::AddRemote(AddRemoteUiStep::Summary)
					}
				},
				AddRemoteUiStep::Summary =>
				{
					ui_state.clone()
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

fn render_add_remote_menu(ui_state: &UiState, current_string: &str, selected_option: usize)
{
	let mut stdout = stdout();
	let _ = queue!(stdout, EnableBlinking);

	match ui_state
	{
		UiState::AddRemote(step) =>
		{
			match step
			{
				AddRemoteUiStep::SettingName =>
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
					let _ = queue!(stdout, MoveTo(4, 3));
					let _ = queue!(stdout, Print("What name or alias would you like to give to the remote?"));
					let _ = queue!(stdout, MoveTo(3, 5));
					let _ = queue!(stdout, SetAttribute(Attribute::Underlined));
					let _ = queue!(stdout, SetBackgroundColor(Color::Blue));
					let _ = queue!(stdout, SetForegroundColor(Color::White));
					let prompt = format!(">> {} ", current_string);
					let _ = queue!(stdout, Print(&prompt));
					let _ = queue!(stdout, SetAttribute(Attribute::Reset));
					let _ = queue!(stdout, SetBackgroundColor(Color::DarkBlue));
					let _ = queue!(stdout, SetForegroundColor(Color::White));
					let _ = queue!(stdout, MoveTo(3 + (prompt.len() as u16) - 1, 5));
					let _ = stdout.flush();
				},
				AddRemoteUiStep::SettingRemoteUrl =>
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
					let _ = queue!(stdout, MoveTo(4, 3));
					let _ = queue!(stdout, Print("Enter the host of your FTP remote."));
					let _ = queue!(stdout, MoveTo(4, 4));
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
					let _ = queue!(stdout, MoveTo(3, 6));
					let _ = queue!(stdout, SetAttribute(Attribute::Underlined));
					let _ = queue!(stdout, SetBackgroundColor(Color::Blue));
					let _ = queue!(stdout, SetForegroundColor(Color::White));
					let prompt = format!(">> {} ", current_string);
					let _ = queue!(stdout, Print(&prompt));
					let _ = queue!(stdout, SetAttribute(Attribute::Reset));
					let _ = queue!(stdout, SetBackgroundColor(Color::DarkBlue));
					let _ = queue!(stdout, SetForegroundColor(Color::White));
					let _ = queue!(stdout, MoveTo(3 + (prompt.len() as u16) - 1, 6));
					let _ = stdout.flush();
				},
				AddRemoteUiStep::SettingRemotePath =>
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
					let _ = queue!(stdout, MoveTo(4, 3));
					let _ = queue!(stdout, Print("Which directory or folder of the remote would you like to sync?"));
					let _ = queue!(stdout, MoveTo(4, 4));
					let _ = queue!(stdout, Print("(For example: "));
					let _ = queue!(stdout, SetAttribute(Attribute::Italic));
					let _ = queue!(stdout, Print("/Movies"));
					let _ = queue!(stdout, SetAttribute(Attribute::Reset));
					let _ = queue!(stdout, SetBackgroundColor(Color::DarkBlue));
					let _ = queue!(stdout, SetForegroundColor(Color::White));
					let _ = queue!(stdout, Print(")"));
					let _ = queue!(stdout, MoveTo(3, 6));
					let _ = queue!(stdout, SetAttribute(Attribute::Underlined));
					let _ = queue!(stdout, SetBackgroundColor(Color::Blue));
					let _ = queue!(stdout, SetForegroundColor(Color::White));
					let prompt = format!(">> /{} ", current_string);
					let _ = queue!(stdout, Print(&prompt));
					let _ = queue!(stdout, SetAttribute(Attribute::Reset));
					let _ = queue!(stdout, SetBackgroundColor(Color::DarkBlue));
					let _ = queue!(stdout, SetForegroundColor(Color::White));
					let _ = queue!(stdout, MoveTo(3 + (prompt.len() as u16) - 1, 6));
					let _ = stdout.flush();
				},
				AddRemoteUiStep::SettingLocalPath =>
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
					let _ = queue!(stdout, MoveTo(4, 3));
					let _ = queue!(stdout, Print("With which local directory or folder would you like to sync?"));
					let _ = queue!(stdout, MoveTo(4, 4));
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
					let _ = queue!(stdout, MoveTo(3, 6));
					let _ = queue!(stdout, SetAttribute(Attribute::Underlined));
					let _ = queue!(stdout, SetBackgroundColor(Color::Blue));
					let _ = queue!(stdout, SetForegroundColor(Color::White));
					let prompt = format!(">> {} ", current_string);
					let _ = queue!(stdout, Print(&prompt));
					let _ = queue!(stdout, SetAttribute(Attribute::Reset));
					let _ = queue!(stdout, SetBackgroundColor(Color::DarkBlue));
					let _ = queue!(stdout, SetForegroundColor(Color::White));
					let _ = queue!(stdout, MoveTo(3 + (prompt.len() as u16) - 1, 6));
					let _ = stdout.flush();
				},
				AddRemoteUiStep::AskingIfNeedsLogin =>
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
				AddRemoteUiStep::SettingRemoteUsername =>
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
					let _ = queue!(stdout, MoveTo(3, 4));
					let _ = queue!(stdout, SetAttribute(Attribute::Underlined));
					let _ = queue!(stdout, SetBackgroundColor(Color::Blue));
					let _ = queue!(stdout, SetForegroundColor(Color::White));
					let prompt = format!(">> {} ", current_string);
					let _ = queue!(stdout, Print(&prompt));
					let _ = queue!(stdout, SetAttribute(Attribute::Reset));
					let _ = queue!(stdout, SetBackgroundColor(Color::DarkBlue));
					let _ = queue!(stdout, SetForegroundColor(Color::White));
					let _ = queue!(stdout, MoveTo(3 + (prompt.len() as u16) - 1, 4));
					let _ = stdout.flush();
				},
				AddRemoteUiStep::SettingRemotePassword =>
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
					let _ = queue!(stdout, SetBackgroundColor(Color::DarkYellow));
					let _ = queue!(stdout, SetForegroundColor(Color::Black));
					let _ = queue!(stdout, MoveTo(4, 3));
					let _ = queue!(stdout, Print("Keep in mind that passwords are stored insecurely as plain text!!"));
					let _ = queue!(stdout, MoveTo(3, 5));
					let _ = queue!(stdout, SetAttribute(Attribute::Underlined));
					let _ = queue!(stdout, SetBackgroundColor(Color::Blue));
					let _ = queue!(stdout, SetForegroundColor(Color::White));
					let mut password_chars = String::with_capacity(current_string.len());
					for _ in 0..current_string.len()
					{
						password_chars.push('*');
					}
					let prompt = format!(">> {} ", password_chars);
					let _ = queue!(stdout, Print(&prompt));
					let _ = queue!(stdout, SetAttribute(Attribute::Reset));
					let _ = queue!(stdout, SetBackgroundColor(Color::DarkBlue));
					let _ = queue!(stdout, SetForegroundColor(Color::White));
					let _ = queue!(stdout, MoveTo(3 + (prompt.len() as u16) - 1, 5));
					let _ = stdout.flush();
				},
				_ =>
				{
					panic_gracefully("[ERROR] idk how to render this");
					std::process::exit(1);
				}
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
	line.push('\u{256D}');
	
	let mut i: u16 = 3;
	loop
	{
		if i > 4 && i < (title.len() as u16) + 4
		{
			line.push_str(title.as_str());
			i += title.len() as u16;
		}
		else
		{
			line.push('\u{2500}');
			i += 1;
		}

		if i >= width  - 1
		{
			break;
		}
	}
	line.push('\u{256E}');
	line.push('\n');

	let _ = queue!(stdout, Print(line.as_str()));

	//Draw side borders
	let mut i: u16 = 2;
	loop
	{
		let _ = queue!(stdout, MoveTo(1, i));
		let _ = queue!(stdout, Print("\u{2502}"));
		let _ = queue!(stdout, MoveTo(width - 2, i));
		let _ = queue!(stdout, Print("\u{2502}"));

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
	line.push('\u{2570}');
	
	let mut i: u16 = 3;
	loop
	{
		line.push('\u{2500}');
		i += 1;

		if i >= width - 1
		{
			break;
		}
	}
	line.push('\u{256F}');
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
					let _ = queue!(stdout, SetBackgroundColor(Color::White));
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

fn kill_program()
{
	let _ = execute!(stdout(), SetBackgroundColor(Color::Reset));
	let _ = execute!(stdout(), SetForegroundColor(Color::Reset));
	let _ = execute!(stdout(), Clear(ClearType::All));
	let _ = execute!(stdout(), Clear(ClearType::Purge));
	let _ = execute!(stdout(), Show);
	let _ = execute!(stdout(), LeaveAlternateScreen);
	let _ = disable_raw_mode();
	let _ = execute!(stdout(), Print("\nGoodbye!\n"));
	std::process::exit(0);
}

fn panic_gracefully(message: &str)
{
	let _ = execute!(stdout(), SetBackgroundColor(Color::Reset));
	let _ = execute!(stdout(), SetForegroundColor(Color::Reset));
	let _ = execute!(stdout(), Clear(ClearType::All));
	let _ = execute!(stdout(), Clear(ClearType::Purge));
	let _ = execute!(stdout(), Show);
	let _ = execute!(stdout(), LeaveAlternateScreen);
	let _ = disable_raw_mode();
	panic!("{}", message);
}

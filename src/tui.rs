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

enum UiState
{
	MainMenu(usize), //usize saves the current remote selected
	AddRemote(AddRemoteUiStep), //AddRemoteUiStep saves the current step of the process.
	RemoteSelected(usize), //This returns the usize to main and exits the ui loop.
}

enum AddRemoteUiStep
{
	SettingName,
	SettingRemoteUrl,
	SettingRemotePath,
	SettingLocalPath,
	SettingRemoteUsername,
	SettingRemotePassword,
}

enum UserInput
{
	Exit,
	Select,
	MoveDown,
	MoveUp,
	Ignore,
}

pub struct RedrawOptions
{
	box_title: String,
	selectable_options: Option<Vec<String>>,
	draw_options_at_coordinates: (u16, u16), //row, column
	selected_option: usize,
}

pub fn start_tui_blocking(selectable_options: &Vec<String>) -> Option<usize>
{
	let _ = execute!(stdout(), EnterAlternateScreen);
	let _ = execute!(stdout(), Hide);

	let mut ui_state = UiState::MainMenu(0);
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
				ui_state = logic_add_remote_menu(&mut ui_state);
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

	let input = match read()
	{
		Ok(value) => value,
		Err(error) =>
		{
			panic_gracefully(format!("[ERROR] Failed to read input!! {error}").as_str());
			std::process::exit(1);
		}
	};

	match process_input_raw_mode(&input)
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

fn logic_add_remote_menu(ui_state: &UiState) -> UiState
{
	match is_raw_mode_enabled()
	{
		Ok(raw_mode_enabled) =>
		{
			if raw_mode_enabled
			{
				match disable_raw_mode()
				{
					Ok(_) => 
					{
						let _ = execute!(stdout(), Show);
						let _ = execute!(stdout(), EnableBlinking);
					},
					Err(error) =>
					{
						panic_gracefully(format!("[ERROR] Failed to disable terminal raw mode!! {}", error).as_str());
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

	match ui_state
	{
		UiState::AddRemote(step) =>
		{
			match step
			{
				AddRemoteUiStep::SettingName =>
				{
					render_add_remote_menu(ui_state);
					let input = read_input_as_string();
					UiState::AddRemote(AddRemoteUiStep::SettingName)
				},
				AddRemoteUiStep::SettingRemoteUrl =>
				{
					UiState::AddRemote(AddRemoteUiStep::SettingRemoteUrl)
				},
				AddRemoteUiStep::SettingRemotePath =>
				{
					UiState::AddRemote(AddRemoteUiStep::SettingRemotePath)
				},
				AddRemoteUiStep::SettingLocalPath =>
				{
					UiState::AddRemote(AddRemoteUiStep::SettingLocalPath)
				},
				AddRemoteUiStep::SettingRemoteUsername =>
				{
					UiState::AddRemote(AddRemoteUiStep::SettingRemoteUsername)
				},
				AddRemoteUiStep::SettingRemotePassword =>
				{
					UiState::AddRemote(AddRemoteUiStep::SettingRemotePassword)
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

fn render_add_remote_menu(ui_state: &UiState)
{
	let mut stdout = stdout();
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
					let _ = queue!(stdout, Print("Name"));
					let _ = queue!(stdout, SetAttribute(Attribute::NoBold));
					let _ = queue!(stdout, MoveTo(3, 4));
					let _ = queue!(stdout, Print("What name or alias would you like to give to the remote?"));
					let _ = queue!(stdout, MoveTo(3, 6));
					let _ = queue!(stdout, Print("> "));
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
					let _ = queue!(stdout, SetBackgroundColor(Color::Yellow));
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

fn process_input_raw_mode(event: &Event) -> UserInput
{
	let key_event = match event.as_key_event()
	{
		Some(value) => value,
		None =>
		{
			return UserInput::Ignore;
		}
	};

	if key_event.modifiers == KeyModifiers::CONTROL
	{
		if key_event.code == KeyCode::Char('c')
		{
			return UserInput::Exit;
		}
	}

	if key_event.code == KeyCode::Down
	{
		return UserInput::MoveDown;
	}

	if key_event.code == KeyCode::Up
	{
		return UserInput::MoveUp;
	}

	if key_event.code == KeyCode::Enter
	{
		return UserInput::Select;
	}

	if key_event.code == KeyCode::Esc
	{
		return UserInput::Exit;
	}

	UserInput::Ignore
}

fn read_input_as_string() -> String
{
	let mut result = String::new();
	match stdin().read_line(&mut result)
	{
		Ok(_) => (),
		Err(error) =>
		{
			panic_gracefully(format!("[ERROR] Failed to read input, {}", error).as_str());
			std::process::exit(1);
		}
	}

	result = result.replace("\n", "");
	result = result.replace("\r", "");

	result
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

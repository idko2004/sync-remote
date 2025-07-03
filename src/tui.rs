use std::
{
	io::
	{
		stdout,
		Stdout,
		Write
	}
};
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

enum UserInput
{
	Exit,
	Select,
	MoveDown,
	MoveUp,
	Ignore,
}

pub fn start_tui_blocking(selectable_options: &Vec<String>) -> Option<usize>
{
	let _ = execute!(stdout(), EnterAlternateScreen);
	let _ = execute!(stdout(), Hide);
	match enable_raw_mode()
	{
		Ok(_) => (),
		Err(error) =>
		{
			println!("Error al activar modo crudo!! {error}");
			return None;
		}
	}

	let selectable_options = selectable_options.clone();

	let mut index_selected_option = 0;

	redraw(&selectable_options, index_selected_option);

	loop
	{
		let input = match read()
		{
			Ok(value) => value,
			Err(error) =>
			{
				let _ = disable_raw_mode();
				println!("Error al leer input!! {error}");
				return None;
			}
		};

		match process_input(&input)
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
				break;
			},
			UserInput::Exit =>
			{
				let _ = execute!(stdout(), Clear(ClearType::All));
				let _ = execute!(stdout(), Clear(ClearType::Purge));
				let _ = execute!(stdout(), Show);
				let _ = execute!(stdout(), LeaveAlternateScreen);
				let _ = disable_raw_mode();
				std::process::exit(0);
			},
			_ => (),
		}

		redraw(&selectable_options, index_selected_option);
		std::thread::sleep(std::time::Duration::from_millis(50));
		//let _ = execute!(stdout(), MoveTo(5, 5));
		//let _ = execute!(stdout(), Print(format!("{:?}", input)));
	}

	let _ = execute!(stdout(), Clear(ClearType::All));
	let _ = execute!(stdout(), Clear(ClearType::Purge));
	let _ = execute!(stdout(), Show);
	let _ = execute!(stdout(), LeaveAlternateScreen);
	let _ = disable_raw_mode();
	
	Some(index_selected_option)
}

fn redraw(selectable_options: &Vec<String>, index_selected_option: usize)
{
	let mut stdout = stdout();

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

	stdout = draw_box(String::from(" Select a remote to sync "), width, height, stdout);
	stdout = draw_options(selectable_options, index_selected_option, stdout);
	let _ = queue!(stdout, MoveTo(0, height));
	let _ = stdout.flush();
}


fn draw_box(title: String, width: u16, height: u16, mut stdout: Stdout) -> Stdout
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

fn draw_options(selectable_options: &Vec<String>, index_selected_option: usize, mut stdout: Stdout) -> Stdout
{
	let column: u16 = 3;
	let mut row: u16 = 2;
	let mut i = 0;

	for item in selectable_options
	{
		let _ = queue!(stdout, MoveTo(column, row));
		if index_selected_option != i
		{
			let _ = queue!(stdout, Print(format!(" _ {item}" )));
		}
		else
		{
			let _ = queue!(stdout, SetBackgroundColor(Color::White));
			let _ = queue!(stdout, SetForegroundColor(Color::Black));
			let _ = queue!(stdout, SetAttribute(Attribute::Bold));
			let _ = queue!(stdout, Print(format!(" > {item} ")));
			let _ = queue!(stdout, SetAttribute(Attribute::Reset));
			let _ = queue!(stdout, SetForegroundColor(Color::Reset));
			let _ = queue!(stdout, SetBackgroundColor(Color::Reset));
		}
		
		i += 1;
		row += 1;
	}

	stdout
}

fn process_input(event: &Event) -> UserInput
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

use std::io::{stdout, Stdout, Write};
use crossterm::{queue, execute, terminal::{EnterAlternateScreen, LeaveAlternateScreen, Clear, ClearType, size}, style::Print, cursor::MoveTo};

pub fn start_tui_blocking(selectable_options: Vec<String>)
{
	//let _ = execute!(stdout(), EnterAlternateScreen);
	redraw(selectable_options);
}

fn redraw(selectable_options: Vec<String>)
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
	stdout = draw_options(selectable_options, 0, stdout);
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
		let _ = queue!(stdout, Print("\u{2502}\n"));

		i += 1;

		if i > height - 2
		{
			break;
		}
	}

	//Drwa bottom border
	let mut line =  String::with_capacity(width as usize);
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

fn draw_options(selectable_options: Vec<String>, index_selected_option: usize, mut stdout: Stdout) -> Stdout
{
	let column: u16 = 4;
	let mut row: u16 = 4;
	let mut i = 0;

	for item in selectable_options
	{
		let _ = queue!(stdout, MoveTo(column, row));
		if index_selected_option != i
		{
			let _ = queue!(stdout, Print(format!("_ {item}")));
		}
		else
		{
			let _ = queue!(stdout, Print(format!("> {item}")));
		}
		
		i += 1;
		row += 1;
	}

	stdout
}


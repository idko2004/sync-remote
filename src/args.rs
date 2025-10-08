use std::env;

pub struct Args
{
	pub wait_to_exit: bool,
	pub log_level: LogLevel,
	pub continue_on_error: bool,
}

#[derive(PartialEq)]
pub enum LogLevel
{
	Default,
	Verbose
}

pub fn check_arguments() -> Args
{
	let mut result = Args
	{
		wait_to_exit: false,
		log_level: LogLevel::Default,
		continue_on_error: false,
	};

	let args: Vec<String> = env::args().collect();
	for arg in args
	{
		match arg.as_str()
		{
			"--wait-to-exit" =>
			{
				result.wait_to_exit = true;
			},
			"--verbose" =>
			{
				result.log_level = LogLevel::Verbose;
			},
			"--continue-on-error" =>
			{
				result.continue_on_error = true;
			}
			_ => ()
		}
	}

	result
}
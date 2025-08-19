use std::env;

pub struct Args
{
	pub wait_to_exit: bool,
}

pub fn check_arguments() -> Args
{
	let mut result = Args
	{
		wait_to_exit: false,
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
			_ => ()
		}
	}

	result
}
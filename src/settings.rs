use crate::encode::settings::GifSettings;
use crate::util;
use crate::util::cmd::Command;
use chrono::Local;
use clap::ArgMatches;

/* General application settings */
#[derive(Clone, Debug)]
pub struct AppSettings {
	pub args: ArgMatches<'static>,
}

impl AppSettings {
	/**
	 * Create a new AppSettings object.
	 *
	 * @return AppSettings
	 */
	pub fn new(args: ArgMatches<'static>) -> Self {
		Self { args }
	}

	/**
	 * Get a Command object from parsed arguments.
	 *
	 * @return Command
	 */
	pub fn get_command(&self) -> Command {
		match self.args.value_of("command") {
			Some(cmd) => {
				let cmd = String::from(cmd);
				if !cmd.contains(' ') {
					Command::new(cmd, Vec::new())
				} else {
					Command::new(String::from("sh"), vec![String::from("-c"), cmd])
				}
			}
			None => panic!("No command specified to run"),
		}
	}

	/**
	 * Get the main color from parsed arguments.
	 *
	 * @return u64
	 */
	pub fn get_color(&self) -> u64 {
		u64::from_str_radix(self.args.value_of("color").unwrap_or("FF00FF"), 16)
			.expect("Failed to parse the color HEX")
	}

	/**
	 * Get FPS value from parsed arguments.
	 *
	 * @return u32
	 */
	pub fn get_fps(&self) -> u32 {
		match self.args.subcommand_matches("record") {
			Some(matches) => matches
				.value_of("fps")
				.unwrap_or_default()
				.parse()
				.unwrap_or(10),
			None => 10,
		}
	}

	/**
	 * Get the output file from parsed arguments.
	 *
	 * @return String
	 */
	pub fn get_output_file(&self) -> String {
		match self.args.subcommand_matches("save") {
			Some(matches) => {
				let mut file_name =
					String::from(matches.value_of("output").unwrap_or_default());
				if matches.is_present("prompt") {
					file_name = rprompt::prompt_reply_stdout("Enter file name: ")
						.unwrap_or(file_name);
				}
				if matches.is_present("date") || matches.is_present("timestamp") {
					util::update_file_name(
						file_name,
						if matches.is_present("date") {
							Local::now().format("%Y%m%dT%H%M%S").to_string()
						} else {
							Local::now().timestamp().to_string()
						},
					)
				} else {
					file_name
				}
			}
			None => String::from("t.gif"),
		}
	}

	/**
	 * Get GIF settings from parsed arguments.
	 *
	 * @return GifSettings
	 */
	pub fn get_gif_settings(&self) -> GifSettings {
		match self.args.subcommand_matches("gif") {
			Some(matches) => GifSettings::new(
				matches
					.value_of("repeat")
					.unwrap_or("-1")
					.parse()
					.unwrap_or_default(),
				matches
					.value_of("speed")
					.unwrap_or_default()
					.parse()
					.unwrap_or(10),
			),
			None => GifSettings::new(-1, 10),
		}
	}
}

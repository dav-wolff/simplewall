use std::{
	error::Error,
	ffi::OsString,
	path::PathBuf
};

mod wallpaper;
use clap::Parser;
use wallpaper::Wallpaper;

mod wayland;

/// Sets a wallpaper (or multiple) as a layer shell surface.
///
/// To set multiple wallpapers, separate the wallpaper arguments using '--' like so:
/// simplewall wallpaper.jpg -- wallpaper2.jpg --namespace overlay-wallpaper
#[derive(Parser, Debug)]
#[command(version, author, about, verbatim_doc_comment)]
struct Args {
	/// The image to use as a wallpaper
	wallpaper: PathBuf,
	/// The namespace to use for the layer shell surface
	#[arg(short, long, default_value = "wallpaper")]
	namespace: String,
	/// More wallpapers
	#[arg(last = true)]
	more: Option<Vec<OsString>>,
}

#[derive(Debug)]
struct WallpaperOptions {
	wallpaper: Wallpaper,
	namespace: String,
}

fn main() {
	let mut wallpapers = Vec::new();
	let mut args = Args::parse();
	
	loop {
		wallpapers.push((args.wallpaper, args.namespace));
		
		match args.more {
			Some(more) => {
				let iter = std::iter::once(OsString::new()) // insert empty element where Parser expects the program name
					.chain(more);
				args = Args::parse_from(iter)
			},
			None => break,
		}
	}
	
	// only load wallpapers after all arguments were parsed without errors
	
	let wallpapers: Vec<_> = wallpapers.into_iter()
		.map(|(path, namespace)| -> Result<_, Box<dyn Error>> {
			Ok(WallpaperOptions {
				wallpaper: Wallpaper::load(&path)?,
				namespace,
			})
		})
		.collect::<Result<_, _>>().unwrap();
	
	assert_eq!(wallpapers.len(), 1, "multiple wallpapers not yet implemented");
	
	wayland::run(wallpapers.into_iter().next().unwrap());
}

use std::path::PathBuf;
use clap::{Parser, Subcommand};
use crate::error::Error;

mod mage_arena;
mod error;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None, arg_required_else_help = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Read the current Mage Arena flag from storage.
    Read {
        /// The bitmap image containing the palette.
        #[clap(short, long, default_value = "palette.bmp")]
        palette_file: PathBuf,

        /// The file to read the flag data into.
        #[clap(short, long, default_value = "flag.bmp")]
        output_file: PathBuf,
    },

    /// Write the image into the Mage Arena flag storage.
    Write {
        /// The bitmap image containing the palette.
        #[clap(short, long, default_value = "palette.bmp")]
        palette_file: PathBuf,
        
        /// The file to read the flag data from.
        #[clap(short, long, default_value = "custom_flag.bmp")]
        input_file: PathBuf,
    }
}

fn main() -> Result<(), Error> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Read { palette_file, output_file }) => {
            mage_arena::read_flag(palette_file, output_file)?;
        },
        
        Some(Commands::Write { palette_file, input_file }) => {
            mage_arena::write_flag(palette_file, input_file)?;
        }

        None => {}
    }

    Ok(())
}

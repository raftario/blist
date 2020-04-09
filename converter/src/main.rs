mod legacy;

use crate::legacy::LegacyPlaylist;
use anyhow::{bail, Result};
use glob::GlobError;
use rayon::prelude::*;
use std::{
    fs::{self, File},
    io::{BufReader, BufWriter},
    path::{Path, PathBuf},
    process,
    time::Instant,
};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
/// Converts legacy Beat Saber playlists to the new format
struct Opt {
    /// Glob patter of files to convert
    #[structopt(name = "GLOB")]
    glob: String,
    /// Prints verbose information
    #[structopt(short, long)]
    verbose: bool,
    /// Skips custom data when converting playlists
    #[structopt(long = "no-custom-data", parse(from_flag = std::ops::Not::not))]
    custom_data: bool,
    /// Exits when an error occurs instead of just displaying it
    #[structopt(long = "exit-on-error")]
    exit_on_error: bool,
    /// Deletes converted files
    #[structopt(long = "delete-converted")]
    delete_converted: bool,
}

macro_rules! exit {
    ($e:expr) => {{
        eprintln!("{}", $e);
        ::std::process::exit(1)
    }};
}

#[inline]
fn convert<P: AsRef<Path>>(
    path: P,
    verbose: bool,
    custom_data: bool,
    exit_on_error: bool,
    delete_converted: bool,
) -> bool {
    let path = path.as_ref();
    if let Err(e) = convert_inner(path, verbose, custom_data, delete_converted) {
        eprintln!("Failed conversion for `{}`: {}", path.display(), e);
        if exit_on_error {
            process::exit(1);
        }
        return false;
    }
    true
}

fn convert_inner<P: AsRef<Path>>(
    path: P,
    verbose: bool,
    custom_data: bool,
    delete_converted: bool,
) -> Result<()> {
    let old_path = path.as_ref();
    let new_path = old_path.with_extension("blist");
    if new_path.exists() {
        bail!("Destination path `{}` already exists", new_path.display());
    }

    if verbose {
        println!(
            "Converting `{}` to `{}`",
            old_path.display(),
            new_path.display(),
        );
    }

    if verbose {
        println!("Reading `{}`", old_path.display());
    }
    let legacy_playlist: LegacyPlaylist = {
        let mut reader = BufReader::new(File::open(old_path)?);
        serde_json::from_reader(&mut reader)?
    };
    let playlist = legacy_playlist.into_playlist(custom_data)?;
    if verbose {
        println!("Writing `{}`", new_path.display());
    }
    {
        let mut writer = BufWriter::new(File::create(&new_path)?);
        playlist.write(&mut writer)?;
    }

    if verbose {
        println!(
            "Done converting `{}` to `{}`",
            old_path.display(),
            new_path.display(),
        );
    }
    if delete_converted {
        if verbose {
            println!("Deleing `{}`", old_path.display());
        }
        fs::remove_file(old_path)?;
    }
    Ok(())
}

fn main() {
    let opt = Opt::from_args();

    let start = Instant::now();

    let paths = match glob::glob(&opt.glob) {
        Ok(p) => match p.collect::<Result<Vec<PathBuf>, GlobError>>() {
            Ok(p) => p,
            Err(e) => exit!(e),
        },
        Err(e) => exit!(e),
    };
    let successful = paths
        .par_iter()
        .map(|p| {
            convert(
                p,
                opt.verbose,
                opt.custom_data,
                opt.exit_on_error,
                opt.delete_converted,
            )
        })
        .filter(|c| *c)
        .count();

    let elapsed = start.elapsed();
    let elapsed_ms = elapsed.as_millis();
    if elapsed_ms > 1000 {
        let elapsed_s = elapsed_ms as f64 / 1000.0;
        println!(
            "Succesfully converted {} playlists in {:.3} s",
            successful, elapsed_s
        )
    } else {
        println!(
            "Succesfully converted {} playlists in {} ms",
            successful, elapsed_ms
        )
    }
}

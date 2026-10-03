//! `cargo deploy` — build an application's binary and put it where you run it from.
//!
//! ```text
//! cargo deploy                     release build -> D:\Programs
//! cargo deploy E:\Tools            somewhere else
//! cargo deploy --debug             the debug build, for once
//! cargo deploy --profile fast      a profile of the package's own
//! ```
//!
//! # Why a theme crate holds this
//!
//! Because it is the crate every Azur application already depends on, and this is a thing
//! they all need and none of them is the place for: getting the binary out of `target` and
//! into wherever it is actually run from. It was two hundred lines in the file explorer and
//! nothing at all in the other two, and the alternative to moving it was copying it twice
//! into files that would then differ only in the name of an executable — or making each
//! repository a workspace to hold one file. Nothing here draws anything and nothing in a
//! window can reach it; see [`crate::render`] for the other decision this crate makes about
//! how an Azur application is *built* rather than about how it looks.
//!
//! # What an application writes
//!
//! Two files, and neither holds any of the work. `.cargo/config.toml`:
//!
//! ```toml
//! [alias]
//! deploy = "run --quiet --release --example deploy --"
//! ```
//!
//! and `examples/deploy.rs`:
//!
//! ```no_run
//! fn main() -> std::process::ExitCode {
//!     azur_egui_theme::deploy::main(azur_egui_theme::deploy::Program {
//!         bin: env!("CARGO_PKG_NAME"),
//!         manifest: env!("CARGO_MANIFEST_DIR"),
//!     })
//! }
//! ```
//!
//! # Why an example and not a script
//!
//! `cargo` has no post-build hook. `build.rs` runs *before* the build and cannot see its
//! own output, and an `[alias]` can only stand for a cargo subcommand — so it can build
//! or it can run, but it cannot build and then copy. What it *can* do is run something
//! that does both, which is what this is.
//!
//! An example target rather than a second `[[bin]]` because a package with two binaries
//! needs `default-run` before `cargo run` means anything again, and the deploy helper has
//! no business being the reason for that. The alias asks for `--release` so that building
//! the helper costs nothing extra: the dependencies it needs are the ones the release
//! build is about to want anyway, rather than a second debug copy of eframe and wgpu on a
//! fresh clone.
//!
//! # Replacing a copy that is running
//!
//! Windows will not let you overwrite the image of a running process, which is exactly
//! the state the destination is in when you have the thing open to see what you just
//! changed. It *will* let you rename it — the image stays mapped from wherever the file
//! went. So a copy that fails for sharing moves the old binary aside and copies again,
//! and the next run clears up what it left. The running window keeps working; it is
//! simply still the old build, which it was anyway.

use std::ffi::{OsStr, OsString};
use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};
use std::time::Instant;

/// Where a binary goes when nothing says otherwise.
pub const DEFAULT_DIR: &str = r"D:\Programs";

/// The environment variable that overrides [`DEFAULT_DIR`]. An argument overrides both.
pub const DIR_VAR: &str = "AZUR_DEPLOY_DIR";

/// The two things only the application knows.
///
/// Both are written out at the call site because `env!` can only answer where it is
/// written: asked in here it would name this crate, and deploy `azur-egui-theme`.
#[derive(Clone, Copy, Debug)]
pub struct Program<'a> {
    /// The `[[bin]]` to deploy, without any `.exe`. `env!("CARGO_PKG_NAME")` for a
    /// package whose one binary is named after it, which is the usual shape.
    pub bin: &'a str,
    /// The directory holding the `Cargo.toml` that builds it:
    /// `env!("CARGO_MANIFEST_DIR")`. Passed so the deploy works from a subdirectory.
    pub manifest: &'a str,
}

/// Read the command line, build, copy, and report: the whole of an `examples/deploy.rs`.
pub fn main(program: Program<'_>) -> ExitCode {
    let asked = match parse(std::env::args().skip(1)) {
        Ok(asked) => asked,
        Err(complaint) => {
            eprintln!("deploy: {complaint}");
            return ExitCode::FAILURE;
        }
    };
    let Asked::Deploy { dir, profile } = asked else {
        println!("cargo deploy [--debug | --profile <name>] [<directory>]");
        println!();
        println!("  builds {} and copies it to the given directory,", program.bin);
        println!("  or ${DIR_VAR}, or {DEFAULT_DIR}");
        return ExitCode::SUCCESS;
    };
    let dir = dir
        .or_else(|| std::env::var_os(DIR_VAR).map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from(DEFAULT_DIR));

    match deploy(program, &dir, &profile) {
        Ok(()) => ExitCode::SUCCESS,
        Err(why) => {
            eprintln!("deploy: {why}");
            ExitCode::FAILURE
        }
    }
}

/// What a command line asked for.
#[derive(Debug, PartialEq, Eq)]
enum Asked {
    Help,
    Deploy {
        dir: Option<PathBuf>,
        profile: String,
    },
}

/// Read the arguments, or say which one was not understood.
///
/// Separate from [`main`] because it is the half worth testing: everything after it
/// either shells out to cargo or touches the disk.
fn parse(args: impl Iterator<Item = String>) -> Result<Asked, String> {
    /// What `--profile` wants after it, once it is clear it is not another flag.
    fn named(name: Option<String>) -> Result<String, String> {
        match name {
            Some(name) if !name.is_empty() && !name.starts_with('-') => Ok(name),
            _ => Err("`--profile` wants the name of one, as in `--profile fast`".to_owned()),
        }
    }

    let mut args = args;
    let mut dir = None;
    let mut profile = String::from("release");
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => return Ok(Asked::Help),
            "--debug" => profile = "dev".to_owned(),
            "--release" => profile = "release".to_owned(),
            "--profile" => profile = named(args.next())?,
            other => match other.strip_prefix("--profile=") {
                Some(name) => profile = named(Some(name.to_owned()))?,
                // A path, unless it looks like a flag — in which case it is a typo, and
                // taking it for a destination would deploy into a directory called
                // `--relase`.
                None if other.starts_with('-') => {
                    return Err(format!("I do not know what `{other}` means"))
                }
                None => dir = Some(PathBuf::from(other)),
            },
        }
    }
    Ok(Asked::Deploy { dir, profile })
}

/// The `--profile` cargo takes for a name, and the directory it writes into.
///
/// The two are the same for every profile a package declares. They differ for exactly one
/// of cargo's own: the default debug profile is called `dev`, writes into `target/debug`,
/// and is not addressable as `--profile debug`.
fn profile(asked: &str) -> (&str, &str) {
    match asked {
        "dev" | "debug" => ("dev", "debug"),
        other => (other, other),
    }
}

fn deploy(program: Program<'_>, dir: &Path, asked: &str) -> std::io::Result<()> {
    let (flag, built_in) = profile(asked);
    // The cargo that is running this, so a `+toolchain` or a rustup override is honoured
    // rather than second-guessed.
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());

    let at = Instant::now();
    let status = Command::new(&cargo)
        .current_dir(program.manifest)
        .args(["build", "--profile", flag, "--bin", program.bin])
        .status()?;
    if !status.success() {
        // Cargo has already said what was wrong, at length and in colour. Adding to it
        // would only push it up the screen.
        return Err(Error::other(format!(
            "the build failed ({status}); nothing was copied"
        )));
    }
    let built = at.elapsed();

    let exe = format!("{}{}", program.bin, std::env::consts::EXE_SUFFIX);
    let from = target_dir()?.join(built_in).join(&exe);
    let meta = std::fs::metadata(&from)
        .map_err(|e| Error::new(e.kind(), format!("{} was not built: {e}", from.display())))?;

    // What was there before, so the report can say whether anything actually changed.
    let to = dir.join(&exe);
    let before = std::fs::metadata(&to).ok().map(|m| m.len());

    std::fs::create_dir_all(dir)
        .map_err(|e| Error::new(e.kind(), format!("{} cannot be created: {e}", dir.display())))?;
    sweep(dir, &exe);

    let parked = install(&from, &to)?;
    let after = std::fs::metadata(&to)?.len();
    if after != meta.len() {
        return Err(Error::other(format!(
            "copied {} bytes but {} is {after} — the copy did not land",
            meta.len(),
            to.display()
        )));
    }

    println!();
    println!("  built    {asked} in {:.1}s", built.as_secs_f32());
    println!("  from     {}", from.display());
    println!(
        "  to       {}  ({:.1} MB{})",
        to.display(),
        after as f64 / (1024.0 * 1024.0),
        match before {
            Some(was) if was == after => ", same size as before".to_owned(),
            Some(was) => format!(", was {:.1} MB", was as f64 / (1024.0 * 1024.0)),
            None => ", new".to_owned(),
        }
    );
    if let Some(parked) = parked {
        println!(
            "  note     it was running, so the old one is now {} — it will go on the next deploy",
            parked.file_name().unwrap_or_default().to_string_lossy()
        );
    }
    Ok(())
}

/// The `target` directory, wherever it has been moved to.
///
/// Taken from this executable's own location rather than from `CARGO_TARGET_DIR`, because
/// that variable is only one of the ways it gets set — `.cargo/config.toml` has
/// `build.target-dir` too — and the running example is *in* the answer:
/// `<target>/<profile>/examples/deploy.exe`.
fn target_dir() -> std::io::Result<PathBuf> {
    let me = std::env::current_exe()?;
    me.parent()
        .and_then(Path::parent)
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .ok_or_else(|| {
            Error::new(
                ErrorKind::NotFound,
                format!("cannot find the target directory from {}", me.display()),
            )
        })
}

/// Copy, moving a running copy out of the way if that is what is in the road.
///
/// Returns where the old one was parked, when it had to be.
fn install(from: &Path, to: &Path) -> std::io::Result<Option<PathBuf>> {
    match std::fs::copy(from, to) {
        Ok(_) => Ok(None),
        Err(e) if in_use(&e) => {
            let parked = park(to)?;
            std::fs::copy(from, to).map_err(|e| {
                // The rename has already happened; say so, or the destination looks empty
                // for no reason anybody could work out.
                Error::new(
                    e.kind(),
                    format!(
                        "moved the old binary to {} and then could not copy: {e}",
                        parked.display()
                    ),
                )
            })?;
            Ok(Some(parked))
        }
        Err(e) => Err(Error::new(
            e.kind(),
            format!("cannot write {}: {e}", to.display()),
        )),
    }
}

/// Whether this is Windows saying "something has that file open".
fn in_use(e: &Error) -> bool {
    /// `ERROR_ACCESS_DENIED` and `ERROR_SHARING_VIOLATION`.
    const BUSY: [i32; 2] = [5, 32];
    e.raw_os_error().is_some_and(|code| BUSY.contains(&code))
        || e.kind() == ErrorKind::PermissionDenied
}

/// The name [`park`] gives a file on its `n`th attempt.
///
/// Its own function so that the test can check [`sweep`] takes back exactly what this
/// hands out: a name `park` invents and `sweep` will not have is a file left behind for
/// good.
fn parked_name(exe: &OsStr, n: u32) -> OsString {
    let mut name = exe.to_os_string();
    name.push(if n == 0 {
        ".old".to_owned()
    } else {
        format!(".old{n}")
    });
    name
}

/// Rename a file out of the way, to a name nothing is using.
fn park(path: &Path) -> std::io::Result<PathBuf> {
    for n in 0..100 {
        let name = parked_name(path.file_name().unwrap_or_default(), n);
        let parked = path.with_file_name(&name);
        if parked.exists() {
            continue;
        }
        std::fs::rename(path, &parked)?;
        return Ok(parked);
    }
    Err(Error::new(
        ErrorKind::AlreadyExists,
        format!(
            "{} is in use and there are a hundred old copies beside it already",
            path.display()
        ),
    ))
}

/// Clear away binaries parked by an earlier deploy, now that whatever held them has quit.
///
/// Best effort by design: one that is *still* running simply stays, and is not worth a word.
fn sweep(dir: &Path, exe: &str) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let exe = exe.to_ascii_lowercase();
    for entry in entries.flatten() {
        if parked_by_us(&entry.file_name().to_string_lossy().to_ascii_lowercase(), &exe) {
            let _ = std::fs::remove_file(entry.path());
        }
    }
}

/// Whether a name is one [`park`] made: `<exe>.old`, or `.old` and a number.
///
/// Both names arrive lowercased. Deliberately not "starts with `<exe>.old`", because this
/// decides what gets deleted, and `azur-file-explorer.exe.old-keep-this` is somebody's
/// file rather than ours.
fn parked_by_us(name: &str, exe: &str) -> bool {
    name.strip_prefix(exe)
        .and_then(|tail| tail.strip_prefix(".old"))
        .is_some_and(|n| n.is_empty() || n.chars().all(|c| c.is_ascii_digit()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parsed(args: &[&str]) -> Result<Asked, String> {
        parse(args.iter().map(|a| (*a).to_owned()))
    }

    fn wanted(dir: Option<&str>, profile: &str) -> Result<Asked, String> {
        Ok(Asked::Deploy {
            dir: dir.map(PathBuf::from),
            profile: profile.to_owned(),
        })
    }

    #[test]
    fn nothing_asked_for_is_a_release_build_where_the_default_says() {
        // And the directory stays `None` rather than resolved here, so that
        // `$AZUR_DEPLOY_DIR` still gets a say: an argument has to beat it, absence
        // must not.
        assert_eq!(parsed(&[]), wanted(None, "release"));
    }

    #[test]
    fn a_bare_argument_is_where_it_goes() {
        assert_eq!(parsed(&[r"E:\Tools"]), wanted(Some(r"E:\Tools"), "release"));
        assert_eq!(
            parsed(&["--debug", r"E:\Tools"]),
            wanted(Some(r"E:\Tools"), "dev")
        );
    }

    #[test]
    fn a_profile_can_be_named_either_way_round() {
        assert_eq!(parsed(&["--profile", "fast"]), wanted(None, "fast"));
        assert_eq!(parsed(&["--profile=fast"]), wanted(None, "fast"));
    }

    #[test]
    fn the_last_word_on_the_profile_wins() {
        assert_eq!(parsed(&["--debug", "--release"]), wanted(None, "release"));
        assert_eq!(parsed(&["--release", "--debug"]), wanted(None, "dev"));
    }

    #[test]
    fn a_flag_nobody_knows_stops_the_deploy() {
        // Rather than being taken for a destination, which is how you deploy into a
        // directory called `--relase`.
        assert!(parsed(&["--relase"]).is_err());
        assert!(parsed(&["-r"]).is_err());
    }

    #[test]
    fn a_profile_with_no_name_stops_the_deploy() {
        assert!(parsed(&["--profile"]).is_err());
        assert!(parsed(&["--profile="]).is_err());
        // The name of a profile is not the next flag; this is a missing argument.
        assert!(parsed(&["--profile", "--debug"]).is_err());
    }

    #[test]
    fn help_is_answered_wherever_it_appears() {
        assert_eq!(parsed(&["-h"]), Ok(Asked::Help));
        assert_eq!(parsed(&[r"E:\Tools", "--help"]), Ok(Asked::Help));
    }

    #[test]
    fn the_debug_profile_is_asked_for_by_the_name_cargo_takes() {
        // `--profile debug` is an error from cargo, and `target/dev` does not exist.
        assert_eq!(profile("debug"), ("dev", "debug"));
        assert_eq!(profile("dev"), ("dev", "debug"));
    }

    #[test]
    fn a_profile_of_the_packages_own_is_its_own_directory() {
        assert_eq!(profile("release"), ("release", "release"));
        assert_eq!(profile("fast"), ("fast", "fast"));
    }

    #[test]
    fn sweeping_takes_what_a_deploy_parked_and_nothing_else() {
        let exe = "azur-file-explorer.exe";
        assert!(parked_by_us("azur-file-explorer.exe.old", exe));
        assert!(parked_by_us("azur-file-explorer.exe.old7", exe));
        // The binary itself, which is the file we are about to write.
        assert!(!parked_by_us(exe, exe));
        assert!(!parked_by_us("azur-file-explorer.exe.old-keep-this", exe));
        assert!(!parked_by_us("azur-file-explorer.exe.older", exe));
        assert!(!parked_by_us("azur-task-manager.exe.old", exe));
        assert!(!parked_by_us("notes.txt", exe));
    }

    #[test]
    fn what_park_makes_is_what_sweep_takes() {
        // The two halves of the running-copy dance, and they are decided in two different
        // functions.
        let exe = "app.exe";
        for n in 0..100 {
            let name = parked_name(OsStr::new(exe), n);
            assert!(parked_by_us(&name.to_string_lossy(), exe), "{name:?}");
        }
    }
}

//! copy-tools copies binaries from its own image into a shared volume, for use
//! as a Kubernetes init container.
//!
//!     copy-tools SRC DSTDIR
//!
//! A file SRC is copied into DSTDIR; a directory SRC has its contents copied
//! into DSTDIR. Any failure exits nonzero, so Kubernetes never starts the app
//! container with missing binaries.
//!
//! Standard library only: every dependency is one more thing a scanner can flag.

use std::os::unix::fs::{symlink, OpenOptionsExt, PermissionsExt};
use std::path::Path;
use std::process::exit;
use std::{env, fs, io};

fn main() {
    let args: Vec<_> = env::args_os().collect();
    if args.len() != 3 {
        eprintln!("usage: copy-tools SRC DSTDIR\n\nCopies file SRC, or the contents of directory SRC, into directory DSTDIR.");
        exit(2);
    }
    let (src, dst) = (Path::new(&args[1]), Path::new(&args[2]));
    if let Err(err) = run(src, dst) {
        eprintln!("copy-tools: copying {} to {}: {err}", src.display(), dst.display());
        exit(1);
    }
    println!("copy-tools: copied {} to {}", src.display(), dst.display());
}

fn run(src: &Path, dst: &Path) -> io::Result<()> {
    if fs::metadata(src)?.is_dir() {
        return copy_dir(src, dst);
    }
    let name = src.file_name().ok_or(io::ErrorKind::InvalidInput)?;
    copy_file(src, &dst.join(name))
}

fn copy_dir(src: &Path, dst: &Path) -> io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let (from, to) = (entry.path(), dst.join(entry.file_name()));
        let kind = entry.file_type()?;
        if kind.is_dir() {
            copy_dir(&from, &to)?;
        } else if kind.is_file() {
            copy_file(&from, &to)?;
        } else if kind.is_symlink() {
            symlink(fs::read_link(&from)?, &to).map_err(|err| at(&to, err))?;
        } else {
            return Err(at(&from, io::ErrorKind::Unsupported.into()));
        }
    }
    Ok(())
}

// Refuses to overwrite, and copies permission bits only, so setuid never reaches the volume.
fn copy_file(src: &Path, dst: &Path) -> io::Result<()> {
    let mut from = fs::File::open(src).map_err(|err| at(src, err))?;
    let mode = from.metadata()?.permissions().mode() & 0o777;
    let mut to = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(mode)
        .open(dst)
        .map_err(|err| at(dst, err))?;
    io::copy(&mut from, &mut to).map_err(|err| at(dst, err))?;
    Ok(())
}

fn at(path: &Path, err: io::Error) -> io::Error {
    io::Error::new(err.kind(), format!("{}: {err}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> std::path::PathBuf {
        let dir = env::temp_dir().join(format!("copy-tools-{}-{name}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn copies_tools() {
        let (src, dst, single) = (temp_dir("src"), temp_dir("dst"), temp_dir("single"));
        let tool = src.join("tool");
        fs::write(&tool, "bin").unwrap();
        fs::set_permissions(&tool, fs::Permissions::from_mode(0o4755)).unwrap();
        fs::create_dir(src.join("sub")).unwrap();
        fs::write(src.join("sub/nested"), "bin").unwrap();
        symlink("tool", src.join("link")).unwrap();

        run(&src, &dst).unwrap();
        run(&tool, &single).unwrap();
        for copied in [dst.join("tool"), single.join("tool")] {
            let mode = fs::metadata(&copied).unwrap().permissions().mode();
            assert_eq!(mode & 0o7111, 0o111, "{}: want executable without setuid", copied.display());
            assert_eq!(fs::read(&copied).unwrap(), b"bin");
        }
        assert_eq!(fs::read(dst.join("sub/nested")).unwrap(), b"bin");
        assert_eq!(fs::read_link(dst.join("link")).unwrap(), Path::new("tool"));

        assert!(run(&tool, &single).is_err(), "existing files must not be overwritten");
        assert!(run(&src.join("missing"), &dst).is_err(), "missing source must fail");
    }
}

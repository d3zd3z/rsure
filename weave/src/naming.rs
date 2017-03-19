//! Weave files will follow a file naming convention.  This determines the names of various temp
//! files and other aspects.  The SCCS conventions are not followed, because they are not save
//! (this crate will never write to a file that already exists).

use ::Result;
use std::path::{Path, PathBuf};
use std::fs::{File, OpenOptions};
use std::io::ErrorKind;

/// A naming convention provides utilities needed to find the involved files, and construct
/// temporary files as part of writing the new weave.  The underlying object should keep the path
/// and base name.
pub trait NamingConvention {
    /// Create a temporary file for writing.  Upon success, returns the full path of the file, and
    /// the opened File for writing to the file.  The path should refer to a new file that did not
    /// exist prior to this call.
    fn temp_file(&self) -> Result<(PathBuf, File)>;

    /// Return the pathname of the primary file.
    fn main_file(&self) -> PathBuf;

    /// Return the pathname of the backup file.
    fn backup_file(&self) -> PathBuf;
}

/// The SimpleNaming is a NamingConvention that has a basename, with the main file having a
/// specified extension, the backup file having a ".bak" extension, and the temp files using a
/// numbered extension starting with ".0".  If the names are intended to be compressed, a ".gz"
/// suffix can also be added.
pub struct SimpleNaming {
    // The directory for the files to be written.
    path: PathBuf,
    // The string for the base filename.
    base: String,
    // The extension to use for the main name.
    ext: String,
    // Are these names to indicate compression?
    compressed: bool,
}

impl SimpleNaming {
    pub fn new<P: AsRef<Path>>(path: P, base: &str, ext: &str, compressed: bool) -> SimpleNaming {
        SimpleNaming {
            path: path.as_ref().to_path_buf(),
            base: base.to_string(),
            ext: ext.to_string(),
            compressed: compressed,
        }
    }

    pub fn make_name(&self, ext: &str) -> PathBuf {
        let name = format!("{}.{}{}", self.base, ext,
                           if self.compressed { ".gz" } else { "" });
        self.path.join(name)
    }
}

impl NamingConvention for SimpleNaming {
    fn main_file(&self) -> PathBuf {
        self.make_name(&self.ext)
    }

    fn backup_file(&self) -> PathBuf {
        self.make_name("bak")
    }

    fn temp_file(&self) -> Result<(PathBuf, File)> {
        let mut n = 0;
        loop {
            let name = self.make_name(&n.to_string());

            match OpenOptions::new().write(true).create_new(true).open(&name) {
                Ok(fd) => return Ok((name, fd)),
                Err(ref e) if e.kind() == ErrorKind::AlreadyExists => (),
                Err(e) => return Err(e.into()),
            }

            n += 1;
        }
    }
}

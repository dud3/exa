use std::io::fs;
use std::io;

use colours::{Plain, Style, Black, Red, Green, Yellow, Blue, Purple, Cyan};
use column::{Column, Permissions, FileName, FileSize};
use format::{formatBinaryBytes, formatDecimalBytes};

// Each file is definitely going to get `stat`ted at least once, if
// only to determine what kind of file it is, so carry the `stat`
// result around with the file for safe keeping.
pub struct File<'a> {
    name: &'a str,
    path: &'a Path,
    stat: io::FileStat,
}

impl<'a> File<'a> {
    pub fn from_path(path: &'a Path) -> File<'a> {
        let filename: &str = path.filename_str().unwrap();

        // We have to use lstat here instad of file.stat(), as it
        // doesn't follow symbolic links. Otherwise, the stat() call
        // will fail if it encounters a link that's target is
        // non-existent.
        let stat: io::FileStat = match fs::lstat(path) {
            Ok(stat) => stat,
            Err(e) => fail!("Couldn't stat {}: {}", filename, e),
        };

        return File { path: path, stat: stat, name: filename };
    }

    pub fn display(&self, column: &Column) -> ~str {
        match *column {
            Permissions => self.permissions(),
            FileName => self.file_colour().paint(self.name.to_owned()),
            FileSize(si) => self.file_size(si),
        }
    }

    fn file_size(&self, si: bool) -> ~str {
        let sizeStr = if si {
            formatBinaryBytes(self.stat.size)
        } else {
            formatDecimalBytes(self.stat.size)
        };

        return if self.stat.kind == io::TypeDirectory {
            Green.normal()
        } else {
            Green.bold()
        }.paint(sizeStr);
    }

    fn type_char(&self) -> ~str {
        return match self.stat.kind {
            io::TypeFile => ~".",
            io::TypeDirectory => Blue.paint("d"),
            io::TypeNamedPipe => Yellow.paint("|"),
            io::TypeBlockSpecial => Purple.paint("s"),
            io::TypeSymlink => Cyan.paint("l"),
            _ => ~"?",
        }
    }


    fn file_colour(&self) -> Style {
        if self.stat.kind == io::TypeDirectory {
            Blue.normal()
        } else if self.stat.perm & io::UserExecute == io::UserExecute {
            Green.normal()
        } else if self.name.ends_with("~") {
            Black.bold()
        } else {
            Plain
        }
    }

    fn permissions(&self) -> ~str {
        let bits = self.stat.perm;
        return format!("{}{}{}{}{}{}{}{}{}{}",
            self.type_char(),
            bit(bits, io::UserRead, ~"r", Yellow.bold()),
            bit(bits, io::UserWrite, ~"w", Red.bold()),
            bit(bits, io::UserExecute, ~"x", Green.bold().underline()),
            bit(bits, io::GroupRead, ~"r", Yellow.normal()),
            bit(bits, io::GroupWrite, ~"w", Red.normal()),
            bit(bits, io::GroupExecute, ~"x", Green.normal()),
            bit(bits, io::OtherRead, ~"r", Yellow.normal()),
            bit(bits, io::OtherWrite, ~"w", Red.normal()),
            bit(bits, io::OtherExecute, ~"x", Green.normal()),
       );
    }
}

fn bit(bits: u32, bit: u32, other: ~str, style: Style) -> ~str {
    if bits & bit == bit {
        style.paint(other)
    } else {
        Black.bold().paint(~"-")
    }
}
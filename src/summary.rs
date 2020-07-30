/* * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * *
 *                                                                           *
 *  Copyright  (C)  2015-2018  Christian Krause                              *
 *                                                                           *
 *  Christian Krause  <christian.krause@mailbox.org>                         *
 *                                                                           *
 * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * *
 *                                                                           *
 *  This file is part of strace-analyzer.                                    *
 *                                                                           *
 *  strace-analyzer is free software: you can redistribute it and/or modify  *
 *  it under the terms of the GNU General Public License as published by     *
 *  the Free Software Foundation, either version 3 of the license, or any    *
 *  later version.                                                           *
 *                                                                           *
 *  strace-analyzer is distributed in the hope that it will be useful, but   *
 *  WITHOUT ANY WARRANTY; without even the implied warranty of               *
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU         *
 *  General Public License for more details.                                 *
 *                                                                           *
 *  You should have received a copy of the GNU General Public License along  *
 *  with strace-analyzer. If not, see <http://www.gnu.org/licenses/>.        *
 *                                                                           *
 * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * */

use config::Config;
use log::*;

use bytesize::ByteSize;
use std::collections::HashMap;

use std::fmt;

#[derive(Clone, Debug)]
pub struct FileDescription {
    pub path: String,
}

impl FileDescription {
    pub fn new(path: String) -> FileDescription {
        FileDescription { path }
    }
}

impl fmt::Display for FileDescription {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "FILE Path:{}", self.path)
    }
}

#[derive(Clone, Debug)]
pub struct SocketDescription {
    bind: String,
    connect: String,
}

impl SocketDescription {
    pub fn new() -> SocketDescription {
        SocketDescription {
            bind: String::new(),
            connect: String::new(),
        }
    }
}

impl fmt::Display for SocketDescription {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "SOCKET Bind:{} Connect:{}", self.bind, self.connect)
    }
}

#[derive(Clone, Debug)]
pub enum GenericFileDescriptor {
    File(FileDescription),
    Socket(SocketDescription),
    Pipe,
}

impl fmt::Display for GenericFileDescriptor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GenericFileDescriptor::File(file_description) => write!(f, "{}", file_description),
            GenericFileDescriptor::Socket(socket_description) => {
                write!(f, "{}", socket_description)
            }
            GenericFileDescriptor::Pipe => write!(f, "PIPE"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Summary {
    pub descriptor: GenericFileDescriptor,
    read_freq: HashMap<u64, u64>,
    write_freq: HashMap<u64, u64>,
    read_bytes: u64,
    write_bytes: u64,
}

impl Summary {
    pub fn new(descriptor: GenericFileDescriptor) -> Summary {
        Summary {
            descriptor,
            read_freq: HashMap::new(),
            write_freq: HashMap::new(),
            read_bytes: 0,
            write_bytes: 0,
        }
    }

    pub fn file(path: String) -> Summary {
        Summary::new(GenericFileDescriptor::File(FileDescription::new(path)))
    }

    pub fn pipe() -> Summary {
        Summary::new(GenericFileDescriptor::Pipe)
    }

    pub fn socket() -> Summary {
        Summary::new(GenericFileDescriptor::Socket(SocketDescription::new()))
    }

    pub fn reset(&mut self) {
        self.read_freq.clear();
        self.write_freq.clear();
        self.read_bytes = 0;
        self.write_bytes = 0;
    }

    pub fn update_read(&mut self, op_size: u64, bytes: u64) {
        let freq = self.read_freq.entry(op_size).or_insert(0);
        *freq += 1;
        self.read_bytes += bytes;
    }

    pub fn update_write(&mut self, op_size: u64, bytes: u64) {
        let freq = self.write_freq.entry(op_size).or_insert(0);
        *freq += 1;
        self.write_bytes += bytes;
    }

    pub fn show(&self, config: &Config) {
        if !config.verbose {
            if let GenericFileDescriptor::File(file_description) = &self.descriptor {
                if file_description.path.starts_with("/bin/")
                    || file_description.path == "/dev/null"
                    || file_description.path.starts_with("/etc/")
                    || file_description.path.starts_with("/lib/")
                    || file_description.path.starts_with("/lib64/")
                    || file_description.path.starts_with("/opt/")
                    || file_description.path.starts_with("/proc/")
                    || file_description.path.starts_with("/run/")
                    || file_description.path.starts_with("/sbin/")
                    || file_description.path.starts_with("/sys/")
                    || file_description.path.starts_with("/tmp/")
                    || file_description.path.starts_with("/usr/")
                    || file_description.path == "STDOUT"
                    || file_description.path == "STDERR"
                    || file_description.path == "STDIN"
                    || file_description.path == "DUP"
                {
                    return;
                }
            }

            if let GenericFileDescriptor::Pipe = self.descriptor {
                return;
            }
        }

        if self.read_freq.is_empty() && self.write_freq.is_empty() {
            debug(format!("no I/O with {}", self.descriptor), config);
            return;
        }

        if !self.read_freq.is_empty() {
            let (op_size, _) = self.read_freq.iter().max().unwrap();
            let n_ops: u64 = self.read_freq.values().sum();

            println!(
                "read {} with {} ops ({} / op) {}",
                humanize(self.read_bytes),
                n_ops,
                humanize(*op_size),
                self.descriptor,
            );
        }

        if !self.write_freq.is_empty() {
            let (op_size, _) = self.write_freq.iter().max().unwrap();
            let n_ops: u64 = self.write_freq.values().sum();

            println!(
                "write {} with {} ops ({} / op) {}",
                humanize(self.write_bytes),
                n_ops,
                humanize(*op_size),
                self.descriptor,
            );
        }
    }
}

fn humanize(bytes: u64) -> String {
    ByteSize(bytes)
        .to_string_as(true)
        .replace("iB", "")
        .replace(" ", "")
        .to_uppercase()
}

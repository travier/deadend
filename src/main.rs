#![forbid(unsafe_code)]

#[macro_use]
extern crate log;

use failure::{Fallible, ResultExt};
use log::LevelFilter;
use std::convert::TryInto;
use std::error::Error;
use std::fs;
use std::fs::Permissions;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use structopt::StructOpt;
use zbus::{dbus_interface, fdo};

#[derive(StructOpt, Debug)]
#[structopt(name = "deadend", about = "FCOS Dead End MOTD")]
struct Opt {
    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[structopt(short = "v", long = "verbose", parse(from_occurrences))]
    verbose: u8,

    /// Enable testing under an unprivileged user with the user session bus
    #[structopt(
        short = "u",
        long = "user",
    )]
    user: bool,
}


struct FcosDeadEnd {
    motd_path: String
}

fn write_motd(reason: &str, path: &str) -> Fallible<()> {
    let mut f = tempfile::Builder::new()
    .prefix(".deadend.")
    .suffix(".motd.partial")
    .tempfile_in(path)
    .with_context(|e| format!("failed to create temporary MOTD file: {}", e))?;

    // Set correct permissions of the temporary file, before moving to
    // the destination (`tempfile` creates files with mode 0600).
    fs::set_permissions(f.path(), Permissions::from_mode(0o664))
        .with_context(|e| format!("failed to set permissions of temporary MOTD file: {}", e))?;

    if reason != "" {
        writeln!(
            f,
            "This release is a dead-end and won't auto-update: {}",
            reason
        )
        .with_context(|e| format!("failed to write MOTD: {}", e))?;
    } else {
        writeln!(
            f,
            "This release is a dead-end and won't auto-update."
        )
        .with_context(|e| format!("failed to write MOTD: {}", e))?;
    }

    f.persist(format!("{}/deadend.motd", path))
        .with_context(|e| format!("failed to persist temporary MOTD file: {}", e))?;

    Ok(())
}

#[dbus_interface(name = "org.coreos.FcosDeadEnd1")]
impl FcosDeadEnd {
    fn write_deadend_reason(&self, reason: &str) -> bool {
        info!("Writing MOTD with reason: {}", reason);
        match write_motd(reason, &self.motd_path) {
            Err(e) => {
                error!("Failed to write MOTD: {}", e);
                false
            },
            Ok(()) => {
                info!("Successfully wrote MOTD");
                true
            }
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let opt = Opt::from_args();

    let level = match opt.verbose {
        0 => LevelFilter::Info,
        1 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    };

    env_logger::Builder::new()
        .filter(None, level)
        .format_timestamp(None)
        .init();

    let connection = if opt.user {
        zbus::Connection::new_session()?
    } else {
        zbus::Connection::new_system()?
    };
    fdo::DBusProxy::new(&connection)?.request_name(
        "org.coreos.FcosDeadEnd",
        fdo::RequestNameFlags::ReplaceExisting.into(),
    )?;

    let mut object_server = zbus::ObjectServer::new(&connection);
    let path = if opt.user {
        String::from("/tmp")
    } else {
        String::from("/run/motd.d")
    };
    let deadend = FcosDeadEnd { motd_path: path };
    object_server.at(&"/org/coreos/FcosDeadEnd".try_into()?, deadend)?;
    loop {
        match object_server.try_handle_next() {
            Ok(None) => {
                return Ok(())
            },
            Ok(Some(message)) => {
                info!("Received: {}", message);
            },
            Err(err) => {
                error!("Error: {}", err);
                return Err(err.into())
            }
        }
    }
}


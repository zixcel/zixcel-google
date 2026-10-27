#![forbid(unsafe_code)]
use std::{
    io::{self},
    process::ExitCode,
};
use zixcel_google_cli::protocol;
use zixcel_google_core::{Error, control, lifecycle};
fn main() -> ExitCode {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args == ["--manifest"] || args == ["--guidance"] || args == ["--zix-provider-v2"] {
        return zixcel_google_cli::guided::entry();
    }
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{{\"error\":\"{}\"}}", error.code());
            ExitCode::from(2)
        }
    }
}
fn run() -> Result<(), Error> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args == ["--manifest"] {
        println!("{}", protocol::manifest());
        return Ok(());
    }
    if args == ["--control-manifest"] {
        println!("{}", control::catalog());
        return Ok(());
    }
    if args == ["--lifecycle-manifest"] {
        println!("{}", lifecycle::catalog());
        return Ok(());
    }
    if args == ["--prepare-lifecycle-v1"] || args == ["--reconcile-lifecycle-v1"] {
        let bytes =
            zixcel_bounded_io::read_bounded(io::stdin(), 65536).map_err(|_| Error::Protocol)?;
        let value = if args == ["--prepare-lifecycle-v1"] {
            serde_json::to_value(lifecycle::prepare(
                serde_json::from_slice(&bytes).map_err(|_| Error::Protocol)?,
            )?)
        } else {
            serde_json::to_value(lifecycle::reconcile(
                serde_json::from_slice(&bytes).map_err(|_| Error::Protocol)?,
            )?)
        }
        .map_err(|_| Error::Protocol)?;
        println!("{value}");
        return Ok(());
    }
    if args == ["--prepare-control-v1"] {
        let bytes =
            zixcel_bounded_io::read_bounded(io::stdin(), 65536).map_err(|_| Error::Protocol)?;
        let input: control::Request =
            serde_json::from_slice(&bytes).map_err(|_| Error::Protocol)?;
        let output = control::prepare(input)?;
        println!(
            "{}",
            serde_json::to_string(&output).map_err(|_| Error::Protocol)?
        );
        return Ok(());
    }
    if args != ["--zix-provider-v1"] {
        return Err(Error::Protocol);
    }
    let bytes = zixcel_bounded_io::read_bounded(io::stdin(), 65536).map_err(|_| Error::Protocol)?;
    let request: protocol::Request = serde_json::from_slice(&bytes).map_err(|_| Error::Protocol)?;
    request.validate()?;
    // A failed IAM query is a typed observation failure, never a fabricated revoked state.
    println!(
        "{}",
        serde_json::to_string(&protocol::execute(&request)).map_err(|_| Error::Protocol)?
    );
    Ok(())
}

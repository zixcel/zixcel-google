#![forbid(unsafe_code)]
use std::{
    io::{self},
    process::ExitCode,
};
use zixcel_google_admin::{Error, protocol};
fn main() -> ExitCode {
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

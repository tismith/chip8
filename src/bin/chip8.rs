//standard includes
extern crate chip8_tismith;
use chip8_tismith::*;

fn main() -> Result<(), exitfailure::ExitFailure> {
    let mut config = utils::cmdline::parse_cmdline();
    config.module_path = Some(module_path!().into());
    utils::logging::configure_logger(&config)?;
    Ok(())
}

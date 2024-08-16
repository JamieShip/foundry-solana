use eyre::EyreHandler;

use std::error::Error;
use yansi::Paint;

#[derive(Debug)]
pub struct Handler;

impl EyreHandler for Handler {
    fn debug(
        &self,
        error: &(dyn Error + 'static),
        f: &mut core::fmt::Formatter<'_>,
    ) -> core::fmt::Result {
        if f.alternate() {
            return core::fmt::Debug::fmt(error, f);
        }
        writeln!(f)?;
        write!(f, "{}", Paint::red(error))?;

        if let Some(cause) = error.source() {
            write!(f, "\n\nContext:")?;

            let multiple = cause.source().is_some();
            let errors = std::iter::successors(Some(cause), |e| (*e).source());

            for (n, error) in errors.enumerate() {
                writeln!(f)?;
                if multiple {
                    write!(f, "- Error #{n}: {error}")?;
                } else {
                    write!(f, "- {error}")?;
                }
            }
        }
        Ok(())
    }
}

pub fn install() {
    let (panic_hook, _) = color_eyre::config::HookBuilder::default()
        .panic_section(
            "This is a bug. Consider reporting it at https://github.com/JamieShip/foundry-solana",
        )
        .into_hooks();
    panic_hook.install();
    if let Err(e) = eyre::set_hook(Box::new(move |_| Box::new(Handler))) {
        debug!("failed to install eyre error hook: {e}");
    }
}

use super::*;
use crate::interactive::{commands::util::expect_u32, error::CommandError};
use colored::*;
use rush_lib::KTEXT_BOT;
use rush_lib::TEXT_BOT;

#[allow(unreachable_code)]
pub(crate) fn context_command() -> Command {
    command(
        "context",
        vec!["c", "ctx"],
        vec![],
        vec!["n"],
        vec![],
        &format!(
            "prints the current and surrounding 3 (or {}) instructions",
            "[n]".magenta(),
        ),
        |_, state, label, args| {
            if label == "__help__" {
                return Ok(format!(
                    "prints the current and surrounding 3 (or {}) instructions",
                    "[n]".magenta(),
                ));
            }

            let f: Option<&dyn Fn(i32) -> String> = None;

            let n = match args.first() {
                Some(arg) => expect_u32(label, &"[n]".bright_magenta(), arg, f),
                None => Ok(3),
            }? as i32;

            if state.exited {
                return Err(CommandError::ProgramExited);
            }

            let binary = state.binary.as_ref().ok_or(CommandError::MustLoadFile)?;
            let runtime = state.runtime.as_ref().unwrap();

            let base_addr = runtime.state().pc();
            for i in (-n)..=n {
                let addr = {
                    let addr = base_addr.wrapping_add((i * 4) as u32);
                    if addr < TEXT_BOT {
                        continue;
                    }

                    if addr < KTEXT_BOT && addr >= (TEXT_BOT + binary.text.len() as u32) {
                        continue;
                    }

                    addr
                };

                let _inst = {
                    if let Ok(inst) = runtime.state().read_mem_word(addr) {
                        inst
                    } else {
                        continue;
                    }
                };

                // let parts = decompile::decompile_inst_into_parts(binary, &state.iset, inst, addr);
                // util::print_inst_parts(binary, &Ok(parts), Some(program), i == 0);
            }

            println!();
            Ok("".into())
        },
    )
}

//! ℵ-OS λ_ℵ shell commands — integrates the ALEPH REPL into the vOS interactive shell.
//!
//! Adds ALEPH-specific commands to the existing shell:
//!   `aleph`     — enter the ALEPH REPL
//!   `aleph <expr>` — evaluate a single ALEPH expression

extern crate alloc;

use alloc::format;

use crate::vga::{self, Color, WRITER};
use crate::aleph;
use crate::aleph_parser;
use crate::aleph_eval::{self, Evaluator, EvalResult};
use crate::aleph_repl::AlephRepl;

/// Handle an `aleph` command from the shell.
/// 
/// - `aleph` (no args) → enter interactive REPL
/// - `aleph <expr>` → evaluate single expression and return
/// 
/// Returns `true` if the user typed `:quit` in the REPL (shell should continue),
/// or `false` if there was an error.
pub fn handle_aleph(args: &str) -> bool {
    let args = args.trim();

    if args.is_empty() {
        // Enter interactive REPL
        let mut repl = AlephRepl::new();
        repl.run();
        return true;
    }

    // Single expression mode
    let mut eval = Evaluator::new();
    match aleph_parser::parse(args) {
        Ok(ast) => {
            let ast_clone = ast.clone();
            match eval.eval(&ast) {
                Ok(result) => {
                    let mut w = WRITER.lock();
                    w.color_code = vga::ColorCode::new(Color::White, Color::Black);
                    match result {
                        EvalResult::Letter(l) => {
                            match ast_clone {
                                aleph_parser::Expr::ProbePhi(_) => {
                                    w.write_string(aleph_eval::format_probe_phi(&l.t).as_str());
                                    w.write_string("\n");
                                }
                                aleph_parser::Expr::ProbeOmega(_) => {
                                    w.write_string(aleph_eval::format_probe_omega(&l.t).as_str());
                                    w.write_string("\n");
                                }
                                aleph_parser::Expr::Tier(_) => {
                                    w.write_string(aleph_eval::format_tier(&l.t).as_str());
                                    w.write_string("\n");
                                }
                                _ => {}
                            }
                            w.write_string(aleph::format_letter(l).as_str());
                            w.write_string("\n");
                        }
                        EvalResult::Unit => {
                            w.write_string(aleph_eval::format_census().as_str());
                        }
                        EvalResult::Distance(d, cs) => {
                            let vc = aleph::veracity_class(d);
                            w.write_string(&format!("  d = {:.4}  [{}]\n", d, vc));
                            if !cs.is_empty() {
                                w.write_string("  conflict_set: {");
                                for (i, &idx) in cs.iter().enumerate() {
                                    if i > 0 { w.write_string(", "); }
                                    w.write_string(aleph::PRIM_NAMES[idx]);
                                }
                                w.write_string("}\n");
                            }
                            if d > 2.4495 {
                                w.color_code = vga::ColorCode::new(Color::Yellow, Color::Black);
                                w.write_string("  !! aspirational gap\n");
                            }
                        }
                    }
                    true
                }
                Err(e) => {
                    let mut w = WRITER.lock();
                    w.color_code = vga::ColorCode::new(Color::LightRed, Color::Black);
                    w.write_string(&format!("[ERROR] {}\n", e));
                    false
                }
            }
        }
        Err(e) => {
            let mut w = WRITER.lock();
            w.color_code = vga::ColorCode::new(Color::LightRed, Color::Black);
            w.write_string(&format!("[PARSE] {}\n", e));
            false
        }
    }
}

/// Print ALEPH-specific help (to be appended to the main help command).
pub fn print_aleph_help() {
    let mut w = WRITER.lock();
    w.color_code = vga::ColorCode::new(Color::White, Color::Black);
    w.write_string("\n");
    w.color_code = vga::ColorCode::new(Color::LightCyan, Color::Black);
    w.write_string("--- ALEPH Commands -------------------------------------------------\n");
    w.color_code = vga::ColorCode::new(Color::White, Color::Black);
    w.write_string("  aleph                   enter the ALEPH REPL\n");
    w.write_string("  aleph <expr>            evaluate a single expression\n");
    w.write_string("                          e.g. aleph mem x shin\n");
    w.write_string("\n");
    w.write_string("  In the ALEPH REPL:\n");
    w.write_string("    :help                 show ALEPH syntax reference\n");
    w.write_string("    :tips                 quick start examples\n");
    w.write_string("    :quit / :q            return to main shell\n");
    w.write_string("    :ls                   list session bindings\n");
    w.write_string("    :clear                clear screen\n");
    w.write_string("    :census               tier distribution\n");
    w.write_string("    :system               22-letter language JOIN\n");
    w.write_string("\n");
}

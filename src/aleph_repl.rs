//! ℵ-OS λ_ℵ interactive REPL for the bare-metal kernel.
//!
//! Reads from the PS/2 keyboard ring buffer, evaluates ALEPH expressions,
//! and prints results to VGA. Manages session state (let bindings).

extern crate alloc;

use alloc::string::String;
use alloc::format;
use alloc::vec::Vec;
use alloc::string::ToString;

use crate::vga::{self, Color, WRITER};
use crate::keyboard;
use crate::aleph;
use crate::aleph_parser;
use crate::aleph_eval::{self, Evaluator, EvalResult};

// ── REPL state ────────────────────────────────────────────────────────────────

pub struct AlephRepl {
    eval: Evaluator,
    input_buf: String,
    brace_depth: usize,
    history: Vec<String>,
    last_result: Option<aleph::Tuple>,  // Last evaluated letter tuple
}

impl AlephRepl {
    pub fn new() -> Self {
        AlephRepl {
            eval: Evaluator::new(),
            input_buf: String::new(),
            brace_depth: 0,
            history: Vec::new(),
            last_result: None,
        }
    }

    /// Print the REPL banner (ASCII-safe for VGA text mode).
    /// All lines ≤ 80 chars to avoid VGA wrap.
    pub fn print_banner(&self) {
        let mut w = WRITER.lock();
        w.color_code = vga::ColorCode::new(Color::Cyan, Color::Black);
        w.write_string("+======================================================================+\n");
        w.color_code = vga::ColorCode::new(Color::White, Color::Black);

        // Line 1: |  ALEPH...[v0.5.0 native]                          |
        // Content: 3 + 30 + 15 = 48, need 31 pad to reach 79 + | = 80
        w.write_string("|  ");
        w.color_code = vga::ColorCode::new(Color::LightCyan, Color::Black);
        w.write_string("ALEPH - Hebrew Type Language  ");
        w.color_code = vga::ColorCode::new(Color::Yellow, Color::Black);
        w.write_string("[v0.5.0 native]");
        w.color_code = vga::ColorCode::new(Color::White, Color::Black);
        for _ in 0..31 { w.write_byte(b' '); }
        w.color_code = vga::ColorCode::new(Color::Cyan, Color::Black);
        w.write_string("|\n");

        // Line 2
        w.write_string("|  ");
        w.color_code = vga::ColorCode::new(Color::White, Color::Black);
        w.write_string("Grammar: SynthOmnicon 12-primitive v0.4.27");
        for _ in 0..25 { w.write_byte(b' '); }
        w.color_code = vga::ColorCode::new(Color::Cyan, Color::Black);
        w.write_string("|\n");

        // Line 3
        w.write_string("|  ");
        w.color_code = vga::ColorCode::new(Color::White, Color::Black);
        w.write_string("Type :help for commands, :quit to exit");
        for _ in 0..27 { w.write_byte(b' '); }
        w.color_code = vga::ColorCode::new(Color::Cyan, Color::Black);
        w.write_string("|\n");

        w.write_string("+======================================================================+\n");
        w.write_string("\n");
        w.color_code = vga::ColorCode::new(Color::White, Color::Black);
        w.write_string("  Welcome! Type ");
        w.color_code = vga::ColorCode::new(Color::Yellow, Color::Black);
        w.write_string(":help");
        w.color_code = vga::ColorCode::new(Color::White, Color::Black);
        w.write_string(" for commands, ");
        w.color_code = vga::ColorCode::new(Color::Yellow, Color::Black);
        w.write_string(":tips");
        w.color_code = vga::ColorCode::new(Color::White, Color::Black);
        w.write_string(" for examples.\n\n");
    }

    /// Print help text.
    fn print_help(&self) {
        let mut w = WRITER.lock();
        w.color_code = vga::ColorCode::new(Color::White, Color::Black);
        w.write_string("\n");
        w.write_string("--- Operations -------------------------------------------------------\n");
        w.write_string("  a x b                   tensor  (P, F bottleneck)\n");
        w.write_string("  a v b                   join    (LUB, no bottleneck)\n");
        w.write_string("  a ^ b                   meet    (GLB)\n");
        w.write_string("  a ::> b                 vav-cast  a to type  b\n");
        w.write_string("  mediate(w, a, b)        triadic: w v (a x b)\n");
        w.write_string("  probe_Phi(a)            report criticality primitive\n");
        w.write_string("  probe_Omega(a)          report topological protection\n");
        w.write_string("  tier(a)                 report ouroboricity tier\n");
        w.write_string("  d(a, b)                 structural distance + conflict set\n");
        w.write_string("  match a { O_0=>x, ...}  tier pattern match\n\n");
        w.write_string("--- Built-ins ------------------------------------------------------\n");
        w.write_string("  system()                JOIN of all 22 letters\n");
        w.write_string("  census()                tier distribution\n\n");
        w.write_string("--- Barriers -------------------------------------------------------\n");
        w.write_string("  palace(n) expr          tier barrier (n=1..7)\n\n");
        w.write_string("--- Session bindings ------------------------------------------------\n");
        w.write_string("  let x = expr            bind result in this session\n\n");
        w.write_string("--- Commands -------------------------------------------------------\n");
        w.write_string("  :help                   this text\n");
        w.write_string("  :tips                   quick start tips\n");
        w.write_string("  :quit  / :q             exit\n");
        w.write_string("  :census                 tier distribution (alias)\n");
        w.write_string("  :system                 22-letter language JOIN\n");
        w.write_string("  :tier <name>            type of a single letter\n");
        w.write_string("  :tuple <name>           full 12-primitive tuple (visual)\n");
        w.write_string("  :explain <name>         detailed type breakdown + C score\n");
        w.write_string("  :ls                     list session bindings\n");
        w.write_string("  :history                show command history\n");
        w.write_string("  :clear                  clear screen\n");
        w.write_string("  :files                  list ALFS files\n");
        w.write_string("  :save name              save last result as name.aleph\n");
        w.write_string("  :save name expr         save expression as name.aleph\n");
        w.write_string("  :load name              load and bind an .aleph file\n");
        w.write_string("  :run name               run an .aleph file\n\n");
        w.write_string("--- Letter names (transliteration or Hebrew glyph) -------------------\n");
        w.write_string("  aleph [A]   bet [B]   gimel [G]   dalet [D]   hei [H]   vav [V]\n");
        w.write_string("  zayin [Z]  chet [C]  tet [T]     yod [Y]     kaf [K]   lamed [L]\n");
        w.write_string("  mem [M]    nun [N]   samech [S]  ayin [E]    pei [P]   tzadi [Q]\n");
        w.write_string("  kuf [U]    resh [R]  shin [X]    tav [O]\n\n");
    }

    /// Print tips.
    fn print_tips(&self) {
        let mut w = WRITER.lock();
        w.color_code = vga::ColorCode::new(Color::White, Color::Black);
        w.write_string("\n");
        w.color_code = vga::ColorCode::new(Color::Yellow, Color::Black);
        w.write_string("--- Quick Start Tips -----------------------------------------------\n\n");
        w.color_code = vga::ColorCode::new(Color::Yellow, Color::Black);
        w.write_string("  Try these examples:\n");
        w.color_code = vga::ColorCode::new(Color::White, Color::Black);
        w.write_string("    aleph x shin         - Tensor two letters\n");
        w.write_string("    mem v vav            - Join operation\n");
        w.write_string("    d(aleph, bet)        - Structural distance\n");
        w.write_string("    tier(shin)           - Check ouroboricity tier\n");
        w.write_string("    probe_Phi(mem)       - Check criticality\n");
        w.write_string("    let x = aleph x mem  - Bind to variable\n");
        w.write_string("    :tuple shin          - Visual 12-primitive bars\n");
        w.write_string("    :ls                  - List session bindings\n\n");
    }

    /// Print census.
    fn print_census(&self) {
        let mut w = WRITER.lock();
        w.color_code = vga::ColorCode::new(Color::White, Color::Black);
        w.write_string(aleph_eval::format_census().as_str());
    }

    /// Print session bindings (:ls).
    fn print_bindings(&self) {
        let mut w = WRITER.lock();
        w.color_code = vga::ColorCode::new(Color::White, Color::Black);
        let env = self.eval.env();
        if env.is_empty() {
            w.write_string("  (no bindings)\n");
            return;
        }
        w.write_string("  Name              Tier       Phi              Omega             P\n");
        w.write_string("  -------------------------------------------------------------------\n");
        for (name, l) in env {
            let tier = aleph::compute_tier(&l.t);
            let phi_n = aleph::PHI_NAMES.get(l.t[8] as usize).copied().unwrap_or("?");
            let om_n = aleph::OMEGA_NAMES.get(l.t[11] as usize).copied().unwrap_or("?");
            // Manual padding since no_std doesn't support {:18s}
            let name_padded = format!("{:<18}", name);
            let tier_padded = format!("{:<9}", aleph::tier_name(tier));
            let phi_padded = format!("{:<10}", phi_n);
            let om_padded = format!("{:<10}", om_n);
            w.write_string(&format!("  {} {} {} {} {}\n",
                name_padded, tier_padded, phi_padded, om_padded, aleph::display_glyph(l)));
        }
    }

    /// List files on the ALFS filesystem.
    fn print_files(&self) {
        let mut w = WRITER.lock();
        w.color_code = vga::ColorCode::new(Color::White, Color::Black);
        if !crate::alfs::is_mounted() {
            w.write_string("  [ERROR] Filesystem not mounted\n");
            return;
        }
        let files = crate::alfs::list();
        if files.is_empty() {
            w.write_string("  (no files)\n");
            return;
        }
        w.write_string("  File              Type        Size\n");
        w.write_string("  ----------------------------------------\n");
        for f in &files {
            let type_str = if f.file_type == crate::alfs::TYPE_ALEPH {
                "aleph"
            } else {
                "data"
            };
            let size = f.sector_count * 512;
            w.write_string(&format!("  {:<18} {:<10} {} bytes\n", f.name, type_str, size));
        }
    }

    /// Print command history (:history).
    fn print_history(&self) {
        let mut w = WRITER.lock();
        w.color_code = vga::ColorCode::new(Color::White, Color::Black);
        if self.history.is_empty() {
            w.write_string("  (no history)\n");
            return;
        }
        for (i, cmd) in self.history.iter().enumerate() {
            w.write_string(&format!("  {:>4}  {}\n", i + 1, cmd));
        }
    }

    /// Print detailed type explanation (:explain).
    fn print_explain(&self, name: &str) {
        if let Some(l) = aleph::resolve_letter(name) {
            let mut w = WRITER.lock();
            w.color_code = vga::ColorCode::new(Color::White, Color::Black);
            w.write_string(aleph::format_explain(l).as_str());
        } else {
            let mut w = WRITER.lock();
            w.color_code = vga::ColorCode::new(Color::LightRed, Color::Black);
            w.write_string(&format!("  [ERROR] Unknown letter: '{}'\n", name));
        }
    }

    /// Load an .aleph file and bind the result to the file's basename.
    fn load_and_bind(&mut self, name: &str) {
        let name = name.trim_end_matches(".aleph");
        let filename = format!("{}.aleph", name);

        let source = match crate::alfs::read_file_string(&filename) {
            Some(s) => s,
            None => {
                let mut w = WRITER.lock();
                w.color_code = vga::ColorCode::new(Color::LightRed, Color::Black);
                w.write_string(&format!("  [ERROR] Cannot read '{}'\n", filename));
                return;
            }
        };

        // Evaluate each line
        for line in source.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') || line.starts_with(':') {
                continue;
            }
            let parts = split_expressions(line);
            for expr_str in parts {
                match aleph_parser::parse(&expr_str) {
                    Ok(ast) => {
                        match self.eval.eval(&ast) {
                            Ok(aleph_eval::EvalResult::Letter(l)) => {
                                // Bind to variable name (file basename)
                                if expr_str.trim().starts_with("let ") {
                                    // Already a let binding
                                } else {
                                    // Auto-bind the result to the file basename
                                    self.eval.bind(name, l.t);
                                }
                            }
                            Ok(_) => {}
                            Err(e) => {
                                let mut w = WRITER.lock();
                                w.color_code = vga::ColorCode::new(Color::LightRed, Color::Black);
                                w.write_string(&format!("  [{}] {}\n", name, e));
                            }
                        }
                    }
                    Err(e) => {
                        let mut w = WRITER.lock();
                        w.color_code = vga::ColorCode::new(Color::LightRed, Color::Black);
                        w.write_string(&format!("  [{}] {}\n", name, e));
                    }
                }
            }
        }
    }

    /// Run an .aleph file — evaluate each line without binding.
    fn run_file(&mut self, name: &str) {
        let name = name.trim_end_matches(".aleph");
        let filename = format!("{}.aleph", name);

        let source = match crate::alfs::read_file_string(&filename) {
            Some(s) => s,
            None => {
                let mut w = WRITER.lock();
                w.color_code = vga::ColorCode::new(Color::LightRed, Color::Black);
                w.write_string(&format!("  [ERROR] Cannot read '{}'\n", filename));
                return;
            }
        };

        let mut w = WRITER.lock();
        w.color_code = vga::ColorCode::new(Color::White, Color::Black);
        w.write_string(&format!("  --- running {} ---\n", filename));

        for line in source.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') || line.starts_with(':') {
                continue;
            }
            let parts = split_expressions(line);
            for expr_str in &parts {
                match aleph_parser::parse(expr_str) {
                    Ok(ast) => {
                        match self.eval.eval(&ast) {
                            Ok(aleph_eval::EvalResult::Letter(l)) => {
                                w.write_string(aleph::format_letter(l).as_str());
                            }
                            Ok(aleph_eval::EvalResult::Distance(d, ref cs)) => {
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
                            }
                            Ok(_) => {}
                            Err(e) => {
                                w.color_code = vga::ColorCode::new(Color::LightRed, Color::Black);
                                w.write_string(&format!("  [{}] {}\n", name, e));
                                w.color_code = vga::ColorCode::new(Color::White, Color::Black);
                            }
                        }
                    }
                    Err(e) => {
                        w.color_code = vga::ColorCode::new(Color::LightRed, Color::Black);
                        w.write_string(&format!("  [{}] {}\n", name, e));
                        w.color_code = vga::ColorCode::new(Color::White, Color::Black);
                    }
                }
            }
        }
    }

    /// Save an expression to the Sefirot filesystem as an .aleph file.
    /// Usage: :save name       — saves last result
    ///        :save name expr  — saves the expression text
    fn save_file(&mut self, args: &str) {
        // Parse args: "name" or "name expr..."
        let args = args.trim();
        if args.is_empty() {
            let mut w = WRITER.lock();
            w.color_code = vga::ColorCode::new(Color::LightRed, Color::Black);
            w.write_string("  Usage: :save name [expression]\n");
            return;
        }

        let (name, content) = if let Some(space_pos) = args.find(' ') {
            // :save name expr...
            let name = &args[..space_pos];
            let expr = &args[space_pos+1..];
            (name.to_string(), expr.to_string())
        } else {
            // :save name — use last result
            let name = args.to_string();
            if let Some(ref t) = self.last_result {
                // Generate ALEPH source that would produce this tuple
                let best = crate::aleph_repl::AlephRepl::find_best(t);
                let content = format!("# Saved: {}\n{}\n", best.name, best.name);
                (name, content)
            } else {
                let mut w = WRITER.lock();
                w.color_code = vga::ColorCode::new(Color::LightRed, Color::Black);
                w.write_string("  [ERROR] No last result to save. Use :save name expr instead.\n");
                return;
            }
        };

        let filename = if name.ends_with(".aleph") { name.clone() } else { format!("{}.aleph", name) };

        // Write to Sefirot filesystem
        let fs = crate::filesystem::fs();
        let sefirah_before = fs.current();
        // Navigate to Chokhmah for .aleph files
        fs.navigate_to(crate::filesystem::Sefirah::Chokhmah);
        fs.write(&filename, content.as_bytes());
        fs.navigate_to(sefirah_before);

        let mut w = WRITER.lock();
        w.color_code = vga::ColorCode::new(Color::White, Color::Black);
        w.write_string(&format!("  Saved '{}' ({} bytes) in Chokhmah (/sys/bin)\n", filename, content.len()));
    }

    /// Find the best-matching letter for a tuple (used for :system).
    fn find_best(t: &aleph::Tuple) -> &'static aleph::LetterDef {
        let mut best_dist = u32::MAX;
        let mut best_idx = 0usize;
        for (i, letter) in aleph::LETTERS.iter().enumerate() {
            let d = aleph::distance_scaled(&letter.t, t);
            if d < best_dist {
                best_dist = d;
                best_idx = i;
            }
        }
        &aleph::LETTERS[best_idx]
    }

    /// Handle a complete input line.
    fn handle_line(&mut self, src: &str) {
        let src = src.trim();
        if src.is_empty() {
            return;
        }

        // Save to history
        self.history.push(src.to_string());

        // REPL commands
        if src == ":help" {
            self.print_help();
            return;
        }
        if src == ":tips" {
            self.print_tips();
            return;
        }
        if src == ":quit" || src == ":q" {
            let mut w = WRITER.lock();
            w.color_code = vga::ColorCode::new(Color::LightCyan, Color::Black);
            w.write_string("\n  Shalom!\n\n");
            return;
        }
        if src == ":census" {
            self.print_census();
            return;
        }
        if src == ":system" {
            let t = aleph::system_language();
            let matched = Self::find_best(&t);
            let mut w = WRITER.lock();
            w.color_code = vga::ColorCode::new(Color::White, Color::Black);
            w.write_string(aleph::format_letter(matched).as_str());
            return;
        }
        if src.starts_with(":tier ") {
            let name = src[6..].trim();
            if let Some(l) = aleph::resolve_letter(name) {
                let tier = aleph::compute_tier(&l.t);
                let mut w = WRITER.lock();
                w.color_code = vga::ColorCode::new(Color::White, Color::Black);
                w.write_string(&format!("  {} ({}) -> {}\n", aleph::display_glyph(l), l.name, aleph::tier_name(tier)));
            } else {
                let mut w = WRITER.lock();
                w.color_code = vga::ColorCode::new(Color::LightRed, Color::Black);
                w.write_string(&format!("  [ERROR] Unknown letter: '{}'\n", name));
            }
            return;
        }
        if src.starts_with(":tuple ") {
            let name = src[7..].trim();
            if let Some(l) = aleph::resolve_letter(name) {
                let mut w = WRITER.lock();
                w.color_code = vga::ColorCode::new(Color::White, Color::Black);
                w.write_string(aleph::format_tuple(l).as_str());
            } else {
                let mut w = WRITER.lock();
                w.color_code = vga::ColorCode::new(Color::LightRed, Color::Black);
                w.write_string(&format!("  [ERROR] Unknown letter: '{}'\n", name));
            }
            return;
        }
        if src.starts_with(":explain ") {
            let name = src[9..].trim();
            self.print_explain(name);
            return;
        }
        if src == ":ls" {
            self.print_bindings();
            return;
        }
        if src == ":history" {
            self.print_history();
            return;
        }
        if src == ":clear" {
            // Clear screen by printing enough newlines
            let mut w = WRITER.lock();
            for _ in 0..25 {
                w.write_string("\n");
            }
            return;
        }
        if src == ":files" {
            self.print_files();
            return;
        }
        if src.starts_with(":save ") {
            let rest = src[6..].trim();
            self.save_file(rest);
            return;
        }
        if src.starts_with(":load ") {
            let name = src[6..].trim();
            self.load_and_bind(name);
            return;
        }
        if src.starts_with(":run ") {
            let name = src[5..].trim();
            self.run_file(name);
            return;
        }

        // Parse and evaluate
        match aleph_parser::parse(src) {
            Ok(ast) => {
                let ast_clone = ast.clone();
                match self.eval.eval(&ast) {
                    Ok(result) => {
                        let mut w = WRITER.lock();
                        w.color_code = vga::ColorCode::new(Color::White, Color::Black);
                        match result {
                            EvalResult::Letter(l) => {
                                // Track last result for :save
                                self.last_result = Some(l.t);
                                // Check if the AST was a probe or tier
                                match ast_clone {
                                    aleph_parser::Expr::ProbePhi(_) => {
                                        w.write_string(aleph_eval::format_probe_phi(&l.t).as_str());
                                    }
                                    aleph_parser::Expr::ProbeOmega(_) => {
                                        w.write_string(aleph_eval::format_probe_omega(&l.t).as_str());
                                    }
                                    aleph_parser::Expr::Tier(_) => {
                                        w.write_string(aleph_eval::format_tier(&l.t).as_str());
                                    }
                                    _ => {}
                                }
                                w.write_string(aleph::format_letter(l).as_str());
                            }
                            EvalResult::Unit => {}
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
                                    w.color_code = vga::ColorCode::new(Color::White, Color::Black);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        let mut w = WRITER.lock();
                        w.color_code = vga::ColorCode::new(Color::LightRed, Color::Black);
                        w.write_string(&format!("  [ERROR] {}\n", e));
                    }
                }
            }
            Err(e) => {
                let mut w = WRITER.lock();
                w.color_code = vga::ColorCode::new(Color::LightRed, Color::Black);
                w.write_string(&format!("  [PARSE] {}\n", e));
            }
        }
    }

    /// Read a single character from the keyboard or serial, blocking with hlt.
    fn read_char_blocking(&self) -> u8 {
        loop {
            x86_64::instructions::interrupts::disable();
            if let Some(b) = keyboard::pop() {
                x86_64::instructions::interrupts::enable();
                return b;
            }
            if let Some(b) = crate::serial::try_read() {
                x86_64::instructions::interrupts::enable();
                return b;
            }
            // Nothing available — wait for next interrupt (timer or keyboard)
            x86_64::instructions::interrupts::enable_and_hlt();
        }
    }

    /// Run the REPL main loop. Reads from keyboard, evaluates, prints to VGA.
    /// Returns when the user types :quit or :q.
    pub fn run(&mut self) -> bool {
        self.print_banner();

        loop {
            // Print prompt (ASCII-safe)
            let prompt = if self.brace_depth > 0 {
                ">  "
            } else {
                "A> "
            };
            {
                let mut w = WRITER.lock();
                w.color_code = vga::ColorCode::new(Color::LightCyan, Color::Black);
                w.write_string(prompt);
                w.color_code = vga::ColorCode::new(Color::White, Color::Black);
            }

            // Read characters until Enter
            let mut line = String::new();
            loop {
                let ch = self.read_char_blocking();

                // Enter
                if ch == b'\n' || ch == b'\r' {
                    let mut w = WRITER.lock();
                    w.write_string("\n");
                    break;
                }

                // Backspace
                if ch == 0x08 || ch == 0x7f {
                    if !line.is_empty() {
                        let removed = line.pop().unwrap();
                        // Update brace depth
                        if removed == '{' {
                            self.brace_depth = self.brace_depth.saturating_sub(1);
                        } else if removed == '}' {
                            self.brace_depth += 1;
                        }
                        // Erase character on screen (VGA) and serial terminal
                        crate::serial::write_byte(0x08);
                        crate::serial::write_byte(b' ');
                        crate::serial::write_byte(0x08);
                        let mut w = WRITER.lock();
                        w.backspace();
                    }
                    continue;
                }

                // Skip non-printable (except tab)
                if ch < 0x20 && ch != b'\t' {
                    continue;
                }

                // Echo to VGA and serial
                crate::serial::write_byte(ch);
                {
                    let mut w = WRITER.lock();
                    w.write_byte(ch);
                }

                // Update brace depth
                if ch == b'{' {
                    self.brace_depth += 1;
                } else if ch == b'}' {
                    self.brace_depth = self.brace_depth.saturating_sub(1);
                }

                line.push(ch as char);
            }

            // Accumulate for multiline (brace balancing)
            self.input_buf.push_str(&line);

            if self.brace_depth > 0 {
                continue; // Wait for closing brace
            }

            let src = core::mem::take(&mut self.input_buf);

            // Check for quit before processing
            if src.trim() == ":quit" || src.trim() == ":q" {
                {
                    let mut w = WRITER.lock();
                    w.color_code = vga::ColorCode::new(Color::LightCyan, Color::Black);
                    w.write_string("\n  Shalom!\n\n");
                }
                return true; // Signal to caller that REPL exited cleanly
            }

            self.handle_line(&src);
        }
    }
}

/// Split a line into individual expression strings.
/// Handles cases like "probe_Phi(x) probe_Omega(x) tier(x)" (three expressions)
/// but NOT "palace(3) light" (one expression).
///
/// Strategy: first try parsing the whole line. If that succeeds, return it as
/// one expression. If it fails, split at spaces at depth 0 and return chunks.
fn split_expressions(line: &str) -> Vec<String> {
    // Try the whole line first
    if aleph_parser::parse(line).is_ok() {
        let mut v = Vec::new();
        v.push(line.into());
        return v;
    }

    // Failed — split at spaces at depth 0
    let mut result = Vec::new();
    let mut current = String::new();
    let mut depth = 0;

    for ch in line.chars() {
        match ch {
            '(' => { current.push(ch); depth += 1; }
            ')' => { current.push(ch); if depth > 0 { depth -= 1; } }
            ' ' | '\t' if depth == 0 => {
                if !current.is_empty() {
                    result.push(current.clone());
                    current.clear();
                }
            }
            _ => { current.push(ch); }
        }
    }
    if !current.is_empty() {
        result.push(current);
    }
    result
}

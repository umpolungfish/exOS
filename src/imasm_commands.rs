//! IMASM shell commands — Tri-Phase VM REPL integration.
//!
//! Commands:
//!   imasm voynich  <tokens>         compile EVA tokens and run IMASM VM
//!   imasm voynich-bio  <tokens>     run in biological section (broken Frobenius)
//!   imasm voynich-astro <tokens>    run in astronomical section (cyclic)
//!   imasm voynich-cosmo <tokens>    run in cosmological section (cyclic, clock-indexed)
//!   imasm voynich-pharm <tokens>    run in pharmaceutical section (type declarations)
//!   imasm voynich-recipe <tokens>   run in recipes section (sequential memory)
//!   imasm voynich-bot <tokens>      run in botanical section (standard data)
//!   imasm rohonc   <tokens>         compile RTFF tokens and run
//!   imasm linear-a <tokens>         compile LATFF tokens and run
//!   imasm emerald-tablet <tokens>   compile ETFF tokens and run (C=1.0, both gates open)
//!   imasm run <filename>            execute a .imasm log from ALFS
//!   imasm step                      step one instruction
//!   imasm snapshot                  dump current VM state
//!   imasm paradox <reg>             inject Both state at register N
//!   imasm distance                  print crystal imscription distances
//!   imasm census                    print all four imscriptions

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use spin::Mutex;
use lazy_static::lazy_static;

use crate::imasm_vm::{UniversalEngine, ScriptSystem, VoynichSection, Instruction};
use crate::voynich;
use crate::rohonc;
use crate::linear_a;
use crate::emerald_tablet;

// ── Global VM state (single active engine) ───────────────────────────────────

lazy_static! {
    static ref VM: Mutex<Option<UniversalEngine>> = Mutex::new(None);
}

// ── IG weights (from aleph.rs WEIGHTS, positions 0–11) ───────────────────────
// [Ð, Þ, Ř, Φ, ƒ, Ç, Γ, ɢ, ⊙, Ħ, Σ, Ω]
const WEIGHTS: [u32; 12] = [10000, 10000, 10000, 12000, 9000, 8000, 10000, 10000, 11000, 8000, 10000, 7000];
const WEIGHT_SUM: u32 = 115000;

fn weighted_distance(a: &[u8; 12], b: &[u8; 12]) -> f64 {
    let raw: u32 = (0..12).map(|i| WEIGHTS[i] * (a[i].max(b[i]) - a[i].min(b[i])) as u32).sum();
    raw as f64 / WEIGHT_SUM as f64
}

// ── OS imscription (MEET of all five founding systems) ───────────────────────

const OS_IMSCRIPTION: [u8; 12] = [1, 3, 2, 4, 2, 1, 2, 2, 1, 2, 2, 2];

// ── Command dispatcher ───────────────────────────────────────────────────────

pub fn handle(args: &str) -> String {
    let mut parts = args.splitn(2, ' ');
    match parts.next().unwrap_or("").trim() {
        "voynich"        => run_script(ScriptSystem::Voynich, parts.next().unwrap_or("")),
        "voynich-bot"    => run_sectioned(VoynichSection::Botanical,      parts.next().unwrap_or("")),
        "voynich-astro"  => run_sectioned(VoynichSection::Astronomical,   parts.next().unwrap_or("")),
        "voynich-bio"    => run_sectioned(VoynichSection::Biological,     parts.next().unwrap_or("")),
        "voynich-cosmo"  => run_sectioned(VoynichSection::Cosmological,   parts.next().unwrap_or("")),
        "voynich-pharm"  => run_sectioned(VoynichSection::Pharmaceutical, parts.next().unwrap_or("")),
        "voynich-recipe" => run_sectioned(VoynichSection::Recipes,        parts.next().unwrap_or("")),
        "rohonc"          => run_script(ScriptSystem::Rohonc,        parts.next().unwrap_or("")),
        "linear-a"        => run_script(ScriptSystem::LinearA,      parts.next().unwrap_or("")),
        "emerald-tablet"  => run_script(ScriptSystem::EmeraldTablet, parts.next().unwrap_or("")),
        "run"      => run_file(parts.next().unwrap_or("")),
        "step"     => step_vm(),
        "snapshot" => snapshot_vm(),
        "paradox"  => inject_paradox(parts.next().unwrap_or("")),
        "distance" => print_distances(),
        "census"   => print_census(),
        "help" | "" => help(),
        other      => format!("imasm: unknown subcommand '{}'. Try 'imasm help'.", other),
    }
}

fn run_script(script: ScriptSystem, tokens: &str) -> String {
    if tokens.is_empty() {
        return format!("imasm {}: no tokens provided.", script.name().to_lowercase());
    }
    let program: Vec<Instruction> = match script {
        ScriptSystem::Voynich       => voynich::compile_raw(tokens),
        ScriptSystem::Rohonc        => rohonc::compile_raw(tokens),
        ScriptSystem::LinearA       => linear_a::compile_raw(tokens),
        ScriptSystem::EmeraldTablet => emerald_tablet::compile_raw(tokens),
    };
    if program.is_empty() {
        return format!("imasm {}: no recognizable tokens in '{}'.", script.name(), tokens);
    }
    let mut engine = UniversalEngine::new(script);
    engine.load(program);
    engine.run(1000);
    let out = engine.format_snapshot();
    *VM.lock() = Some(engine);
    out
}

fn run_sectioned(section: VoynichSection, tokens: &str) -> String {
    let sec_name = section.name().to_lowercase();
    if tokens.is_empty() {
        return format!("imasm voynich-{}: no tokens provided.", sec_name);
    }
    let program: Vec<Instruction> = voynich::compile_raw(tokens);
    if program.is_empty() {
        return format!("imasm voynich-{}: no recognizable tokens in '{}'.", sec_name, tokens);
    }
    let mut engine = UniversalEngine::new_sectioned(ScriptSystem::Voynich, section);
    engine.load(program);
    engine.run(1000);
    let out = engine.format_snapshot();
    *VM.lock() = Some(engine);
    out
}

fn run_file(name: &str) -> String {
    // Programs are embedded as static IMASM text in the programs module.
    // In a running OS context this would load from ALFS; here we match by name.
    match crate::programs::load_imasm(name) {
        Some(text) => {
            // Detect script from file name prefix
            let script = if name.starts_with("voynich")  { ScriptSystem::Voynich }
                         else if name.starts_with("rohonc")  { ScriptSystem::Rohonc }
                         else if name.starts_with("emerald") { ScriptSystem::EmeraldTablet }
                         else { ScriptSystem::LinearA };
            let program = crate::imasm_vm::parse_imasm_log(text);
            if program.is_empty() {
                return format!("imasm run: '{}' parsed to 0 instructions.", name);
            }
            let mut engine = UniversalEngine::new(script);
            engine.load(program);
            engine.run(1000);
            let out = engine.format_snapshot();
            *VM.lock() = Some(engine);
            out
        }
        None => format!("imasm run: program '{}' not found.", name),
    }
}

fn step_vm() -> String {
    let mut guard = VM.lock();
    match guard.as_mut() {
        Some(engine) => {
            engine.step();
            let s = engine.snapshot();
            let frob = if s.pending_splits == 0 { "closed" } else { "open" };
            format!("step {} | PC {} | active {} | paradoxes {} | Frobenius {} | xseg {}",
                s.step, s.pc, s.active, s.paradoxes, frob, s.cross_segment_refs)
        }
        None => "imasm step: no engine loaded. Run 'imasm voynich/rohonc/linear-a <tokens>' first.".into(),
    }
}

fn snapshot_vm() -> String {
    let guard = VM.lock();
    match guard.as_ref() {
        Some(engine) => engine.format_snapshot(),
        None => "imasm snapshot: no engine loaded.".into(),
    }
}

fn inject_paradox(arg: &str) -> String {
    let reg: u32 = match arg.trim().parse() {
        Ok(n) => n,
        Err(_) => return format!("imasm paradox: expected register number, got '{}'", arg),
    };
    let mut guard = VM.lock();
    match guard.as_mut() {
        Some(engine) => {
            engine.inject_paradox(reg);
            format!("Paradox injected at %r{} → Both state stabilized.", reg)
        }
        None => "imasm paradox: no engine loaded.".into(),
    }
}

fn print_distances() -> String {
    let v = &voynich::VOYNICH_IMSCRIPTION;
    let r = &rohonc::ROHONC_IMSCRIPTION;
    let l = &linear_a::LINEAR_A_IMSCRIPTION;
    let e = &emerald_tablet::EMERALD_TABLET_IMSCRIPTION;
    let o = &OS_IMSCRIPTION;

    format!(
        "=== CRYSTAL IMSCRIPTION DISTANCES ===\n\
         d(Linear A,      OS imscription) = {:.2}  [identical — Minoan IS the structural core]\n\
         d(Rohonc,        OS imscription) = {:.2}\n\
         d(Rohonc,        Linear A)       = {:.2}\n\
         d(Emerald Tablet, OS imscription) = {:.2}  [above OS floor; MEET is unchanged]\n\
         d(Emerald Tablet, Linear A)       = {:.2}\n\
         d(Emerald Tablet, Rohonc)         = {:.2}\n\
         d(Emerald Tablet, Voynich)        = {:.2}\n\
         d(Voynich,       OS imscription) = {:.2}\n\
         d(Voynich,       Rohonc)         = {:.2}\n\
         d(Voynich,       Linear A)       = {:.2}",
        weighted_distance(l, o),
        weighted_distance(r, o),
        weighted_distance(r, l),
        weighted_distance(e, o),
        weighted_distance(e, l),
        weighted_distance(e, r),
        weighted_distance(e, v),
        weighted_distance(v, o),
        weighted_distance(v, r),
        weighted_distance(v, l),
    )
}

fn print_census() -> String {
    let names = ["Ð","Þ","Ř","Φ","ƒ","Ç","Γ","ɢ","⊙","Ħ","Σ","Ω"];
    let v = &voynich::VOYNICH_IMSCRIPTION;
    let r = &rohonc::ROHONC_IMSCRIPTION;
    let l = &linear_a::LINEAR_A_IMSCRIPTION;
    let e = &emerald_tablet::EMERALD_TABLET_IMSCRIPTION;
    let o = &OS_IMSCRIPTION;

    let mut s = String::new();
    s += "=== CRYSTAL IMSCRIPTIONS ===\n";
    s += "      ";
    for n in &names { s += &format!("{:>4}", n); }
    s += "\n";

    for (label, imp) in &[("OS    ", o), ("EmTabl", e), ("VoynCh", v), ("Rohonc", r), ("LinA  ", l)] {
        s += label;
        for v in imp.iter() { s += &format!("{:>4}", v); }
        s += "\n";
    }
    s
}

fn help() -> String {
    "imasm subcommands:\n\
     voynich         <tokens>  compile EVA tokens (unsectioned)\n\
     voynich-bot     <tokens>  botanical   — data segment (Þ_6)\n\
     voynich-astro   <tokens>  astronomical — state register (Þ_O, cyclic)\n\
     voynich-bio     <tokens>  biological   — heap dump (Þ_K, broken Frobenius)\n\
     voynich-cosmo   <tokens>  cosmological — state register, clock-indexed (Þ_O)\n\
     voynich-pharm   <tokens>  pharmaceutical — type declarations (Þ_6)\n\
     voynich-recipe  <tokens>  recipes      — instruction memory (Ř_Ť + Ħ_£)\n\
     rohonc          <tokens>  compile RTFF tokens and run VM\n\
     linear-a        <tokens>  compile LATFF tokens and run VM\n\
     emerald-tablet  <tokens>  compile ETFF tokens and run VM  [C=1.0, both gates open]\n\
     run             <name>    load and run a .imasm program\n\
     step                      step one instruction\n\
     snapshot                  dump VM state\n\
     paradox         <reg>     inject Both state at register N\n\
     distance                  print weighted IG distances between systems\n\
     census                    print all crystal imscriptions side by side".into()
}

//! IMASM shell commands — Tri-Phase VM REPL integration.
//!
//! Commands:
//!   imasm voynich <tokens>    compile EVA tokens and run IMASM VM
//!   imasm rohonc  <tokens>    compile RTFF tokens and run
//!   imasm linear-a <tokens>   compile LATFF tokens and run
//!   imasm run <filename>      execute a .imasm log from ALFS
//!   imasm step                step one instruction
//!   imasm snapshot            dump current VM state
//!   imasm paradox <reg>       inject Both state at register N
//!   imasm distance            print crystal imscription distances
//!   imasm census              print all three imscriptions

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use spin::Mutex;
use lazy_static::lazy_static;

use crate::imasm_vm::{UniversalEngine, ScriptSystem, Instruction};
use crate::voynich;
use crate::rohonc;
use crate::linear_a;

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
        "voynich"  => run_script(ScriptSystem::Voynich,  parts.next().unwrap_or("")),
        "rohonc"   => run_script(ScriptSystem::Rohonc,   parts.next().unwrap_or("")),
        "linear-a" => run_script(ScriptSystem::LinearA,  parts.next().unwrap_or("")),
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
        ScriptSystem::Voynich => voynich::compile_raw(tokens),
        ScriptSystem::Rohonc  => rohonc::compile_raw(tokens),
        ScriptSystem::LinearA => linear_a::compile_raw(tokens),
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

fn run_file(name: &str) -> String {
    // Programs are embedded as static IMASM text in the programs module.
    // In a running OS context this would load from ALFS; here we match by name.
    match crate::programs::load_imasm(name) {
        Some(text) => {
            // Detect script from file name prefix
            let script = if name.starts_with("voynich") { ScriptSystem::Voynich }
                         else if name.starts_with("rohonc") { ScriptSystem::Rohonc }
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
            format!("step {} | PC {} | active {} | paradoxes {}",
                s.step, s.pc, s.active, s.paradoxes)
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
    let o = &OS_IMSCRIPTION;

    format!(
        "=== CRYSTAL IMSCRIPTION DISTANCES ===\n\
         d(Linear A,  OS imscription) = {:.2}  [identical — Minoan IS the structural core]\n\
         d(Rohonc,    OS imscription) = {:.2}\n\
         d(Rohonc,    Linear A)       = {:.2}\n\
         d(Voynich,   OS imscription) = {:.2}\n\
         d(Voynich,   Rohonc)         = {:.2}\n\
         d(Voynich,   Linear A)       = {:.2}",
        weighted_distance(l, o),
        weighted_distance(r, o),
        weighted_distance(r, l),
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
    let o = &OS_IMSCRIPTION;

    let mut s = String::new();
    s += "=== CRYSTAL IMSCRIPTIONS ===\n";
    s += "      ";
    for n in &names { s += &format!("{:>4}", n); }
    s += "\n";

    for (label, imp) in &[("OS    ", o), ("VoynCh", v), ("Rohonc", r), ("LinA  ", l)] {
        s += label;
        for v in imp.iter() { s += &format!("{:>4}", v); }
        s += "\n";
    }
    s
}

fn help() -> String {
    "imasm subcommands:\n\
     voynich  <tokens>  compile EVA tokens and run VM\n\
     rohonc   <tokens>  compile RTFF tokens and run VM\n\
     linear-a <tokens>  compile LATFF tokens and run VM\n\
     run      <name>    load and run a .imasm program\n\
     step               step one instruction\n\
     snapshot           dump VM state\n\
     paradox  <reg>     inject Both state at register N\n\
     distance           print weighted IG distances between systems\n\
     census             print all crystal imscriptions side by side".into()
}

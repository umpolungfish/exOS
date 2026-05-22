// ParaASM shell commands for exOS
// Usage: para <subcommand> [args]
//
// Subcommands:
//   para load  <asm>   assemble and load inline ParaASM (use \n for newlines)
//   para run   [N]     run N steps (default 100)
//   para step  [N]     step N instructions (default 1)
//   para regs          show active registers
//   para snap          full VM snapshot
//   para reset         clear all registers and reset PC
//   para loop  [N]     run N steps of a looping program (default 10000)
//   para shor  [N a]   Belnap Shor pipeline (verification suite or single instance)
//   para help          this message

extern crate alloc;

use alloc::string::String;
use alloc::format;
use spin::Mutex;
use lazy_static::lazy_static;

use crate::para_vm::ParaVM;
use crate::para_shor_commands;
use crate::para_align_commands;
use crate::para_rh_commands;
use crate::para_ym_commands;
use crate::para_nreg_commands;

lazy_static! {
    static ref PARA_VM: Mutex<ParaVM> = Mutex::new(ParaVM::new());
}

pub fn handle(args: &str) -> String {
    let mut parts = args.splitn(2, ' ');
    match parts.next().unwrap_or("").trim() {
        "load"  => load(parts.next().unwrap_or("")),
        "run"   => run_n(parts.next().unwrap_or("100")),
        "step"  => step_n(parts.next().unwrap_or("1")),
        "regs"  => regs(),
        "snap"  => snap(),
        "reset" => reset(),
        "loop"  => run_loop(parts.next().unwrap_or("10000")),
        "shor"  => para_shor_commands::handle(parts.next().unwrap_or("")),
        "align" => para_align_commands::handle(parts.next().unwrap_or("")),
        "rh"   => para_rh_commands::handle(parts.next().unwrap_or("")),
        "ym"   => para_ym_commands::handle(parts.next().unwrap_or("")),
        "nreg" => para_nreg_commands::handle(parts.next().unwrap_or("")),
        "help" | "" => help(),
        other => format!("para: unknown subcommand '{}'. Try 'para help'.", other),
    }
}

fn load(text: &str) -> String {
    if text.is_empty() {
        return "para load: provide inline ParaASM. Use \\n for newlines.\n\
                Example: para load .loop:\\nENGAGR %r0\\nFSPLIT %r0 %r1 %r2\\nFFUSE %r1 %r2 %r0\\nJMP loop".into();
    }
    // expand literal \n sequences into real newlines
    let expanded = text.replace("\\n", "\n");
    let mut vm = PARA_VM.lock();
    vm.load(&expanded);
    format!("Loaded {} instructions, {} labels.", vm.program.len(), vm.labels.len())
}

fn run_n(arg: &str) -> String {
    let n: u64 = arg.trim().parse().unwrap_or(100);
    let mut vm = PARA_VM.lock();
    if vm.program.is_empty() {
        return "para run: no program loaded. Use 'para load <asm>'.".into();
    }
    vm.run(n);
    vm.format_snapshot()
}

fn step_n(arg: &str) -> String {
    let n: u64 = arg.trim().parse().unwrap_or(1);
    let mut vm = PARA_VM.lock();
    if vm.program.is_empty() {
        return "para step: no program loaded.".into();
    }
    for _ in 0..n { vm.step(); }
    format!("pc={}  steps={}  halted={}", vm.pc, vm.steps, vm.halted)
}

fn regs() -> String {
    let vm = PARA_VM.lock();
    let active = vm.active_regs();
    if active.is_empty() {
        return "  (all registers N)".into();
    }
    let mut s = String::new();
    for (i, belief, paradoxes, fixed) in &active {
        let tag = if *fixed { " [FIXED]" } else { "" };
        s += &format!("  %r{:<2} = {}{}  paradoxes={}\n", i, belief.name(), tag, paradoxes);
    }
    s
}

fn snap() -> String {
    PARA_VM.lock().format_snapshot()
}

fn reset() -> String {
    PARA_VM.lock().reset_state();
    "ParaVM reset.".into()
}

fn run_loop(arg: &str) -> String {
    let n: u64 = arg.trim().parse().unwrap_or(10_000);
    let mut vm = PARA_VM.lock();
    if vm.program.is_empty() {
        return "para loop: no program loaded. Use 'para load <asm>'.".into();
    }
    vm.run(n);
    format!("steps={}  total_paradoxes={}", vm.steps, vm.total_paradoxes())
}

fn help() -> String {
    "para subcommands:\n\
     load  <asm>     assemble and load inline ParaASM (\\n for newlines)\n\
     run   [N]       run N steps (default 100)\n\
     step  [N]       step N instructions (default 1)\n\
     regs            show active registers\n\
     snap            full VM snapshot\n\
     reset           clear registers, reset PC\n\
     loop  [N]       run N steps of a looping program (default 10000)\n\
     shor  [N a]     Belnap Shor pipeline — full visual suite or single instance\n\
     shor  loop [N]  coherence accumulator — N cycles (default 40), 8-instance table\n\
     align           Dialetheic Alignment Theorem — DAT + seq algebra + P vs NP\n\
     align bifur     bifurcation point + DAT tri-equivalence\n\
     align seq       measurement sequence algebra (QCI_Sequences.lean)\n\
     align pvsnp     P vs NP bridge (BelnapCircuit + one-way barrier)\n\
     align shor N a  dialetheicShor framing for one instance\n\
     rh              RH Bridge — functional eq ↔ bnot, strip map\n\
     rh    frobenius  Frobenius fixed point analysis\n\
     rh    strip      critical strip state map\n\
     ym              YM Bridge — mass gap, BRST↔Frobenius, confinement\n\
     ym    gap        covering relation and mass gap\n\
     ym    brst       BRST ↔ Frobenius correspondence\n\
     nreg            n-Register generalization — SIC tensor + ratio table\n\
     nreg  ratio      coherence ratio table n=1..8\n\
     nreg  sic        SIC-POVM per-qubit axioms\n\
     \n\
     ParaASM ISA:\n\
       ENGAGR  %rN             band(r,bnot(r)): B stays B; T/F collapse\n\
       FSPLIT  %src %d1 %d2    delta: B→(T,F) comultiplication; others copy\n\
       FFUSE   %s1  %s2 %dst   mu:   Belnap join s1 v s2 -> dst\n\
       IFIX    %rN             collapse to T, mark FIXED\n\
       MOVE    %src %dst       copy register\n\
       CLEAR   %rN             reset to N\n\
       JMP     .lbl            unconditional jump\n\
       JB/JT/JF/JN  %rN .lbl  conditional branch on belief\n\
       CALL    .lbl            push PC, jump\n\
       RET                     pop and return\n\
       HALT                    stop VM\n\
       PUSH    %rN             push belief to stack\n\
       POP     %rN             pop belief from stack\n\
       EMIT    %rN             print register state to serial\n\
       READ    %rN             read belief (returns N in kernel)\n\
     \n\
     Frobenius loop (seed r0=B first; μ∘δ=id: B→(T,F)→B, P(n)=4n):\n\
       para load ENGAGR %r0\\nFSPLIT %r0 %r1 %r2\\nFFUSE %r1 %r2 %r0\\nJMP .loop\n\
       para loop 12\n\
       para regs\n\
     \n\
     IFIX stability demo (T v B = B, Theorem 3 Case B):\n\
       para load .loop:\\nENGAGR %r0\\nFSPLIT %r0 %r1 %r2\\nIFIX %r2\\nFFUSE %r1 %r2 %r0\\nJMP loop\n\
       para loop 9\n\
       para regs".into()
}

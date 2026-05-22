// ParaWASM shell commands for exOS
// Usage: wasm <subcommand> [args]
//
//   wasm i32  <n>          push i32_const n (tagged T)
//   wasm i64  <n>          push i64_const n (tagged T)
//   wasm drop              drop top of stack
//   wasm nop               no-op
//   wasm unreachable       set frob_invariant → F
//   wasm checkpoint        snapshot current stack
//   wasm verify            check all values designated → B, else F
//   wasm assert            assert_invariant
//   wasm snap              show VM state
//   wasm reset             clear state
//   wasm demo  [n]         frobenius_empty_stack demo (default n=42)
//   wasm help              this message

extern crate alloc;

use alloc::string::String;
use alloc::format;
use spin::Mutex;
use lazy_static::lazy_static;

use crate::para_wasm::{WasmRuntime, WasmInstr, demo_frobenius_empty_stack};

lazy_static! {
    static ref WASM_RT: Mutex<WasmRuntime> = Mutex::new(WasmRuntime::new());
}

pub fn handle(args: &str) -> String {
    let mut parts = args.splitn(2, ' ');
    let sub = parts.next().unwrap_or("").trim();
    let rest = parts.next().unwrap_or("").trim();

    match sub {
        "i32"          => push_instr(WasmInstr::I32Const(rest.parse().unwrap_or(0))),
        "i64"          => push_instr(WasmInstr::I64Const(rest.parse().unwrap_or(0))),
        "drop"         => exec(WasmInstr::Drop),
        "nop"          => exec(WasmInstr::Nop),
        "unreachable"  => exec(WasmInstr::Unreachable),
        "checkpoint"   => exec(WasmInstr::Checkpoint),
        "verify"       => exec(WasmInstr::Verify),
        "assert"       => exec(WasmInstr::AssertInvariant),
        "snap"         => WASM_RT.lock().format_snapshot(),
        "reset"        => { *WASM_RT.lock() = WasmRuntime::new(); "ParaWASM reset.".into() }
        "demo"         => {
            let n: u64 = rest.parse().unwrap_or(42);
            demo_frobenius_empty_stack(n)
        }
        "help" | ""    => help(),
        other          => format!("wasm: unknown subcommand '{}'. Try 'wasm help'.", other),
    }
}

fn push_instr(instr: WasmInstr) -> String {
    let mut rt = WASM_RT.lock();
    rt.program.push(instr.clone());
    crate::para_wasm::exec_one(&mut rt.state, &instr);
    rt.format_snapshot()
}

fn exec(instr: WasmInstr) -> String {
    let mut rt = WASM_RT.lock();
    crate::para_wasm::exec_one(&mut rt.state, &instr);
    rt.format_snapshot()
}

fn help() -> String {
    "wasm subcommands:\n\
     i32  <n>       push i32_const n (tag=T)\n\
     i64  <n>       push i64_const n (tag=T)\n\
     drop           drop top of stack\n\
     nop            no-op\n\
     unreachable    set frob_invariant → F\n\
     checkpoint     snapshot current stack\n\
     verify         all designated (T|B)? → B, else F\n\
     assert         assert_invariant (frobTagBin with self)\n\
     snap           show VM state\n\
     reset          clear all state\n\
     demo [n]       frobenius_empty_stack: checkpoint+i32_const n+verify → B\n\
     \n\
     Belnap tagging: every WASM value carries a B4 belief (N/T/F/B).\n\
     Constants are tagged T. verify checks designated values (T or B).\n\
     frobTagBin = meet in approximation order; identity: meet(t,B)=t.\n\
     \n\
     frobenius_empty_stack demo:\n\
       wasm checkpoint\n\
       wasm i32 42\n\
       wasm verify\n\
       wasm snap\n\
       → frob_invariant=B  (theorem proven in SelfVerifyingWASM.lean)".into()
}

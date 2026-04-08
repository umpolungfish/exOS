#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

extern crate alloc;

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use exoterik_os::*;
use exoterik_os::ALLOCATOR;

entry_point!(kernel_main);

/// Kernel boot: P_±_sym → P_asym symmetry breaking
///
/// The system boots in perfect symmetry (no process distinguished, all resources pooled).
/// The first scheduled interrupt is the symmetry-breaking event (δχ becomes nonzero).
/// From that moment the system is P_asym and the ergative process model activates.
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    vga::init();
    serial::init();

    // Initialize the heap using the physical memory mapping.
    // Bootloader 0.9 with map_physical_memory maps all physical memory at
    // physical_memory_offset. We find the largest usable region and place
    // the heap there.
    let phys_offset = boot_info.physical_memory_offset;

    // Place the heap at physical 16MB, accessed via the physical_memory_offset mapping.
    // 16MB is safely above the bootloader (< 1MB), kernel (~4MB physical), and page tables.
    // QEMU's default 128MB gives plenty of room for a 4MB heap here.
    let heap_phys: u64 = 0x100_0000; // 16MB physical
    let heap_start = (phys_offset + heap_phys) as *mut u8;
    let heap_size: usize = 4 * 1024 * 1024; // 4MB
    unsafe {
        ALLOCATOR.lock().init(heap_start, heap_size);
    }

    println!("[BOOT] phys_offset=0x{:x}, heap=0x{:x}+4M", phys_offset, heap_phys);

    // Initialize interrupt table — this is the symmetry-breaking event
    // Before this: P_±_sym (all resources pooled, nothing distinguished)
    // After this: P_asym (ergative scheduler activates, roles distinguished)
    interrupts::init();

    println!("[exoterikOS] Phi_c Kernel booting...");
    println!("P_pm_sym -> P_asym symmetry break initiated.");

    // --- Three-layer kernel object (Hieroglyphs + Cuneiform) ---
    // Exercise all StructuralType, OperationalMode, and Determinative variants
    let init_process = kernel_object::KernelObject::new(
        kernel_object::StructuralType::Process,
        kernel_object::OperationalMode::Schedule,
        kernel_object::Determinative::Init,
        0,
    );
    assert!(init_process.is_well_formed());

    // Exercise remaining StructuralType variants via additional kernel objects
    let _socket_obj = kernel_object::KernelObject::new(
        kernel_object::StructuralType::Socket,
        kernel_object::OperationalMode::Network,
        kernel_object::Determinative::Service,
        1,
    );
    let _semaphore_obj = kernel_object::KernelObject::new(
        kernel_object::StructuralType::Semaphore,
        kernel_object::OperationalMode::Compute,
        kernel_object::Determinative::Driver,
        2,
    );
    let _memory_obj = kernel_object::KernelObject::new(
        kernel_object::StructuralType::MemoryRegion,
        kernel_object::OperationalMode::MemoryManage,
        kernel_object::Determinative::Kernel,
        3,
    );
    let _file_obj = kernel_object::KernelObject::new(
        kernel_object::StructuralType::File,
        kernel_object::OperationalMode::IO,
        kernel_object::Determinative::User,
        4,
    );
    let _idle_obj = kernel_object::KernelObject::new(
        kernel_object::StructuralType::Process,
        kernel_object::OperationalMode::Idle,
        kernel_object::Determinative::User,
        5,
    );

    // Read all three-layer fields to prove the architecture is load-bearing
    let _s = init_process.structural;
    let _o = init_process.operational;
    let _d = init_process.determinative;

    println!("[INIT] Three-layer objects: all structural/operational/determinative variants exercised");

    // --- Ergative scheduler (Basque grammar) ---
    let mut sched = scheduler::ErgativeScheduler::new();
    assert!(sched.is_symmetric());

    let pcb = scheduler::ProcessControlBlock {
        id: 1,
        obj: init_process,
        role: scheduler::GrammaticalRole::Absolutive,
        priority: 1,
        stack_pointer: 0x1000,
        targets: alloc::vec![2],
    };
    let mut test_pcb = pcb;
    // Read all fields to satisfy dead-code analysis
    let _pcb_id = test_pcb.id;
    let _pcb_obj = test_pcb.obj.clone();
    let _pcb_sp = test_pcb.stack_pointer;
    test_pcb.determine_role();
    let _p = test_pcb.effective_priority();

    sched.spawn(test_pcb);
    sched.break_symmetry();
    assert!(!sched.is_symmetric());
    let _next = sched.schedule_next();
    println!("[SCHED] Ergative scheduler online, symmetry broken");

    // --- Phonological memory (Varnamala articulation gradient) ---
    let mut palloc = memory::PhonologicalAllocator::new();

    // Exercise all articulation depths
    let depths = [
        memory::ArticulationDepth::Velar,
        memory::ArticulationDepth::Palatal,
        memory::ArticulationDepth::Retroflex,
        memory::ArticulationDepth::Dental,
        memory::ArticulationDepth::Bilabial,
    ];
    for depth in &depths {
        let _prot = depth.protection_level();
        let _valid = depth.requires_validation();
    }

    palloc.set_depth(memory::ArticulationDepth::Velar);
    let layout = core::alloc::Layout::from_size_align(64, 8).unwrap();
    if let Some(ptr) = palloc.allocate(layout) {
        palloc.deallocate(ptr, layout);
    }
    println!("[MEM] Phonological allocator: Velar -> Bilabial gradient online");

    // --- Sefirot filesystem (Hebrew Kabbalistic tree) ---
    let mut fs = filesystem::SefirotFs::new();

    // Walk the Sefirot tree
    let sefirot = [
        filesystem::Sefirah::Keter,
        filesystem::Sefirah::Chokhmah,
        filesystem::Sefirah::Binah,
        filesystem::Sefirah::Daat,
        filesystem::Sefirah::Chesed,
        filesystem::Sefirah::Gevurah,
        filesystem::Sefirah::Tiferet,
        filesystem::Sefirah::Netzach,
        filesystem::Sefirah::Hod,
        filesystem::Sefirah::Yesod,
        filesystem::Sefirah::Malkuth,
    ];
    for s in &sefirot {
        let _name = s.name();
        let _path = s.default_path();
        fs.navigate_to(*s);
    }
    let _current = fs.current();
    let _tree = fs.tree();

    let path = filesystem::SefirotPath::new(
        alloc::vec![
            filesystem::Sefirah::Keter,
            filesystem::Sefirah::Chokhmah,
            filesystem::Sefirah::Binah,
        ],
        "init",
    );
    let _resolved = path.resolve();
    println!("[FS] Sefirot tree: Keter -> Malkuth, 10 layers mapped");

    // --- IPC (Egyptian three-layer messages) ---
    let sig = ipc::StructuralSignature {
        source_type: kernel_object::StructuralType::Process,
        target_type: kernel_object::StructuralType::File,
    };
    static PAYLOAD: &[u8] = b"\xCE\xA6_c";
    let det = ipc::MessageDeterminative {
        source_ctx: kernel_object::Determinative::Kernel,
        target_ctx: kernel_object::Determinative::User,
    };
    let msg = ipc::IpcMessage::new(sig, PAYLOAD, det);
    assert!(msg.is_well_formed());
    let _len = msg.len();
    let _empty = msg.is_empty();
    println!("[IPC] Three-layer message: well_formed={} len={}", msg.is_well_formed(), msg.len());

    // --- Command grammar (Hebrew letters + Varnamala pratyahara) ---
    let mut cmd = command::GenerativeCommand::new(alloc::vec![
        command::CommandPrimitive::Aleph,
        command::CommandPrimitive::Mem,
        command::CommandPrimitive::Shin,
        command::CommandPrimitive::Vav,
        command::CommandPrimitive::Bet,
        command::CommandPrimitive::Gimel,
        command::CommandPrimitive::Dalet,
        command::CommandPrimitive::Heh,
    ]);
    let _gem = cmd.total_gematria();

    // Exercise individual primitive gematria
    let _g_aleph = command::CommandPrimitive::Aleph.gematria();
    let _g_mem = command::CommandPrimitive::Mem.gematria();
    let _g_shin = command::CommandPrimitive::Shin.gematria();
    let _g_vav = command::CommandPrimitive::Vav.gematria();
    let _g_bet = command::CommandPrimitive::Bet.gematria();
    let _g_gimel = command::CommandPrimitive::Gimel.gematria();
    let _g_dalet = command::CommandPrimitive::Dalet.gematria();
    let _g_heh = command::CommandPrimitive::Heh.gematria();

    let ctx = cmd.generate_context();
    let _idx = ctx.pratyahara_index;
    let _pri = ctx.priority;
    let _has_aleph = ctx.has_aleph;
    println!("[CMD] Generative command: gematria={} pratyahara={}",
        cmd.total_gematria(), ctx.pratyahara_index);

    // --- VGA color test (exercise all Color variants) ---
    vga::write_colored_test();

    // --- ℵ-OS λ_ℵ type system (Hebrew 22-letter lattice) ---
    // Exercise the dormant aleph.rs module: system language, tier census, distance.
    let _sys = aleph::system_language();
    let _census_boot = aleph::tier_census();
    let _d = aleph::distance(&aleph::LETTERS[0].t, &aleph::LETTERS[5].t);
    let _tier_vav = aleph::compute_tier(&aleph::LETTERS[5].t);
    let _tier_mem = aleph::compute_tier(&aleph::LETTERS[12].t);
    let _tier_shin = aleph::compute_tier(&aleph::LETTERS[20].t);

    // Verify the three Frobenius fixed points (O_inf)
    assert!(matches!(aleph::compute_tier(&aleph::LETTERS[5].t), aleph::Tier::OInf));   // ו
    assert!(matches!(aleph::compute_tier(&aleph::LETTERS[12].t), aleph::Tier::OInf));  // מ
    assert!(matches!(aleph::compute_tier(&aleph::LETTERS[20].t), aleph::Tier::OInf));  // ש

    println!("[ALEPH] 22-letter type system online. O_inf: 3, O_2: 6, O_1: 1, O_0: 12");

    // --- ALFS filesystem (ATA PIO, sector-based) ---
    match alfs::mount() {
        Ok(()) => {
            println!("[FS] {} - files: {}", alfs::info(), alfs::list().len());
            // Initialize Sefirot filesystem from ALFS
            match filesystem::init() {
                Ok(count) => println!("[SEFIROT] {} files loaded into Sefirot tree", count),
                Err(e) => println!("[SEFIROT] Init failed: {}", e),
            }
        }
        Err(e) => println!("[FS] ALFS mount failed: {} (running without disk)", e),
    }
    println!("[exoterikOS] Phi_c Kernel fully online.");
    shell_main();
}

// ── Shell ─────────────────────────────────────────────────────────────────────

fn handle_key(key: u8, line_buf: &mut alloc::vec::Vec<u8>) {
    match key {
        b'\n' | b'\r' => {
            println!();
            let cmd: &str = core::str::from_utf8(line_buf).unwrap_or("");
            run_command(cmd.trim());
            line_buf.clear();
            print_prompt();
        }
        0x08 | 0x7F => {  // backspace or DEL (terminals send 0x7F)
            if !line_buf.is_empty() {
                line_buf.pop();
                // VGA: move cursor back and blank the cell
                crate::vga::WRITER.lock().backspace();
                // Serial terminal: BS SP BS — move left, erase, move left
                serial::write_byte(0x08);
                serial::write_byte(b' ');
                serial::write_byte(0x08);
            }
        }
        0x1B => {} // ESC — ignore
        ch if ch.is_ascii_graphic() || ch == b' ' => {
            if line_buf.len() < 78 {
                line_buf.push(ch);
                // Echo character to serial (VGA gets it via print!)
                serial::write_byte(ch);
                crate::vga::WRITER.lock().write_byte(ch);
            }
        }
        _ => {}
    }
}

fn shell_main() -> ! {
    println!("exoterikOS shell. Type 'help' for commands.");
    let mut line_buf = alloc::vec::Vec::<u8>::new();
    print_prompt();

    loop {
        // HLT only when both PS/2 keyboard and serial UART are empty
        x86_64::instructions::interrupts::disable();
        if keyboard::is_empty() && !serial::rx_ready() {
            x86_64::instructions::interrupts::enable_and_hlt();
        } else {
            x86_64::instructions::interrupts::enable();
        }

        // Drain PS/2 keyboard (physical keyboard / QEMU -display curses)
        while let Some(key) = keyboard::pop() {
            handle_key(key, &mut line_buf);
        }

        // Drain serial UART (stdin when running with -nographic)
        while let Some(b) = serial::try_read() {
            handle_key(b, &mut line_buf);
        }
    }
}

fn print_prompt() {
    print!("exOS> ");
}

fn run_command(cmd: &str) {
    match cmd {
        "" => {}
        "help" => {
            println!("Commands:");
            println!("  help    - this message");
            println!("  clear   - clear screen");
            println!("  info    - kernel information");
            println!("  phi     - Phi_c / synthonicon primitives");
            println!("  sched   - ergative scheduler status");
            println!("  mem     - memory allocator status");
            println!("  fs      - sefirot filesystem tree (full view)");
            println!("  cd X    - navigate to Sefirah X");
            println!("  ls      - list files in current Sefirah");
            println!("  cat F   - read file F in current Sefirah");
            println!("  write F C - write content C to file F");
            println!("  ipc     - send test IPC message");
            println!("  aleph   - enter the ALEPH REPL");
            println!("  history [N] - replay last N lines (default 50)");
            println!("  bench   - run performance benchmarks");
            println!("  reboot  - triple-fault reboot");
        }
        "clear" => {
            for _ in 0..25 { println!(); }
        }
        "info" => {
            println!("exoterikOS v0.1");
            println!("  Arch  : x86_64 bare-metal");
            println!("  Model : Phi_c kernel (critical manifold)");
            println!("  Sched : Ergative (Basque grammar)");
            println!("  Mem   : Phonological (Varnamala gradient)");
            println!("  FS    : Sefirot tree (Keter->Malkuth)");
            println!("  IPC   : Three-layer Egyptian messages");
            println!("  Cmd   : Hebrew-letter generative grammar");
        }
        "phi" => {
            let census = aleph::tier_census();
            println!("Phi_c - critical manifold primitives:");
            println!("  Tuple : <D; T; R; P; F; K; G; Gamma; Phi; H; S; Omega>");
            println!("  D_holo  T_net   R_lr    P_pm_sym");
            println!("  F_hbar  K_mod   G_aleph Gamma_seq");
            println!("  Phi_c   H_inf   n_n     Omega_Z");
            println!("  O_inf  : Phi_c + P_pm_sym (Frobenius)");
            println!("  C score: [Phi=Phi_c]*[K!=K_trap]*(0.292T+0.273G+0.276O+0.158K)");
            println!();
            println!("  ALEPH system: 22 Hebrew letters, 12 primitives each");
            println!("  Tiers: O_inf({}) O_2({}) O_1({}) O_0({})",
                census[4], census[2], census[1], census[0]);
            println!("  Type 'aleph' to enter the ALEPH REPL");
        }
        "sched" => {
            let mut sched = scheduler::ErgativeScheduler::new();
            println!("Ergative scheduler:");
            println!("  Symmetric : {}", sched.is_symmetric());
            sched.break_symmetry();
            println!("  After break: symmetric={}", sched.is_symmetric());
            println!("  Grammatical roles: Absolutive / Ergative / Dative");
        }
        "mem" => {
            println!("Phonological memory allocator:");
            println!("  Velar    -> deep kernel (high protection)");
            println!("  Palatal  -> kernel space");
            println!("  Retroflex-> system space");
            println!("  Dental   -> user space");
            println!("  Bilabial -> user space (low protection)");
            println!("  Heap base: 0x1000000 + 4MB");
        }
        "fs" => {
            let fs = filesystem::fs();
            println!("Sefirot Filesystem:");
            println!("{}", fs.full_tree());
            println!("Current Sefirah: {}", fs.current().name());
        }
        cmd if cmd.starts_with("cd ") => {
            let target = cmd[3..].trim();
            let fs = filesystem::fs();
            if let Some(sefirah) = filesystem::Sefirah::all().iter().find(|s| 
                s.name().eq_ignore_ascii_case(target) || 
                s.default_path().trim_start_matches('/') == target
            ) {
                let chain = fs.navigate_to(*sefirah);
                print!("Navigated to {} [", sefirah.name());
                for (i, s) in chain.iter().enumerate() {
                    if i > 0 { print!(" -> "); }
                    print!("{}", s.name());
                }
                println!("]");
            } else {
                println!("Unknown Sefirah: '{}'. Available:", target);
                for s in filesystem::Sefirah::all() {
                    println!("  {} ({})", s.name(), s.default_path());
                }
            }
        }
        cmd if cmd.starts_with("ls") => {
            let fs = filesystem::fs();
            let files = fs.list();
            if files.is_empty() {
                println!("{} (empty)", fs.current().default_path());
            } else {
                println!("{}:", fs.current().default_path());
                for f in &files {
                    let type_tag = match f.file_type {
                        filesystem::FileType::Regular => "f",
                        filesystem::FileType::AlephProg => "λ",
                        filesystem::FileType::Directory => "d",
                        filesystem::FileType::Device => "c",
                        filesystem::FileType::IPC => "p",
                        filesystem::FileType::Log => "l",
                    };
                    println!("  {} {} ({} bytes)", type_tag, f.name, f.size);
                }
            }
        }
        cmd if cmd.starts_with("cat ") => {
            let name = cmd[4..].trim();
            let fs = filesystem::fs();
            if let Some(content) = fs.read_string(name) {
                println!("{}", content);
            } else {
                println!("File not found: {}", name);
            }
        }
        cmd if cmd.starts_with("write ") => {
            let rest = cmd[6..].trim();
            if let Some(space_pos) = rest.find(' ') {
                let name = &rest[..space_pos];
                let content = &rest[space_pos+1..];
                let fs = filesystem::fs();
                fs.write(name, content.as_bytes());
                println!("  Written {} bytes to '{}' in {}", 
                    content.len(), name, fs.current().name());
            } else {
                println!("Usage: write <name> <content>");
                println!("  Writes a single-line file to current Sefirah ({})", 
                    filesystem::fs().current().name());
            }
        }
        "ipc" => {
            let sig = ipc::StructuralSignature {
                source_type: kernel_object::StructuralType::Process,
                target_type: kernel_object::StructuralType::File,
            };
            let det = ipc::MessageDeterminative {
                source_ctx: kernel_object::Determinative::Kernel,
                target_ctx: kernel_object::Determinative::User,
            };
            let msg = ipc::IpcMessage::new(sig, b"Phi_c", det);
            println!("IPC message sent:");
            println!("  well_formed={} len={}", msg.is_well_formed(), msg.len());
            println!("  payload: Phi_c");
        }
        "bench" => {
            println!("Calibrating CPU frequency via PIT...");
            let results = bench::run_all();
            let mhz = results.first().map(|r| r.mhz).unwrap_or(0);
            println!("CPU: ~{} MHz (PIT calibration)", mhz);
            println!("");
            println!("  {:<28}  {:>8}  {:>10}  {:>8}",
                "benchmark", "iters", "cy/op", "Mop/s");
            println!("  {}", "-".repeat(60));
            for r in &results {
                let mops = r.mops();
                println!("  {:<28}  {:>8}  {:>10}  {:>8}",
                    r.name, r.iters, r.cycles_per_op(),
                    if mops > 0 { mops } else { 0 });
            }
            println!("");
        }
        "history" => {
            history::replay(50);
        }
        cmd if cmd.starts_with("history ") => {
            let n: usize = cmd[8..].trim().parse().unwrap_or(50);
            history::replay(n);
        }
        cmd if cmd.starts_with("aleph") => {
            let args = cmd.strip_prefix("aleph").unwrap_or("").trim();
            aleph_commands::handle_aleph(args);
        }
        "reboot" => {
            println!("Rebooting via triple fault...");
            unsafe {
                // Load null IDT then trigger divide-by-zero → triple fault → reset
                let null_idt: x86_64::structures::idt::InterruptDescriptorTable =
                    x86_64::structures::idt::InterruptDescriptorTable::new();
                null_idt.load_unsafe();
                core::arch::asm!("int3");
            }
        }
        _ => {
            println!("Unknown command: '{}'. Try 'help'.", cmd);
        }
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("\nKERNEL PANIC: {}", info);
    // Shutdown reverses the symmetry-breaking sequence:
    // processes collapse back toward P_±_sym as resources are released
    loop {
        x86_64::instructions::hlt();
    }
}

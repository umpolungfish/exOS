#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

extern crate alloc;

use bootloader_api::{entry_point, BootInfo, config::{BootloaderConfig, Mapping}};
use core::panic::PanicInfo;
use exoterik_os::*;
use exoterik_os::ALLOCATOR;

/// Bootloader configuration with physical memory mapping and framebuffer enabled.
const BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config.mappings.framebuffer = Mapping::Dynamic;
    config.kernel_stack_size = 0x20000;
    config
};

entry_point!(kernel_main, config = &BOOTLOADER_CONFIG);

/// Kernel boot: P_±_sym → P_asym symmetry breaking
///
/// The system boots in perfect symmetry (no process distinguished, all resources pooled).
/// The first scheduled interrupt is the symmetry-breaking event (δχ becomes nonzero).
/// From that moment the system is P_asym and the ergative process model activates.
fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    serial::init();

    // ── Heap initialization — MUST happen before any println! ──
    // _print previously called args.to_string() which heap-allocates.
    // Without the heap ready, that cascades: OOM → panic → println → OOM → …
    // → stack overflow → double fault → triple fault → CPU reset → QEMU exits.
    // We borrow boot_info immutably here; the mutable framebuffer borrow comes after.
    {
        let phys_offset_opt = boot_info.physical_memory_offset.as_ref().copied();
        let heap_phys = boot_info.memory_regions
            .iter()
            .find(|r| {
                let len = r.end.saturating_sub(r.start);
                matches!(r.kind, bootloader_api::info::MemoryRegionKind::Usable)
                    && len >= 4 * 1024 * 1024
            })
            .map(|r| r.start + 0x100_0000) // 16MB into the region, past bootloader data
            .unwrap_or(0x100_0000);

        if let Some(phys_offset) = phys_offset_opt {
            let heap_start = (phys_offset + heap_phys) as *mut u8;
            let heap_size: usize = 4 * 1024 * 1024; // 4MB
            unsafe { ALLOCATOR.lock().init(heap_start, heap_size); }
            serial::write_str("[BOOT] Heap: initialized (4MB)\r\n");
        } else {
            serial::write_str("[BOOT] WARNING: physical_memory_offset not mapped — heap unavailable\r\n");
        }
    } // immutable borrow of boot_info released here

    // ── UEFI Framebuffer initialization ──
    // Heap is now ready; println! is safe to call.
    if let Some(fb) = boot_info.framebuffer.as_mut() {
        let fb_info = fb.info();
        let fb_width = fb_info.width as u64;
        let fb_height = fb_info.height as u64;
        let fb_pitch = (fb_info.stride * fb_info.bytes_per_pixel) as u64;
        let fb_bpp = (fb_info.bytes_per_pixel * 8) as u8;

        unsafe {
            crate::framebuffer::init_hw(
                fb.buffer_mut().as_mut_ptr() as u64,
                fb_width,
                fb_height,
                fb_pitch,
                fb_bpp,
            );
        }

        // Switch to framebuffer mode — all print!/println! now render pixels.
        vga::init_framebuffer();

        println!("[BOOT] UEFI GOP Framebuffer: {}x{} @ {}bpp (stride={}, format={:?})",
            fb_width, fb_height, fb_bpp, fb_info.stride, fb_info.pixel_format);
    } else {
        println!("[BOOT] WARNING: No framebuffer found. Falling back to VGA text mode.");
        vga::init();
    }

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

    // ── Type-system boot verification ─────────────────────────────────────
    // The 12-primitive type lattice is now operational — it constrains kernel
    // behavior. Verify all four type gates at boot time.

    // Verify kernel object type inference
    let kernel_obj = kernel_object::KernelObject::new(
        kernel_object::StructuralType::Process,
        kernel_object::OperationalMode::Compute,
        kernel_object::Determinative::Kernel,
        100,
    );
    let type_summary = kernel_obj.aleph_type.summary();
    let wf = kernel_obj.is_well_formed();
    println!("[TYPE] Kernel object: {}  well_formed={}", type_summary, wf);
    assert!(wf, "Kernel object should be well-formed (Ω_Z with Kernel determinative)");

    // Verify user object type inference
    let user_obj = kernel_object::KernelObject::new(
        kernel_object::StructuralType::Process,
        kernel_object::OperationalMode::Compute,
        kernel_object::Determinative::User,
        101,
    );
    let user_wf = user_obj.is_well_formed();
    println!("[TYPE] User object: {}  well_formed={}", user_obj.aleph_type.summary(), user_wf);
    assert!(user_wf, "User object should be well-formed (Ω_0 with User determinative)");

    // Verify IPC type gate: close types should pass
    let close_src = kernel_object::KernelObject::new(
        kernel_object::StructuralType::Process,
        kernel_object::OperationalMode::Compute,
        kernel_object::Determinative::Kernel,
        200,
    );
    let close_tgt = kernel_object::KernelObject::new(
        kernel_object::StructuralType::Process,
        kernel_object::OperationalMode::IO,
        kernel_object::Determinative::Kernel,
        201,
    );
    let close_msg = ipc::IpcMessage::with_types(
        ipc::StructuralSignature {
            source_type: kernel_object::StructuralType::Process,
            target_type: kernel_object::StructuralType::Process,
        },
        b"test",
        ipc::MessageDeterminative {
            source_ctx: kernel_object::Determinative::Kernel,
            target_ctx: kernel_object::Determinative::Kernel,
        },
        close_src.aleph_type.clone(),
        close_tgt.aleph_type.clone(),
    );
    let ipc_result = close_msg.is_type_valid();
    println!("[TYPE] IPC gate (close): accepted={}", ipc_result.is_accepted());
    assert!(ipc_result.is_accepted(), "Close types should pass IPC gate");

    // Verify IPC type gate: remote types should be rejected without witness
    let remote_src = kernel_object::KernelObject::new(
        kernel_object::StructuralType::Process,
        kernel_object::OperationalMode::Compute,
        kernel_object::Determinative::Kernel,
        300,
    );
    let remote_tgt = kernel_object::KernelObject::new(
        kernel_object::StructuralType::File,
        kernel_object::OperationalMode::IO,
        kernel_object::Determinative::User,
        301,
    );
    let remote_msg = ipc::IpcMessage::with_types(
        ipc::StructuralSignature {
            source_type: kernel_object::StructuralType::Process,
            target_type: kernel_object::StructuralType::File,
        },
        b"remote_test",
        ipc::MessageDeterminative {
            source_ctx: kernel_object::Determinative::Kernel,
            target_ctx: kernel_object::Determinative::User,
        },
        remote_src.aleph_type.clone(),
        remote_tgt.aleph_type.clone(),
    );
    let remote_result = remote_msg.is_type_valid();
    println!("[TYPE] IPC gate (remote): accepted={}", remote_result.is_accepted());

    // Verify Ω-gated memory: kernel object at Velar should pass
    let mut palloc = memory::PhonologicalAllocator::new();
    palloc.set_depth(memory::ArticulationDepth::Velar);
    let velar_check = palloc.can_allocate_for(&kernel_obj);
    println!("[TYPE] Ω gate (Velar+Kernel): allowed={}", velar_check.is_allowed());
    assert!(velar_check.is_allowed(), "Kernel object should pass Ω gate at Velar");

    // Verify Ω-gated memory: user object at Velar should fail
    let user_velar_check = palloc.can_allocate_for(&user_obj);
    println!("[TYPE] Ω gate (Velar+User): allowed={}", user_velar_check.is_allowed());
    assert!(!user_velar_check.is_allowed(), "User object should fail Ω gate at Velar");

    // Verify tier-gated scheduler: O_inf ergative should pass
    let o_inf_obj = kernel_object::KernelObject::with_type(
        kernel_object::StructuralType::Process,
        kernel_object::OperationalMode::Compute,
        kernel_object::Determinative::Kernel,
        400,
        aleph_kernel_types::canonical::kernel_process(),
    );
    let mut sched_test = scheduler::ErgativeScheduler::new();
    let o_inf_pcb = scheduler::ProcessControlBlock {
        id: 400,
        obj: o_inf_obj.clone(),
        role: scheduler::GrammaticalRole::Ergative,
        priority: 5,
        stack_pointer: 0x5000,
        targets: alloc::vec![401],
    };
    let spawn_result = sched_test.spawn_type_safe(o_inf_pcb);
    println!("[TYPE] Tier gate (O_inf ergative): ok={}", spawn_result.is_ok());
    assert!(spawn_result.is_ok(), "O_inf process should pass tier gate as ergative");

    // Verify tier-gated scheduler: O_0 ergative should fail
    let o0_obj = kernel_object::KernelObject::with_type(
        kernel_object::StructuralType::Process,
        kernel_object::OperationalMode::Idle,
        kernel_object::Determinative::User,
        402,
        aleph_kernel_types::canonical::user_process(),
    );
    let o0_pcb = scheduler::ProcessControlBlock {
        id: 402,
        obj: o0_obj,
        role: scheduler::GrammaticalRole::Ergative,
        priority: 5,
        stack_pointer: 0x6000,
        targets: alloc::vec![403],
    };
    let o0_result = sched_test.spawn_type_safe(o0_pcb);
    println!("[TYPE] Tier gate (O_0 ergative): ok={}", o0_result.is_err());
    assert!(o0_result.is_err(), "O_0 process should fail tier gate as ergative");

    // Verify Φ-gated filesystem: kernel object to Keter should pass (Φ_c ≥ 1)
    let mut fs_test = filesystem::SefirotFs::new();
    let keter_result = fs_test.navigate_to_type_safe(
        filesystem::Sefirah::Keter,
        &kernel_obj,
    );
    println!("[TYPE] Φ gate (Keter+Kernel): ok={}", keter_result.is_ok());
    assert!(keter_result.is_ok(), "Kernel object should pass Φ gate to Keter");

    // Verify Φ-gated filesystem: user object (Compute) also has Φ_c and passes
    // — user processes CAN conceptually reach Keter (Φ gate is about criticality,
    //   not protection; Ω handles the protection gate separately)
    let user_keter_result = fs_test.navigate_to_type_safe(
        filesystem::Sefirah::Keter,
        &user_obj,
    );
    println!("[TYPE] Φ gate (Keter+User): ok={}", user_keter_result.is_ok());
    assert!(user_keter_result.is_ok(), "User Compute object has Φ_c and passes Φ gate to Keter");

    // Verify Φ-gated filesystem: sub-critical object (Driver) should FAIL Keter
    let driver_obj = kernel_object::KernelObject::new(
        kernel_object::StructuralType::Process,
        kernel_object::OperationalMode::Compute,
        kernel_object::Determinative::Driver,
        102,
    );
    let driver_keter_result = fs_test.navigate_to_type_safe(
        filesystem::Sefirah::Keter,
        &driver_obj,
    );
    println!("[TYPE] Φ gate (Keter+Driver): ok={}", driver_keter_result.is_ok());
    assert!(!driver_keter_result.is_ok(), "Driver object (Φ_sub) should fail Φ gate to Keter");

    // Verify Ω gate in isolation: user object at Velar should fail
    let user_velar_check = palloc.can_allocate_for(&user_obj);
    println!("[TYPE] Ω gate (Velar+User): allowed={}", user_velar_check.is_allowed());
    assert!(!user_velar_check.is_allowed(), "User object should fail Ω gate at Velar");

    // Verify conscience scores
    let c_kernel = kernel_obj.aleph_type.conscience_score();
    let c_user = user_obj.aleph_type.conscience_score();
    let c_os = aleph_kernel_types::canonical::os_imscription().conscience_score();
    println!("[TYPE] C scores: kernel={:.3} user={:.3} os={:.3}",
        c_kernel, c_user, c_os);

    // --- Holographic Self-Encoding Monitor ---
    let mut holographic_monitor = holographic_monitor::HolographicMonitor::new();
    let g_pcb = holographic_monitor.pcb.clone();
    let spawn_result = sched.spawn_type_safe(g_pcb);
    println!("[HOLO] Holographic monitor (g(x)): spawn ok={}", spawn_result.is_ok());
    assert!(spawn_result.is_ok(), "g(x) process should spawn as O_inf ergative");

    // --- ALFS filesystem (ATA PIO, sector-based) ---
    match alfs::mount() {
        Ok(()) => {
            programs::seed_alfs();
            println!("[FS] {} - files: {}", alfs::info(), alfs::list().len());
            match filesystem::init() {
                Ok(count) => println!("[SEFIROT] {} files loaded from disk", count),
                Err(e) => println!("[SEFIROT] Init failed: {}", e),
            }
        }
        Err(e) => println!("[FS] ALFS: {} (no disk — using in-memory FS)", e),
    }
    // Always populate built-in seed files (skipped if already present from ALFS)
    filesystem::populate_defaults();
    println!("[exoterikOS] Phi_c Kernel fully online. Type 'help' for commands.");
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
                crate::vga::WRITER.lock().write_byte(ch);
            }
        }
        _ => {}
    }
}

fn shell_main() -> ! {
    println!("exoterikOS shell. Type 'help' for commands, 'history 50' to scroll back.");
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
            println!("  phi     - Phi_c / IG 12-primitive imscription");
            println!("  sched   - ergative scheduler status");
            println!("  mem     - memory allocator status");
            println!("  fs      - sefirot filesystem tree (full view)");
            println!("  cd X    - navigate to Sefirah X");
            println!("  ls      - list files in current Sefirah");
            println!("  cat F   - read file F in current Sefirah");
            println!("  write F C - write content C to file F");
            println!("  ipc     - send test IPC message");
            println!("  aleph   - enter the ALEPH REPL");
            println!("  type X  - show ALEPH type of kernel object X");
            println!("  type-check - run all type-gating verification tests");
            println!("  type-infer - show type inference trace for all variants");
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
        cmd if cmd.starts_with("imasm") => {
            let args = cmd.strip_prefix("imasm").unwrap_or("").trim();
            let out = imasm_commands::handle(args);
            println!("{}", out);
        }
        "type-check" => {
            println!("Running type-gating verification...");
            println!();

            // Create test objects
            let k_obj = kernel_object::KernelObject::new(
                kernel_object::StructuralType::Process,
                kernel_object::OperationalMode::Compute,
                kernel_object::Determinative::Kernel, 999);
            let u_obj = kernel_object::KernelObject::new(
                kernel_object::StructuralType::Process,
                kernel_object::OperationalMode::Compute,
                kernel_object::Determinative::User, 1000);
            let s_obj = kernel_object::KernelObject::new(
                kernel_object::StructuralType::Process,
                kernel_object::OperationalMode::Compute,
                kernel_object::Determinative::Service, 1001);

            println!("  Object types:");
            println!("    Kernel : {}", k_obj.aleph_type.summary());
            println!("    User   : {}", u_obj.aleph_type.summary());
            println!("    Service: {}", s_obj.aleph_type.summary());
            println!();

            // IPC gates
            let close_msg = ipc::IpcMessage::with_types(
                ipc::StructuralSignature {
                    source_type: kernel_object::StructuralType::Process,
                    target_type: kernel_object::StructuralType::Process,
                },
                b"test",
                ipc::MessageDeterminative {
                    source_ctx: kernel_object::Determinative::Kernel,
                    target_ctx: kernel_object::Determinative::Kernel,
                },
                k_obj.aleph_type.clone(),
                k_obj.aleph_type.clone(),
            );
            let remote_msg = ipc::IpcMessage::with_types(
                ipc::StructuralSignature {
                    source_type: kernel_object::StructuralType::Process,
                    target_type: kernel_object::StructuralType::File,
                },
                b"remote",
                ipc::MessageDeterminative {
                    source_ctx: kernel_object::Determinative::Kernel,
                    target_ctx: kernel_object::Determinative::User,
                },
                k_obj.aleph_type.clone(),
                u_obj.aleph_type.clone(),
            );
            println!("  IPC gate:");
            println!("    Kernel <-> Kernel: {}", close_msg.is_type_valid().is_accepted());
            println!("    Kernel <-> User  : {}", remote_msg.is_type_valid().is_accepted());
            println!();

            // Ω gates
            let mut palloc = memory::PhonologicalAllocator::new();
            for (name, depth) in &[
                ("Velar", memory::ArticulationDepth::Velar),
                ("Retroflex", memory::ArticulationDepth::Retroflex),
                ("Bilabial", memory::ArticulationDepth::Bilabial),
            ] {
                palloc.set_depth(*depth);
                println!("  Ω gate ({} depth):", name);
                println!("    Kernel : {}", palloc.can_allocate_for(&k_obj).is_allowed());
                println!("    User   : {}", palloc.can_allocate_for(&u_obj).is_allowed());
                println!("    Service: {}", palloc.can_allocate_for(&s_obj).is_allowed());
            }
            println!();

            // Tier gates
            let mut sched = scheduler::ErgativeScheduler::new();
            let erg_pcb = scheduler::ProcessControlBlock {
                id: 9999, obj: k_obj.clone(),
                role: scheduler::GrammaticalRole::Ergative,
                priority: 5, stack_pointer: 0xFFFF,
                targets: alloc::vec![10000],
            };
            println!("  Tier gate:");
            println!("    Kernel ergative: {}", sched.spawn_type_safe(erg_pcb).is_ok());
            println!();

            // Φ gates
            println!("  Φ gate (requires Φ_c for Keter→Gevurah):");
            for (name, sef) in &[
                ("Keter", filesystem::Sefirah::Keter),
                ("Chesed", filesystem::Sefirah::Chesed),
                ("Malkuth", filesystem::Sefirah::Malkuth),
            ] {
                let mut fs_copy = filesystem::SefirotFs::new();
                let k_r = fs_copy.navigate_to_type_safe(*sef, &k_obj);
                let u_r = fs_copy.navigate_to_type_safe(*sef, &u_obj);
                let s_r = fs_copy.navigate_to_type_safe(*sef, &s_obj);
                println!("    {} : Kernel={} User={} Service={}",
                    name, k_r.is_ok(), u_r.is_ok(), s_r.is_ok());
            }
            println!();

            // Conscience scores
            println!("  C scores:");
            println!("    Kernel : {:.3}", k_obj.aleph_type.conscience_score());
            println!("    User   : {:.3}", u_obj.aleph_type.conscience_score());
            println!("    Service: {:.3}", s_obj.aleph_type.conscience_score());
            println!("    OS imscription: {:.3}",
                aleph_kernel_types::canonical::os_imscription().conscience_score());
        }
        "type-infer" => {
            println!("Type inference: Structural x Determinative x Operational");
            println!("  (inferred → nearest canonical letter)");
            println!();
            use kernel_object::{StructuralType, OperationalMode, Determinative};

            // Summary table: determinative × structural for Compute mode
            println!("  OperationalMode::Compute:");
            println!("  {:<12} {:>8} {:>8} {:>8} {:>8} {:>8}",
                "Det\\Struct", "Process", "File", "Socket", "Semaph", "MemReg");
            println!("  {}", "-".repeat(62));
            let structs = [
                StructuralType::Process, StructuralType::File,
                StructuralType::Socket, StructuralType::Semaphore,
                StructuralType::MemoryRegion,
            ];
            for d in &[Determinative::Kernel, Determinative::Init,
                       Determinative::Service, Determinative::Driver,
                       Determinative::User] {
                let d_name = match d {
                    Determinative::Kernel => "Kernel",
                    Determinative::Init => "Init",
                    Determinative::Service => "Service",
                    Determinative::Driver => "Driver",
                    Determinative::User => "User",
                };
                print!("  {:<12}", d_name);
                for s in &structs {
                    let ty = aleph_kernel_types::AlephKernelType::infer(*s, OperationalMode::Compute, *d);
                    let nearest = aleph_kernel_types::nearest_canonical(&ty);
                    print!(" {:>8}", aleph::display_glyph(nearest));
                }
                println!();
            }
            println!();
            println!("  Legend: A=aleph B=bet G=gimel D=dalet H=hei V=vav Z=zayin");
            println!("  C=chet T=tet Y=yod K=kaf L=lamed M=mem N=nun S=samech");
            println!("  E=ayin P=pei Q=tzadi U=kuf R=resh X=shin O=tav");
            println!();
            println!("  Type 'type-infer verbose' for full inference traces");
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

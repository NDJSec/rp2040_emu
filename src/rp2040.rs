use unicorn_engine::{Arch, MemType, Mode, Permission, RegisterARM, Unicorn, SECOND_SCALE};
use unicorn_engine::HookType;
use libc::size_t;
use crate::uf2_parser::UF2File;

pub struct RP2040<'a> {
    cpu: Unicorn<'a, ()>,  // Storing Unicorn directly, no references
}

struct MemoryRegion {
    name: &'static str,
    base: u64,
    size: u64,
    permissions: Permission,
}

impl<'a> RP2040<'a> {
    pub fn new() -> Self {
        let memory_regions: [MemoryRegion; 7] = [
            MemoryRegion {
                name: "Boot ROM",
                base: 0x0000_0000,
                size: 0x4000, // 16 kB readonly memory
                permissions: Permission::READ | Permission::EXEC,
            },
            MemoryRegion {
                name: "XIP Flash",
                base: 0x1000_0000,
                size: 0x1000_0000,
                permissions: Permission::READ | Permission::EXEC,
            },
            MemoryRegion {
                name: "SRAM",
                base: 0x2000_0000,
                size: 0x1000_0000, // SRAM 264KB
                permissions: Permission::ALL,
            },
            MemoryRegion {
                name: "APB Peripherals",
                base: 0x4000_0000,
                size: 0x1000_0000,
                permissions: Permission::READ | Permission::WRITE,
            },
            MemoryRegion {
                name: "APB Lite Peripherals",
                base: 0x5000_0000,
                size: 0x1000_0000,
                permissions: Permission::READ | Permission::WRITE,
            },
            MemoryRegion {
                name: "SIO",
                base: 0xd000_0000,
                size: 0x1000_0000,
                permissions: Permission::READ | Permission::WRITE,
            },
            MemoryRegion {
                name: "Private Peripheral Bus (PPB)",
                base: 0xE000_0000,
                size: 0x2000_0000,
                permissions: Permission::READ | Permission::WRITE,
            },
        ];

        let mut cpu = Unicorn::new(Arch::ARM, Mode::THUMB)
            .expect("failed to initialize Unicorn instance");

        for region in &memory_regions {
            cpu.mem_map(region.base, region.size as size_t, region.permissions)
                .expect(&format!(
                    "Failed to map memory region: {} at 0x{:08X}",
                    region.name, region.base
                ));
            println!(
                "Mapped {} at 0x{:08X} (size: {} KB)",
                region.name,
                region.base,
                region.size / 1024
            );
        }
        RP2040 { cpu }
    }

    pub fn write_flash(&mut self, program: UF2File) {
        for block in program.blocks {
            self.cpu.mem_write(block.target_addr as u64, &block.data).expect("Failed to write block");
        }
    }

    pub fn start_emulation(&mut self) {
        let callback = move |uc: &mut Unicorn<'_, ()>| {
            // Read the program counter (PC)
            let pc = uc
                .reg_read(RegisterARM::PC)
                .expect("Failed to read PC");

            // Print the error address
            println!("Error Address: 0x{:08X}", pc);

            // Attempt to read the instruction at the PC address
            let mut buffer = [0u8; 4]; // ARM instructions can be 4 bytes
            if let Ok(_) = uc.mem_read(pc, &mut buffer) {
                println!(
                    "Failing Instruction: {:02X} {:02X} {:02X} {:02X}",
                    buffer[0], buffer[1], buffer[2], buffer[3]
                );
            } else {
                println!("Failed to read instruction memory at 0x{:08X}", pc);
            }

            false // Continue the emulation
        };
        let mem_callback = Box::new(|uc: &mut Unicorn<'_, ()>, access: MemType, address: u64, size: usize, value: i64| {
            println!(
                "Memory op detected! Type: {:?} Address: 0x{:08X}, Size: {} bytes, Value: 0x{:X}, SP: 0x{:08X}",
                access, address, size, value, uc.reg_read(RegisterARM::SP).expect("Failed to read PC")
            );
            // Return `true` to prevent the emulator from crashing on this invalid memory access
            true
        });

        self.cpu.add_code_hook(0x1000_4894, 0x1000_72D8, Box::new(|uc: &mut Unicorn<'_, ()>, address, size| {
            let pc = uc.reg_read(RegisterARM::PC).expect("Failed to read PC");
            let sp = uc.reg_read(RegisterARM::SP).expect("Failed to read SP");
            println!("Executing instruction at PC: 0x{:08X}, size: {}", pc, size);
            println!("PC: 0x{:08X}, SP: 0x{:08X}", pc, sp);

            let r0 = uc.reg_read(RegisterARM::R0).expect("Failed to read R0");
            println!("R0: 0x{:08X}", r0);
        }))
            .expect("Failed to add code hook");

        self.cpu.add_mem_hook(HookType::MEM_ALL, 0, u64::MAX, mem_callback).expect("Failed to create mem hook");
        self.cpu.add_insn_invalid_hook(callback).expect("Failed to hook code");
        self.cpu.reg_write(RegisterARM::SP, 0x20042000).expect("Failed to write SP");
        self.cpu.emu_start(0x1000_4894 | 1, 0x1000_490E, 10 * SECOND_SCALE, 0)
            .expect("Failed to start emulation");
    }
}
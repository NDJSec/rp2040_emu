use unicorn_engine::{Arch, Mode, Permission, Unicorn, SECOND_SCALE};

pub struct RP2040<'a> {
    cpu: Unicorn<'a, ()>,  // Storing Unicorn directly, no references
}

impl<'a> RP2040<'a> {
    pub fn new() -> Self {
        let arm_code32: Vec<u8> = vec![0x17, 0x00, 0x40, 0xe2]; // sub r0, #23

        // Create the Unicorn instance directly
        let mut unicorn = Unicorn::new(Arch::ARM, Mode::LITTLE_ENDIAN)
            .expect("failed to initialize Unicorn instance");

        // Map memory and write the ARM code
        unicorn.mem_map(0x1000, 0x4000, Permission::ALL)
            .expect("failed to map code page");
        unicorn.mem_write(0x1000, &arm_code32)
            .expect("failed to write instructions");

        // Return the struct with the Unicorn instance
        RP2040 { cpu: unicorn }
    }

    // Example method that could start the emulation
    pub fn start_emulation(&mut self) {
        // You can interact with `self.cpu` directly here
        // Uncomment the line below to start the emulation
        let arm_code32: Vec<u8> = vec![0x17, 0x00, 0x40, 0xe2]; // sub r0, #23
        self.cpu.emu_start(0x1000, 0x1000 + arm_code32.len() as u64, 10 * SECOND_SCALE, 1000)
            .expect("Failed to start emulation");
    }
}
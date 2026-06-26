#![no_std]
#![no_main]

mod cmds;
use cmds::Cmds;

use core::ptr::{read_volatile, write_volatile};
use flash_algorithm::*;

// ── QSPI peripheral (AHB3 at 0x52005000) ──────────────────────────────────
const QSPI: u32 = 0x5200_5000;
macro_rules! reg { ($off:expr) => { (QSPI + $off) as *mut u32 } }
const CR:   *mut u32 = reg!(0x00);
const DCR:  *mut u32 = reg!(0x04);
const SR:   *mut u32 = reg!(0x08);
const FCR:  *mut u32 = reg!(0x0C);
const DLR:  *mut u32 = reg!(0x10);
const CCR:  *mut u32 = reg!(0x14);
const AR:   *mut u32 = reg!(0x18);
const DR:   *mut u32 = reg!(0x20);
const DR8:  *mut u8  = reg!(0x20) as *mut u8;

// SR bits
const SR_TCF:  u32 = 1 << 1; // transfer complete
const SR_FTF:  u32 = 1 << 2; // FIFO threshold
const SR_BUSY: u32 = 1 << 5;

// ── RCC ───────────────────────────────────────────────────────────────────
const RCC: u32       = 0x5802_4400;
const AHB3ENR: *mut u32 = (RCC + 0xD4) as *mut u32; // bit 14 = QSPIEN
const AHB4ENR: *mut u32 = (RCC + 0xE0) as *mut u32; // bit1=GPIOB, 3=GPIOD, 4=GPIOE

// ── GPIO bases ────────────────────────────────────────────────────────────
const GPIOB: u32 = 0x5802_0400;
const GPIOD: u32 = 0x5802_0C00;
const GPIOE: u32 = 0x5802_1000;

// ── Flash constants ───────────────────────────────────────────────────────
const FLASH_BASE: u32 = 0x9000_0000;

// ── CCR helpers ───────────────────────────────────────────────────────────
// FMODE: 00=indirect write, 01=indirect read
// IMODE/ADMODE/DMODE: 00=none, 01=1-line, 10=2-line, 11=4-line
// ADSIZE: 10=24-bit, 11=32-bit

#[inline(always)]
fn ccr_cmd1(instr: u8) -> u32 {
    // Instruction only, 1-line, indirect write, no addr, no data
    (instr as u32) | (0b01 << 8)
}

#[inline(always)]
fn ccr_read1(instr: u8) -> u32 {
    // 1-line instr + 1-line data, indirect read, no addr
    (instr as u32) | (0b01 << 8) | (0b01 << 24) | (0b01 << 26)
}

#[inline(always)]
fn ccr_write4b(instr: u8) -> u32 {
    // [9:8]=IMODE=01(1-line) [11:10]=ADMODE=01(1-line) [13:12]=ADSIZE=11(32-bit) [25:24]=DMODE=01(1-line)
    (instr as u32) | (0b01 << 8) | (0b01 << 10) | (0b11 << 12) | (0b01 << 24)
}

#[inline(always)]
fn ccr_erase4b(instr: u8) -> u32 {
    // [9:8]=IMODE=01(1-line) [11:10]=ADMODE=01(1-line) [13:12]=ADSIZE=11(32-bit) no data
    (instr as u32) | (0b01 << 8) | (0b01 << 10) | (0b11 << 12)
}

// ── Low-level QSPI helpers ────────────────────────────────────────────────

fn wait_busy() {
    unsafe { while (read_volatile(SR) & SR_BUSY) != 0 {} }
}

fn clear_flags() {
    unsafe { write_volatile(FCR, 0x1F); }
}

// Send instruction-only command (e.g. WREN, Bulk Erase)
fn send_cmd(instr: u8) {
    unsafe {
        wait_busy();
        clear_flags();
        write_volatile(CCR, ccr_cmd1(instr));
        wait_busy();
    }
}

// Read status from both chips: DFM returns BK1 byte then BK2 byte (DLR=1 = 2 logical bytes).
// Returns packed byte: BK1 WIP at bit 0, BK2 WIP at bit 4 (for use with 0x11 mask).
fn read_status() -> u8 {
    unsafe {
        wait_busy();
        clear_flags();
        write_volatile(DLR, 1); // 2 logical bytes (BK1 status + BK2 status)
        write_volatile(CCR, ccr_read1(Cmds::ReadStatusRegister as u8));
        while (read_volatile(SR) & SR_TCF) == 0 {}
        let word = read_volatile(DR);
        clear_flags();
        let bk1 = (word & 0xFF) as u8;
        let bk2 = ((word >> 8) & 0xFF) as u8;
        bk1 | (bk2 << 4) // BK1 WIP→bit0, BK2 WIP→bit4
    }
}

// Poll until both WIP bits clear (DFM: bit0=BK1, bit4=BK2)
fn wait_wip() {
    loop {
        if (read_status() & 0x11) == 0 {
            break;
        }
    }
}

// ── GPIO helpers ──────────────────────────────────────────────────────────

unsafe fn gpio_setup_af(base: u32, pin: u8, af: u8) {
    let moder   = (base + 0x00) as *mut u32;
    let ospeedr = (base + 0x08) as *mut u32;
    let afrl    = (base + 0x20) as *mut u32;
    let afrh    = (base + 0x24) as *mut u32;

    // MODER: AF = 0b10
    let mut v = read_volatile(moder);
    v &= !(0b11 << (pin * 2));
    v |=   0b10 << (pin * 2);
    write_volatile(moder, v);

    // OSPEEDR: Very High = 0b11
    let mut v = read_volatile(ospeedr);
    v |= 0b11 << (pin * 2);
    write_volatile(ospeedr, v);

    // AFR: 4-bit field per pin
    if pin < 8 {
        let mut v = read_volatile(afrl);
        v &= !(0xF << (pin * 4));
        v |=  (af as u32) << (pin * 4);
        write_volatile(afrl, v);
    } else {
        let p = pin - 8;
        let mut v = read_volatile(afrh);
        v &= !(0xF << (p * 4));
        v |=  (af as u32) << (p * 4);
        write_volatile(afrh, v);
    }
}

// ── Flash algorithm ───────────────────────────────────────────────────────

struct Algorithm;

algorithm!(Algorithm, {
    device_name: "MT25QL512_DFM_WomoLIN",
    device_type: DeviceType::Ext8Bit,
    flash_address: 0x90000000,
    flash_size:    0x08000000,   // 128 MB logical (2 × 64 MB chips in DFM)
    page_size:     0x200,        // 512 B logical page  (2 × 256 B physical)
    empty_value:   0xFF,
    program_time_out: 1000,
    erase_time_out:   20000,
    sectors: [{
        size:    0x2000,         // 8 KB logical sector (2 × 4 KB physical)
        address: 0x0,
    }]
});

impl FlashAlgorithm for Algorithm {
    fn new(
        _address: u32,
        _clock: u32,
        _function: Function,
    ) -> Result<Self, ErrorCode> {
        unsafe {
            // ── RCC: enable GPIOB, GPIOD, GPIOE, QSPI ──────────────────
            write_volatile(AHB4ENR, read_volatile(AHB4ENR) | (1<<1) | (1<<3) | (1<<4));
            write_volatile(AHB3ENR, read_volatile(AHB3ENR) | (1<<14));
            // Ensure clocks propagate (read-back)
            let _ = read_volatile(AHB3ENR);

            // ── GPIO AF configuration ────────────────────────────────────
            // PB2 → QSPI_CLK     AF9
            gpio_setup_af(GPIOB, 2, 9);
            // PB6 → QSPI_BK1_NCS AF10
            gpio_setup_af(GPIOB, 6, 10);
            // PD11 → BK1_IO0  AF9
            gpio_setup_af(GPIOD, 11, 9);
            // PD12 → BK1_IO1  AF9
            gpio_setup_af(GPIOD, 12, 9);
            // PD13 → BK1_IO3  AF9
            gpio_setup_af(GPIOD, 13, 9);
            // PE2  → BK1_IO2  AF9
            gpio_setup_af(GPIOE, 2, 9);
            // PE7  → BK2_IO0  AF10
            gpio_setup_af(GPIOE, 7, 10);
            // PE8  → BK2_IO1  AF10
            gpio_setup_af(GPIOE, 8, 10);
            // PE9  → BK2_IO2  AF10
            gpio_setup_af(GPIOE, 9, 10);
            // PE10 → BK2_IO3  AF10
            gpio_setup_af(GPIOE, 10, 10);

            // ── Abort any ongoing QSPI transaction ───────────────────────
            write_volatile(CR, read_volatile(CR) | (1 << 1)); // ABORT
            while (read_volatile(CR) & (1 << 1)) != 0 {}

            // ── QSPI CR: disable first ────────────────────────────────────
            write_volatile(CR, 0);

            // ── DCR: FSIZE=26 (total 128 MB in DFM), CSHT=1 ─────────────
            // FSIZE bits [20:16], CSHT bits [10:8]
            write_volatile(DCR, (26_u32 << 16) | (1_u32 << 8));

            // ── CR: DFM=1(bit6), PRESCALER=1(bits[31:24]), EN=1(bit0) ───
            // PRESCALER=1 → divide AHB by 2 (safe conservative rate)
            write_volatile(CR, (1_u32 << 24) | (1_u32 << 6) | (1_u32 << 0));

            wait_busy();
        }
        Ok(Self)
    }

    fn erase_all(&mut self) -> Result<(), ErrorCode> {
        send_cmd(Cmds::WriteEnable as u8);
        send_cmd(Cmds::BulkErase as u8);
        wait_wip();
        Ok(())
    }

    fn erase_sector(&mut self, address: u32) -> Result<(), ErrorCode> {
        // Use the logical offset directly. In DFM mode the QUADSPI hardware
        // already divides AR by 2 when sending the physical address to each chip
        // (AR[ADSIZE:1] is sent). Manual division would cause a double-shift.
        let offset = address - FLASH_BASE;
        send_cmd(Cmds::WriteEnable as u8);
        unsafe {
            wait_busy();
            clear_flags();
            write_volatile(CCR, ccr_erase4b(Cmds::FourByte4KbSubsectorErase as u8));
            write_volatile(AR, offset);
            wait_busy();
        }
        wait_wip();
        Ok(())
    }

    fn program_page(&mut self, address: u32, data: &[u8]) -> Result<(), ErrorCode> {
        let offset = address - FLASH_BASE;
        send_cmd(Cmds::WriteEnable as u8);
        unsafe {
            wait_busy();
            clear_flags();
            write_volatile(DLR, (data.len() as u32) - 1);
            write_volatile(CCR, ccr_write4b(Cmds::FourBytePageProgram as u8));
            write_volatile(AR, offset);
            // Write data into FIFO
            for &byte in data {
                while (read_volatile(SR) & (SR_FTF | SR_TCF)) == 0 {}
                write_volatile(DR8, byte);
            }
            while (read_volatile(SR) & SR_TCF) == 0 {}
            clear_flags();
            wait_busy();
        }
        wait_wip();
        Ok(())
    }
}

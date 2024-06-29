#![no_std]
#![no_main]

use cortex_m_rt::entry;
use flash_algorithm::{FlashAlgorithm, Function};
use panic_probe as _;
use stm32h730xx_is25lp128::Algorithm;
use rtt_target::rprintln;

#[entry]
fn main() -> ! {
    let addr = 0x0_u32;
    let data = "Hello SPI!".as_bytes();

    let mut algo = Algorithm::new(0, 0, Function::Verify).unwrap();

    algo.erase_all().unwrap();
    // algo.erase_sector(addr).unwrap();
    algo.program_page(addr, data).unwrap();
    match algo.verify(addr, data.len() as u32, Some(data)) {
        Ok(_) => rprintln!("Verified."),
        Err(_) => rprintln!("Not verified."),
    }
    loop {}
}

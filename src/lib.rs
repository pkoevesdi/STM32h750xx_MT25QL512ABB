#![no_std]
#![no_main]

// use core::mem::MaybeUninit;

use core::cmp::min;
use panic_probe as _;
use flash_algorithm::*;
use rtt_target::{rprintln, rtt_init_print};
use stm32h7xx_hal::gpio::Speed;
use stm32h7xx_hal::pac::OCTOSPI1;
use stm32h7xx_hal::{pac, prelude::*, xspi::Octospi, xspi::OctospiMode, xspi::OctospiWord};

mod cmds;
mod macros;

pub struct Algorithm {
    octospi: Octospi<OCTOSPI1>,
}

algorithm!(Algorithm, {
    flash_address: 0x0000000,
    flash_size: 0x1000000,
    page_size: 0x100,
    empty_value: 0xff,
    sectors: [{
        size: 0x1000,
        address: 0x0000000,
    }]
});

impl FlashAlgorithm for Algorithm {
    fn new(_address: u32, _clock: u32, _function: Function) -> Result<Self, ErrorCode> {
        rtt_init_print!();
        // rprintln!("Initialising Flash...");

        let dp = pac::Peripherals::take().unwrap();

        // Constrain and Freeze power
        let pwr = dp.PWR.constrain();
        let pwrcfg = pwr.freeze();

        // Constrain and Freeze clock
        let rcc = dp.RCC.constrain();
        let ccdr = rcc.sys_ck(64.MHz());
        let ccdr=ccdr.freeze(pwrcfg, &dp.SYSCFG);
        
        // Acquire the GPIO peripherals. This also enables the clock for
        // the GPIOs in the RCC register.
        let gpioa = dp.GPIOA.split(ccdr.peripheral.GPIOA);
        let gpioc = dp.GPIOC.split(ccdr.peripheral.GPIOC);
        let gpioe = dp.GPIOE.split(ccdr.peripheral.GPIOE);
        
        // "All GPIOs have to be configured in very high-speed configuration." - AN5050, p. 30
        let _ncs = gpioc.pc11.into_alternate::<9>().speed(Speed::VeryHigh);
        let _clk = gpioa.pa3.into_alternate::<12>().speed(Speed::VeryHigh);
        let _io0 = gpioc.pc9.into_alternate::<9>().speed(Speed::VeryHigh);
        let _io1 = gpioc.pc10.into_alternate::<9>().speed(Speed::VeryHigh);
        let _io2 = gpioe.pe2.into_alternate::<9>().speed(Speed::VeryHigh);
        let _io3 = gpioa.pa6.into_alternate::<6>().speed(Speed::VeryHigh);
        
        // Initialise the OCTOSPI peripheral.
        let mut octospi =
        dp.OCTOSPI1
        .octospi_unchecked(1.MHz(), &ccdr.clocks, ccdr.peripheral.OCTOSPI1);
    
    // switch to QPI mode
    octospi
    .write_extended(
        OctospiWord::U8(cmds::Cmds::Qpien as u8),
        OctospiWord::None,
        OctospiWord::None,
        &[],
        )
        .unwrap();
    
    // Change bus mode
    octospi.configure_mode(OctospiMode::FourBit).unwrap();
    
    rprintln!("Freezing clock...");
    Ok(Self { octospi })
    }
    
    fn erase_all(&mut self) -> Result<(), ErrorCode> {
        rprintln!("Erasing All...");

        write_enable!(self);

        self.octospi
            .write_extended(
                OctospiWord::U8(cmds::Cmds::Cer as u8),
                OctospiWord::None,
                OctospiWord::None,
                &[],
            )
            .unwrap();

        wait_for_flash!(self);

        Err(ErrorCode::new(0x70d0).unwrap())
    }

    fn erase_sector(&mut self, addr: u32) -> Result<(), ErrorCode> {
        rprintln!("Erasing sector addr: {:#2x}...", addr);

        write_enable!(self);

        self.octospi
            .write_extended(
                OctospiWord::U8(cmds::Cmds::Ser as u8),
                OctospiWord::U24(addr),
                OctospiWord::None,
                &[],
            )
            .unwrap();

        wait_for_flash!(self);
        Ok(())
    }

    fn program_page(&mut self, addr: u32, data: &[u8]) -> Result<(), ErrorCode> {
        rprintln!(
            "Programming Page addr: {:#02x}, size: {} with data: {:2x?}...",
            addr,
            data.len(),
            data
        );

        write_enable!(self);

        self.octospi
            .write_extended(
                OctospiWord::U8(cmds::Cmds::Pp as u8),
                OctospiWord::U24(addr),
                OctospiWord::None,
                data,
            )
            .unwrap();

        wait_for_flash!(self);

        Ok(())
    }

    fn verify(&mut self, addr: u32, size: u32, data: Option<&[u8]>) -> Result<(), ErrorCode> {
        const BUFSIZE: usize = 64;

        let data = data.unwrap_or_default();
        let size = min(size as usize, data.len());

        rprintln!(
            "Reading Page addr: {:#02x}, size: {}, comparing with data: {:2x?}...",
            addr,
            size,
            data
        );

        let mut read = [0_u8; BUFSIZE];

        for i in (0..size).step_by(BUFSIZE) {
            let lim = min(size - i, BUFSIZE);
            self.octospi
                .read_extended(
                    OctospiWord::U8(cmds::Cmds::Frqio as u8),
                    OctospiWord::U24(addr + i as u32),
                    OctospiWord::U8(0xa0),
                    4,
                    &mut read[0..lim],
                )
                .unwrap();
            wait_for_flash!(self);
            rprintln!(
                "Read page addr: {:#02x}, size: {}, data: {:2x?}",
                addr + i as u32,
                lim,
                &read[0..lim]
            );
   
            if &read[0..lim] != &data[i..i + lim] {
                return Err(ErrorCode::new(0x1).unwrap());
            }
        }
        Ok(())
    }
}

impl Drop for Algorithm {
    fn drop(&mut self) {
        // TODO: Add code here to uninitialize the flash algorithm.
    }
}

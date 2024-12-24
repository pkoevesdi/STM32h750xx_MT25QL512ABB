#![no_std] // from template
#![no_main] // from template

use flash_algorithm::*; // from template
use rtt_target::{rprintln, rtt_init_print}; // from template

mod cmds;
mod macros;

use stm32h7xx_hal::gpio::Speed;
use stm32h7xx_hal::pac::QUADSPI;
// use stm32h7xx_hal::xspi::BankSelect;
use stm32h7xx_hal::{pac, prelude::*, xspi::Qspi, xspi::QspiMode, xspi::QspiWord};

// struct Algorithm; // from template

pub struct Algorithm {
    quadspi: Qspi<QUADSPI>,
}

// from initialization:
algorithm!(Algorithm, {
    flash_address: 0x90000000,
    flash_size: 0x8000000,
    page_size: 0x100,
    empty_value: 0xFF,
    sectors: [{
        size: 0x10000,
        address: 0x0,
    }]
});

// Rest: empty functions from template
impl FlashAlgorithm for Algorithm {
    fn new(_address: u32, _clock: u32, _function: Function) -> Result<Self, ErrorCode> {
        rtt_init_print!(); // from template
        rprintln!("Init"); // from template

        let dp = pac::Peripherals::take().unwrap();

        // Constrain and Freeze power
        let pwr = dp.PWR.constrain();
        let pwrcfg = pwr.freeze();

        // Constrain and Freeze clock
        let rcc = dp
            .RCC
            .constrain()
            .use_hse(1.MHz()) // use (and thus test) external clock - "Will result in a hang if an external oscillator is not connected or it fails to start." - https://docs.rs/stm32h7xx-hal/latest/stm32h7xx_hal/rcc/struct.Rcc.html#method.use_hse
            .sys_ck(64.MHz());

        rprintln!("            Freezing the core clocks...");
        let ccdr = rcc.freeze(pwrcfg, &dp.SYSCFG);

        rprintln!("            hse_ck: {}", ccdr.clocks.hse_ck().unwrap());
        rprintln!("            sys_ck: {}", ccdr.clocks.sys_ck());
        rprintln!("            hclk: {:}", ccdr.clocks.hclk());

        let gpiob = dp.GPIOB.split(ccdr.peripheral.GPIOB);
        let gpioc = dp.GPIOC.split(ccdr.peripheral.GPIOC);
        let gpiod = dp.GPIOD.split(ccdr.peripheral.GPIOD);
        let gpioe = dp.GPIOE.split(ccdr.peripheral.GPIOE);

        // "All GPIOs have to be configured in very high-speed configuration." - AN5050, p. 30
        let clk = gpiob.pb2.into_alternate::<9>().speed(Speed::VeryHigh);
        let _bk1_ncs = gpiob.pb6.into_alternate::<10>().speed(Speed::VeryHigh);
        let _bk2_ncs = gpioc.pc11.into_alternate::<9>().speed(Speed::VeryHigh);
        let bk1_io0 = gpiod.pd11.into_alternate::<9>().speed(Speed::VeryHigh);
        let bk1_io1 = gpiod.pd12.into_alternate::<9>().speed(Speed::VeryHigh);
        let bk1_io2 = gpioe.pe2.into_alternate::<9>().speed(Speed::VeryHigh);
        let bk1_io3 = gpiod.pd13.into_alternate::<9>().speed(Speed::VeryHigh);
        let _bk2_io0 = gpioe.pe7.into_alternate::<10>().speed(Speed::VeryHigh);
        let _bk2_io1 = gpioe.pe8.into_alternate::<10>().speed(Speed::VeryHigh);
        let _bk2_io2 = gpioe.pe9.into_alternate::<10>().speed(Speed::VeryHigh);
        let _bk2_io3 = gpioe.pe10.into_alternate::<10>().speed(Speed::VeryHigh);

        // Initialise the SPI peripheral.
        let mut quadspi = dp.QUADSPI.bank1(
            (clk, bk1_io0, bk1_io1, bk1_io2, bk1_io3),
            75.MHz(),
            &ccdr.clocks,
            ccdr.peripheral.QSPI,
        );

        // switch to QPI mode
        quadspi
            .write_extended(
                QspiWord::U8(cmds::Cmds::Qpien as u8),
                QspiWord::None,
                QspiWord::None,
                &[],
            )
            .unwrap();

        quadspi
            .inner_mut()
            .dcr
            .modify(|_, w| unsafe { w.fsize().bits(26) }); // set flash size to 2^27 bytes (= 2*512 Mbit)

        // Change bus mode
        quadspi.configure_mode(QspiMode::OneBit).unwrap();

        rprintln!("            done.");

        Ok(Self { quadspi })

        // Ok(Self) // from template
    }

    fn erase_all(&mut self) -> Result<(), ErrorCode> {
        rprintln!("Erase All");// from template
        // TODO: Add code here that erases the entire flash.
        Err(ErrorCode::new(0x70d0).unwrap())// from template
    }

    fn erase_sector(&mut self, addr: u32) -> Result<(), ErrorCode> {
        rprintln!("Erase sector addr:{}", addr);// from template
        // TODO: Add code here that erases a page to flash.
        Ok(())// from template
    }

    fn program_page(&mut self, addr: u32, data: &[u8]) -> Result<(), ErrorCode> {
        rprintln!("Program Page addr:{} size:{}", addr, data.len());// from template
        // TODO: Add code here that writes a page to flash.
        Ok(())// from template
    }
}

impl Drop for Algorithm {
    fn drop(&mut self) {
        // TODO: Add code here to uninitialize the flash algorithm.
    }
}

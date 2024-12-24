#![no_std] // from template
#![no_main] // from template

use core::borrow::{Borrow, BorrowMut};

use panic_probe as _;

use flash_algorithm::*; // from template
use rtt_target::{rprintln, rtt_init_print}; // from template

use stm32h7xx_hal::gpio::Speed;
use stm32h7xx_hal::pac::QUADSPI;
use stm32h7xx_hal::xspi::BankSelect;
use stm32h7xx_hal::{pac, prelude::*, xspi::Qspi, xspi::QspiError, xspi::QspiMode, xspi::QspiWord};

// struct Algorithm; // from template

pub struct Algorithm {
    quadspi: Qspi<QUADSPI>,
}


// MT25QL512ABB
// from initialization:
algorithm!(Algorithm, {
    device_name: "MT25QL512ABB",
    device_type: DeviceType::ExtSpi,
    flash_address: 0x90000000,
    flash_size: 0x4000000,
    page_size: 0x100,
    empty_value: 0xFF,
    program_time_out: 1000,
    erase_time_out: 20000,
    sectors: [{
        size: 0x1000,
        address: 0x0,
    }]
});

fn wait_for_finish(qspi: &mut Qspi<QUADSPI>) -> u8 {
    let mut read = [1; 1];
    while read[0] & 1 == 1 {
        qspi.read(0x05, &mut read).unwrap();
    }
    read[0]
}

fn wait_for_finish_dual(qspi: &mut Qspi<QUADSPI>) -> u8 {
    let mut read = [1; 2];
    while read[1] & 1 == 1 {
        qspi.read(0x05, &mut read).unwrap();
    }
    read[0]
}

fn wren(qspi: &mut Qspi<QUADSPI>) -> Result<(), QspiError> {
    let res = qspi.write_extended(QspiWord::U8(0x06), QspiWord::None, QspiWord::None, &[]);
    wait_for_finish(qspi);
    res
}

fn nord(qspi: &mut Qspi<QUADSPI>, addr: u32, data: &mut [u8]) -> Result<(), QspiError> {
    // NORMAL READ
    // take an offset as 0x9000000 into account (the address should start with 0)
    let address = addr - 0x90000000;
    let mut offset = 0;
    while offset < data.len() {
        let chunk_size = core::cmp::min(32, data.len() - offset);
        let res = qspi.read_extended(
            QspiWord::U8(0x03),
            QspiWord::U24(address + offset as u32),
            QspiWord::None,
            0,
            &mut data[offset..offset + chunk_size],
        );
        match res {
            Ok(_) => {
                // rprintln!("Read chunk done");
            }
            Err(e) => {
                return Err(e);
            }
        }
        offset += chunk_size;
    }
    Ok(())
}

fn pp(qspi: &mut Qspi<QUADSPI>, addr: u32, data: &[u8]) -> Result<(), QspiError> {
    let address = addr - 0x90000000;
    let mut offset = 0;

    while offset < data.len() {
        let chunk_size = core::cmp::min(32, data.len() - offset);
        let chunk = &data[offset..offset + chunk_size];

        let _ = wren(qspi);

        // PAGE PROGRAM OPERATION (PP, 02h)
        let res = qspi.write_extended(
            QspiWord::U8(0x02),
            QspiWord::U24(address + offset as u32),
            QspiWord::None,
            chunk,
        );

        match res {
            Ok(_) => {
                wait_for_finish(qspi);
            }
            Err(e) => {
                return Err(e);
            }
        }

        offset += chunk_size;
    }

    Ok(())
}

fn ser(qspi: &mut Qspi<QUADSPI>, inst: u8, addr: u32) -> Result<(), QspiError> {
    // SECTOR ERASE

    let address = addr - 0x90000000;

    let _ = wren(qspi);
    let res = qspi.write_extended(QspiWord::U8(inst), QspiWord::U24(address), QspiWord::None, &[]);
    wait_for_finish(qspi);
    res
}

fn ser_all(qspi: &mut Qspi<QUADSPI>, inst: u8) -> Result<(), QspiError> {
    // ERASE ENTIRE CHIP

    let _ = wren(qspi);
    let res = qspi.write_extended(QspiWord::U8(inst), QspiWord::None, QspiWord::None, &[]);
    wait_for_finish(qspi);
    res
}

// Rest: empty functions from template
impl FlashAlgorithm for Algorithm {
    fn new(_address: u32, _clock: u32, function: Function) -> Result<Self, ErrorCode> {
        rtt_init_print!(); // from template
        rprintln!("Init with function {:?}", function); // from template

        let dp = unsafe { pac::Peripherals::steal() };

        // Constrain and Freeze power
        let pwr = dp.PWR.constrain();
        let pwrcfg = pwr.freeze();

        // Constrain and Freeze clock
        let rcc = dp
            .RCC
            .constrain()
            .use_hse(25.MHz()) // use (and thus test) external clock - "Will result in a hang if an external oscillator is not connected or it fails to start." - https://docs.rs/stm32h7xx-hal/latest/stm32h7xx_hal/rcc/struct.Rcc.html#method.use_hse
            .sys_ck(180.MHz())
            .pll1_q_ck(45.MHz());

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
    }

    fn erase_all(&mut self) -> Result<(), ErrorCode> {

        let res = ser_all(self.quadspi.borrow_mut(), 0x60);

        match res {
            Ok(_) => {
                Ok(())
            }
            Err(e) => {
                Err(ErrorCode::new(42 as u32).unwrap()) // from template
            }
        }
    }

    fn erase_sector(&mut self, addr: u32) -> Result<(), ErrorCode> {
        let fsel = self.quadspi.inner_mut().cr.read().fsel().bit() as i8 + 1;

        let res = ser(self.quadspi.borrow_mut(), 0x20, addr);
        match res {
            Ok(_) => {
                // wait_for_finish(self.quadspi.borrow_mut());
                // rprintln!("Erase sector done");
                Ok(())
            }
            Err(e) => {
                // rprintln!("Error: {:?}", e);
                Err(ErrorCode::new(0x70d0).unwrap())
            }
        }

        // ERROR probe_rs::flashing::flasher: RTT could not be initialized: RTT control block not found in target memory.
        // - Make sure RTT is initialized on the target, AND that there are NO target breakpoints before RTT initialization.
        // - For VSCode and probe-rs-debugger users, using `halt_after_reset:true` in your `launch.json` file will prevent RTT
        // initialization from happening on time.
        // - Depending on the target, sleep modes can interfere with RTT.
    }

    fn verify(&mut self, address: u32, size: u32, data: Option<&[u8]>) -> Result<(), ErrorCode> 
    {
        let mut array: [u8; 256] = [0; 256];

        nord(self.quadspi.borrow_mut(), address, &mut array);

        // compare the read data with the data
        if let Some(data) = data {
            if array != data {
                rprintln!("Verify failed");
                return Err(ErrorCode::new(42 as u32).unwrap());
            }
        }

        Ok(())
    }

    fn program_page(&mut self, addr: u32, data: &[u8]) -> Result<(), ErrorCode> {

        let res = pp(self.quadspi.borrow_mut(), addr, data);

        match res {
            Ok(_) => {
                Ok(())
            }
            Err(e) => {
                Err(ErrorCode::new(42 as u32).unwrap()) // from template
            }
        }
    }

    fn read_flash(
        &mut self,
        address: u32,
        data: &mut [u8]
    ) -> Result<(), ErrorCode> {

        // TODO: rtt print doesn't work in this function!

        let res = nord(self.quadspi.borrow_mut(), address, data);
        match res {
            Ok(_) => {
                Ok(())
            }
            Err(e) => {
                Err(ErrorCode::new(42 as u32).unwrap()) // from template
            }
        }
    }
}

impl Drop for Algorithm {
    fn drop(&mut self) {
        rprintln!("Drop");// from template
        // drop the dp pack
        unsafe { pac::Peripherals::steal() };

        // read first 32 bytes from flash for simple verification
        let mut buf= [0; 32];
        nord(self.quadspi.borrow_mut(), 0x90000000, &mut buf);
        rprintln!("Read after drop: {:x?}", buf);
    }
}

#[allow(dead_code)]
pub(crate) enum Cmds {
    Nord = 0x03,      // Normal Read Mode
    Frd = 0x0B,       // Fast Read Mode
    Frdio = 0xBB,     // Fast Read Dual I/O
    Frdo = 0x3B,      // Fast Read Dual Output
    Frqio = 0xEB,     // Fast Read Quad I/O
    Frqo = 0x6B,      // Fast Read Quad Output
    Frdtr = 0x0D,     // Fast Read DTR Mode
    Frddtr = 0xBD,    // Fast Read Dual I/O DTR
    Frqdtr = 0xED,    // Fast Read Quad I/O DTR
    Pp = 0x02,        // Input Page Program
    Ppq = 0x32,       // Quad Input Page Program
    PpqAlt = 0x38,    // Quad Input Page Program Alternative
    Ser = 0x20,       // Sector Erase
    SerAlt = 0xD7,    // Sector Erase Alternative
    Ber32 = 0x52,     // Block Erase 32KB
    Ber64 = 0xD8,     // Block Erase 64KB
    Cer = 0x60,       // Chip Erase
    CerAlt = 0xC7,    // Chip Erase Alternative
    Wren = 0x06,      // Write Enable
    Wrdi = 0x04,      // Write Disable
    Rdsr = 0x05,      // Read Status Register
    Wrsr = 0x01,      // Write Status Register
    Rdfr = 0x48,      // Read Function Register
    Wrfr = 0x42,      // Write Function Register
    Qpien = 0x35,     // Enter QPI mode
    Qpidi = 0xF5,     // Exit QPI mode
    Persus = 0x75,    // Suspend during program/erase
    PersusAlt = 0xB0, // Suspend during program/erase Alternative
    Perrsm = 0x7A,    // Resume program/erase
    PerrsmAlt = 0x30, // Resume program/erase Alternative
    Dp = 0xB9,        // Deep Power Down
    Rdid = 0xAB,      // Read ID
    // Rdpd = 0xAB,      // Release Power Down
    Srp = 0xC0,       // Set Read Parameters
    Rdjdid = 0x9F,    // Read JEDEC ID Command
    Rdmdid = 0x90,    // Read Manufacturer & Device ID
    Rdjdidq = 0xAF,   // Read JEDEC ID QPI mode
    Rduid = 0x4B,     // Read Unique ID
    Rdsfdp = 0x5A,    // SFDP Read
    Rsten = 0x66,     // Software Reset Enable
    Rst = 0x99,       // Software Reset
    Irer = 0x64,      // Erase Information Row
    Irp = 0x62,       // Program Information Row
    Irrd = 0x68,      // Read Information Row
    Secunlock = 0x26, // Sector Unlock
    Seclock = 0x24,   // Sector Lock
}
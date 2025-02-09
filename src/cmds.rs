#[allow(dead_code)]
pub(crate) enum Cmds {
    // SPI Commands of NOR flash MT25QL512ABB
    // extracted from data sheet https://www.micron.com/products/storage/nor-flash/serial-nor/part-catalog/part-detail/mt25ql512abb8e12-0aat

    // Software RESET Operations
    ResetEnable = 0x66,
    ResetMemory = 0x99,
    // READ ID Operations
    ReadId = 0x9E,
    ReadIdAlternative = 0x9F,
    MultipleIOReadId = 0xAF,
    ReadSerialFlashDiscoveryParameter = 0x5A,
    // READ MEMORY Operations
    Read = 0x03,
    FastRead = 0x0B,
    DualOutputFastRead = 0x3B,
    DualInputOutputFastRead = 0xBB,
    QuadOutputFastRead = 0x6B,
    QuadInputOutputFastRead = 0xEB,
    DtrFastRead = 0x0D,
    DtrDualOutputFastRead = 0x3D,
    DtrDualInputOutputFastRead = 0xBD,
    DtrQuadOutputFastRead = 0x6D,
    DtrQuadInputOutputFastRead = 0xED,
    QuadInputOutputWordRead = 0xE7,
    // READ MEMORY Operations with FOUR-BYTE Address
    FourByteRead = 0x13,
    FourByteFastRead = 0x0C,
    FourByteDualOutputFastRead = 0x3C,
    FourByteDualInputOutputFastRead = 0xBC,
    FourByteQuadOutputFastRead = 0x6C,
    FourByteQuadInputOutputFastRead = 0xEC,
    FourByteDtrFastRead = 0x0E,
    FourByteDtrDualInputOutputFastRead = 0xBE,
    FourByteDtrQuadInputOutputFastRead = 0xEE,
    // WRITE Operations
    WriteEnable = 0x06,
    WriteDisable = 0x04,
    // READ REGISTER Operations
    ReadStatusRegister = 0x05,
    ReadFlagStatusRegister = 0x70,
    ReadNonvolatileConfiguRationRegister = 0xB5,
    ReadVolatileConfigurationregister = 0x85,
    ReadEnhancedVolatileConFigurationRegister = 0x65,
    ReadExtendedAddressRegIster = 0xC8,
    ReadGeneralPurposeReadRegister = 0x96,
    // WRITE REGISTER Operations
    WriteStatusRegister = 0x01,
    WriteNonvolatileConfiguRationRegister = 0xB1,
    WriteVolatileConfiguraTionRegister = 0x81,
    WriteEnhancedVolatileConfigurationRegister = 0x61,
    WriteExtendedAddressRegIster = 0xC5,
    // CLEAR FLAG STATUS REGISTER Operation
    ClearFlagStatusRegister = 0x50,
    // PROGRAM Operations
    PageProgram = 0x02,
    DualInputFastProgram = 0xA2,
    ExtendedDualInputFastProgram = 0xD2,
    QuadInputFastProgram = 0x32,
    ExtendedQuadInputFastProgram = 0x38,
    // PROGRAM Operations with FOUR-BYTE Address
    FourBytePageProgram = 0x12,
    FourByteQuadInputFastProGram = 0x34,
    FourByteQuadInputExtendedFastProgram = 0x3E,
    // ERASE Operations
    Subsector32KbErase = 0x52,
    Subsector4KbErase = 0x20,
    SectorErase = 0xD8,
    BulkErase = 0xC7,
    BulkEraseAlternative = 0x60,
    // ERASE Operations with FOUR-BYTE Address
    FourByteSectorErase = 0xDC,
    FourByte4KbSubsectorErase = 0x21,
    FourByte32KbSubsectorErase = 0x5C,
    // SUSPEND/RESUME Operations
    ProgramEraseSuspend = 0x75,
    ProgramEraseResume = 0x7A,
    // ONE-TIME PROGRAMMABLE (OTP) Operations
    ReadOtpArray = 0x4B,
    ProgramOtpArray = 0x42,
    // FOUR-BYTE ADDRESS MODE Operations
    EnterFourByteAddressMode = 0xB7,
    ExitFourByteAddressMode = 0xE9,
    // QUAD PROTOCOL Operations
    EnterQuadInputOutputMode = 0x35,
    ResetQuadInputOutputMode = 0xF5,
    // Deep Power-Down Operations
    EnterDeepPowerDown = 0xB9,
    ReleaseFromDeepPowerDown = 0xAB,
    // ADVANCED SECTOR PROTECTION Operations
    ReadSectorProtection = 0x2D,
    ProgramSectorProtection = 0x2C,
    ReadVolatileLockBits = 0xE8,
    WriteVolatileLockBits = 0xE5,
    ReadNonvolatileLockBits = 0xE2,
    WriteNonvolatileLockBits = 0xE3,
    EraseNonvolatileLockBits = 0xE4,
    ReadGlobalFreezeBit = 0xA7,
    WriteGlobalFreezeBit = 0xA6,
    ReadPassword = 0x27,
    WritePassword = 0x28,
    UnlockPassword = 0x29,
    // ADVANCED SECTOR PROTECTION Operations with FOUR-BYTE Address
    FourByteReadVolatileLockBits = 0xE0,
    FourByteWriteVolatileLockBits = 0xE1,
    // ADVANCED FUNCTION INTERFACE Operations
    InterfaceActivation = 0x9B,
    // CyclicRedundancyCheck = 0x9B,
    // CyclicRedundancyCheckAlternative = 0x27,
}

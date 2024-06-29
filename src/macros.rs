#![macro_use]

#[macro_export]
macro_rules! write_enable {
    ($self:ident) => {
        // Set WREN bit
        $self
            .octospi
            .write_extended(
                OctospiWord::U8(cmds::Cmds::Wren as u8),
                OctospiWord::None,
                OctospiWord::None,
                &[],
            )
            .unwrap();
        wait_for_flash!($self);
    };
}

#[macro_export]
macro_rules! wait_for_flash {
    ($self:ident) => {
        while $self.octospi.is_busy().is_err() {}

        let mut read = [0xff];
        while read[0] & 1_u8 == 1 {
            $self
                .octospi
                .read(cmds::Cmds::Rdsr as u8, &mut read)
                .unwrap();
        }
    };
}

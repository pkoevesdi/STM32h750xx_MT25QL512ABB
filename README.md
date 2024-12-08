# Flash Algorithm for STM32h750 with dual external NOR flash MT25QL512ABB  

This is a flash algorithm for usage with `probe-rs`.
Works with the current (2024-11-30) `master` of `probe-rs`, which can be intalled with `cargo install probe-rs-tools target-gen --git https://github.com/probe-rs/probe-rs --locked --force`

It is instantiated by
```bash
cargo generate gh:probe-rs/flash-algorithm-template --name=stm32h750xx_mt25_ql512 \
-d target-arch=thumbv7em-none-eabihf \
-d ram-start-address=0x20000000 \
-d ram-size=0x20000 \
-d flash-start-address=0x90000000 \
-d flash-size=0x8000000 \
-d flash-page-size=0x100 \
-d flash-sector-size=0x10000 \
-d empty-byte-value=0xff
```

## Developing the algorithm

Just run `cargo run`. It spits out the flash algo in the probe-rs YAML format and downloads it onto a target and makes a test run.
You will also be able to see RTT messages.

You can find the generated YAML in `target/definition.yaml`.

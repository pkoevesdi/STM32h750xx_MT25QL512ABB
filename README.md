# Flash Algorithm for STM32H750 — Dual External NOR Flash MT25QL512ABB (DFM)

Flash algorithm for use with [probe-rs](https://probe.rs/), targeting the WomoLIN.power v3 board:
two Micron MT25QL512ABB chips wired in **Dual Flash Mode (DFM)** — byte-interleaved, 128 MB logical.

**No patched probe-rs needed.** This implementation uses
[`flash-algorithm`](https://crates.io/crates/flash-algorithm) v0.7.0, which is natively supported
by standard probe-rs 0.24 and later (including 0.31).

This is a rewrite of the original single-bank algorithm (`stm32h750xx_mt25_ql512`), adding DFM
support and removing the `stm32h7xx-hal` dependency in favour of direct MMIO register access.
The full MT25QL512ABB command set from the Micron datasheet is available in `src/cmds.rs`.

## Hardware

- **MCU**: STM32H750VBTx (C2 revision), 180 MHz
- **Flash**: 2 × Micron MT25QL512ABB, CE# tied together to PB6 (BK1_NCS); PC11 (BK2_NCS) unused
- **Mode**: QSPI Dual Flash Mode (DFM) — QUADSPI CR bit 6
- **Logical address space**: `0x90000000–0x98000000` (128 MB)

## Flash Configuration

| Parameter        | Value                 | Notes                                         |
|------------------|-----------------------|-----------------------------------------------|
| Flash base       | `0x90000000`          |                                               |
| Flash size       | `0x08000000` (128 MB) | 2 × 64 MB chips byte-interleaved              |
| Page size        | `0x200` (512 B)       | 2 × 256 B physical pages                      |
| Sector size      | `0x2000` (8 KB)       | 2 × 4 KB physical subsectors                  |
| Empty value      | `0xFF`                |                                               |
| FSIZE            | 26                    | DCR bits [20:16]                              |
| XIP read command | `0x6C`                | 4-Byte Quad Output Fast Read                  |
| Program command  | `0x12`                | 4-Byte Page Program (`FourBytePageProgram`)   |
| Erase command    | `0x21`                | 4-Byte 4 KB Subsector Erase                   |
| Address width    | 32-bit                | ADSIZE = 0b11                                 |
| Dummy cycles     | 8                     | Required for 0x6C read at speed               |

## DFM Address Rule

In DFM the QUADSPI peripheral sends `AR >> 1` (`AR[ADSIZE:1]`) as the physical address to each
chip. **Do not shift the logical offset manually** — write the raw logical offset
(`address − 0x90000000`) directly to AR; the hardware halves it automatically.
Manual shifting causes a double-shift and puts data at wrong physical addresses.

## QSPI Pin Mapping

| Signal   | Pin  | AF  |
|----------|------|-----|
| CLK      | PB2  | AF9 |
| BK1_NCS  | PB6  | AF10|
| BK1 IO0  | PD11 | AF9 |
| BK1 IO1  | PD12 | AF9 |
| BK1 IO2  | PE2  | AF9 |
| BK1 IO3  | PD13 | AF9 |
| BK2 IO0  | PE7  | AF10|
| BK2 IO1  | PE8  | AF10|
| BK2 IO2  | PE9  | AF10|
| BK2 IO3  | PE10 | AF10|

BK2_NCS (PC11) is not connected on the WomoLIN.power v3 board; BK1_NCS selects both chips.
BK2 IO pins are configured in AF mode so DFM data lines are driven correctly.

## Setup

No special probe-rs patch is required. Standard probe-rs 0.24+ works:

```sh
cargo install probe-rs-tools
```

Build and generate the YAML definition (produces `target/definition.yaml`):

```sh
cargo run
```

Copy the relevant `flash_algorithms` entry into your project's chip description file and reference
it in the `memories` section for the QSPI range (`0x90000000–0x98000000`).

## Differences from the original single-bank version

| Feature           | Original (`stm32h750xx_mt25_ql512`) | This version (`stm32h750xx_mt25_ql512_dfm`) |
|-------------------|--------------------------------------|----------------------------------------------|
| Flash mode        | Single-bank, 1-bit SPI               | Dual Flash Mode (DFM), 4-bit QSPI            |
| Logical size      | 64 MB (one chip)                     | 128 MB (two chips byte-interleaved)          |
| Page size         | `0x100` (256 B)                      | `0x200` (512 B)                              |
| Sector size       | `0x10000` (64 KB)                    | `0x2000` (8 KB)                              |
| Address width     | 24-bit                               | 32-bit                                       |
| HAL dependency    | `stm32h7xx-hal`                      | None — direct MMIO                           |
| probe-rs patch    | Required (toxxin fork)               | Not required (standard probe-rs 0.24+)       |
| flash-algorithm   | 0.6                                  | 0.7.0                                        |

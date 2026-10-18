#![cfg_attr(not(feature = "std"), no_std)]

//! # ToSeven DM01 / DM02 Firmware Recompilation (`V6510C`)
//!
//! This crate provides a functionally identical, `#![no_std]`-compatible Rust recompilation
//! of the ToSeven DM01 (`TO7-DM01-K260521A-V6510C.Pack`) and DM02 (`TO7-DM02-K260526-V6510C.Pack`)
//! motor controller firmware, along with `.Pack` container encryption/decryption and Flash EEPROM
//! configuration management.
//!
//! ## Core Subsystems
//! - [`pack`] — `.Pack` firmware image container unpacker, header parser, XOR cipher (`0xA8`/`0x67` for DM01, `0xF7`/`0xE2` for DM02), and bit-exact repacker.
//! - [`eeprom`] — 5-block (480-byte) Flash EEPROM configuration storage (`0xA5` XOR cipher, `A5 01 FE` magic, 16-entry nibble CRC-16, and embedded DM01 `TS1000W` & DM02 `TS500W` factory defaults).
//! - [`power_curve`] — Exact `V6510C` integer-math power curve pipeline (Global Mode Ceiling vs. Level Power Ceiling, Torque Sensor auto-zero drift tracking & 60 RPM cadence scaling, PAS curve, Throttle slew-rate limiter, Arbitrator, Speed Soft-Taper, and LVC 3.0V cushion).
//! - [`foc`] — Field-Oriented Control (FOC) engine with Clarke/Park transforms, quarter-wave sine/cosine lookup, Hall sensor + PLL observer, dual $I_d/I_q$ PI controllers, and 7-segment SVPWM with `[0.06, 0.94]` dead-time shaping.
//! - [`protocol`] — Display UART communication protocol with `0x69` stream XOR cipher and CRC-16 verification.
//! - [`tuning`] — Source-level modular tuning profiles (uncapped cadence, progressive torque curves, unified assist ceilings).
//! - [`stock`] — Pinned SHA-256 hashes of known official `.Pack` releases.
//! - [`sha256`] — Dependency-free `no_std` SHA-256.
//! - [`config`], [`sensors`], [`control`], [`thermal`], [`battery`], [`faults`], [`hal`] — High-level hardware abstraction, telemetry, and motor controller state machines.

pub mod battery;
pub mod config;
pub mod control;
pub mod eeprom;
pub mod faults;
pub mod foc;
pub mod hal;
pub mod pac;
pub mod pack;
pub mod power_curve;
pub mod protocol;
pub mod sensors;
pub mod sha256;
pub mod stock;
pub mod thermal;
pub mod tuning;

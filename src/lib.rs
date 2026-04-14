// Copyright (C) 2026  Jorge Andre Castro
// GPL-2.0-or-later

//! # embassy-max4466
//!
//! Driver async `no_std` pour le capteur micro MAX4466, basé sur Embassy.
//!
//! ## Features optionnelles
//!
//! | Feature          | Description                              |
//! |------------------|------------------------------------------|
//! | `filter-ema`     | Active le filtre EMA (moyenne exponentielle mobile) |
//! | `filter-median`  | Active le filtre médian (fenêtre glissante de 5 samples) |
//!
//! Par défaut, aucun filtre n'est activé : `read_raw` retourne la valeur ADC brute.
//!
//! ## Exemple minimal
//!
//! ```rust,ignore
//! use embassy_max4466::Max4466;
//! use embassy_rp::adc::{Adc, Channel, Config};
//!
//! #[embassy_executor::task]
//! async fn micro_task(adc: Adc<'static, Async>, ch: Channel<'static>) {
//!     let mut mic = Max4466::new(adc, ch);
//!     mic.calibrate().await;
//!
//!     loop {
//!         let amplitude = mic.read_amplitude(50).await;
//!         defmt::info!("Amplitude: {}", amplitude);
//!     }
//! }
//! ```
//!
//! ## Exemple avec signal global
//!
//! ```rust,ignore
//! use embassy_max4466::signals::MIC_SIGNAL;
//!
//! #[embassy_executor::task]
//! async fn afficher_micro() {
//!     loop {
//!         let data = MIC_SIGNAL.wait().await;
//!         defmt::info!("Amplitude: {}  Raw: {}", data.amplitude, data.raw);
//!     }
//! }
//! ```

#![no_std]
#![allow(async_fn_in_trait)]

pub mod driver;
pub mod signals;

#[cfg(feature = "filter-ema")]
pub mod filter_ema;

#[cfg(feature = "filter-median")]
pub mod filter_median;

pub use driver::Max4466;

/// Données publiées sur [`signals::MIC_SIGNAL`].
#[derive(Clone, Copy)]
pub struct MicData {
    /// Valeur ADC brute (0–4095 sur 12 bits).
    pub raw: u16,
    /// Amplitude crête-à-crête sur la fenêtre de mesure.
    pub amplitude: u16,
}
// Copyright (C) 2026 Jorge Andre Castro
// GPL-2.0-or-later

#![no_std]

//! Driver async `no_std` pour le capteur MAX4466 (microphone amplifié).
//! Compatible avec toutes les cartes Embassy via `embassy-rp` / `embassy-stm32`
//! et `embassy-time` pour les délais précis.
//!
//! ## Principe de fonctionnement
//!
//! Le MAX4466 délivre un signal analogique centré autour de VCC/2. Ce driver :
//! 1. **Calibre** le point zéro (DC offset) en moyennant N lectures au repos.
//! 2. **Lit** l'amplitude crête-à-crête sur une fenêtre temporelle.
//! 3. (optionnel) Applique les traitements activés par features.
//!
//! ## Features
//!
//! | Feature          | Contenu                                                   |
//! |------------------|-----------------------------------------------------------|
//! | *(défaut)*       | `read_raw`, `read_amplitude`, `calibrate`                 |
//! | `filter-smooth`  | [`filters`] — moyenne glissante (passe-bas)               |
//! | `utils-map`      | [`mapping`] — `map_range`, `to_db`, `to_percent`          |
//! | `stats-average`  | [`stats`] — moyenne, variance, RMS sur fenêtre            |
//! | `noise-gate`     | [`gate`] — seuil d'amplitude, silence si < threshold      |
//! | `full`           | Active toutes les features ci-dessus                      |
//!
//! ## Exemple minimal  Embassy RP2350
//!
//! ```rust,ignore
//! use embassy_rp::adc::{Adc, Async, Channel};
//! use embassy_max4466::Max4466;
//!
//! #[embassy_executor::task]
//! async fn mic_task(adc: Adc<'static, Async>, channel: Channel<'static>) {
//!     let mut mic = Max4466::new(adc, channel);
//!     mic.calibrate().await;
//!
//!     loop {
//!         let amplitude = mic.read_amplitude(50).await;
//!         defmt::info!("Amplitude: {}", amplitude);
//!         embassy_time::Timer::after_millis(100).await;
//!     }
//! }
//! ```
//!
//! ## Exemple avec features
//!
//! ```rust,ignore
//! use embassy_max4466::{Max4466, gate::NoiseGate};
//!
//! // Avec noise-gate + stats-average
//! let amplitude = mic.read_amplitude(50).await;
//!
//! let gate = NoiseGate::new(200);
//! if let Some(signal) = gate.process(amplitude) {
//!     let db = embassy_max4466::mapping::to_db(signal);
//!     defmt::info!("Signal: {} dB", db);
//! }
//! ```

pub mod signals;

#[cfg(feature = "filter-smooth")]
pub mod filters;

#[cfg(feature = "utils-map")]
pub mod mapping;

#[cfg(feature = "stats-average")]
pub mod stats;

#[cfg(feature = "noise-gate")]
pub mod gate;

use embassy_rp::adc::{Adc, Async, Channel};
use embassy_time::Timer;

/// Données brutes lues depuis le capteur MAX4466.
#[derive(Clone, Copy, Debug)]
pub struct MicData {
    /// Amplitude crête-à-crête sur la fenêtre de lecture (en unités ADC 12 bits).
    pub amplitude: u16,
    /// Valeur brute instantanée du dernier échantillon (centré sur `zero_point`).
    pub raw: u16,
}

/// Erreurs possibles lors de la lecture du capteur MAX4466.
#[derive(Debug, PartialEq)]
pub enum Max4466Error {
    /// Erreur de lecture ADC (valeur non disponible, remplacée par `zero_point`).
    AdcRead,
}

/// Driver pour le microphone amplifié MAX4466.
///
/// Utilise l'ADC Embassy en mode asynchrone (`Adc<'d, Async>`).
/// La calibration est recommandée avant la première lecture afin d'établir
/// le point zéro (DC offset, typiquement VCC/2 ≈ 2048 sur 12 bits).
pub struct Max4466<'d> {
    adc: Adc<'d, Async>,
    channel: Channel<'d>,
    /// Point zéro calibré (DC offset). Valeur par défaut : 2048 (VCC/2, 12 bits).
    pub zero_point: u16,
}

impl<'d> Max4466<'d> {
    /// Crée un nouveau driver MAX4466.
    ///
    /// Le `zero_point` est initialisé à 2048 (milieu de plage 12 bits).
    /// Appelez [`calibrate`](Self::calibrate) pour affiner cette valeur.
    pub fn new(adc: Adc<'d, Async>, channel: Channel<'d>) -> Self {
        Self {
            adc,
            channel,
            zero_point: 2048,
        }
    }

    /// Calibre le point zéro en moyennant `samples` lectures au repos.
    ///
    /// À appeler **avant** toute mesure, dans un environnement silencieux.
    /// Espacée de 500 µs par lecture pour couvrir plusieurs cycles secteur (50/60 Hz).
    ///
    /// # Arguments
    ///
    /// * `samples` nombre de lectures (défaut recommandé : 128 à 256)
    pub async fn calibrate_n(&mut self, samples: u32) {
        let mut sum: u64 = 0;
        for _ in 0..samples {
            sum += self.adc.read(&mut self.channel).await.unwrap_or(2048) as u64;
            Timer::after_micros(500).await;
        }
        self.zero_point = (sum / samples as u64) as u16;
    }

    /// Calibration rapide avec 128 échantillons.
    ///
    /// Équivalent à `calibrate_n(128)`.
    pub async fn calibrate(&mut self) {
        self.calibrate_n(128).await;
    }

    /// Lit la valeur brute instantanée de l'ADC (non centrée).
    ///
    /// Retourne `zero_point` en cas d'erreur ADC.
    pub async fn read_raw(&mut self) -> u16 {
        self.adc
            .read(&mut self.channel)
            .await
            .unwrap_or(self.zero_point)
    }

    /// Lit l'amplitude crête-à-crête sur une fenêtre de `window_ms` millisecondes.
    ///
    /// Échantillonnage à ~5 kHz (200 µs par lecture). Pour une fenêtre de 50 ms,
    /// cela donne ~250 échantillons.
    ///
    /// # Arguments
    ///
    /// * `window_ms`  durée de la fenêtre d'acquisition en millisecondes
    ///
    /// # Retour
    ///
    /// `amplitude = max - min` sur la fenêtre, en unités ADC.
    pub async fn read_amplitude(&mut self, window_ms: u32) -> u16 {
        let mut min: u16 = 4095;
        let mut max: u16 = 0;

        // ~5 kHz : 1 lecture toutes les 200 µs → window_ms * 5 échantillons
        let samples = window_ms * 5;

        for _ in 0..samples {
            let val = self.read_raw().await;
            if val > max { max = val; }
            if val < min { min = val; }
            Timer::after_micros(200).await;
        }

        if max > min { max - min } else { 0 }
    }

    /// Lit une mesure complète (`MicData`) : amplitude sur fenêtre + dernière valeur brute.
    ///
    /// # Arguments
    ///
    /// * `window_ms` durée de la fenêtre d'acquisition en millisecondes
    pub async fn read(&mut self, window_ms: u32) -> MicData {
        let mut min: u16 = 4095;
        let mut max: u16 = 0;
        let mut last_raw = self.zero_point;

        let samples = window_ms * 5;

        for _ in 0..samples {
            let val = self.read_raw().await;
            last_raw = val;
            if val > max { max = val; }
            if val < min { min = val; }
            Timer::after_micros(200).await;
        }

        MicData {
            amplitude: if max > min { max - min } else { 0 },
            raw: last_raw,
        }
    }
}
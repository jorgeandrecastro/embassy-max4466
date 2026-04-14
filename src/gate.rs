// Copyright (C) 2026 Jorge Andre Castro
// GPL-2.0-or-later

//! Noise gate ignore les signaux en dessous d'un seuil d'amplitude.
//!
//! Feature : `noise-gate`
//!
//! ## Principe
//!
//! Un noise gate laisse passer le signal uniquement lorsque son amplitude
//! dépasse un seuil (`threshold`). En dessous, le signal est considéré
//! comme du bruit ambiant et ignoré (`None`).
//!
//! Un **hold time** optionnel évite les coupures brusques : une fois le
//! seuil franchi, le gate reste ouvert pendant `hold_ticks` appels à
//! [`NoiseGate::process`], même si l'amplitude redescend sous le seuil.
//!
//! ## Exemple
//!
//! ```rust,ignore
//! use embassy_max4466::gate::NoiseGate;
//!
//! // Gate simple : seuil à 150 unités ADC
//! let gate = NoiseGate::new(150);
//!
//! loop {
//!     let amp = mic.read_amplitude(50).await;
//!     match gate.process(amp) {
//!         Some(signal) => defmt::info!("Signal: {}", signal),
//!         None         => { /* silence */ }
//!     }
//! }
//! ```
//!
//! ## Exemple avec hold time
//!
//! ```rust,ignore
//! use embassy_max4466::gate::NoiseGate;
//!
//! // Reste ouvert 10 ticks après le dernier dépassement
//! let mut gate = NoiseGate::with_hold(150, 10);
//!
//! loop {
//!     let amp = mic.read_amplitude(50).await;
//!     if let Some(signal) = gate.process(amp) {
//!         defmt::info!("Signal: {}", signal);
//!     }
//! }
//! ```

/// Noise gate avec seuil configurable et hold time optionnel.
pub struct NoiseGate {
    /// Amplitude minimale pour que le gate s'ouvre.
    threshold: u16,
    /// Nombre de ticks pendant lesquels le gate reste ouvert après un dépassement.
    hold_ticks: u16,
    /// Compteur de hold courant.
    hold_counter: u16,
}

impl NoiseGate {
    /// Crée un noise gate avec seuil fixe et sans hold time.
    ///
    /// # Arguments
    ///
    /// * `threshold`  amplitude minimale (en unités ADC) pour ouvrir le gate
    pub const fn new(threshold: u16) -> Self {
        Self {
            threshold,
            hold_ticks: 0,
            hold_counter: 0,
        }
    }

    /// Crée un noise gate avec seuil et hold time.
    ///
    /// # Arguments
    ///
    /// * `threshold`   amplitude minimale pour ouvrir le gate
    /// * `hold_ticks`  nombre d'appels à [`process`](Self::process) pendant lesquels
    ///                  le gate reste ouvert après que l'amplitude soit redescendue
    pub const fn with_hold(threshold: u16, hold_ticks: u16) -> Self {
        Self {
            threshold,
            hold_ticks,
            hold_counter: 0,
        }
    }

    /// Traite une amplitude et retourne `Some(amplitude)` si le gate est ouvert,
    /// `None` si le signal est sous le seuil (et hors hold time).
    pub fn process(&mut self, amplitude: u16) -> Option<u16> {
        if amplitude >= self.threshold {
            self.hold_counter = self.hold_ticks;
            Some(amplitude)
        } else if self.hold_counter > 0 {
            self.hold_counter -= 1;
            Some(amplitude)
        } else {
            None
        }
    }

    /// Modifie le seuil à la volée.
    pub fn set_threshold(&mut self, threshold: u16) {
        self.threshold = threshold;
    }

    /// Retourne le seuil courant.
    pub fn threshold(&self) -> u16 {
        self.threshold
    }

    /// Remet le compteur de hold à zéro (ferme le gate immédiatement).
    pub fn reset(&mut self) {
        self.hold_counter = 0;
    }
}
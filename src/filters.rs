// Copyright (C) 2026 Jorge Andre Castro
// GPL-2.0-or-later

//! Filtre passe-bas par moyenne glissante.
//!
//! Feature : `filter-smooth`
//!
//! ## Principe
//!
//! Maintient un buffer circulaire de `N` échantillons et retourne
//! leur moyenne. Réduit le bruit haute fréquence au prix d'une
//! latence proportionnelle à la taille du buffer.
//!
//! ## Exemple
//!
//! ```rust,ignore
//! use embassy_max4466::filters::SmoothFilter;
//!
//! let mut filter: SmoothFilter<8> = SmoothFilter::new();
//!
//! loop {
//!     let raw = mic.read_amplitude(20).await;
//!     let smooth = filter.feed(raw);
//!     defmt::info!("Amplitude lissée : {}", smooth);
//! }
//! ```

/// Filtre à moyenne glissante sur `N` échantillons.
///
/// `N` doit être une puissance de 2 pour que la division soit optimisée
/// en décalage de bits par le compilateur.
///
/// Valeurs typiques : 4, 8, 16, 32.
pub struct SmoothFilter<const N: usize> {
    buf: [u16; N],
    idx: usize,
    sum: u32,
    filled: bool,
}

impl<const N: usize> SmoothFilter<N> {
    /// Crée un filtre vide. Tous les échantillons internes sont à zéro.
    pub const fn new() -> Self {
        Self {
            buf: [0u16; N],
            idx: 0,
            sum: 0,
            filled: false,
        }
    }

    /// Alimente le filtre avec un nouvel échantillon et retourne la moyenne courante.
    ///
    /// Avant que le buffer soit plein, la moyenne est calculée sur les
    /// échantillons déjà reçus (pas de "warm-up" artificiel à zéro).
    pub fn feed(&mut self, sample: u16) -> u16 {
        // Soustrait l'ancienne valeur à la position `idx`
        self.sum = self.sum.saturating_sub(self.buf[self.idx] as u32);
        // Insère le nouvel échantillon
        self.buf[self.idx] = sample;
        self.sum += sample as u32;

        self.idx += 1;
        if self.idx >= N {
            self.idx = 0;
            self.filled = true;
        }

        let count = if self.filled { N } else { self.idx } as u32;
        (self.sum / count) as u16
    }

    /// Remet le filtre à zéro.
    pub fn reset(&mut self) {
        self.buf = [0u16; N];
        self.idx = 0;
        self.sum = 0;
        self.filled = false;
    }
}

impl<const N: usize> Default for SmoothFilter<N> {
    fn default() -> Self {
        Self::new()
    }
}
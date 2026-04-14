// Copyright (C) 2026 Jorge Andre Castro
// GPL-2.0-or-later

//! Statistiques sur une fenêtre glissante d'amplitudes.
//!
//! Feature : `stats-average`
//!
//! ## Types disponibles
//!
//! - [`RollingStats`]  accumule N échantillons et calcule moyenne, variance, RMS, pic.
//!
//! ## Exemple
//!
//! ```rust,ignore
//! use embassy_max4466::stats::RollingStats;
//!
//! let mut stats: RollingStats<32> = RollingStats::new();
//!
//! loop {
//!     let amp = mic.read_amplitude(20).await;
//!     stats.feed(amp);
//!
//!     if stats.is_full() {
//!         defmt::info!(
//!             "avg={} rms={} peak={} var={}",
//!             stats.mean(), stats.rms(), stats.peak(), stats.variance()
//!         );
//!     }
//! }
//! ```

/// Accumulateur de statistiques sur une fenêtre glissante de `N` échantillons.
///
/// Calcule : moyenne, variance, RMS (root mean square) et valeur de pic.
///
/// Toutes les opérations sont en arithmétique entière, sans `f32`.
pub struct RollingStats<const N: usize> {
    buf: [u16; N],
    idx: usize,
    filled: bool,
}

impl<const N: usize> RollingStats<N> {
    /// Crée un accumulateur vide.
    pub const fn new() -> Self {
        Self {
            buf: [0u16; N],
            idx: 0,
            filled: false,
        }
    }

    /// Ajoute un échantillon dans la fenêtre.
    pub fn feed(&mut self, sample: u16) {
        self.buf[self.idx] = sample;
        self.idx += 1;
        if self.idx >= N {
            self.idx = 0;
            self.filled = true;
        }
    }

    /// Nombre d'échantillons valides dans la fenêtre (≤ N).
    fn count(&self) -> usize {
        if self.filled { N } else { self.idx }
    }

    /// `true` si la fenêtre est entièrement remplie (N échantillons).
    pub fn is_full(&self) -> bool {
        self.filled
    }

    /// Moyenne arithmétique des échantillons dans la fenêtre.
    ///
    /// Retourne 0 si aucun échantillon.
    pub fn mean(&self) -> u16 {
        let n = self.count();
        if n == 0 { return 0; }

        let sum: u32 = self.buf[..n].iter().map(|&x| x as u32).sum();
        (sum / n as u32) as u16
    }

    /// Valeur maximale observée dans la fenêtre courante.
    pub fn peak(&self) -> u16 {
        let n = self.count();
        if n == 0 { return 0; }

        self.buf[..n].iter().copied().max().unwrap_or(0)
    }

    /// Variance des échantillons (en unités ADC²).
    ///
    /// Formule : `E[x²] - E[x]²` (variance non biaisée simplifiée).
    /// Retourne 0 si moins de 2 échantillons.
    pub fn variance(&self) -> u32 {
        let n = self.count();
        if n < 2 { return 0; }

        let mean = self.mean() as u32;
        let sum_sq: u64 = self.buf[..n].iter().map(|&x| (x as u32) * (x as u32) as u32).map(|v| v as u64).sum();
        let mean_sq = (sum_sq / n as u64) as u32;

        mean_sq.saturating_sub(mean * mean)
    }

    /// RMS (root mean square) des échantillons en unités ADC.
    ///
    /// Approximation entière par racine carrée (Newton-Raphson, 8 itérations).
    /// Précision : ±1 LSB.
    pub fn rms(&self) -> u16 {
        let n = self.count();
        if n == 0 { return 0; }

        let sum_sq: u64 = self.buf[..n].iter().map(|&x| (x as u64) * (x as u64)).sum();
        let mean_sq = sum_sq / n as u64;

        isqrt_u64(mean_sq) as u16
    }

    /// Remet la fenêtre à zéro.
    pub fn reset(&mut self) {
        self.buf = [0u16; N];
        self.idx = 0;
        self.filled = false;
    }
}

impl<const N: usize> Default for RollingStats<N> {
    fn default() -> Self {
        Self::new()
    }
}

/// Racine carrée entière (isqrt) par méthode de Newton-Raphson.
///
/// Utilisée en interne par [`RollingStats::rms`].
fn isqrt_u64(n: u64) -> u64 {
    if n == 0 { return 0; }

    let mut x = n;
    let mut y = (x + 1) / 2;

    while y < x {
        x = y;
        y = (x + n / x) / 2;
    }

    x
}
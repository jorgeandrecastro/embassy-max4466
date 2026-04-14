// Copyright (C) 2026 Jorge Andre Castro
// GPL-2.0-or-later

//! Filtre EMA (Exponential Moving Average) — feature `filter-ema`.
//!
//! Lisse le signal ADC en appliquant :
//!
//! ```text
//! output = alpha * input + (1 - alpha) * previous_output
//! ```
//!
//! `alpha` est encodé en virgule fixe Q8 (0–255) pour éviter `f32` en `no_std`.
//! Valeur par défaut : 32 ≈ 0.125 (lissage fort, adapté audio basse fréquence).

/// Filtre EMA en virgule fixe Q8.
pub struct EmaFilter {
    /// Coefficient alpha en Q8 : alpha_real = alpha_q8 / 256.
    alpha_q8: u16,
    /// Accumulateur Q8 (valeur filtrée × 256).
    acc: u32,
    /// Indique si le filtre a reçu au moins une valeur.
    initialized: bool,
}

impl EmaFilter {
    /// Crée un filtre EMA avec alpha par défaut (32/256 ≈ 0.125).
    pub const fn new() -> Self {
        Self {
            alpha_q8: 32,
            acc: 0,
            initialized: false,
        }
    }

    /// Crée un filtre EMA avec un alpha personnalisé.
    ///
    /// `alpha_q8` ∈ [1, 255] :
    /// - 255 → pas de lissage (sortie = entrée)
    /// - 1   → lissage maximal (réponse très lente)
    pub const fn with_alpha(alpha_q8: u16) -> Self {
        Self {
            alpha_q8,
            acc: 0,
            initialized: false,
        }
    }

    /// Soumet une nouvelle valeur et retourne la sortie filtrée.
    pub fn update(&mut self, input: u16) -> u16 {
        let input_q8 = (input as u32) << 8;

        if !self.initialized {
            self.acc = input_q8;
            self.initialized = true;
        } else {
            // acc = alpha * input_q8 + (256 - alpha) * acc  (tout en Q8*Q8 = Q16, /256 → Q8)
            let alpha = self.alpha_q8 as u32;
            self.acc = (alpha * input_q8 + (256 - alpha) * self.acc) >> 8;
        }

        // Retour en valeur entière (arrondi)
        ((self.acc + 128) >> 8) as u16
    }

    /// Réinitialise le filtre (prochain `update` repart de la valeur brute).
    pub fn reset(&mut self) {
        self.initialized = false;
        self.acc = 0;
    }
}

impl Default for EmaFilter {
    fn default() -> Self {
        Self::new()
    }
}
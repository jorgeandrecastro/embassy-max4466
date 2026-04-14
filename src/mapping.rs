// Copyright (C) 2026 Jorge Andre Castro
// GPL-2.0-or-later

//! Utilitaires de mapping pour les valeurs ADC du MAX4466.
//!
//! Feature : `utils-map`
//!
//! ## Fonctions disponibles
//!
//! - [`map_range`]  — reéchantillonne une valeur d'une plage vers une autre
//! - [`to_percent`] — convertit une amplitude en pourcentage (0–100)
//! - [`to_db`]      — convertit une amplitude en décibels relatifs (dBFS)
//!
//! ## Exemple
//!
//! ```rust,ignore
//! use embassy_max4466::mapping::{to_db, to_percent, map_range};
//!
//! let amplitude = mic.read_amplitude(50).await;
//!
//! let pct = to_percent(amplitude, 4095);          // 0–100 %
//! let db  = to_db(amplitude, 4095);               // dBFS
//! let led = map_range(amplitude, 0, 4095, 0, 255); // PWM LED
//! ```

/// Remappage linéaire d'une valeur depuis `[in_min, in_max]` vers `[out_min, out_max]`.
///
/// Équivalent à la fonction `map()` d'Arduino.
/// La valeur est saturée dans la plage de sortie.
///
/// # Exemples
///
/// ```rust,ignore
/// // Amplitude ADC 12 bits → luminosité PWM 8 bits
/// let pwm = map_range(amplitude, 0, 4095, 0, 255);
/// ```
pub fn map_range(value: u16, in_min: u16, in_max: u16, out_min: u16, out_max: u16) -> u16 {
    if in_max == in_min { return out_min; }

    let value = value.clamp(in_min, in_max);
    let num = (value - in_min) as u32 * (out_max - out_min) as u32;
    let den = (in_max - in_min) as u32;

    out_min + (num / den) as u16
}

/// Convertit une amplitude en pourcentage de la plage ADC.
///
/// # Arguments
///
/// * `amplitude` — valeur crête-à-crête en unités ADC
/// * `adc_max`   — valeur maximale de l'ADC (ex : 4095 pour 12 bits)
///
/// # Retour
///
/// Valeur dans `[0, 100]`.
pub fn to_percent(amplitude: u16, adc_max: u16) -> u8 {
    if adc_max == 0 { return 0; }
    let pct = (amplitude as u32 * 100) / adc_max as u32;
    pct.min(100) as u8
}

/// Convertit une amplitude en décibels relatifs (dBFS — Full Scale).
///
/// Utilise une approximation entière de `20 * log10(amplitude / adc_max)`.
/// Retourne `-96` si l'amplitude est nulle (silence numérique).
///
/// # Arguments
///
/// * `amplitude` — valeur crête-à-crête en unités ADC
/// * `adc_max`   — valeur maximale de l'ADC (ex : 4095 pour 12 bits)
///
/// # Retour
///
/// Valeur en dBFS dans `[-96, 0]` (entier signé).
pub fn to_db(amplitude: u16, adc_max: u16) -> i8 {
    if amplitude == 0 || adc_max == 0 { return -96; }

    // Approximation : 20 * log10(x) ≈ table de décalage sur les bits
    // On utilise une approche par puissances de 2 pour éviter les flottants.
    // Précision ≈ ±3 dB, suffisant pour des indicateurs VU.
    let ratio_x1000 = (amplitude as u32 * 1000) / adc_max as u32;

    // log10 approché par lookup sur les ordres de grandeur
    let db = if ratio_x1000 >= 1000 {  0i16 }
        else if ratio_x1000 >= 708  { -3  }
        else if ratio_x1000 >= 501  { -6  }
        else if ratio_x1000 >= 355  { -9  }
        else if ratio_x1000 >= 251  { -12 }
        else if ratio_x1000 >= 178  { -15 }
        else if ratio_x1000 >= 126  { -18 }
        else if ratio_x1000 >= 89   { -21 }
        else if ratio_x1000 >= 63   { -24 }
        else if ratio_x1000 >= 45   { -27 }
        else if ratio_x1000 >= 32   { -30 }
        else if ratio_x1000 >= 22   { -33 }
        else if ratio_x1000 >= 16   { -36 }
        else if ratio_x1000 >= 11   { -39 }
        else if ratio_x1000 >= 8    { -42 }
        else if ratio_x1000 >= 6    { -45 }
        else if ratio_x1000 >= 4    { -48 }
        else if ratio_x1000 >= 3    { -51 }
        else if ratio_x1000 >= 2    { -54 }
        else                        { -60 };

    db as i8
}
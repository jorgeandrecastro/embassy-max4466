// Copyright (C) 2026 Jorge Andre Castro
// GPL-2.0-or-later

//! Driver principal `Max4466`.
//!
//! Utilise directement les types ADC d'Embassy via des génériques contraints
//!  par le trait `AdcReader` défini ici pas d'abstraction maison.

use embassy_time::Timer;

#[cfg(feature = "filter-ema")]
use crate::filter_ema::EmaFilter;

#[cfg(feature = "filter-median")]
use crate::filter_median::MedianFilter;

/// Trait abstrait minimal pour lire un canal ADC de façon async.
///
/// Embassy ne fournit pas (encore) de trait ADC dans `embedded-hal-async`.
/// On définit ici un trait minimal que l'utilisateur implémente pour son HAL
/// (embassy-rp, embassy-stm32, etc.) en une ligne.
///
/// ## Implémentation pour embassy-rp
///
/// ```rust,ignore
/// use embassy_max4466::AdcReader;
/// use embassy_rp::adc::{Adc, Async, Channel};
///
/// pub struct RpAdc<'d> {
///     adc: Adc<'d, Async>,
///     channel: Channel<'d>,
/// }
///
/// impl AdcReader for RpAdc<'_> {
///     async fn read(&mut self) -> u16 {
///         self.adc.read(&mut self.channel).await.unwrap_or(0)
///     }
/// }
/// ```
pub trait AdcReader {
    /// Lit une valeur ADC brute (généralement 0–4095 sur 12 bits).
    async fn read(&mut self) -> u16;
}

/// Driver pour le capteur microphone MAX4466.
///
/// Générique sur `A : AdcReader` pour rester compatible avec tout HAL Embassy
/// sans dépendance sur un crate HAL spécifique.
pub struct Max4466<A: AdcReader> {
    adc: A,
    /// Point zéro (offset DC) déterminé lors de la calibration.
    zero_point: u16,

    #[cfg(feature = "filter-ema")]
    ema: EmaFilter,

    #[cfg(feature = "filter-median")]
    median: MedianFilter,
}

impl<A: AdcReader> Max4466<A> {
    /// Crée un nouveau driver.
    ///
    /// `zero_point` est initialisé à 2048 (milieu de plage 12 bits).
    /// Appeler [`calibrate`](Self::calibrate) pour l'affiner.
    pub fn new(adc: A) -> Self {
        Self {
            adc,
            zero_point: 2048,

            #[cfg(feature = "filter-ema")]
            ema: EmaFilter::new(),

            #[cfg(feature = "filter-median")]
            median: MedianFilter::new(),
        }
    }

    /// Calibre le point zéro en moyennant 128 lectures espacées de 500 µs.
    ///
    /// À appeler au démarrage, micro sans signal (silence).
    pub async fn calibrate(&mut self) {
        let mut sum: u32 = 0;
        for _ in 0..128 {
            sum += self.adc.read().await as u32;
            Timer::after_micros(500).await;
        }
        self.zero_point = (sum / 128) as u16;
    }

    /// Lit une valeur ADC brute, sans aucun post-traitement.
    pub async fn read_raw(&mut self) -> u16 {
        self.adc.read().await
    }

    /// Lit une valeur filtrée.
    ///
    /// - Avec `filter-ema`    : retourne la sortie du filtre EMA.
    /// - Avec `filter-median` : retourne la sortie du filtre médian.
    /// - Sans aucune feature  : équivalent à [`read_raw`](Self::read_raw).
    ///
    /// Si les deux features sont activées en même temps, EMA est appliqué
    /// en premier, puis médian.
    pub async fn read_filtered(&mut self) -> u16 {
        let raw = self.adc.read().await;

        #[allow(unused_mut, unused_variables)]
        let mut value = raw;

        #[cfg(feature = "filter-ema")]
        {
            value = self.ema.update(value);
        }

        #[cfg(feature = "filter-median")]
        {
            value = self.median.update(value);
        }

        value
    }

    /// Mesure l'amplitude crête-à-crête sur une fenêtre de `window_ms` ms.
    ///
    /// Effectue ~5 lectures par milliseconde (espacées de 200 µs).
    /// Retourne `max - min` des valeurs lues.
    ///
    /// Publie automatiquement le résultat sur [`crate::signals::MIC_SIGNAL`].
    pub async fn read_amplitude(&mut self, window_ms: u32) -> u16 {
        let samples = window_ms * 5;
        let mut min = u16::MAX;
        let mut max = u16::MIN;

        for _ in 0..samples {
            let val = self.read_filtered().await;
            if val > max {
                max = val;
            }
            if val < min {
                min = val;
            }
            Timer::after_micros(200).await;
        }

        let amplitude = if max > min { max - min } else { 0 };

        // Publication sur le signal global (toujours disponible).
        let raw = self.adc.read().await;
        crate::signals::MIC_SIGNAL.signal(crate::MicData { raw, amplitude });

        amplitude
    }

    /// Retourne le point zéro courant (offset DC calibré).
    pub fn zero_point(&self) -> u16 {
        self.zero_point
    }
}
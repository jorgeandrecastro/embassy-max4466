// Copyright (C) 2026 Jorge Andre Castro
// GPL-2.0-or-later

//! Signal global portant la dernière mesure publiée par [`crate::Max4466`].
//!
//! ## Utilisation
//!
//! Le signal [`MIC_SIGNAL`] est un canal "last-value" : seule la dernière
//! mesure est conservée. Si personne ne consomme les données entre deux
//! lectures, l'ancienne valeur est écrasée.
//!
//! ```rust,ignore
//! use embassy_max4466::signals::MIC_SIGNAL;
//!
//! #[embassy_executor::task]
//! async fn afficher_mic() {
//!     loop {
//!         let data = MIC_SIGNAL.wait().await;
//!         defmt::info!("Amplitude: {}  Raw: {}", data.amplitude, data.raw);
//!     }
//! }
//! ```
//!
//! ## Publication depuis la tâche de lecture
//!
//! ```rust,ignore
//! use embassy_max4466::{Max4466, signals::MIC_SIGNAL};
//!
//! #[embassy_executor::task]
//! async fn mic_task(adc: Adc<'static, Async>, channel: Channel<'static>) {
//!     let mut mic = Max4466::new(adc, channel);
//!     mic.calibrate().await;
//!     loop {
//!         let data = mic.read(50).await;
//!         MIC_SIGNAL.signal(data);
//!         embassy_time::Timer::after_millis(100).await;
//!     }
//! }
//! ```

use crate::MicData;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};

/// Signal global portant la dernière mesure publiée par [`crate::Max4466`].
///
/// Utilise un mutex section critique (`CriticalSectionRawMutex`),
/// compatible avec les environnements sans OS (bare-metal, `no_std`).
///
/// # Comportement
///
/// - `signal(data)` publie une nouvelle mesure (écrase la précédente si non lue)
/// - `wait()`        attend de manière asynchrone la prochaine mesure disponible
pub static MIC_SIGNAL: Signal<CriticalSectionRawMutex, MicData> = Signal::new();
# embassy-max4466

Driver async `no_std` pour le capteur **MAX4466** (microphone amplifié à gain réglable),
basé sur [Embassy](https://embassy.dev/).

## Fonctionnalités

| Feature          | Par défaut | Description                                            |
|------------------|:----------:|--------------------------------------------------------|
| *(core)*         | ✅          | `read_raw`, `read_amplitude`, `read`, `calibrate`      |
| `filter-smooth`  | ❌          | Filtre passe-bas par moyenne glissante (`SmoothFilter`) |
| `utils-map`      | ❌          | `map_range`, `to_percent`, `to_db` (dBFS)              |
| `stats-average`  | ❌          | `RollingStats` : moyenne, variance, RMS, pic            |
| `noise-gate`     | ❌          | `NoiseGate` : seuil + hold time                        |
| `full`           | ❌          | Active toutes les features ci-dessus                   |

## Installation

```toml
[dependencies]
embassy-max4466 = { version = "0.1.0" }

# Avec options
embassy-max4466 = { version = "0.1.0", features = ["filter-smooth", "noise-gate"] }

# Tout activer
embassy-max4466 = { version = "0.1.0", features = ["full"] }
```

## Exemple minimal

```rust
use embassy_rp::adc::{Adc, Async, Channel};
use embassy_max4466::Max4466;

#[embassy_executor::task]
async fn mic_task(adc: Adc<'static, Async>, channel: Channel<'static>) {
    let mut mic = Max4466::new(adc, channel);
    mic.calibrate().await; // ~64 ms de calibration

    loop {
        let data = mic.read(50).await;
        defmt::info!("Amplitude: {}  Raw: {}", data.amplitude, data.raw);
        embassy_time::Timer::after_millis(100).await;
    }
}
```

## Exemple avec toutes les features

```rust
use embassy_max4466::{
    Max4466,
    filters::SmoothFilter,
    mapping::{to_db, to_percent},
    stats::RollingStats,
    gate::NoiseGate,
    signals::MIC_SIGNAL,
};

#[embassy_executor::task]
async fn mic_task(adc: Adc<'static, Async>, channel: Channel<'static>) {
    let mut mic = Max4466::new(adc, channel);
    mic.calibrate().await;

    let mut filter: SmoothFilter<8> = SmoothFilter::new();
    let mut stats: RollingStats<32> = RollingStats::new();
    let mut gate = NoiseGate::with_hold(150, 5);

    loop {
        let amp = mic.read_amplitude(50).await;

        let smoothed = filter.feed(amp);
        stats.feed(smoothed);

        if let Some(signal) = gate.process(smoothed) {
            let db  = to_db(signal, 4095);
            let pct = to_percent(signal, 4095);
            defmt::info!("{}% ({}dBFS)  rms={}", pct, db, stats.rms());
        }

        // Publier pour d'autres tâches Embassy
        MIC_SIGNAL.signal(mic.read(1).await);

        embassy_time::Timer::after_millis(100).await;
    }
}
```

## Signal global

```rust
use embassy_max4466::signals::MIC_SIGNAL;

#[embassy_executor::task]
async fn display_task() {
    loop {
        let data = MIC_SIGNAL.wait().await;
        // utiliser data.amplitude, data.raw
    }
}
```

## Calibration

La calibration établit le DC offset du MAX4466 (typiquement VCC/2 ≈ 2048 en 12 bits).
À appeler dans un environnement silencieux avant la première lecture.

```rust
mic.calibrate().await;         // 128 échantillons, ~64 ms
mic.calibrate_n(256).await;    // 256 échantillons, ~128 ms
```

## Licence

GPL-2.0-or-later — Copyright (C) 2026 Jorge Andre Castro
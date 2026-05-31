# embassy-max4466

[![Documentation](https://docs.rs/embassy-max4466/badge.svg)](https://docs.rs/embassy-max4466)
[![Crate](https://img.shields.io/crates/v/embassy-max4466.svg)](https://crates.io/crates/embassy-max4466)
[![License: GPL-2.0](https://img.shields.io/badge/license-GPL--2.0--or--later-blue.svg)](LICENSE)

Driver async `no_std` pour le capteur microphone **MAX4466**, basé sur [Embassy](https://embassy.dev).

# La version 0.2.0 de embassy-max4466
est adoptée afin de stabiliser l’intégration avec l’écosystème Embassy et de réduire les conflits de dépendances. Cette mise à jour vise à améliorer la cohérence avec les autres crates du projet et à fiabiliser l’utilisation du driver microphone dans un environnement embarqué.

```[dependencies]
embassy-time = "0.5"
embassy-sync = "0.8"
```

# La version 0.1.1 fournit un exemple clé en main 

## Features

| Feature          | Description                                        |
|------------------|----------------------------------------------------|
| *(défaut)*       | Valeur ADC brute, aucun filtre                     |
| `filter-ema`     | Filtre EMA (lissage exponentiel, alpha configurable) |
| `filter-median`  | Filtre médian (fenêtre glissante de 5 samples)     |

Les deux features sont cumulables.


## Utilisation

### Cargo.toml

```toml
# Valeur brute uniquement
embassy-max4466 = { version = "0.2.0" }

# Avec filtre EMA
embassy-max4466 = { version = "0.2.0", features = ["filter-ema"] }

# Avec filtre médian
embassy-max4466 = { version = "0.2.0", features = ["filter-median"] }

# Les deux
embassy-max4466 = { version = "0.2.0", features = ["filter-ema", "filter-median"] }
```

### Implémentation du trait `AdcReader` (exemple embassy-rp)

```rust
use embassy_max4466::driver::AdcReader;
use embassy_rp::adc::{Adc, Async, Channel};

pub struct RpAdc<'d> {
    adc: Adc<'d, Async>,
    channel: Channel<'d>,
}

impl AdcReader for RpAdc<'_> {
    async fn read(&mut self) -> u16 {
        self.adc.read(&mut self.channel).await.unwrap_or(0)
    }
}
```

### Exemple complet

```rust
use embassy_executor::Spawner;
use embassy_max4466::{Max4466, signals::MIC_SIGNAL};

#[embassy_executor::task]
async fn micro_task(adc: RpAdc<'static>) {
    let mut mic = Max4466::new(adc);
    mic.calibrate().await;

    loop {
        let amplitude = mic.read_amplitude(50).await;
        defmt::info!("Amplitude: {}", amplitude);
    }
}

#[embassy_executor::task]
async fn afficher_signal() {
    loop {
        let data = MIC_SIGNAL.wait().await;
        defmt::info!("raw={} amplitude={}", data.raw, data.amplitude);
    }
}
```


## Exemple simple

```rust
use embassy_executor::Spawner;
use embassy_max4466::{Max4466, signals::MIC_SIGNAL};
use embassy_max4466::driver::AdcReader;

#[embassy_executor::task]
async fn audio_task(mut adc: impl AdcReader) {
    let mut mic = Max4466::new(adc);
    mic.calibrate().await;

    loop {
        let amplitude = mic.read_amplitude(50).await;
        defmt::info!("Amplitude: {}", amplitude);
        Timer::after_millis(100).await;
    }
}

#[embassy_executor::task]
async fn signal_handler() {
    loop {
        let data = MIC_SIGNAL.wait().await;
        defmt::info!("Raw: {}, Amplitude: {}", data.raw, data.amplitude);
    }
}
```

## Licence

GPL-2.0-or-later
# embassy-max4466

Driver async `no_std` pour le capteur microphone **MAX4466**, basé sur [Embassy](https://embassy.dev).

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
embassy-max4466 = { version = "0.1.1" }

# Avec filtre EMA
embassy-max4466 = { version = "0.1.1", features = ["filter-ema"] }

# Avec filtre médian
embassy-max4466 = { version = "0.1.1", features = ["filter-median"] }

# Les deux
embassy-max4466 = { version = "0.1.1", features = ["filter-ema", "filter-median"] }
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


# Exemple Embassy , Oled ( ma crate disponible aussi ) et blink pico 
Dans cette exemple on utilise tout Filtre Emmma et Median pour une meilleure crate.
```rust

#![no_std]
#![no_main]

use cortex_m_rt as _;
use embassy_executor::Spawner;
use embassy_rp::gpio::{Level, Output, Pull};
use embassy_rp::i2c::{Config as I2cConfig, I2c, Async};
use embassy_rp::adc::{Adc, Channel, Config as AdcConfig, InterruptHandler, Async as AdcAsync};
use embassy_time::{Delay, Duration, Timer};
use hd44780_i2c_nostd::LcdI2c;
use {panic_halt as _, embassy_rp as _};
use core::fmt::Write;
use heapless::String;

//  Mon DRIVER 🦅 
use embassy_max4466::{Max4466, signals::MIC_SIGNAL};
use embassy_max4466::driver::AdcReader;

use rp2350_linker as _;
use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::{I2C0, ADC, PIN_26};
use embassy_rp::Peri; 

bind_interrupts!(struct Irqs {
    I2C0_IRQ => embassy_rp::i2c::InterruptHandler<I2C0>;
    ADC_IRQ_FIFO => InterruptHandler;
});

//  PONT ENTRE Le DRIVER ET EMBASSY-RP 
pub struct RpAdc {
    adc: Adc<'static, AdcAsync>,
    channel: Channel<'static>,
}

impl AdcReader for RpAdc {
    async fn read(&mut self) -> u16 {
        self.adc.read(&mut self.channel).await.unwrap_or(0)
    }
}

// TASK : MICROPHONE 
#[embassy_executor::task]
async fn mic_task(adc_p: Peri<'static, ADC>, pin_p: Peri<'static, PIN_26>) {
    let adc = Adc::new(adc_p, Irqs, AdcConfig::default());
    let channel = Channel::new_pin(pin_p, Pull::None);
    
    // On utilise ton wrapper pour satisfaire AdcReader
    let reader = RpAdc { adc, channel };
    let mut mic = Max4466::new(reader);

    Timer::after(Duration::from_millis(200)).await;
    mic.calibrate().await; 

    loop {
        // Ta crate gère déjà la publication sur MIC_SIGNAL !
        let _amplitude = mic.read_amplitude(40).await;
        Timer::after(Duration::from_millis(10)).await;
    }
}

//  TASK : DISPLAY JC-OS STYLE 
#[embassy_executor::task]
async fn display_task(mut lcd: LcdI2c<I2c<'static, I2C0, Async>>) {
    let mut delay = Delay;
    Timer::after(Duration::from_millis(500)).await;
    
    if lcd.init(&mut delay).await.is_ok() {
        let _ = lcd.set_backlight(true);
        let _ = lcd.clear(&mut delay).await;
        let _ = lcd.write_str("  JC-OS KERNEL", &mut delay).await;
        let _ = lcd.set_cursor(1, 0, &mut delay).await;
        let _ = lcd.write_str("  EAGLE ON 🦅", &mut delay).await;
    }

    Timer::after(Duration::from_secs(2)).await;

    loop {
        // On récupère les données directement depuis le signal de ta crate
        let data = MIC_SIGNAL.wait().await;
        
        let mut s: String<16> = String::new();
        let _ = lcd.set_cursor(0, 0, &mut delay).await;
        
        // Affichage de l'amplitude (le filtrage EMA/Median est géré dans la crate)
        let _ = write!(s, "LEVEL: {:<5}    ", data.amplitude); 
        let _ = lcd.write_str(s.as_str(), &mut delay).await;

        let _ = lcd.set_cursor(1, 0, &mut delay).await;
        if data.amplitude > 2000 {
            let _ = lcd.write_str("!! TROP BRUYANT !!", &mut delay).await;
        } else {
            let _ = lcd.write_str("SOUND STABLE    ", &mut delay).await;
        }
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(embassy_rp::config::Config::default());

    let mut i2c_config = I2cConfig::default();
    i2c_config.frequency = 100_000;
    let i2c = I2c::new_async(p.I2C0, p.PIN_5, p.PIN_4, Irqs, i2c_config);
    let lcd = LcdI2c::new(i2c, 0x3F); 

    spawner.spawn(mic_task(p.ADC, p.PIN_26)).unwrap();
    spawner.spawn(display_task(lcd)).unwrap();

    let mut led = Output::new(p.PIN_25, Level::Low);
    loop {
        led.toggle();
        Timer::after_millis(150).await; 
    }
}
```

## Licence

GPL-2.0-or-later
// Copyright (C) 2026 Jorge Andre Castro
// GPL-2.0-or-later

//! Filtre médian à fenêtre glissante feature `filter-median`.
//!
//! Conserve les 5 dernières valeurs ADC et retourne leur médiane.
//! Fenêtre de taille 5 : bon compromis latence / rejet des pics parasites.
//!
//! Entièrement `no_std`, pas d'allocation dynamique.

const WINDOW: usize = 5;

/// Filtre médian sur fenêtre glissante de 5 échantillons.
pub struct MedianFilter {
    buf: [u16; WINDOW],
    pos: usize,
    count: usize,
}

impl MedianFilter {
    /// Crée un filtre médian non initialisé.
    pub const fn new() -> Self {
        Self {
            buf: [0u16; WINDOW],
            pos: 0,
            count: 0,
        }
    }

    /// Soumet une nouvelle valeur et retourne la médiane courante.
    ///
    /// Tant que la fenêtre n'est pas pleine, la médiane est calculée
    /// sur les échantillons disponibles.
    pub fn update(&mut self, input: u16) -> u16 {
        self.buf[self.pos] = input;
        self.pos = (self.pos + 1) % WINDOW;
        if self.count < WINDOW {
            self.count += 1;
        }
        self.median()
    }

    /// Calcule la médiane des `count` dernières valeurs.
    fn median(&self) -> u16 {
        let n = self.count;
        // Copie partielle dans un tableau de travail (tri par insertion).
        let mut tmp = [0u16; WINDOW];
        tmp[..n].copy_from_slice(&self.buf[..n]);
        insertion_sort(&mut tmp[..n]);
        tmp[n / 2]
    }

    /// Réinitialise le filtre.
    pub fn reset(&mut self) {
        self.buf = [0u16; WINDOW];
        self.pos = 0;
        self.count = 0;
    }
}

impl Default for MedianFilter {
    fn default() -> Self {
        Self::new()
    }
}

/// Tri par insertion en place  O(n²) mais n ≤ 5, donc parfaitement adapté.
fn insertion_sort(arr: &mut [u16]) {
    let n = arr.len();
    for i in 1..n {
        let key = arr[i];
        let mut j = i;
        while j > 0 && arr[j - 1] > key {
            arr[j] = arr[j - 1];
            j -= 1;
        }
        arr[j] = key;
    }
}
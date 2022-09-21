// Standard set of cards
pub fn get_default_cards() -> &'static [u32] {
    &[
        100, 75, 50, 25, // Big numbers
        10, 10, 9, 9, 8, 8, 7, 7, 6, 6, 5, 5, 4, 4, 3, 3, 2, 2, 1, 1,
    ]
}

// Set of cards used in special editions of the show
pub fn get_special_cards() -> &'static [u32] {
    &[
        87, 62, 37, 12, // Big numbers
        10, 10, 9, 9, 8, 8, 7, 7, 6, 6, 5, 5, 4, 4, 3, 3, 2, 2, 1, 1,
    ]
}

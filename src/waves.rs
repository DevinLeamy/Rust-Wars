use crate::aliens::Wave;

impl Wave {
    pub fn get_data(path: &str) -> Vec<&str> {
        match path {
            "assets/waves/wave_0.txt" => vec![
                "AAAAAAAAAA",
                "#AA#AA#AA#",
                "##########",
                "#AA#AA#AA#",
                "AAAAAAAAAA"
            ],
            "assets/waves/wave_1.txt" => vec![
                "aa#a##a#aa",
                "#aaraaraa#",
                "##########",
                "#aaraaraa#",
                "raaa##aaar",
            ],
            "assets/waves/wave_2.txt" => vec![
                "aa#zrrz#aa",
                "raaraaraar",
                "zr#rzzr#rz",
                "raa#aa#aar",
                "#aaa##aaa#",
            ],
            "assets/waves/wave_3.txt" => vec![
                "razzrrzzar",
                "rraraararr",
                "zrrrzzrrrz",
                "raazaazaar",
                "zaaazzaaaz",
            ],
            _ => panic!("Not a valid wave")
        }
    }
}

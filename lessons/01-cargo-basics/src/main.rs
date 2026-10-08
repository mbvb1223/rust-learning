use cargo_basics::{celsius_to_fahrenheit, describe};

fn main() {
    for celsius in (-10..=40).step_by(10) {
        let celsius = f64::from(celsius);
        let fahrenheit = celsius_to_fahrenheit(celsius);
        println!(
            "{celsius:>5.1} °C = {fahrenheit:>5.1} °F  {}",
            describe(celsius)
        );
    }
}

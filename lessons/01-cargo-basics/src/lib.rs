//! Lesson 01 — temperature converter. Replace every `todo!()` until `cargo test` passes.

/// `F = C × 9/5 + 32`
pub fn celsius_to_fahrenheit(celsius: f64) -> f64 {
    celsius * 9.0 / 5.0 + 32.0
}

/// `C = (F − 32) × 5/9`
pub fn fahrenheit_to_celsius(fahrenheit: f64) -> f64 {
    (fahrenheit - 32.0) * 5.0 / 9.0
}

/// Integer-only conversion. Integer division truncates toward zero, so `37` → `98`.
pub fn celsius_to_fahrenheit_whole(celsius: i32) -> i32 {
    celsius * 9 / 5 + 32
}

/// Rounds half away from zero, like PHP's `round($value, $decimals)`.
/// Negative `decimals` round to tens, hundreds, …
pub fn round_to(value: f64, decimals: i32) -> f64 {
    let factor = 10f64.powi(decimals);
    let value = (value * factor).round();
    value / factor
}

/// `"freezing"` at or below 0 °C, `"cold"` below 15, `"mild"` below 25, otherwise `"hot"`.
pub fn describe(celsius: f64) -> &'static str {
    if celsius <= 0.0 {
        "freezing"
    } else if celsius < 15.0 {
        "cold"
    } else if celsius < 25.0 {
        "mild"
    } else {
        "hot"
    }
}

/// Average Fahrenheit value of every whole Celsius degree in `from..=to`. Assumes `from <= to`.
pub fn average_fahrenheit(from: i32, to: i32) -> f64 {
    let mut sum = 0.0;
    let mut count = 0;
    for celsius in from..=to {
        sum += celsius_to_fahrenheit(f64::from(celsius));
        count += 1;
    }
    sum / f64::from(count)
}

/// Counting up from `start`, the first whole Celsius degree whose Fahrenheit value is at least `threshold`.
pub fn first_celsius_reaching(start: i32, threshold: f64) -> i32 {
    let mut celsius = start;
    loop {
        if celsius_to_fahrenheit(f64::from(celsius)) >= threshold {
            break celsius;
        }
        celsius += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_close(actual: f64, expected: f64) {
        assert!(
            (actual - expected).abs() < 1e-9,
            "expected {expected}, got {actual}"
        );
    }

    #[test]
    fn converts_celsius_to_fahrenheit() {
        assert_close(celsius_to_fahrenheit(0.0), 32.0);
        assert_close(celsius_to_fahrenheit(100.0), 212.0);
        assert_close(celsius_to_fahrenheit(-40.0), -40.0);
        assert_close(celsius_to_fahrenheit(37.0), 98.6);
    }

    #[test]
    fn converts_fahrenheit_to_celsius() {
        assert_close(fahrenheit_to_celsius(32.0), 0.0);
        assert_close(fahrenheit_to_celsius(212.0), 100.0);
        assert_close(fahrenheit_to_celsius(-40.0), -40.0);
        assert_close(fahrenheit_to_celsius(98.6), 37.0);
    }

    #[test]
    fn round_trip_returns_the_original_value() {
        for celsius in -50..=50 {
            let celsius = f64::from(celsius);
            assert_close(
                fahrenheit_to_celsius(celsius_to_fahrenheit(celsius)),
                celsius,
            );
        }
    }

    #[test]
    fn whole_conversion_uses_integer_arithmetic() {
        assert_eq!(celsius_to_fahrenheit_whole(0), 32);
        assert_eq!(celsius_to_fahrenheit_whole(100), 212);
        assert_eq!(celsius_to_fahrenheit_whole(-40), -40);
        assert_eq!(celsius_to_fahrenheit_whole(37), 98);
    }

    #[test]
    fn rounds_like_php() {
        assert_close(round_to(98.600_000_1, 1), 98.6);
        assert_close(round_to(36.666, 2), 36.67);
        assert_close(round_to(2.5, 0), 3.0);
        assert_close(round_to(-2.5, 0), -3.0);
        assert_close(round_to(1234.5, -2), 1200.0);
    }

    #[test]
    fn describes_temperature_ranges() {
        assert_eq!(describe(-5.0), "freezing");
        assert_eq!(describe(0.0), "freezing");
        assert_eq!(describe(0.1), "cold");
        assert_eq!(describe(14.9), "cold");
        assert_eq!(describe(15.0), "mild");
        assert_eq!(describe(24.9), "mild");
        assert_eq!(describe(25.0), "hot");
    }

    #[test]
    fn averages_fahrenheit_over_a_range() {
        assert_close(average_fahrenheit(0, 0), 32.0);
        assert_close(average_fahrenheit(0, 100), 122.0);
        assert_close(average_fahrenheit(-40, -40), -40.0);
    }

    #[test]
    fn finds_first_degree_reaching_threshold() {
        assert_eq!(first_celsius_reaching(0, 100.0), 38);
        assert_eq!(first_celsius_reaching(-50, -40.0), -40);
        assert_eq!(first_celsius_reaching(20, 0.0), 20);
    }
}

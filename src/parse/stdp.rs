use std::num::{NonZeroI32, NonZeroU32};

use super::Parser;

/// Беззнаковые числа (ненулевые)
#[derive(Debug)]
pub struct U32;
impl U32 {
    const HEX_PREFIX: &str = "0x";
    const HEX_RADIX: u32 = 16;
    const DEC_RADIX: u32 = 10;

    /// Отделить префикс системы счисления, вернуть остаток и radix
    fn strip_radix_prefix(input: &str) -> (&str, u32) {
        input
            .strip_prefix(Self::HEX_PREFIX)
            .map_or((input, Self::DEC_RADIX), |rest| (rest, Self::HEX_RADIX))
    }

    /// Позиция первого символа, который не является цифрой в данной системе
    fn find_digit_end(s: &str, radix: u32) -> usize {
        s.char_indices()
            .find_map(|(i, c)| (!c.is_digit(radix)).then_some(i))
            .unwrap_or(s.len())
    }
}
impl Parser for U32 {
    type Dest = NonZeroU32;
    fn parse<'a>(&self, input: &'a str) -> Result<(&'a str, Self::Dest), ()> {
        let (after_prefix, radix) = Self::strip_radix_prefix(input);
        let digit_end = Self::find_digit_end(after_prefix, radix);
        let digit_str = &after_prefix[..digit_end];
        let remaining = &after_prefix[digit_end..];
        let value = u32::from_str_radix(digit_str, radix).map_err(|_| ())?;
        let non_zero = NonZeroU32::new(value).ok_or(())?;
        Ok((remaining, non_zero))
    }
}

/// Знаковые числа (ненулевые)
#[derive(Debug)]
#[allow(dead_code)]
pub struct I32;
impl I32 {
    /// Конец цифровой части (первый символ - знак или цифра - пропускается)
    fn find_digit_end(input: &str) -> usize {
        input
            .char_indices()
            .skip(1)
            .find_map(|(i, c)| (!c.is_ascii_digit()).then_some(i))
            .unwrap_or(input.len())
    }
}
impl Parser for I32 {
    type Dest = NonZeroI32;
    fn parse<'a>(&self, input: &'a str) -> Result<(&'a str, Self::Dest), ()> {
        let digit_end = Self::find_digit_end(input);
        let digit_str = &input[..digit_end];
        let remaining = &input[digit_end..];
        let value: i32 = digit_str.parse().map_err(|_| ())?;
        let non_zero = NonZeroI32::new(value).ok_or(())?;
        Ok((remaining, non_zero))
    }
}

/// Шестнадцатеричные байты (пригодится при парсинге блобов)
#[derive(Debug, Clone)]
pub struct Byte;
impl Byte {
    const HEX_BYTE_LEN: usize = 2;
}
impl Parser for Byte {
    type Dest = u8;
    fn parse<'a>(&self, input: &'a str) -> Result<(&'a str, Self::Dest), ()> {
        let (hex_str, remaining) =
            input.split_at_checked(Self::HEX_BYTE_LEN).ok_or(())?;
        let value = u8::from_str_radix(hex_str, 16).map_err(|_| ())?;
        Ok((remaining, value))
    }
}

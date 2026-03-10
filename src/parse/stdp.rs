use super::Parser;

/// Беззнаковые числа (ненулевые)
#[derive(Debug)]
pub struct U32;
impl Parser for U32 {
    type Dest = u32;
    fn parse<'a>(&self, input: &'a str) -> Result<(&'a str, Self::Dest), ()> {
        let (remaining, is_hex) = input
            .strip_prefix("0x")
            .map_or((input, false), |remaining| (remaining, true));
        let end_idx = remaining
            .char_indices()
            .find_map(|(idx, c)| match (is_hex, c) {
                (true, 'a'..='f' | '0'..='9' | 'A'..='F') => None,
                (false, '0'..='9') => None,
                _ => Some(idx),
            })
            .unwrap_or(remaining.len());
        let value = u32::from_str_radix(&remaining[..end_idx], if is_hex { 16 } else { 10 })
            .map_err(|_| ())?;
        let value = std::num::NonZeroU32::new(value).ok_or(())?; // в наших логах нет нулей, ноль в операции - фикция
        Ok((&remaining[end_idx..], value.get()))
    }
}
/// Знаковые числа
#[derive(Debug)]
pub struct I32;
impl Parser for I32 {
    type Dest = i32;
    fn parse<'a>(&self, input: &'a str) -> Result<(&'a str, Self::Dest), ()> {
        let end_idx = input
            .char_indices()
            .skip(1)
            .find_map(|(idx, c)| (!c.is_ascii_digit()).then_some(idx))
            .unwrap_or(input.len());
        let value = input[..end_idx].parse().map_err(|_| ())?;
        if value == 0 {
            return Err(()); // в наших логах нет нулей, ноль в операции - фикция
        }
        Ok((&input[end_idx..], value))
    }
}
/// Шестнадцатеричные байты (пригодится при парсинге блобов)
#[derive(Debug, Clone)]
pub struct Byte;
impl Parser for Byte {
    type Dest = u8;
    fn parse<'a>(&self, input: &'a str) -> Result<(&'a str, Self::Dest), ()> {
        let (to_parse, remaining) = input.split_at_checked(2).ok_or(())?;
        if !to_parse.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(());
        }
        let value = u8::from_str_radix(to_parse, 16).map_err(|_| ())?;
        Ok((remaining, value))
    }
}
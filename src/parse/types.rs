use std::num::NonZeroU32;

use super::Parsable;
use super::Parser;
use super::combinators::{Map, Take, map, take, Alt, Tag, Delimited, Unquote, alt2, tag, delimited, unquote, All, StripWhitespace, Permutation, KeyValue, all2, strip_whitespace, permutation2, key_value, List, list};
use super::stdp;

pub const AUTHDATA_SIZE: usize = 1024;

// подсказка: довольно много места на стэке
/// Данные для авторизации
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthData(pub(crate) [u8; AUTHDATA_SIZE]);
impl Parsable for AuthData {
    type Parser = Map<Take<stdp::Byte>, fn(Vec<u8>) -> Self>;
    fn parser() -> Self::Parser {
        map(take(AUTHDATA_SIZE, stdp::Byte), |authdata| {
            Self(authdata.try_into().unwrap_or([0; AUTHDATA_SIZE]))
        })
    }
}

/// Конструкция 'либо-либо'
pub enum Either<Left, Right> {
    Left(Left),
    Right(Right),
}

/// Статус, которые можно парсить
pub enum Status {
    Ok,
    Err(String),
}
impl Parsable for Status {
    type Parser = Alt<(
        Map<Tag, fn(()) -> Self>,
        Map<Delimited<Tag, Unquote, Tag>, fn(String) -> Self>,
    )>;
    fn parser() -> Self::Parser {
        const fn to_ok(_: ()) -> Status {
            Status::Ok
        }
        const fn to_err(error: String) -> Status {
            Status::Err(error)
        }
        alt2(
            map(tag("Ok"), to_ok),
            map(delimited(tag("Err("), unquote(), tag(")")), to_err),
        )
    }
}

/// Пара 'сокращённое название предмета' - 'его описание'
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssetDsc {
    // `dsc` aka `description`
    pub id: String,
    pub dsc: String,
}
impl Parsable for AssetDsc {
    type Parser = Map<
        Delimited<
            All<(StripWhitespace<Tag>, StripWhitespace<Tag>)>,
            Permutation<(KeyValue<Unquote>, KeyValue<Unquote>)>,
            StripWhitespace<Tag>,
        >,
        fn((String, String)) -> Self,
    >;
    fn parser() -> Self::Parser {
        // комбинаторы парсеров - это круто
        map(
            delimited(
                all2(
                    strip_whitespace(tag("AssetDsc")),
                    strip_whitespace(tag("{")),
                ),
                permutation2(
                    key_value("id", unquote()),
                    key_value("dsc", unquote()),
                ),
                strip_whitespace(tag("}")),
            ),
            |(id, dsc)| Self { id, dsc },
        )
    }
}
/// Сведение о предмете в некотором количестве
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Backet {
    pub asset_id: String,
    pub count: NonZeroU32,
}
impl Parsable for Backet {
    type Parser = Map<
        Delimited<
            All<(StripWhitespace<Tag>, StripWhitespace<Tag>)>,
            Permutation<(KeyValue<Unquote>, KeyValue<stdp::U32>)>,
            StripWhitespace<Tag>,
        >,
        fn((String, NonZeroU32)) -> Self,
    >;
    fn parser() -> Self::Parser {
        map(
            delimited(
                all2(
                    strip_whitespace(tag("Backet")),
                    strip_whitespace(tag("{")),
                ),
                permutation2(
                    key_value("asset_id", unquote()),
                    key_value("count", stdp::U32),
                ),
                strip_whitespace(tag("}")),
            ),
            |(asset_id, count)| Self { asset_id, count },
        )
    }
}
/// Фиатные деньги конкретного пользователя
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserCash {
    pub user_id: String,
    pub count: NonZeroU32,
}
impl Parsable for UserCash {
    type Parser = Map<
        Delimited<
            All<(StripWhitespace<Tag>, StripWhitespace<Tag>)>,
            Permutation<(KeyValue<Unquote>, KeyValue<stdp::U32>)>,
            StripWhitespace<Tag>,
        >,
        fn((String, NonZeroU32)) -> Self,
    >;
    fn parser() -> Self::Parser {
        map(
            delimited(
                all2(
                    strip_whitespace(tag("UserCash")),
                    strip_whitespace(tag("{")),
                ),
                permutation2(
                    key_value("user_id", unquote()),
                    key_value("count", stdp::U32),
                ),
                strip_whitespace(tag("}")),
            ),
            |(user_id, count)| Self { user_id, count },
        )
    }
}
/// [Backet] конкретного пользователя
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserBacket {
    pub user_id: String,
    pub backet: Backet,
}
impl Parsable for UserBacket {
    type Parser = Map<
        Delimited<
            All<(StripWhitespace<Tag>, StripWhitespace<Tag>)>,
            Permutation<(
                KeyValue<Unquote>,
                KeyValue<<Backet as Parsable>::Parser>,
            )>,
            StripWhitespace<Tag>,
        >,
        fn((String, Backet)) -> Self,
    >;
    fn parser() -> Self::Parser {
        map(
            delimited(
                all2(
                    strip_whitespace(tag("UserBacket")),
                    strip_whitespace(tag("{")),
                ),
                permutation2(
                    key_value("user_id", unquote()),
                    key_value("backet", Backet::parser()),
                ),
                strip_whitespace(tag("}")),
            ),
            |(user_id, backet)| Self { user_id, backet },
        )
    }
}
/// [Бакеты](Backet) конкретного пользователя
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserBackets {
    pub user_id: String,
    pub backets: Vec<Backet>,
}
impl Parsable for UserBackets {
    type Parser = Map<
        Delimited<
            All<(StripWhitespace<Tag>, StripWhitespace<Tag>)>,
            Permutation<(
                KeyValue<Unquote>,
                KeyValue<List<<Backet as Parsable>::Parser>>,
            )>,
            StripWhitespace<Tag>,
        >,
        fn((String, Vec<Backet>)) -> Self,
    >;
    fn parser() -> Self::Parser {
        map(
            delimited(
                all2(
                    strip_whitespace(tag("UserBackets")),
                    strip_whitespace(tag("{")),
                ),
                permutation2(
                    key_value("user_id", unquote()),
                    key_value("backets", list(Backet::parser())),
                ),
                strip_whitespace(tag("}")),
            ),
            |(user_id, backets)| Self { user_id, backets },
        )
    }
}
/// Список опубликованных бакетов
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Announcements(pub(crate) Vec<UserBackets>);
impl Parsable for Announcements {
    type Parser = Map<
        List<<UserBackets as Parsable>::Parser>,
        fn(Vec<UserBackets>) -> Self,
    >;
    fn parser() -> Self::Parser {
        const fn from_vec(vec: Vec<UserBackets>) -> Announcements {
            Announcements(vec)
        }
        map(list(UserBackets::parser()), from_vec)
    }
}

/// Дженерик-обёртка для парсинга любого [Parsable] типа
fn just_parse<T: Parsable>(
    input: &str,
) -> Result<(&str, T), ()> {
    T::parser().parse(input)
}

// Обратная совместимость: алиасы для прежних функций
#[allow(clippy::missing_errors_doc, clippy::result_unit_err)]
pub fn just_parse_asset_dsc(
    input: &str,
) -> Result<(&str, AssetDsc), ()> {
    just_parse(input)
}
#[allow(clippy::missing_errors_doc, clippy::result_unit_err)]
pub fn just_parse_backet(input: &str) -> Result<(&str, Backet), ()> {
    just_parse(input)
}
#[allow(clippy::missing_errors_doc, clippy::result_unit_err)]
pub fn just_user_cash(input: &str) -> Result<(&str, UserCash), ()> {
    just_parse(input)
}
#[allow(clippy::missing_errors_doc, clippy::result_unit_err)]
pub fn just_user_backet(
    input: &str,
) -> Result<(&str, UserBacket), ()> {
    just_parse(input)
}
#[allow(clippy::missing_errors_doc, clippy::result_unit_err)]
pub fn just_user_backets(
    input: &str,
) -> Result<(&str, UserBackets), ()> {
    just_parse(input)
}
#[allow(clippy::missing_errors_doc, clippy::result_unit_err)]
pub fn just_parse_anouncements(
    input: &str,
) -> Result<(&str, Announcements), ()> {
    just_parse(input)
}

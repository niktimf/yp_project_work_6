use super::Parsable;
use super::Parser;
use super::combinators::*;
use super::stdp;

pub(crate) const AUTHDATA_SIZE: usize = 1024;

// подсказка: довольно много места на стэке
/// Данные для авторизации
#[derive(Debug, Clone, PartialEq)]
pub struct AuthData(pub(crate) [u8; AUTHDATA_SIZE]);
impl Parsable for AuthData {
    type Parser = Map<Take<stdp::Byte>, fn(Vec<u8>) -> Self>;
    fn parser() -> Self::Parser {
        map(take(AUTHDATA_SIZE, stdp::Byte), |authdata| {
            AuthData(authdata.try_into().unwrap_or([0; AUTHDATA_SIZE]))
        })
    }
}

/// Конструкция 'либо-либо'
pub(crate) enum Either<Left, Right> {
    Left(Left),
    Right(Right),
}

/// Статус, которые можно парсить
pub(crate) enum Status {
    Ok,
    Err(String),
}
impl Parsable for Status {
    type Parser = Alt<(
        Map<Tag, fn(()) -> Self>,
        Map<Delimited<Tag, Unquote, Tag>, fn(String) -> Self>,
    )>;
    fn parser() -> Self::Parser {
        fn to_ok(_: ()) -> Status {
            Status::Ok
        }
        fn to_err(error: String) -> Status {
            Status::Err(error)
        }
        alt2(
            map(tag("Ok"), to_ok),
            map(delimited(tag("Err("), unquote(), tag(")")), to_err),
        )
    }
}

/// Пара 'сокращённое название предмета' - 'его описание'
#[derive(Debug, Clone, PartialEq)]
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
                permutation2(key_value("id", unquote()), key_value("dsc", unquote())),
                strip_whitespace(tag("}")),
            ),
            |(id, dsc)| AssetDsc { id, dsc },
        )
    }
}
/// Сведение о предмете в некотором количестве
#[derive(Debug, Clone, PartialEq)]
pub struct Backet {
    pub asset_id: String,
    pub count: u32,
}
impl Parsable for Backet {
    type Parser = Map<
        Delimited<
            All<(StripWhitespace<Tag>, StripWhitespace<Tag>)>,
            Permutation<(KeyValue<Unquote>, KeyValue<stdp::U32>)>,
            StripWhitespace<Tag>,
        >,
        fn((String, u32)) -> Self,
    >;
    fn parser() -> Self::Parser {
        map(
            delimited(
                all2(strip_whitespace(tag("Backet")), strip_whitespace(tag("{"))),
                permutation2(
                    key_value("asset_id", unquote()),
                    key_value("count", stdp::U32),
                ),
                strip_whitespace(tag("}")),
            ),
            |(asset_id, count)| Backet { asset_id, count },
        )
    }
}
/// Фиатные деньги конкретного пользователя
#[derive(Debug, Clone, PartialEq)]
pub struct UserCash {
    pub user_id: String,
    pub count: u32,
}
impl Parsable for UserCash {
    type Parser = Map<
        Delimited<
            All<(StripWhitespace<Tag>, StripWhitespace<Tag>)>,
            Permutation<(KeyValue<Unquote>, KeyValue<stdp::U32>)>,
            StripWhitespace<Tag>,
        >,
        fn((String, u32)) -> Self,
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
            |(user_id, count)| UserCash { user_id, count },
        )
    }
}
/// [Backet] конкретного пользователя
#[derive(Debug, Clone, PartialEq)]
pub struct UserBacket {
    pub user_id: String,
    pub backet: Backet,
}
impl Parsable for UserBacket {
    type Parser = Map<
        Delimited<
            All<(StripWhitespace<Tag>, StripWhitespace<Tag>)>,
            Permutation<(KeyValue<Unquote>, KeyValue<<Backet as Parsable>::Parser>)>,
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
            |(user_id, backet)| UserBacket { user_id, backet },
        )
    }
}
/// [Бакеты](Backet) конкретного пользователя
#[derive(Debug, Clone, PartialEq)]
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
            |(user_id, backets)| UserBackets { user_id, backets },
        )
    }
}
/// Список опубликованных бакетов
#[derive(Debug, Clone, PartialEq)]
pub struct Announcements(pub(crate) Vec<UserBackets>);
impl Parsable for Announcements {
    type Parser = Map<List<<UserBackets as Parsable>::Parser>, fn(Vec<UserBackets>) -> Self>;
    fn parser() -> Self::Parser {
        fn from_vec(vec: Vec<UserBackets>) -> Announcements {
            Announcements(vec)
        }
        map(list(UserBackets::parser()), from_vec)
    }
}

/// Дженерик-обёртка для парсинга любого [Parsable] типа
pub(crate) fn just_parse<'a, T: Parsable>(input: &'a str) -> Result<(&'a str, T), ()> {
    T::parser().parse(input)
}

// Обратная совместимость: алиасы для прежних функций
pub fn just_parse_asset_dsc<'a>(input: &'a str) -> Result<(&'a str, AssetDsc), ()> {
    just_parse(input)
}
pub fn just_parse_backet<'a>(input: &'a str) -> Result<(&'a str, Backet), ()> {
    just_parse(input)
}
pub fn just_user_cash<'a>(input: &'a str) -> Result<(&'a str, UserCash), ()> {
    just_parse(input)
}
pub fn just_user_backet<'a>(input: &'a str) -> Result<(&'a str, UserBacket), ()> {
    just_parse(input)
}
pub fn just_user_backets<'a>(input: &'a str) -> Result<(&'a str, UserBackets), ()> {
    just_parse(input)
}
pub fn just_parse_anouncements<'a>(input: &'a str) -> Result<(&'a str, Announcements), ()> {
    just_parse(input)
}

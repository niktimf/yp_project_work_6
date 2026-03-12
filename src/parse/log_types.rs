use std::num::NonZeroU32;

use super::Parsable;
use super::combinators::{
    All, Alt, Delimited, KeyValue, Map, Permutation, Preceded, StripWhitespace,
    Tag, Unquote, all2, alt2, alt3, alt4, alt8, delimited, key_value, map,
    permutation2, permutation3, preceded, strip_whitespace, tag, unquote,
};
use super::stdp;
use super::types::{Announcements, AuthData, UserBacket, UserCash};

/// Все виды логов
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LogKind {
    System(SystemLogKind),
    App(AppLogKind),
}
/// Все виды [системных](LogKind) логов
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SystemLogKind {
    Error(SystemLogErrorKind),
    Trace(SystemLogTraceKind),
}
/// Trace [системы](SystemLogKind)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SystemLogTraceKind {
    SendRequest(String),
    GetResponse(String),
}
/// Error [системы](SystemLogKind)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SystemLogErrorKind {
    NetworkError(String),
    AccessDenied(String),
}
/// Все виды [логов приложения](LogKind) логов
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppLogKind {
    Error(AppLogErrorKind),
    Trace(AppLogTraceKind),
    Journal(AppLogJournalKind),
}
/// Error [приложения](AppLogKind)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppLogErrorKind {
    LackOf(String),
    SystemError(String),
}
/// Trace [приложения](AppLogKind)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppLogTraceKind {
    Connect(Box<AuthData>),
    SendRequest(String),
    Check(Announcements),
    GetResponse(String),
}
/// Создание пользователя
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateUser {
    pub user_id: String,
    pub authorized_capital: NonZeroU32,
}
/// Удаление пользователя
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeleteUser {
    pub user_id: String,
}
/// Регистрация ассета
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegisterAsset {
    pub asset_id: String,
    pub user_id: String,
    pub liquidity: NonZeroU32,
}
/// Отмена регистрации ассета
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnregisterAsset {
    pub asset_id: String,
    pub user_id: String,
}
/// Журнал [приложения](AppLogKind), самые высокоуровневые события
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppLogJournalKind {
    CreateUser(CreateUser),
    DeleteUser(DeleteUser),
    RegisterAsset(RegisterAsset),
    UnregisterAsset(UnregisterAsset),
    DepositCash(UserCash),
    WithdrawCash(UserCash),
    BuyAsset(UserBacket),
    SellAsset(UserBacket),
}
impl Parsable for SystemLogErrorKind {
    type Parser = Preceded<
        Tag,
        Alt<(
            Map<
                Preceded<StripWhitespace<Tag>, StripWhitespace<Unquote>>,
                fn(String) -> Self,
            >,
            Map<
                Preceded<StripWhitespace<Tag>, StripWhitespace<Unquote>>,
                fn(String) -> Self,
            >,
        )>,
    >;
    fn parser() -> Self::Parser {
        preceded(
            tag("Error"),
            alt2(
                map(
                    preceded(
                        strip_whitespace(tag("NetworkError")),
                        strip_whitespace(unquote()),
                    ),
                    Self::NetworkError,
                ),
                map(
                    preceded(
                        strip_whitespace(tag("AccessDenied")),
                        strip_whitespace(unquote()),
                    ),
                    Self::AccessDenied,
                ),
            ),
        )
    }
}
impl Parsable for SystemLogTraceKind {
    type Parser = Preceded<
        Tag,
        Alt<(
            Map<
                Preceded<StripWhitespace<Tag>, StripWhitespace<Unquote>>,
                fn(String) -> Self,
            >,
            Map<
                Preceded<StripWhitespace<Tag>, StripWhitespace<Unquote>>,
                fn(String) -> Self,
            >,
        )>,
    >;
    fn parser() -> Self::Parser {
        preceded(
            tag("Trace"),
            alt2(
                map(
                    preceded(
                        strip_whitespace(tag("SendRequest")),
                        strip_whitespace(unquote()),
                    ),
                    Self::SendRequest,
                ),
                map(
                    preceded(
                        strip_whitespace(tag("GetResponse")),
                        strip_whitespace(unquote()),
                    ),
                    Self::GetResponse,
                ),
            ),
        )
    }
}
impl Parsable for SystemLogKind {
    type Parser = StripWhitespace<
        Preceded<
            Tag,
            Alt<(
                Map<
                    <SystemLogTraceKind as Parsable>::Parser,
                    fn(SystemLogTraceKind) -> Self,
                >,
                Map<
                    <SystemLogErrorKind as Parsable>::Parser,
                    fn(SystemLogErrorKind) -> Self,
                >,
            )>,
        >,
    >;
    fn parser() -> Self::Parser {
        strip_whitespace(preceded(
            tag("System::"),
            alt2(
                map(SystemLogTraceKind::parser(), Self::Trace),
                map(SystemLogErrorKind::parser(), Self::Error),
            ),
        ))
    }
}
impl Parsable for AppLogErrorKind {
    type Parser = Preceded<
        Tag,
        Alt<(
            Map<
                Preceded<StripWhitespace<Tag>, StripWhitespace<Unquote>>,
                fn(String) -> Self,
            >,
            Map<
                Preceded<StripWhitespace<Tag>, StripWhitespace<Unquote>>,
                fn(String) -> Self,
            >,
        )>,
    >;
    fn parser() -> Self::Parser {
        preceded(
            tag("Error"),
            alt2(
                map(
                    preceded(
                        strip_whitespace(tag("LackOf")),
                        strip_whitespace(unquote()),
                    ),
                    Self::LackOf,
                ),
                map(
                    preceded(
                        strip_whitespace(tag("SystemError")),
                        strip_whitespace(unquote()),
                    ),
                    Self::SystemError,
                ),
            ),
        )
    }
}
impl Parsable for AppLogTraceKind {
    type Parser = Preceded<
        Tag,
        Alt<(
            Map<
                Preceded<
                    StripWhitespace<Tag>,
                    StripWhitespace<<AuthData as Parsable>::Parser>,
                >,
                fn(AuthData) -> Self,
            >,
            Map<
                Preceded<StripWhitespace<Tag>, StripWhitespace<Unquote>>,
                fn(String) -> Self,
            >,
            Map<
                Preceded<
                    StripWhitespace<Tag>,
                    StripWhitespace<<Announcements as Parsable>::Parser>,
                >,
                fn(Announcements) -> Self,
            >,
            Map<
                Preceded<StripWhitespace<Tag>, StripWhitespace<Unquote>>,
                fn(String) -> Self,
            >,
        )>,
    >;
    fn parser() -> Self::Parser {
        preceded(
            tag("Trace"),
            alt4(
                map(
                    preceded(
                        strip_whitespace(tag("Connect")),
                        strip_whitespace(AuthData::parser()),
                    ),
                    |authdata| Self::Connect(Box::new(authdata)),
                ),
                map(
                    preceded(
                        strip_whitespace(tag("SendRequest")),
                        strip_whitespace(unquote()),
                    ),
                    Self::SendRequest,
                ),
                map(
                    preceded(
                        strip_whitespace(tag("Check")),
                        strip_whitespace(Announcements::parser()),
                    ),
                    Self::Check,
                ),
                map(
                    preceded(
                        strip_whitespace(tag("GetResponse")),
                        strip_whitespace(unquote()),
                    ),
                    Self::GetResponse,
                ),
            ),
        )
    }
}
impl Parsable for CreateUser {
    type Parser = Map<
        Delimited<
            Tag,
            Permutation<(KeyValue<Unquote>, KeyValue<stdp::U32>)>,
            Tag,
        >,
        fn((String, NonZeroU32)) -> Self,
    >;
    fn parser() -> Self::Parser {
        map(
            delimited(
                tag("{"),
                permutation2(
                    key_value("user_id", unquote()),
                    key_value("authorized_capital", stdp::U32),
                ),
                tag("}"),
            ),
            |(user_id, authorized_capital)| Self {
                user_id,
                authorized_capital,
            },
        )
    }
}
impl Parsable for DeleteUser {
    type Parser =
        Map<Delimited<Tag, KeyValue<Unquote>, Tag>, fn(String) -> Self>;
    fn parser() -> Self::Parser {
        map(
            delimited(tag("{"), key_value("user_id", unquote()), tag("}")),
            |user_id| Self { user_id },
        )
    }
}
impl Parsable for RegisterAsset {
    type Parser = Map<
        Delimited<
            Tag,
            Permutation<(
                KeyValue<Unquote>,
                KeyValue<Unquote>,
                KeyValue<stdp::U32>,
            )>,
            Tag,
        >,
        fn((String, String, NonZeroU32)) -> Self,
    >;
    fn parser() -> Self::Parser {
        map(
            delimited(
                tag("{"),
                permutation3(
                    key_value("asset_id", unquote()),
                    key_value("user_id", unquote()),
                    key_value("liquidity", stdp::U32),
                ),
                tag("}"),
            ),
            |(asset_id, user_id, liquidity)| Self {
                asset_id,
                user_id,
                liquidity,
            },
        )
    }
}
impl Parsable for UnregisterAsset {
    type Parser = Map<
        Delimited<
            Tag,
            Permutation<(KeyValue<Unquote>, KeyValue<Unquote>)>,
            Tag,
        >,
        fn((String, String)) -> Self,
    >;
    fn parser() -> Self::Parser {
        map(
            delimited(
                tag("{"),
                permutation2(
                    key_value("asset_id", unquote()),
                    key_value("user_id", unquote()),
                ),
                tag("}"),
            ),
            |(asset_id, user_id)| Self { asset_id, user_id },
        )
    }
}
impl Parsable for AppLogJournalKind {
    type Parser = Preceded<
        Tag,
        Alt<(
            Map<
                Preceded<
                    StripWhitespace<Tag>,
                    <CreateUser as Parsable>::Parser,
                >,
                fn(CreateUser) -> Self,
            >,
            Map<
                Preceded<
                    StripWhitespace<Tag>,
                    <DeleteUser as Parsable>::Parser,
                >,
                fn(DeleteUser) -> Self,
            >,
            Map<
                Preceded<
                    StripWhitespace<Tag>,
                    <RegisterAsset as Parsable>::Parser,
                >,
                fn(RegisterAsset) -> Self,
            >,
            Map<
                Preceded<
                    StripWhitespace<Tag>,
                    <UnregisterAsset as Parsable>::Parser,
                >,
                fn(UnregisterAsset) -> Self,
            >,
            Map<
                Preceded<StripWhitespace<Tag>, <UserCash as Parsable>::Parser>,
                fn(UserCash) -> Self,
            >,
            Map<
                Preceded<StripWhitespace<Tag>, <UserCash as Parsable>::Parser>,
                fn(UserCash) -> Self,
            >,
            Map<
                Preceded<
                    StripWhitespace<Tag>,
                    <UserBacket as Parsable>::Parser,
                >,
                fn(UserBacket) -> Self,
            >,
            Map<
                Preceded<
                    StripWhitespace<Tag>,
                    <UserBacket as Parsable>::Parser,
                >,
                fn(UserBacket) -> Self,
            >,
        )>,
    >;
    fn parser() -> Self::Parser {
        preceded(
            tag("Journal"),
            alt8(
                map(
                    preceded(
                        strip_whitespace(tag("CreateUser")),
                        CreateUser::parser(),
                    ),
                    Self::CreateUser,
                ),
                map(
                    preceded(
                        strip_whitespace(tag("DeleteUser")),
                        DeleteUser::parser(),
                    ),
                    Self::DeleteUser,
                ),
                map(
                    preceded(
                        strip_whitespace(tag("RegisterAsset")),
                        RegisterAsset::parser(),
                    ),
                    Self::RegisterAsset,
                ),
                map(
                    preceded(
                        strip_whitespace(tag("UnregisterAsset")),
                        UnregisterAsset::parser(),
                    ),
                    Self::UnregisterAsset,
                ),
                map(
                    preceded(
                        strip_whitespace(tag("DepositCash")),
                        UserCash::parser(),
                    ),
                    Self::DepositCash,
                ),
                map(
                    preceded(
                        strip_whitespace(tag("WithdrawCash")),
                        UserCash::parser(),
                    ),
                    Self::WithdrawCash,
                ),
                map(
                    preceded(
                        strip_whitespace(tag("BuyAsset")),
                        UserBacket::parser(),
                    ),
                    Self::BuyAsset,
                ),
                map(
                    preceded(
                        strip_whitespace(tag("SellAsset")),
                        UserBacket::parser(),
                    ),
                    Self::SellAsset,
                ),
            ),
        )
    }
}
impl Parsable for AppLogKind {
    type Parser = StripWhitespace<
        Preceded<
            Tag,
            Alt<(
                Map<
                    <AppLogErrorKind as Parsable>::Parser,
                    fn(AppLogErrorKind) -> Self,
                >,
                Map<
                    <AppLogTraceKind as Parsable>::Parser,
                    fn(AppLogTraceKind) -> Self,
                >,
                Map<
                    <AppLogJournalKind as Parsable>::Parser,
                    fn(AppLogJournalKind) -> Self,
                >,
            )>,
        >,
    >;
    fn parser() -> Self::Parser {
        strip_whitespace(preceded(
            tag("App::"),
            alt3(
                map(AppLogErrorKind::parser(), Self::Error),
                map(AppLogTraceKind::parser(), Self::Trace),
                map(AppLogJournalKind::parser(), Self::Journal),
            ),
        ))
    }
}
impl Parsable for LogKind {
    type Parser = StripWhitespace<
        Alt<(
            Map<<SystemLogKind as Parsable>::Parser, fn(SystemLogKind) -> Self>,
            Map<<AppLogKind as Parsable>::Parser, fn(AppLogKind) -> Self>,
        )>,
    >;
    fn parser() -> Self::Parser {
        strip_whitespace(alt2(
            map(SystemLogKind::parser(), Self::System),
            map(AppLogKind::parser(), Self::App),
        ))
    }
}
/// Строка логов, [лог](AppLogKind) с `request_id`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogLine {
    pub kind: LogKind,
    pub request_id: NonZeroU32,
}
impl Parsable for LogLine {
    type Parser = Map<
        All<(
            <LogKind as Parsable>::Parser,
            StripWhitespace<Preceded<Tag, stdp::U32>>,
        )>,
        fn((LogKind, NonZeroU32)) -> Self,
    >;
    fn parser() -> Self::Parser {
        map(
            all2(
                LogKind::parser(),
                strip_whitespace(preceded(tag("requestid="), stdp::U32)),
            ),
            |(kind, request_id)| Self { kind, request_id },
        )
    }
}

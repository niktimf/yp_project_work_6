use std::num::NonZeroU32;

use super::Parsable;
use super::combinators::{Preceded, Tag, Alt, Map, StripWhitespace, Unquote, preceded, tag, alt2, map, strip_whitespace, unquote, alt4, Delimited, Permutation, KeyValue, alt8, delimited, permutation2, key_value, permutation3, alt3, All, all2};
use super::stdp;
use super::types::{AuthData, Announcements, UserCash, UserBacket};

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
/// Журнал [приложения](AppLogKind), самые высокоуровневые события
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppLogJournalKind {
    CreateUser {
        user_id: String,
        authorized_capital: NonZeroU32,
    },
    DeleteUser {
        user_id: String,
    },
    RegisterAsset {
        asset_id: String,
        user_id: String,
        liquidity: NonZeroU32,
    },
    UnregisterAsset {
        asset_id: String,
        user_id: String,
    },
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
                map(SystemLogTraceKind::parser(), |trace| {
                    Self::Trace(trace)
                }),
                map(SystemLogErrorKind::parser(), |error| {
                    Self::Error(error)
                }),
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
impl Parsable for AppLogJournalKind {
    type Parser = Preceded<
        Tag,
        Alt<(
            Map<
                Preceded<
                    StripWhitespace<Tag>,
                    Delimited<
                        Tag,
                        Permutation<(KeyValue<Unquote>, KeyValue<stdp::U32>)>,
                        Tag,
                    >,
                >,
                fn((String, NonZeroU32)) -> Self,
            >,
            Map<
                Preceded<
                    StripWhitespace<Tag>,
                    Delimited<Tag, KeyValue<Unquote>, Tag>,
                >,
                fn(String) -> Self,
            >,
            Map<
                Preceded<
                    StripWhitespace<Tag>,
                    Delimited<
                        Tag,
                        Permutation<(
                            KeyValue<Unquote>,
                            KeyValue<Unquote>,
                            KeyValue<stdp::U32>,
                        )>,
                        Tag,
                    >,
                >,
                fn((String, String, NonZeroU32)) -> Self,
            >,
            Map<
                Preceded<
                    StripWhitespace<Tag>,
                    Delimited<
                        Tag,
                        Permutation<(KeyValue<Unquote>, KeyValue<Unquote>)>,
                        Tag,
                    >,
                >,
                fn((String, String)) -> Self,
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
    #[allow(clippy::too_many_lines)]
    fn parser() -> Self::Parser {
        preceded(
            tag("Journal"),
            alt8(
                map(
                    preceded(
                        strip_whitespace(tag("CreateUser")),
                        delimited(
                            tag("{"),
                            permutation2(
                                key_value("user_id", unquote()),
                                key_value("authorized_capital", stdp::U32),
                            ),
                            tag("}"),
                        ),
                    ),
                    |(user_id, authorized_capital)| {
                        Self::CreateUser {
                            user_id,
                            authorized_capital,
                        }
                    },
                ),
                map(
                    preceded(
                        strip_whitespace(tag("DeleteUser")),
                        delimited(
                            tag("{"),
                            key_value("user_id", unquote()),
                            tag("}"),
                        ),
                    ),
                    |user_id| Self::DeleteUser { user_id },
                ),
                map(
                    preceded(
                        strip_whitespace(tag("RegisterAsset")),
                        delimited(
                            tag("{"),
                            permutation3(
                                key_value("asset_id", unquote()),
                                key_value("user_id", unquote()),
                                key_value("liquidity", stdp::U32),
                            ),
                            tag("}"),
                        ),
                    ),
                    |(asset_id, user_id, liquidity)| {
                        Self::RegisterAsset {
                            asset_id,
                            user_id,
                            liquidity,
                        }
                    },
                ),
                map(
                    preceded(
                        strip_whitespace(tag("UnregisterAsset")),
                        delimited(
                            tag("{"),
                            permutation2(
                                key_value("asset_id", unquote()),
                                key_value("user_id", unquote()),
                            ),
                            tag("}"),
                        ),
                    ),
                    |(asset_id, user_id)| Self::UnregisterAsset {
                        asset_id,
                        user_id,
                    },
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
                map(AppLogErrorKind::parser(), |error| {
                    Self::Error(error)
                }),
                map(AppLogTraceKind::parser(), |trace| {
                    Self::Trace(trace)
                }),
                map(AppLogJournalKind::parser(), |journal| {
                    Self::Journal(journal)
                }),
            ),
        ))
    }
}
impl Parsable for LogKind {
    type Parser = StripWhitespace<
        Alt<(
            Map<
                <SystemLogKind as Parsable>::Parser,
                fn(SystemLogKind) -> Self,
            >,
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

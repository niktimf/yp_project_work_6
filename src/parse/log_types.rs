use super::Parsable;
use super::combinators::*;
use super::stdp;
use super::types::*;

/// Все виды логов
#[derive(Debug, Clone, PartialEq)]
pub enum LogKind {
    System(SystemLogKind),
    App(AppLogKind),
}
/// Все виды [системных](LogKind) логов
#[derive(Debug, Clone, PartialEq)]
pub enum SystemLogKind {
    Error(SystemLogErrorKind),
    Trace(SystemLogTraceKind),
}
/// Trace [системы](SystemLogKind)
#[derive(Debug, Clone, PartialEq)]
pub enum SystemLogTraceKind {
    SendRequest(String),
    GetResponse(String),
}
/// Error [системы](SystemLogKind)
#[derive(Debug, Clone, PartialEq)]
pub enum SystemLogErrorKind {
    NetworkError(String),
    AccessDenied(String),
}
/// Все виды [логов приложения](LogKind) логов
#[derive(Debug, Clone, PartialEq)]
pub enum AppLogKind {
    Error(AppLogErrorKind),
    Trace(AppLogTraceKind),
    Journal(AppLogJournalKind),
}
/// Error [приложения](AppLogKind)
#[derive(Debug, Clone, PartialEq)]
pub enum AppLogErrorKind {
    LackOf(String),
    SystemError(String),
}
/// Trace [приложения](AppLogKind)
#[derive(Debug, Clone, PartialEq)]
pub enum AppLogTraceKind {
    Connect(Box<AuthData>),
    SendRequest(String),
    Check(Announcements),
    GetResponse(String),
}
/// Журнал [приложения](AppLogKind), самые высокоуровневые события
#[derive(Debug, Clone, PartialEq)]
pub enum AppLogJournalKind {
    CreateUser {
        user_id: String,
        authorized_capital: u32,
    },
    DeleteUser {
        user_id: String,
    },
    RegisterAsset {
        asset_id: String,
        user_id: String,
        liquidity: u32,
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
                fn(String) -> SystemLogErrorKind,
            >,
            Map<
                Preceded<StripWhitespace<Tag>, StripWhitespace<Unquote>>,
                fn(String) -> SystemLogErrorKind,
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
                    |error| SystemLogErrorKind::NetworkError(error),
                ),
                map(
                    preceded(
                        strip_whitespace(tag("AccessDenied")),
                        strip_whitespace(unquote()),
                    ),
                    |error| SystemLogErrorKind::AccessDenied(error),
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
                fn(String) -> SystemLogTraceKind,
            >,
            Map<
                Preceded<StripWhitespace<Tag>, StripWhitespace<Unquote>>,
                fn(String) -> SystemLogTraceKind,
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
                    |request| SystemLogTraceKind::SendRequest(request),
                ),
                map(
                    preceded(
                        strip_whitespace(tag("GetResponse")),
                        strip_whitespace(unquote()),
                    ),
                    |response| SystemLogTraceKind::GetResponse(response),
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
                    fn(SystemLogTraceKind) -> SystemLogKind,
                >,
                Map<
                    <SystemLogErrorKind as Parsable>::Parser,
                    fn(SystemLogErrorKind) -> SystemLogKind,
                >,
            )>,
        >,
    >;
    fn parser() -> Self::Parser {
        strip_whitespace(preceded(
            tag("System::"),
            alt2(
                map(SystemLogTraceKind::parser(), |trace| {
                    SystemLogKind::Trace(trace)
                }),
                map(SystemLogErrorKind::parser(), |error| {
                    SystemLogKind::Error(error)
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
                fn(String) -> AppLogErrorKind,
            >,
            Map<
                Preceded<StripWhitespace<Tag>, StripWhitespace<Unquote>>,
                fn(String) -> AppLogErrorKind,
            >,
        )>,
    >;
    fn parser() -> Self::Parser {
        preceded(
            tag("Error"),
            alt2(
                map(
                    preceded(strip_whitespace(tag("LackOf")), strip_whitespace(unquote())),
                    |error| AppLogErrorKind::LackOf(error),
                ),
                map(
                    preceded(
                        strip_whitespace(tag("SystemError")),
                        strip_whitespace(unquote()),
                    ),
                    |error| AppLogErrorKind::SystemError(error),
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
                Preceded<StripWhitespace<Tag>, StripWhitespace<<AuthData as Parsable>::Parser>>,
                fn(AuthData) -> AppLogTraceKind,
            >,
            Map<
                Preceded<StripWhitespace<Tag>, StripWhitespace<Unquote>>,
                fn(String) -> AppLogTraceKind,
            >,
            Map<
                Preceded<
                    StripWhitespace<Tag>,
                    StripWhitespace<<Announcements as Parsable>::Parser>,
                >,
                fn(Announcements) -> AppLogTraceKind,
            >,
            Map<
                Preceded<StripWhitespace<Tag>, StripWhitespace<Unquote>>,
                fn(String) -> AppLogTraceKind,
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
                    |authdata| AppLogTraceKind::Connect(Box::new(authdata)),
                ),
                map(
                    preceded(
                        strip_whitespace(tag("SendRequest")),
                        strip_whitespace(unquote()),
                    ),
                    |trace| AppLogTraceKind::SendRequest(trace),
                ),
                map(
                    preceded(
                        strip_whitespace(tag("Check")),
                        strip_whitespace(Announcements::parser()),
                    ),
                    |announcements| AppLogTraceKind::Check(announcements),
                ),
                map(
                    preceded(
                        strip_whitespace(tag("GetResponse")),
                        strip_whitespace(unquote()),
                    ),
                    |trace| AppLogTraceKind::GetResponse(trace),
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
                    Delimited<Tag, Permutation<(KeyValue<Unquote>, KeyValue<stdp::U32>)>, Tag>,
                >,
                fn((String, u32)) -> AppLogJournalKind,
            >,
            Map<
                Preceded<StripWhitespace<Tag>, Delimited<Tag, KeyValue<Unquote>, Tag>>,
                fn(String) -> AppLogJournalKind,
            >,
            Map<
                Preceded<
                    StripWhitespace<Tag>,
                    Delimited<
                        Tag,
                        Permutation<(KeyValue<Unquote>, KeyValue<Unquote>, KeyValue<stdp::U32>)>,
                        Tag,
                    >,
                >,
                fn((String, String, u32)) -> AppLogJournalKind,
            >,
            Map<
                Preceded<
                    StripWhitespace<Tag>,
                    Delimited<Tag, Permutation<(KeyValue<Unquote>, KeyValue<Unquote>)>, Tag>,
                >,
                fn((String, String)) -> AppLogJournalKind,
            >,
            Map<
                Preceded<StripWhitespace<Tag>, <UserCash as Parsable>::Parser>,
                fn(UserCash) -> AppLogJournalKind,
            >,
            Map<
                Preceded<StripWhitespace<Tag>, <UserCash as Parsable>::Parser>,
                fn(UserCash) -> AppLogJournalKind,
            >,
            Map<
                Preceded<StripWhitespace<Tag>, <UserBacket as Parsable>::Parser>,
                fn(UserBacket) -> AppLogJournalKind,
            >,
            Map<
                Preceded<StripWhitespace<Tag>, <UserBacket as Parsable>::Parser>,
                fn(UserBacket) -> AppLogJournalKind,
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
                        delimited(
                            tag("{"),
                            permutation2(
                                key_value("user_id", unquote()),
                                key_value("authorized_capital", stdp::U32),
                            ),
                            tag("}"),
                        ),
                    ),
                    |(user_id, authorized_capital)| AppLogJournalKind::CreateUser {
                        user_id,
                        authorized_capital,
                    },
                ),
                map(
                    preceded(
                        strip_whitespace(tag("DeleteUser")),
                        delimited(tag("{"), key_value("user_id", unquote()), tag("}")),
                    ),
                    |user_id| AppLogJournalKind::DeleteUser { user_id },
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
                    |(asset_id, user_id, liquidity)| AppLogJournalKind::RegisterAsset {
                        asset_id,
                        user_id,
                        liquidity,
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
                    |(asset_id, user_id)| AppLogJournalKind::UnregisterAsset { asset_id, user_id },
                ),
                map(
                    preceded(strip_whitespace(tag("DepositCash")), UserCash::parser()),
                    |user_cash| AppLogJournalKind::DepositCash(user_cash),
                ),
                map(
                    preceded(strip_whitespace(tag("WithdrawCash")), UserCash::parser()),
                    |user_cash| AppLogJournalKind::DepositCash(user_cash),
                ),
                map(
                    preceded(strip_whitespace(tag("BuyAsset")), UserBacket::parser()),
                    |user_backet| AppLogJournalKind::BuyAsset(user_backet),
                ),
                map(
                    preceded(strip_whitespace(tag("SellAsset")), UserBacket::parser()),
                    |user_backet| AppLogJournalKind::SellAsset(user_backet),
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
                Map<<AppLogErrorKind as Parsable>::Parser, fn(AppLogErrorKind) -> AppLogKind>,
                Map<<AppLogTraceKind as Parsable>::Parser, fn(AppLogTraceKind) -> AppLogKind>,
                Map<<AppLogJournalKind as Parsable>::Parser, fn(AppLogJournalKind) -> AppLogKind>,
            )>,
        >,
    >;
    fn parser() -> Self::Parser {
        strip_whitespace(preceded(
            tag("App::"),
            alt3(
                map(AppLogErrorKind::parser(), |error| AppLogKind::Error(error)),
                map(AppLogTraceKind::parser(), |trace| AppLogKind::Trace(trace)),
                map(AppLogJournalKind::parser(), |journal| {
                    AppLogKind::Journal(journal)
                }),
            ),
        ))
    }
}
impl Parsable for LogKind {
    type Parser = StripWhitespace<
        Alt<(
            Map<<SystemLogKind as Parsable>::Parser, fn(SystemLogKind) -> LogKind>,
            Map<<AppLogKind as Parsable>::Parser, fn(AppLogKind) -> LogKind>,
        )>,
    >;
    fn parser() -> Self::Parser {
        strip_whitespace(alt2(
            map(SystemLogKind::parser(), |system| LogKind::System(system)),
            map(AppLogKind::parser(), |app| LogKind::App(app)),
        ))
    }
}
/// Строка логов, [лог](AppLogKind) с `request_id`
#[derive(Debug, Clone, PartialEq)]
pub struct LogLine {
    pub kind: LogKind,
    pub request_id: u32,
}
impl Parsable for LogLine {
    type Parser = Map<
        All<(
            <LogKind as Parsable>::Parser,
            StripWhitespace<Preceded<Tag, stdp::U32>>,
        )>,
        fn((LogKind, u32)) -> Self,
    >;
    fn parser() -> Self::Parser {
        map(
            all2(
                LogKind::parser(),
                strip_whitespace(preceded(tag("requestid="), stdp::U32)),
            ),
            |(kind, request_id)| LogLine { kind, request_id },
        )
    }
}

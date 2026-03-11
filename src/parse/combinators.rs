use super::Parser;

/// Обернуть строку в кавычки, экранировав кавычки, которые в строке уже есть
pub fn quote(input: &str) -> String {
    let mut result = String::from("\"");
    result.extend(
        input
            .chars()
            .flat_map(|c| match c {
                '\\' | '"' => ['\\', c].into_iter().take(2),
                _ => [c, ' '].into_iter().take(1),
            }),
    );
    result.push('"');
    result
}
/// Распарсить строку, которую ранее [обернули в кавычки](quote)
// `"abc\"def\\ghi"nice` -> (`abcd"def\ghi`, `nice`)
pub fn do_unquote(input: &str) -> Result<(&str, String), ()> {
    let mut result = String::new();
    let mut escaped_now = false;
    let mut chars = input.strip_prefix("\"").ok_or(())?.chars();
    while let Some(c) = chars.next() {
        match (c, escaped_now) {
            ('"' | '\\', true) => {
                result.push(c);
                escaped_now = false;
            }
            ('\\', false) => escaped_now = true,
            ('"', false) => return Ok((chars.as_str(), result)),
            (c, _) => {
                result.push(c);
                escaped_now = false;
            }
        }
    }
    Err(()) // строка кончилась, не закрыв кавычку
}
/// Распарсить строку, обёрную в кавычки
/// (сокращённая версия [`do_unquote`], в которой вложенные кавычки не предусмотрены)
pub fn do_unquote_non_escaped(input: &str) -> Result<(&str, &str), ()> {
    let input = input.strip_prefix("\"").ok_or(())?;
    let quote_byteidx = input.find('"').ok_or(())?;
    if 0 == quote_byteidx
        || Some("\\") == input.get(quote_byteidx - 1..quote_byteidx)
    {
        return Err(());
    }
    Ok((&input[1 + quote_byteidx..], &input[..quote_byteidx]))
}
/// Парсер кавычек
#[derive(Debug, Clone)]
pub struct Unquote;
impl Parser for Unquote {
    type Dest = String;
    fn parse<'a>(&self, input: &'a str) -> Result<(&'a str, Self::Dest), ()> {
        do_unquote(input)
    }
}
/// Конструктор [Unquote]
pub const fn unquote() -> Unquote {
    Unquote
}
/// Парсер, возвращающий результат как есть
#[derive(Debug, Clone)]
pub struct AsIs;
impl Parser for AsIs {
    type Dest = String;
    fn parse<'a>(&self, input: &'a str) -> Result<(&'a str, Self::Dest), ()> {
        Ok(("", input.to_string()))
    }
}
/// Парсер константных строк
/// (аналог `nom::bytes::complete::tag`)
#[derive(Debug, Clone)]
pub struct Tag {
    pub(crate) tag: &'static str,
}
impl Parser for Tag {
    type Dest = ();
    fn parse<'a>(&self, input: &'a str) -> Result<(&'a str, Self::Dest), ()> {
        Ok((input.strip_prefix(self.tag).ok_or(())?, ()))
    }
}
/// Конструктор [Tag]
pub const fn tag(tag: &'static str) -> Tag {
    Tag { tag }
}
/// Парсер [тэга](Tag), обёрнутого в кавычки
#[derive(Debug, Clone)]
pub struct QuotedTag(pub(crate) Tag);
impl Parser for QuotedTag {
    type Dest = ();
    fn parse<'a>(&self, input: &'a str) -> Result<(&'a str, Self::Dest), ()> {
        let (remaining, candidate) = do_unquote_non_escaped(input)?;
        if !self.0.parse(candidate)?.0.is_empty() {
            return Err(());
        }
        Ok((remaining, ()))
    }
}
/// Конструктор [`QuotedTag`]
pub const fn quoted_tag(tag: &'static str) -> QuotedTag {
    QuotedTag(Tag { tag })
}
/// Комбинатор, пробрасывающий строку без лидирующих пробелов
#[derive(Debug, Clone)]
pub struct StripWhitespace<T> {
    pub(crate) parser: T,
}
impl<T: Parser> Parser for StripWhitespace<T> {
    type Dest = T::Dest;
    fn parse<'a>(&self, input: &'a str) -> Result<(&'a str, Self::Dest), ()> {
        self.parser
            .parse(input.trim_start())
            .map(|(remaining, parsed)| (remaining.trim_start(), parsed))
    }
}
/// Конструктор [`StripWhitespace`]
pub const fn strip_whitespace<T: Parser>(parser: T) -> StripWhitespace<T> {
    StripWhitespace { parser }
}
/// Комбинатор, чтобы распарсить нужное, окружённое в начале и в конце чем-то
/// обязательным, не участвующем в результате.
/// Пробрасывает строку в парсер1, оставшуюся строку после первого
/// парсинга - в парсер2, оставшуюся строку после второго парсинга - в парсер3.
/// Результат парсера2 будет результатом этого комбинатора, а оставшейся
/// строкой - строка, оставшаяся после парсера3.
/// (аналог `delimited` из `nom`)
#[derive(Debug, Clone)]
pub struct Delimited<Prefix, T, Suffix> {
    pub(crate) prefix_to_ignore: Prefix,
    pub(crate) dest_parser: T,
    pub(crate) suffix_to_ignore: Suffix,
}
impl<Prefix, T, Suffix> Parser for Delimited<Prefix, T, Suffix>
where
    Prefix: Parser,
    T: Parser,
    Suffix: Parser,
{
    type Dest = T::Dest;
    fn parse<'a>(&self, input: &'a str) -> Result<(&'a str, Self::Dest), ()> {
        let (remaining, _) = self.prefix_to_ignore.parse(input)?;
        let (remaining, result) = self.dest_parser.parse(remaining)?;
        self.suffix_to_ignore
            .parse(remaining)
            .map(|(remaining, _)| (remaining, result))
    }
}
/// Конструктор [Delimited]
pub const fn delimited<Prefix, T, Suffix>(
    prefix_to_ignore: Prefix,
    dest_parser: T,
    suffix_to_ignore: Suffix,
) -> Delimited<Prefix, T, Suffix>
where
    Prefix: Parser,
    T: Parser,
    Suffix: Parser,
{
    Delimited {
        prefix_to_ignore,
        dest_parser,
        suffix_to_ignore,
    }
}
/// Комбинатор-отображение. Парсит дочерним парсером, преобразует результат так,
/// как вызывающему хочется
#[derive(Debug, Clone)]
pub struct Map<T, M> {
    pub(crate) parser: T,
    pub(crate) map: M,
}
impl<T: Parser, Dest: Sized, M: Fn(T::Dest) -> Dest> Parser for Map<T, M> {
    type Dest = Dest;
    fn parse<'a>(&self, input: &'a str) -> Result<(&'a str, Self::Dest), ()> {
        self.parser
            .parse(input)
            .map(|(remaining, pre_result)| (remaining, (self.map)(pre_result)))
    }
}
/// Конструктор [Map]
pub const fn map<T: Parser, Dest: Sized, M: Fn(T::Dest) -> Dest>(
    parser: T,
    map: M,
) -> Map<T, M> {
    Map { parser, map }
}
/// Комбинатор с отбрасываемым префиксом, упрощённая версия [Delimited]
/// (аналог `preceeded` из `nom`)
#[derive(Debug, Clone)]
pub struct Preceded<Prefix, T> {
    pub(crate) prefix_to_ignore: Prefix,
    pub(crate) dest_parser: T,
}
impl<Prefix, T> Parser for Preceded<Prefix, T>
where
    Prefix: Parser,
    T: Parser,
{
    type Dest = T::Dest;
    fn parse<'a>(&self, input: &'a str) -> Result<(&'a str, Self::Dest), ()> {
        let (remaining, _) = self.prefix_to_ignore.parse(input)?;
        self.dest_parser.parse(remaining)
    }
}
/// Конструктор [Preceded]
pub const fn preceded<Prefix, T>(
    prefix_to_ignore: Prefix,
    dest_parser: T,
) -> Preceded<Prefix, T>
where
    Prefix: Parser,
    T: Parser,
{
    Preceded {
        prefix_to_ignore,
        dest_parser,
    }
}
/// Комбинатор, который требует, чтобы все дочерние парсеры отработали,
/// (аналог `all` из `nom`)
#[derive(Debug, Clone)]
pub struct All<T> {
    pub(crate) parser: T,
}
impl<A0, A1> Parser for All<(A0, A1)>
where
    A0: Parser,
    A1: Parser,
{
    type Dest = (A0::Dest, A1::Dest);
    fn parse<'a>(&self, input: &'a str) -> Result<(&'a str, Self::Dest), ()> {
        let (remaining, a0) = self.parser.0.parse(input)?;
        self.parser
            .1
            .parse(remaining)
            .map(|(remaining, a1)| (remaining, (a0, a1)))
    }
}
/// Конструктор [All] для двух парсеров
/// (в Rust нет чего-то, вроде variadic templates из C++)
pub const fn all2<A0: Parser, A1: Parser>(a0: A0, a1: A1) -> All<(A0, A1)> {
    All { parser: (a0, a1) }
}
impl<A0, A1, A2> Parser for All<(A0, A1, A2)>
where
    A0: Parser,
    A1: Parser,
    A2: Parser,
{
    type Dest = (A0::Dest, A1::Dest, A2::Dest);
    fn parse<'a>(&self, input: &'a str) -> Result<(&'a str, Self::Dest), ()> {
        let (remaining, a0) = self.parser.0.parse(input)?;
        let (remaining, a1) = self.parser.1.parse(remaining)?;
        self.parser
            .2
            .parse(remaining)
            .map(|(remaining, a2)| (remaining, (a0, a1, a2)))
    }
}
/// Конструктор [All] для трёх парсеров
/// (в Rust нет чего-то, вроде variadic templates из C++)
pub const fn all3<A0: Parser, A1: Parser, A2: Parser>(
    a0: A0,
    a1: A1,
    a2: A2,
) -> All<(A0, A1, A2)> {
    All {
        parser: (a0, a1, a2),
    }
}
impl<A0, A1, A2, A3> Parser for All<(A0, A1, A2, A3)>
where
    A0: Parser,
    A1: Parser,
    A2: Parser,
    A3: Parser,
{
    type Dest = (A0::Dest, A1::Dest, A2::Dest, A3::Dest);
    fn parse<'a>(&self, input: &'a str) -> Result<(&'a str, Self::Dest), ()> {
        let (remaining, a0) = self.parser.0.parse(input)?;
        let (remaining, a1) = self.parser.1.parse(remaining)?;
        let (remaining, a2) = self.parser.2.parse(remaining)?;
        self.parser
            .3
            .parse(remaining)
            .map(|(remaining, a3)| (remaining, (a0, a1, a2, a3)))
    }
}
/// Конструктор [All] для четырёх парсеров
/// (в Rust нет чего-то, вроде variadic templates из C++)
pub const fn all4<A0: Parser, A1: Parser, A2: Parser, A3: Parser>(
    a0: A0,
    a1: A1,
    a2: A2,
    a3: A3,
) -> All<(A0, A1, A2, A3)> {
    All {
        parser: (a0, a1, a2, a3),
    }
}
/// Комбинатор, который вытаскивает значения из пары `"ключ":значение,`.
/// Для простоты реализации, запятая всегда нужна в конце пары ключ-значение,
/// простое '"ключ":значение' читаться не будет
#[derive(Debug, Clone)]
pub struct KeyValue<T> {
    pub(crate) parser: Delimited<
        All<(StripWhitespace<QuotedTag>, StripWhitespace<Tag>)>,
        StripWhitespace<T>,
        StripWhitespace<Tag>,
    >,
}
impl<T> Parser for KeyValue<T>
where
    T: Parser,
{
    type Dest = T::Dest;
    fn parse<'a>(&self, input: &'a str) -> Result<(&'a str, Self::Dest), ()> {
        self.parser.parse(input)
    }
}
/// Конструктор [`KeyValue`]
pub const fn key_value<T: Parser>(
    key: &'static str,
    value_parser: T,
) -> KeyValue<T> {
    KeyValue {
        parser: delimited(
            all2(strip_whitespace(quoted_tag(key)), strip_whitespace(tag(":"))),
            strip_whitespace(value_parser),
            strip_whitespace(tag(",")),
        ),
    }
}
/// Комбинатор, который возвращает результаты дочерних парсеров, если их
/// удалось применить друг после друга в любом порядке. Результат возвращается в
/// том порядке, в каком `Permutation` был сконструирован
/// (аналог `permutation` из `nom`)
#[derive(Debug, Clone)]
pub struct Permutation<T> {
    pub(crate) parsers: T,
}
impl<A0, A1> Parser for Permutation<(A0, A1)>
where
    A0: Parser,
    A1: Parser,
{
    type Dest = (A0::Dest, A1::Dest);
    fn parse<'a>(&self, input: &'a str) -> Result<(&'a str, Self::Dest), ()> {
        match self.parsers.0.parse(input) {
            Ok((remaining, a0)) => self
                .parsers
                .1
                .parse(remaining)
                .map(|(remaining, a1)| (remaining, (a0, a1))),
            Err(()) => {
                self.parsers.1.parse(input).and_then(|(remaining, a1)| {
                    self.parsers
                        .0
                        .parse(remaining)
                        .map(|(remaining, a0)| (remaining, (a0, a1)))
                })
            }
        }
    }
}
/// Конструктор [Permutation] для двух парсеров
/// (в Rust нет чего-то, вроде variadic templates из C++)
pub const fn permutation2<A0: Parser, A1: Parser>(
    a0: A0,
    a1: A1,
) -> Permutation<(A0, A1)> {
    Permutation { parsers: (a0, a1) }
}
impl<A0, A1, A2> Parser for Permutation<(A0, A1, A2)>
where
    A0: Parser,
    A1: Parser,
    A2: Parser,
{
    type Dest = (A0::Dest, A1::Dest, A2::Dest);
    fn parse<'a>(&self, input: &'a str) -> Result<(&'a str, Self::Dest), ()> {
        match self.parsers.0.parse(input) {
            Ok((remaining, a0)) => match self.parsers.1.parse(remaining) {
                Ok((remaining, a1)) => self
                    .parsers
                    .2
                    .parse(remaining)
                    .map(|(remaining, a2)| (remaining, (a0, a1, a2))),
                Err(()) => self.parsers.2.parse(remaining).and_then(
                    |(remaining, a2)| {
                        self.parsers
                            .1
                            .parse(remaining)
                            .map(|(remaining, a1)| (remaining, (a0, a1, a2)))
                    },
                ),
            },
            Err(()) => match self.parsers.1.parse(input) {
                Ok((remaining, a1)) => match self.parsers.0.parse(remaining) {
                    Ok((remaining, a0)) => self
                        .parsers
                        .2
                        .parse(remaining)
                        .map(|(remaining, a2)| (remaining, (a0, a1, a2))),
                    Err(()) => self.parsers.2.parse(remaining).and_then(
                        |(remaining, a2)| {
                            self.parsers.0.parse(remaining).map(
                                |(remaining, a0)| (remaining, (a0, a1, a2)),
                            )
                        },
                    ),
                },
                Err(()) => {
                    self.parsers.2.parse(input).and_then(|(remaining, a2)| {
                        match self.parsers.0.parse(remaining) {
                            Ok((remaining, a0)) => {
                                self.parsers.1.parse(remaining).map(
                                    |(remaining, a1)| (remaining, (a0, a1, a2)),
                                )
                            }
                            Err(()) => {
                                self.parsers.1.parse(remaining).and_then(
                                    |(remaining, a1)| {
                                        self.parsers.0.parse(remaining).map(
                                            |(remaining, a0)| {
                                                (remaining, (a0, a1, a2))
                                            },
                                        )
                                    },
                                )
                            }
                        }
                    })
                }
            },
        }
    }
}
/// Конструктор [Permutation] для трёх парсеров
/// (в Rust нет чего-то, вроде variadic templates из C++)
pub const fn permutation3<A0: Parser, A1: Parser, A2: Parser>(
    a0: A0,
    a1: A1,
    a2: A2,
) -> Permutation<(A0, A1, A2)> {
    Permutation {
        parsers: (a0, a1, a2),
    }
}
/// Комбинатор списка из любого числа элементов, которые надо читать
/// вложенным парсером. Граница списка определяется квадратными (`[`&`]`)
/// скобками.
/// Для простоты реализации, после каждого элемента списка должна быть запятая
#[derive(Debug, Clone)]
pub struct List<T> {
    pub(crate) parser: T,
}
impl<T: Parser> Parser for List<T> {
    type Dest = Vec<T::Dest>;
    fn parse<'a>(&self, input: &'a str) -> Result<(&'a str, Self::Dest), ()> {
        let mut remaining =
            input.trim_start().strip_prefix('[').ok_or(())?.trim_start();
        let mut result = Vec::new();
        while !remaining.is_empty() {
            if let Some(rest) = remaining.strip_prefix(']') { return Ok((rest.trim_start(), result)) } else {
                let (new_remaining, item) = self.parser.parse(remaining)?;
                remaining = new_remaining
                    .trim_start()
                    .strip_prefix(',')
                    .ok_or(())?
                    .trim_start();
                result.push(item);
            }
        }
        Err(()) // строка кончилась, не закрыв скобку
    }
}
/// Конструктор для [List]
pub const fn list<T: Parser>(parser: T) -> List<T> {
    List { parser }
}
/// Комбинатор, который вернёт тот результат, который будет успешно
/// получен первым из дочерних комбинаторов
/// (аналог `alt` из `nom`)
#[derive(Debug, Clone)]
pub struct Alt<T> {
    pub(crate) parser: T,
}
impl<A0, A1, Dest> Parser for Alt<(A0, A1)>
where
    A0: Parser<Dest = Dest>,
    A1: Parser<Dest = Dest>,
{
    type Dest = Dest;
    fn parse<'a>(&self, input: &'a str) -> Result<(&'a str, Self::Dest), ()> {
        self.parser
            .0
            .parse(input)
            .or_else(|()| self.parser.1.parse(input))
    }
}
/// Конструктор [Alt] для двух парсеров
/// (в Rust нет чего-то, вроде variadic templates из C++)
pub const fn alt2<Dest, A0: Parser<Dest = Dest>, A1: Parser<Dest = Dest>>(
    a0: A0,
    a1: A1,
) -> Alt<(A0, A1)> {
    Alt { parser: (a0, a1) }
}
impl<A0, A1, A2, Dest> Parser for Alt<(A0, A1, A2)>
where
    A0: Parser<Dest = Dest>,
    A1: Parser<Dest = Dest>,
    A2: Parser<Dest = Dest>,
{
    type Dest = Dest;
    fn parse<'a>(&self, input: &'a str) -> Result<(&'a str, Self::Dest), ()> {
        self.parser
            .0
            .parse(input)
            .or_else(|()| self.parser.1.parse(input))
            .or_else(|()| self.parser.2.parse(input))
    }
}
/// Конструктор [Alt] для трёх парсеров
/// (в Rust нет чего-то, вроде variadic templates из C++)
pub const fn alt3<
    Dest,
    A0: Parser<Dest = Dest>,
    A1: Parser<Dest = Dest>,
    A2: Parser<Dest = Dest>,
>(
    a0: A0,
    a1: A1,
    a2: A2,
) -> Alt<(A0, A1, A2)> {
    Alt {
        parser: (a0, a1, a2),
    }
}
impl<A0, A1, A2, A3, Dest> Parser for Alt<(A0, A1, A2, A3)>
where
    A0: Parser<Dest = Dest>,
    A1: Parser<Dest = Dest>,
    A2: Parser<Dest = Dest>,
    A3: Parser<Dest = Dest>,
{
    type Dest = Dest;
    fn parse<'a>(&self, input: &'a str) -> Result<(&'a str, Self::Dest), ()> {
        self.parser
            .0
            .parse(input)
            .or_else(|()| self.parser.1.parse(input))
            .or_else(|()| self.parser.2.parse(input))
            .or_else(|()| self.parser.3.parse(input))
    }
}
/// Конструктор [Alt] для четырёх парсеров
/// (в Rust нет чего-то, вроде variadic templates из C++)
pub const fn alt4<
    Dest,
    A0: Parser<Dest = Dest>,
    A1: Parser<Dest = Dest>,
    A2: Parser<Dest = Dest>,
    A3: Parser<Dest = Dest>,
>(
    a0: A0,
    a1: A1,
    a2: A2,
    a3: A3,
) -> Alt<(A0, A1, A2, A3)> {
    Alt {
        parser: (a0, a1, a2, a3),
    }
}
impl<A0, A1, A2, A3, A4, A5, A6, A7, Dest> Parser
    for Alt<(A0, A1, A2, A3, A4, A5, A6, A7)>
where
    A0: Parser<Dest = Dest>,
    A1: Parser<Dest = Dest>,
    A2: Parser<Dest = Dest>,
    A3: Parser<Dest = Dest>,
    A4: Parser<Dest = Dest>,
    A5: Parser<Dest = Dest>,
    A6: Parser<Dest = Dest>,
    A7: Parser<Dest = Dest>,
{
    type Dest = Dest;
    fn parse<'a>(&self, input: &'a str) -> Result<(&'a str, Self::Dest), ()> {
        self.parser
            .0
            .parse(input)
            .or_else(|()| self.parser.1.parse(input))
            .or_else(|()| self.parser.2.parse(input))
            .or_else(|()| self.parser.3.parse(input))
            .or_else(|()| self.parser.4.parse(input))
            .or_else(|()| self.parser.5.parse(input))
            .or_else(|()| self.parser.6.parse(input))
            .or_else(|()| self.parser.7.parse(input))
    }
}
/// Конструктор [Alt] для восьми парсеров
/// (в Rust нет чего-то, вроде variadic templates из C++)
pub const fn alt8<
    Dest,
    A0: Parser<Dest = Dest>,
    A1: Parser<Dest = Dest>,
    A2: Parser<Dest = Dest>,
    A3: Parser<Dest = Dest>,
    A4: Parser<Dest = Dest>,
    A5: Parser<Dest = Dest>,
    A6: Parser<Dest = Dest>,
    A7: Parser<Dest = Dest>,
>(
    a0: A0,
    a1: A1,
    a2: A2,
    a3: A3,
    a4: A4,
    a5: A5,
    a6: A6,
    a7: A7,
) -> Alt<(A0, A1, A2, A3, A4, A5, A6, A7)> {
    Alt {
        parser: (a0, a1, a2, a3, a4, a5, a6, a7),
    }
}

/// Комбинатор для применения дочернего парсера N раз
/// (аналог `take` из `nom`)
pub struct Take<T> {
    pub(crate) count: usize,
    pub(crate) parser: T,
}
impl<T: Parser> Parser for Take<T> {
    type Dest = Vec<T::Dest>;
    fn parse<'a>(&self, input: &'a str) -> Result<(&'a str, Self::Dest), ()> {
        (0..self.count).try_fold(
            (input, Vec::with_capacity(self.count)),
            |(remaining, mut result), _| {
                let (new_remaining, item) = self.parser.parse(remaining)?;
                result.push(item);
                Ok((new_remaining, result))
            },
        )
    }
}
/// Конструктор `Take`
pub const fn take<T: Parser>(count: usize, parser: T) -> Take<T> {
    Take { count, parser }
}

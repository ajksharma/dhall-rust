use itertools::Itertools;
use pest::iterators::Pair;
use pest::prec_climber as pcl;
use pest::prec_climber::PrecClimber;
use pest::Parser;
use std::borrow::Cow;
use std::rc::Rc;

use dhall_generated_parser::{DhallParser, Rule};
use dhall_proc_macros::{make_parser, parse_children};

use crate::map::{DupTreeMap, DupTreeSet};
use crate::ExprF::*;
use crate::*;

// This file consumes the parse tree generated by pest and turns it into
// our own AST. All those custom macros should eventually moved into
// their own crate because they are quite general and useful. For now they
// are here and hopefully you can figure out how they work.

pub(crate) type ParsedRawExpr = RawExpr<!>;
pub(crate) type ParsedExpr = Expr<!>;
type ParsedText = InterpolatedText<ParsedExpr>;
type ParsedTextContents = InterpolatedTextContents<ParsedExpr>;

pub type ParseError = pest::error::Error<Rule>;

pub type ParseResult<T> = Result<T, ParseError>;

#[derive(Debug, Clone)]
struct ParseInput<'input, Rule>
where
    Rule: std::fmt::Debug + Copy + std::hash::Hash + Ord,
{
    pair: Pair<'input, Rule>,
    original_input_str: Rc<str>,
}

impl<'input> ParseInput<'input, Rule> {
    fn error(&self, message: String) -> ParseError {
        let message = format!(
            "{} while matching on:\n{}",
            message,
            debug_pair(self.pair.clone())
        );
        let e = pest::error::ErrorVariant::CustomError { message };
        pest::error::Error::new_from_span(e, self.pair.as_span())
    }
    fn parse(input_str: &'input str, rule: Rule) -> ParseResult<Self> {
        let mut pairs = DhallParser::parse(rule, input_str)?;
        // TODO: proper errors
        let pair = pairs.next().unwrap();
        assert_eq!(pairs.next(), None);
        Ok(ParseInput {
            original_input_str: input_str.to_string().into(),
            pair,
        })
    }
    fn with_pair(&self, new_pair: Pair<'input, Rule>) -> Self {
        ParseInput {
            pair: new_pair,
            original_input_str: self.original_input_str.clone(),
        }
    }
    fn as_span(&self) -> Span {
        Span::make(self.original_input_str.clone(), self.pair.as_span())
    }
    fn as_str(&self) -> &'input str {
        self.pair.as_str()
    }
}

fn debug_pair(pair: Pair<Rule>) -> String {
    use std::fmt::Write;
    let mut s = String::new();
    fn aux(s: &mut String, indent: usize, prefix: String, pair: Pair<Rule>) {
        let indent_str = "| ".repeat(indent);
        let rule = pair.as_rule();
        let contents = pair.as_str();
        let mut inner = pair.into_inner();
        let mut first = true;
        while let Some(p) = inner.next() {
            if first {
                first = false;
                let last = inner.peek().is_none();
                if last && p.as_str() == contents {
                    let prefix = format!("{}{:?} > ", prefix, rule);
                    aux(s, indent, prefix, p);
                    continue;
                } else {
                    writeln!(
                        s,
                        r#"{}{}{:?}: "{}""#,
                        indent_str, prefix, rule, contents
                    )
                    .unwrap();
                }
            }
            aux(s, indent + 1, "".into(), p);
        }
        if first {
            writeln!(
                s,
                r#"{}{}{:?}: "{}""#,
                indent_str, prefix, rule, contents
            )
            .unwrap();
        }
    }
    aux(&mut s, 0, "".into(), pair);
    s
}

#[derive(Debug)]
enum Either<A, B> {
    Left(A),
    Right(B),
}

impl crate::Builtin {
    pub fn parse(s: &str) -> Option<Self> {
        use crate::Builtin::*;
        match s {
            "Bool" => Some(Bool),
            "Natural" => Some(Natural),
            "Integer" => Some(Integer),
            "Double" => Some(Double),
            "Text" => Some(Text),
            "List" => Some(List),
            "Optional" => Some(Optional),
            "None" => Some(OptionalNone),
            "Natural/build" => Some(NaturalBuild),
            "Natural/fold" => Some(NaturalFold),
            "Natural/isZero" => Some(NaturalIsZero),
            "Natural/even" => Some(NaturalEven),
            "Natural/odd" => Some(NaturalOdd),
            "Natural/toInteger" => Some(NaturalToInteger),
            "Natural/show" => Some(NaturalShow),
            "Natural/subtract" => Some(NaturalSubtract),
            "Integer/toDouble" => Some(IntegerToDouble),
            "Integer/show" => Some(IntegerShow),
            "Double/show" => Some(DoubleShow),
            "List/build" => Some(ListBuild),
            "List/fold" => Some(ListFold),
            "List/length" => Some(ListLength),
            "List/head" => Some(ListHead),
            "List/last" => Some(ListLast),
            "List/indexed" => Some(ListIndexed),
            "List/reverse" => Some(ListReverse),
            "Optional/fold" => Some(OptionalFold),
            "Optional/build" => Some(OptionalBuild),
            "Text/show" => Some(TextShow),
            _ => None,
        }
    }
}

// Trim the shared indent off of a vec of lines, as defined by the Dhall semantics of multiline
// literals.
fn trim_indent(lines: &mut Vec<ParsedText>) {
    let is_indent = |c: char| c == ' ' || c == '\t';

    // There is at least one line so this is safe
    let last_line_head = lines.last().unwrap().head();
    let indent_chars = last_line_head
        .char_indices()
        .take_while(|(_, c)| is_indent(*c));
    let mut min_indent_idx = match indent_chars.last() {
        Some((i, _)) => i,
        // If there is no indent char, then no indent needs to be stripped
        None => return,
    };

    for line in lines.iter() {
        // Ignore empty lines
        if line.is_empty() {
            continue;
        }
        // Take chars from line while they match the current minimum indent.
        let indent_chars = last_line_head[0..=min_indent_idx]
            .char_indices()
            .zip(line.head().chars())
            .take_while(|((_, c1), c2)| c1 == c2);
        match indent_chars.last() {
            Some(((i, _), _)) => min_indent_idx = i,
            // If there is no indent char, then no indent needs to be stripped
            None => return,
        };
    }

    // Remove the shared indent from non-empty lines
    for line in lines.iter_mut() {
        if !line.is_empty() {
            line.head_mut().replace_range(0..=min_indent_idx, "");
        }
    }
}

lazy_static::lazy_static! {
    static ref PRECCLIMBER: PrecClimber<Rule> = {
        use Rule::*;
        // In order of precedence
        let operators = vec![
            import_alt,
            bool_or,
            natural_plus,
            text_append,
            list_append,
            bool_and,
            combine,
            prefer,
            combine_types,
            natural_times,
            bool_eq,
            bool_ne,
            equivalent,
        ];
        PrecClimber::new(
            operators
                .into_iter()
                .map(|op| pcl::Operator::new(op, pcl::Assoc::Left))
                .collect(),
        )
    };
}

#[make_parser]
impl _ {
    fn EOI(_: ParseInput<Rule>) -> ParseResult<()> {
        Ok(())
    }

    fn simple_label(input: ParseInput<Rule>) -> ParseResult<Label> {
        Ok(Label::from(input.as_str().trim().to_owned()))
    }
    fn quoted_label(input: ParseInput<Rule>) -> ParseResult<Label> {
        Ok(Label::from(input.as_str().trim().to_owned()))
    }
    fn label(input: ParseInput<Rule>) -> ParseResult<Label> {
        Ok(parse_children!(input;
            [simple_label(l)] => l,
            [quoted_label(l)] => l,
        ))
    }

    fn double_quote_literal(
        input: ParseInput<Rule>,
    ) -> ParseResult<ParsedText> {
        Ok(parse_children!(input;
            [double_quote_chunk(chunks)..] => {
                chunks.collect()
            }
        ))
    }

    fn double_quote_chunk(
        input: ParseInput<Rule>,
    ) -> ParseResult<ParsedTextContents> {
        Ok(parse_children!(input;
            [interpolation(e)] => {
                InterpolatedTextContents::Expr(e)
            },
            [double_quote_escaped(s)] => {
                InterpolatedTextContents::Text(s)
            },
            [double_quote_char(s)] => {
                InterpolatedTextContents::Text(s.to_owned())
            },
        ))
    }
    fn double_quote_escaped(input: ParseInput<Rule>) -> ParseResult<String> {
        Ok(match input.as_str() {
            "\"" => "\"".to_owned(),
            "$" => "$".to_owned(),
            "\\" => "\\".to_owned(),
            "/" => "/".to_owned(),
            "b" => "\u{0008}".to_owned(),
            "f" => "\u{000C}".to_owned(),
            "n" => "\n".to_owned(),
            "r" => "\r".to_owned(),
            "t" => "\t".to_owned(),
            // "uXXXX" or "u{XXXXX}"
            s => {
                use std::convert::{TryFrom, TryInto};

                let s = &s[1..];
                let s = if &s[0..1] == "{" {
                    &s[1..s.len() - 1]
                } else {
                    &s[0..s.len()]
                };

                if s.len() > 8 {
                    Err(input.error(format!(
                        "Escape sequences can't have more than 8 chars: \"{}\"",
                        s
                    )))?
                }

                // pad with zeroes
                let s: String = std::iter::repeat('0')
                    .take(8 - s.len())
                    .chain(s.chars())
                    .collect();

                // `s` has length 8, so `bytes` has length 4
                let bytes: &[u8] = &hex::decode(s).unwrap();
                let i = u32::from_be_bytes(bytes.try_into().unwrap());
                let c = char::try_from(i).unwrap();
                match i {
                    0xD800..=0xDFFF => {
                        let c_ecapsed = c.escape_unicode();
                        Err(input.error(format!("Escape sequences can't contain surrogate pairs: \"{}\"", c_ecapsed)))?
                    }
                    0x0FFFE..=0x0FFFF
                    | 0x1FFFE..=0x1FFFF
                    | 0x2FFFE..=0x2FFFF
                    | 0x3FFFE..=0x3FFFF
                    | 0x4FFFE..=0x4FFFF
                    | 0x5FFFE..=0x5FFFF
                    | 0x6FFFE..=0x6FFFF
                    | 0x7FFFE..=0x7FFFF
                    | 0x8FFFE..=0x8FFFF
                    | 0x9FFFE..=0x9FFFF
                    | 0xAFFFE..=0xAFFFF
                    | 0xBFFFE..=0xBFFFF
                    | 0xCFFFE..=0xCFFFF
                    | 0xDFFFE..=0xDFFFF
                    | 0xEFFFE..=0xEFFFF
                    | 0xFFFFE..=0xFFFFF
                    | 0x10_FFFE..=0x10_FFFF => {
                        let c_ecapsed = c.escape_unicode();
                        Err(input.error(format!("Escape sequences can't contain non-characters: \"{}\"", c_ecapsed)))?
                    }
                    _ => {}
                }
                std::iter::once(c).collect()
            }
        })
    }
    fn double_quote_char<'a>(
        input: ParseInput<'a, Rule>,
    ) -> ParseResult<&'a str> {
        Ok(input.as_str())
    }

    fn single_quote_literal(
        input: ParseInput<Rule>,
    ) -> ParseResult<ParsedText> {
        Ok(parse_children!(input;
            [single_quote_continue(lines)] => {
                let newline: ParsedText = "\n".to_string().into();

                let mut lines: Vec<ParsedText> = lines
                    .into_iter()
                    .rev()
                    .map(|l| l.into_iter().rev().collect::<ParsedText>())
                    .collect();

                trim_indent(&mut lines);

                lines
                    .into_iter()
                    .intersperse(newline)
                    .flat_map(InterpolatedText::into_iter)
                    .collect::<ParsedText>()
            }
        ))
    }
    fn single_quote_char<'a>(
        input: ParseInput<'a, Rule>,
    ) -> ParseResult<&'a str> {
        Ok(input.as_str())
    }
    fn escaped_quote_pair<'a>(_: ParseInput<'a, Rule>) -> ParseResult<&'a str> {
        Ok("''")
    }
    fn escaped_interpolation<'a>(
        _: ParseInput<'a, Rule>,
    ) -> ParseResult<&'a str> {
        Ok("${")
    }
    fn interpolation(input: ParseInput<Rule>) -> ParseResult<ParsedExpr> {
        Ok(parse_children!(input;
            [expression(e)] => e
        ))
    }

    // Returns a vec of lines in reversed order, where each line is also in reversed order.
    fn single_quote_continue(
        input: ParseInput<Rule>,
    ) -> ParseResult<Vec<Vec<ParsedTextContents>>> {
        Ok(parse_children!(input;
            [interpolation(c), single_quote_continue(lines)] => {
                let c = InterpolatedTextContents::Expr(c);
                let mut lines = lines;
                lines.last_mut().unwrap().push(c);
                lines
            },
            [escaped_quote_pair(c), single_quote_continue(lines)] => {
                let mut lines = lines;
                // TODO: don't allocate for every char
                let c = InterpolatedTextContents::Text(c.to_owned());
                lines.last_mut().unwrap().push(c);
                lines
            },
            [escaped_interpolation(c), single_quote_continue(lines)] => {
                let mut lines = lines;
                // TODO: don't allocate for every char
                let c = InterpolatedTextContents::Text(c.to_owned());
                lines.last_mut().unwrap().push(c);
                lines
            },
            [single_quote_char(c), single_quote_continue(lines)] => {
                let mut lines = lines;
                if c == "\n" || c == "\r\n" {
                    lines.push(vec![]);
                } else {
                    // TODO: don't allocate for every char
                    let c = InterpolatedTextContents::Text(c.to_owned());
                    lines.last_mut().unwrap().push(c);
                }
                lines
            },
            [] => {
                vec![vec![]]
            },
        ))
    }

    fn builtin(input: ParseInput<Rule>) -> ParseResult<ParsedExpr> {
        let s = input.as_str();
        let span = input.as_span();
        Ok(spanned(
            span,
            match crate::Builtin::parse(s) {
                Some(b) => Builtin(b),
                None => {
                    match s {
                        "True" => BoolLit(true),
                        "False" => BoolLit(false),
                        "Type" => Const(crate::Const::Type),
                        "Kind" => Const(crate::Const::Kind),
                        "Sort" => Const(crate::Const::Sort),
                        _ => Err(input
                            .error(format!("Unrecognized builtin: '{}'", s)))?,
                    }
                }
            },
        ))
    }

    fn NaN(_: ParseInput<Rule>) -> ParseResult<()> {
        Ok(())
    }
    fn minus_infinity_literal(_: ParseInput<Rule>) -> ParseResult<()> {
        Ok(())
    }
    fn plus_infinity_literal(_: ParseInput<Rule>) -> ParseResult<()> {
        Ok(())
    }

    fn numeric_double_literal(
        input: ParseInput<Rule>,
    ) -> ParseResult<core::Double> {
        let s = input.as_str().trim();
        match s.parse::<f64>() {
            Ok(x) if x.is_infinite() => Err(input.error(format!(
                "Overflow while parsing double literal '{}'",
                s
            ))),
            Ok(x) => Ok(NaiveDouble::from(x)),
            Err(e) => Err(input.error(format!("{}", e))),
        }
    }

    fn double_literal(input: ParseInput<Rule>) -> ParseResult<core::Double> {
        Ok(parse_children!(input;
            [numeric_double_literal(n)] => n,
            [minus_infinity_literal(_)] => std::f64::NEG_INFINITY.into(),
            [plus_infinity_literal(_)] => std::f64::INFINITY.into(),
            [NaN(_)] => std::f64::NAN.into(),
        ))
    }

    fn natural_literal(input: ParseInput<Rule>) -> ParseResult<core::Natural> {
        input
            .as_str()
            .trim()
            .parse()
            .map_err(|e| input.error(format!("{}", e)))
    }

    fn integer_literal(input: ParseInput<Rule>) -> ParseResult<core::Integer> {
        input
            .as_str()
            .trim()
            .parse()
            .map_err(|e| input.error(format!("{}", e)))
    }

    fn identifier(input: ParseInput<Rule>) -> ParseResult<ParsedExpr> {
        let span = input.as_span();
        Ok(parse_children!(input;
            [variable(v)] => {
                spanned(span, Var(v))
            },
            [builtin(e)] => e,
        ))
    }

    fn variable(input: ParseInput<Rule>) -> ParseResult<V<Label>> {
        Ok(parse_children!(input;
            [label(l), natural_literal(idx)] => {
                V(l, idx)
            },
            [label(l)] => {
                V(l, 0)
            },
        ))
    }

    fn unquoted_path_component<'a>(
        input: ParseInput<'a, Rule>,
    ) -> ParseResult<&'a str> {
        Ok(input.as_str())
    }
    fn quoted_path_component<'a>(
        input: ParseInput<'a, Rule>,
    ) -> ParseResult<&'a str> {
        Ok(input.as_str())
    }
    fn path_component(input: ParseInput<Rule>) -> ParseResult<String> {
        Ok(parse_children!(input;
            [unquoted_path_component(s)] => s.to_string(),
            [quoted_path_component(s)] => {
                const RESERVED: &percent_encoding::AsciiSet =
                    &percent_encoding::CONTROLS
                    .add(b'=').add(b':').add(b'/').add(b'?')
                    .add(b'#').add(b'[').add(b']').add(b'@')
                    .add(b'!').add(b'$').add(b'&').add(b'\'')
                    .add(b'(').add(b')').add(b'*').add(b'+')
                    .add(b',').add(b';');
                s.chars()
                    .map(|c| {
                        // Percent-encode ascii chars
                        if c.is_ascii() {
                            percent_encoding::utf8_percent_encode(
                                &c.to_string(),
                                RESERVED,
                            ).to_string()
                        } else {
                            c.to_string()
                        }
                    })
                    .collect()
            },
        ))
    }
    fn path(input: ParseInput<Rule>) -> ParseResult<Vec<String>> {
        Ok(parse_children!(input;
            [path_component(components)..] => {
                components.collect()
            }
        ))
    }

    fn local(
        input: ParseInput<Rule>,
    ) -> ParseResult<(FilePrefix, Vec<String>)> {
        Ok(parse_children!(input;
            [parent_path(l)] => l,
            [here_path(l)] => l,
            [home_path(l)] => l,
            [absolute_path(l)] => l,
        ))
    }

    fn parent_path(
        input: ParseInput<Rule>,
    ) -> ParseResult<(FilePrefix, Vec<String>)> {
        Ok(parse_children!(input;
            [path(p)] => (FilePrefix::Parent, p)
        ))
    }
    fn here_path(
        input: ParseInput<Rule>,
    ) -> ParseResult<(FilePrefix, Vec<String>)> {
        Ok(parse_children!(input;
            [path(p)] => (FilePrefix::Here, p)
        ))
    }
    fn home_path(
        input: ParseInput<Rule>,
    ) -> ParseResult<(FilePrefix, Vec<String>)> {
        Ok(parse_children!(input;
            [path(p)] => (FilePrefix::Home, p)
        ))
    }
    fn absolute_path(
        input: ParseInput<Rule>,
    ) -> ParseResult<(FilePrefix, Vec<String>)> {
        Ok(parse_children!(input;
            [path(p)] => (FilePrefix::Absolute, p)
        ))
    }

    fn scheme(input: ParseInput<Rule>) -> ParseResult<Scheme> {
        Ok(match input.as_str() {
            "http" => Scheme::HTTP,
            "https" => Scheme::HTTPS,
            _ => unreachable!(),
        })
    }

    fn http_raw(input: ParseInput<Rule>) -> ParseResult<URL<ParsedExpr>> {
        Ok(parse_children!(input;
            [scheme(sch), authority(auth), path(p)] => URL {
                scheme: sch,
                authority: auth,
                path: p,
                query: None,
                headers: None,
            },
            [scheme(sch), authority(auth), path(p), query(q)] => URL {
                scheme: sch,
                authority: auth,
                path: p,
                query: Some(q),
                headers: None,
            },
        ))
    }

    fn authority(input: ParseInput<Rule>) -> ParseResult<String> {
        Ok(input.as_str().to_owned())
    }

    fn query(input: ParseInput<Rule>) -> ParseResult<String> {
        Ok(input.as_str().to_owned())
    }

    fn http(input: ParseInput<Rule>) -> ParseResult<URL<ParsedExpr>> {
        Ok(parse_children!(input;
        [http_raw(url)] => url,
        [http_raw(url), import_expression(e)] =>
            URL { headers: Some(e), ..url },
        ))
    }

    fn env(input: ParseInput<Rule>) -> ParseResult<String> {
        Ok(parse_children!(input;
            [bash_environment_variable(s)] => s,
            [posix_environment_variable(s)] => s,
        ))
    }
    fn bash_environment_variable(
        input: ParseInput<Rule>,
    ) -> ParseResult<String> {
        Ok(input.as_str().to_owned())
    }
    fn posix_environment_variable(
        input: ParseInput<Rule>,
    ) -> ParseResult<String> {
        Ok(parse_children!(input;
            [posix_environment_variable_character(chars)..] => {
                chars.collect()
            },
        ))
    }
    fn posix_environment_variable_character<'a>(
        input: ParseInput<'a, Rule>,
    ) -> ParseResult<Cow<'a, str>> {
        Ok(match input.as_str() {
            "\\\"" => Cow::Owned("\"".to_owned()),
            "\\\\" => Cow::Owned("\\".to_owned()),
            "\\a" => Cow::Owned("\u{0007}".to_owned()),
            "\\b" => Cow::Owned("\u{0008}".to_owned()),
            "\\f" => Cow::Owned("\u{000C}".to_owned()),
            "\\n" => Cow::Owned("\n".to_owned()),
            "\\r" => Cow::Owned("\r".to_owned()),
            "\\t" => Cow::Owned("\t".to_owned()),
            "\\v" => Cow::Owned("\u{000B}".to_owned()),
            s => Cow::Borrowed(s),
        })
    }

    fn missing(_: ParseInput<Rule>) -> ParseResult<()> {
        Ok(())
    }

    fn import_type(
        input: ParseInput<Rule>,
    ) -> ParseResult<ImportLocation<ParsedExpr>> {
        Ok(parse_children!(input;
            [missing(_)] => {
                ImportLocation::Missing
            },
            [env(e)] => {
                ImportLocation::Env(e)
            },
            [http(url)] => {
                ImportLocation::Remote(url)
            },
            [local((prefix, p))] => {
                ImportLocation::Local(prefix, p)
            },
        ))
    }

    fn hash(input: ParseInput<Rule>) -> ParseResult<Hash> {
        let s = input.as_str().trim();
        let protocol = &s[..6];
        let hash = &s[7..];
        if protocol != "sha256" {
            Err(input.error(format!("Unknown hashing protocol '{}'", protocol)))?
        }
        Ok(Hash::SHA256(hex::decode(hash).unwrap()))
    }

    fn import_hashed(
        input: ParseInput<Rule>,
    ) -> ParseResult<crate::Import<ParsedExpr>> {
        Ok(parse_children!(input;
        [import_type(location)] =>
            crate::Import {mode: ImportMode::Code, location, hash: None },
        [import_type(location), hash(h)] =>
        crate::Import {mode: ImportMode::Code, location, hash: Some(h) },
        ))
    }

    fn Text(_: ParseInput<Rule>) -> ParseResult<()> {
        Ok(())
    }
    fn Location(_: ParseInput<Rule>) -> ParseResult<()> {
        Ok(())
    }

    fn import(input: ParseInput<Rule>) -> ParseResult<ParsedExpr> {
        let span = input.as_span();
        Ok(parse_children!(input;
            [import_hashed(imp)] => {
                spanned(span, Import(crate::Import {
                    mode: ImportMode::Code,
                    ..imp
                }))
            },
            [import_hashed(imp), Text(_)] => {
                spanned(span, Import(crate::Import {
                    mode: ImportMode::RawText,
                    ..imp
                }))
            },
            [import_hashed(imp), Location(_)] => {
                spanned(span, Import(crate::Import {
                    mode: ImportMode::Location,
                    ..imp
                }))
            },
        ))
    }

    fn lambda(_: ParseInput<Rule>) -> ParseResult<()> {
        Ok(())
    }
    fn forall(_: ParseInput<Rule>) -> ParseResult<()> {
        Ok(())
    }
    fn arrow(_: ParseInput<Rule>) -> ParseResult<()> {
        Ok(())
    }
    fn merge(_: ParseInput<Rule>) -> ParseResult<()> {
        Ok(())
    }
    fn assert(_: ParseInput<Rule>) -> ParseResult<()> {
        Ok(())
    }
    fn if_(_: ParseInput<Rule>) -> ParseResult<()> {
        Ok(())
    }
    fn in_(_: ParseInput<Rule>) -> ParseResult<()> {
        Ok(())
    }
    fn toMap(_: ParseInput<Rule>) -> ParseResult<()> {
        Ok(())
    }

    fn empty_list_literal(input: ParseInput<Rule>) -> ParseResult<ParsedExpr> {
        let span = input.as_span();
        Ok(parse_children!(input;
            [application_expression(e)] => {
                spanned(span, EmptyListLit(e))
            },
        ))
    }

    fn expression(input: ParseInput<Rule>) -> ParseResult<ParsedExpr> {
        let span = input.as_span();
        Ok(parse_children!(input;
            [lambda(()), label(l), expression(typ),
                    arrow(()), expression(body)] => {
                spanned(span, Lam(l, typ, body))
            },
            [if_(()), expression(cond), expression(left),
                    expression(right)] => {
                spanned(span, BoolIf(cond, left, right))
            },
            [let_binding(bindings).., in_(()), expression(final_expr)] => {
                bindings.rev().fold(
                    final_expr,
                    |acc, x| unspanned(Let(x.0, x.1, x.2, acc))
                )
            },
            [forall(()), label(l), expression(typ),
                    arrow(()), expression(body)] => {
                spanned(span, Pi(l, typ, body))
            },
            [operator_expression(typ), arrow(()), expression(body)] => {
                spanned(span, Pi("_".into(), typ, body))
            },
            [merge(()), import_expression(x), import_expression(y),
                    application_expression(z)] => {
                spanned(span, Merge(x, y, Some(z)))
            },
            [empty_list_literal(e)] => e,
            [assert(()), expression(x)] => {
                spanned(span, Assert(x))
            },
            [toMap(()), import_expression(x), application_expression(y)] => {
                spanned(span, ToMap(x, Some(y)))
            },
            [operator_expression(e)] => e,
            [operator_expression(e), expression(annot)] => {
                spanned(span, Annot(e, annot))
            },
        ))
    }

    fn let_binding(
        input: ParseInput<Rule>,
    ) -> ParseResult<(Label, Option<ParsedExpr>, ParsedExpr)> {
        Ok(parse_children!(input;
            [label(name), expression(annot), expression(expr)] =>
                (name, Some(annot), expr),
            [label(name), expression(expr)] =>
                (name, None, expr),
        ))
    }

    fn List(_: ParseInput<Rule>) -> ParseResult<()> {
        Ok(())
    }
    fn Optional(_: ParseInput<Rule>) -> ParseResult<()> {
        Ok(())
    }

    #[prec_climb(application_expression, PRECCLIMBER)]
    fn operator_expression(
        input: ParseInput<Rule>,
        l: ParsedExpr,
        op: Pair<Rule>,
        r: ParsedExpr,
    ) -> ParseResult<ParsedExpr> {
        use crate::BinOp::*;
        use Rule::*;
        let op = match op.as_rule() {
            import_alt => ImportAlt,
            bool_or => BoolOr,
            natural_plus => NaturalPlus,
            text_append => TextAppend,
            list_append => ListAppend,
            bool_and => BoolAnd,
            combine => RecursiveRecordMerge,
            prefer => RightBiasedRecordMerge,
            combine_types => RecursiveRecordTypeMerge,
            natural_times => NaturalTimes,
            bool_eq => BoolEQ,
            bool_ne => BoolNE,
            equivalent => Equivalence,
            r => Err(input.error(format!("Rule {:?} isn't an operator", r)))?,
        };

        Ok(unspanned(BinOp(op, l, r)))
    }

    fn Some_(_: ParseInput<Rule>) -> ParseResult<()> {
        Ok(())
    }

    fn application_expression(
        input: ParseInput<Rule>,
    ) -> ParseResult<ParsedExpr> {
        Ok(parse_children!(input;
            [first_application_expression(e)] => e,
            [first_application_expression(first),
                    import_expression(rest)..] => {
                rest.fold(first, |acc, e| unspanned(App(acc, e)))
            },
        ))
    }

    fn first_application_expression(
        input: ParseInput<Rule>,
    ) -> ParseResult<ParsedExpr> {
        let span = input.as_span();
        Ok(parse_children!(input;
            [Some_(()), import_expression(e)] => {
                spanned(span, SomeLit(e))
            },
            [merge(()), import_expression(x), import_expression(y)] => {
                spanned(span, Merge(x, y, None))
            },
            [toMap(()), import_expression(x)] => {
                spanned(span, ToMap(x, None))
            },
            [import_expression(e)] => e,
        ))
    }

    fn import_expression(input: ParseInput<Rule>) -> ParseResult<ParsedExpr> {
        Ok(parse_children!(input;
            [selector_expression(e)] => e,
            [import(e)] => e,
        ))
    }

    fn selector_expression(input: ParseInput<Rule>) -> ParseResult<ParsedExpr> {
        Ok(parse_children!(input;
            [primitive_expression(e)] => e,
            [primitive_expression(first), selector(rest)..] => {
                rest.fold(first, |acc, e| unspanned(match e {
                    Either::Left(l) => Field(acc, l),
                    Either::Right(ls) => Projection(acc, ls),
                }))
            },
        ))
    }

    fn selector(
        input: ParseInput<Rule>,
    ) -> ParseResult<Either<Label, DupTreeSet<Label>>> {
        Ok(parse_children!(input;
            [label(l)] => Either::Left(l),
            [labels(ls)] => Either::Right(ls),
            [expression(_e)] => unimplemented!("selection by expression"), // TODO
        ))
    }

    fn labels(input: ParseInput<Rule>) -> ParseResult<DupTreeSet<Label>> {
        Ok(parse_children!(input;
            [label(ls)..] => ls.collect(),
        ))
    }

    fn primitive_expression(
        input: ParseInput<Rule>,
    ) -> ParseResult<ParsedExpr> {
        let span = input.as_span();
        Ok(parse_children!(input;
            [double_literal(n)] => spanned(span, DoubleLit(n)),
            [natural_literal(n)] => spanned(span, NaturalLit(n)),
            [integer_literal(n)] => spanned(span, IntegerLit(n)),
            [double_quote_literal(s)] => spanned(span, TextLit(s)),
            [single_quote_literal(s)] => spanned(span, TextLit(s)),
            [empty_record_type(e)] => e,
            [empty_record_literal(e)] => e,
            [non_empty_record_type_or_literal(e)] => e,
            [union_type(e)] => e,
            [non_empty_list_literal(e)] => e,
            [identifier(e)] => e,
            [expression(e)] => e,
        ))
    }

    fn empty_record_literal(
        input: ParseInput<Rule>,
    ) -> ParseResult<ParsedExpr> {
        let span = input.as_span();
        Ok(spanned(span, RecordLit(Default::default())))
    }

    fn empty_record_type(input: ParseInput<Rule>) -> ParseResult<ParsedExpr> {
        let span = input.as_span();
        Ok(spanned(span, RecordType(Default::default())))
    }

    fn non_empty_record_type_or_literal(
        input: ParseInput<Rule>,
    ) -> ParseResult<ParsedExpr> {
        let span = input.as_span();
        Ok(parse_children!(input;
            [label(first_label), non_empty_record_type(rest)] => {
                let (first_expr, mut map) = rest;
                map.insert(first_label, first_expr);
                spanned(span, RecordType(map))
            },
            [label(first_label), non_empty_record_literal(rest)] => {
                let (first_expr, mut map) = rest;
                map.insert(first_label, first_expr);
                spanned(span, RecordLit(map))
            },
        ))
    }

    fn non_empty_record_type(
        input: ParseInput<Rule>,
    ) -> ParseResult<(ParsedExpr, DupTreeMap<Label, ParsedExpr>)> {
        Ok(parse_children!(input;
            [expression(expr), record_type_entry(entries)..] => {
                (expr, entries.collect())
            }
        ))
    }

    fn record_type_entry(
        input: ParseInput<Rule>,
    ) -> ParseResult<(Label, ParsedExpr)> {
        Ok(parse_children!(input;
            [label(name), expression(expr)] => (name, expr)
        ))
    }

    fn non_empty_record_literal(
        input: ParseInput<Rule>,
    ) -> ParseResult<(ParsedExpr, DupTreeMap<Label, ParsedExpr>)> {
        Ok(parse_children!(input;
            [expression(expr), record_literal_entry(entries)..] => {
                (expr, entries.collect())
            }
        ))
    }

    fn record_literal_entry(
        input: ParseInput<Rule>,
    ) -> ParseResult<(Label, ParsedExpr)> {
        Ok(parse_children!(input;
            [label(name), expression(expr)] => (name, expr)
        ))
    }

    fn union_type(input: ParseInput<Rule>) -> ParseResult<ParsedExpr> {
        let span = input.as_span();
        Ok(parse_children!(input;
            [empty_union_type(_)] => {
                spanned(span, UnionType(Default::default()))
            },
            [union_type_entry(entries)..] => {
                spanned(span, UnionType(entries.collect()))
            },
        ))
    }

    fn empty_union_type(_: ParseInput<Rule>) -> ParseResult<()> {
        Ok(())
    }

    fn union_type_entry(
        input: ParseInput<Rule>,
    ) -> ParseResult<(Label, Option<ParsedExpr>)> {
        Ok(parse_children!(input;
            [label(name), expression(expr)] => (name, Some(expr)),
            [label(name)] => (name, None),
        ))
    }

    fn non_empty_list_literal(
        input: ParseInput<Rule>,
    ) -> ParseResult<ParsedExpr> {
        let span = input.as_span();
        Ok(parse_children!(input;
            [expression(items)..] => spanned(
                span,
                NEListLit(items.collect())
            )
        ))
    }

    fn final_expression(input: ParseInput<Rule>) -> ParseResult<ParsedExpr> {
        Ok(parse_children!(input;
            [expression(e), EOI(_)] => e
        ))
    }
}

pub fn parse_expr(s: &str) -> ParseResult<ParsedExpr> {
    let input = ParseInput::parse(s, Rule::final_expression)?;
    Parsers::final_expression(input)
}

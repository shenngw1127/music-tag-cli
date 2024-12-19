use cfgrammar::Span;
use log::{debug, error};
use lrlex::{DefaultLexerTypes, lrlex_mod};
use lrpar::{lrpar_mod, NonStreamingLexer};
use wildmatch::WildMatchPattern;

// Using `lrlex_mod!` brings the lexer for `calc.l` into scope.
// By default, the module name will be `where_l`
// (i.e. the file name, minus any extensions, with a suffix of `_l`).
lrlex_mod!("where.l");
// Using `lrpar_mod!` brings the parser for `calc.y` into scope.
// By default, the module name will be`where_y`
// (i.e. the file name, minus any extensions, with a suffix of `_y`).
lrpar_mod!("where.y");

use where_y::Expr;

use crate::model::MyTag;
use crate::op::ReadTag;

fn eval(lexer: &dyn NonStreamingLexer<DefaultLexerTypes<u32>>,
        e: Expr,
) -> Result<WhereClause, (Span, &'static str)> {
    match e {
        Expr::Or { span, lhs, rhs } => {
            debug!("span: {:?}, lhs: {:?}, rhs: {:?}", span, lhs, rhs);
            let lhs = Box::new(eval(lexer, *lhs)?);
            let rhs = Box::new(eval(lexer, *rhs)?);
            Ok(WhereClause::LogicOp(LogicOp { op: "OR", lhs, rhs }))
        }
        Expr::And { span, lhs, rhs } => {
            debug!("span: {:?}, lhs: {:?}, rhs: {:?}", span, lhs, rhs);
            let lhs = Box::new(eval(lexer, *lhs)?);
            let rhs = Box::new(eval(lexer, *rhs)?);
            Ok(WhereClause::LogicOp(LogicOp { op: "AND", lhs, rhs }))
        }
        Expr::Not { span, inner } => {
            debug!("span: {:?}, inner: {:?}", span, inner);
            let inner = Box::new(eval(lexer, *inner)?);
            Ok(WhereClause::NotOp(NotOp { inner }))
        }
        Expr::Comparator { span, op, lhs, rhs } => {
            debug!("span: {:?}, lhs: {:?}, rhs: {:?}", span, lhs, rhs);
            let op = match eval(lexer, *op)? {
                WhereClause::CompOp(op) => op,
                _ => return Err((span, "op is not a validate comparison operator")),
            };
            let key = match eval(lexer, *lhs)? {
                WhereClause::CompKey(key) => key,
                _ => return Err((span, "left is not a key")),
            };
            match MyTag::from_str(&key) {
                Ok(ref tag) => {
                    if tag.is_text() || tag.is_date() {
                        let value = match eval(lexer, *rhs)? {
                            WhereClause::TextValue(value) => value,
                            _ => return Err((span, "right is not a value")),
                        };
                        Ok(WhereClause::TextComp(TextComp { op, tag, value }))
                    } else if tag.is_numeric() {
                        let value = match eval(lexer, *rhs)? {
                            WhereClause::NumValue(value) => value,
                            _ => return Err((span, "right is not a value")),
                        };
                        Ok(WhereClause::NumComp(NumComp { op, tag, value }))
                    } else {
                        Err((span, "left is an unsupported key"))
                    }
                }
                _ => return Err((span, "left is not a key")),
            }
        }
        Expr::ComparatorOp { span } => {
            let span_str = lexer.span_str(span);
            debug!("span_str: {}", span_str);
            Ok(WhereClause::CompOp(span_str.to_owned()))
        }
        Expr::TextTag { span } => {
            let span_str = lexer.span_str(span);
            debug!("span_str: {}", span_str);
            Ok(WhereClause::CompKey(span_str.to_owned()))
        }
        Expr::TextValue { span } => {
            let span_str = lexer.span_str(span);
            debug!("span_str: {}", span_str);
            if span_str.chars().count() < 2 {
                Err((span, "value is not a string!"))
            } else {
                let first_char = span_str.chars().nth(0).unwrap();
                let last_char = span_str.chars().rev().nth(0).unwrap();
                if first_char != '\'' || last_char != '\'' {
                    Err((span, "value is not a string!"))
                } else {
                    Ok(WhereClause::TextValue(
                        span_str[1..span_str.len() - 1].replace("\'\'", "\'")))
                }
            }
        }
        Expr::NumTag { span } => {
            let span_str = lexer.span_str(span);
            debug!("span_str: {}", span_str);
            Ok(WhereClause::CompKey(span_str.to_owned()))
        }
        Expr::NumValue { span } => {
            let span_str = lexer.span_str(span);
            debug!("span_str: {}", span_str);
            let v = span_str.parse::<u32>()
                .map_err(|_| (span, "cannot be represented as a u32"))?;
            Ok(WhereClause::NumValue(v))
        }
    }
}

#[derive(Debug)]
pub enum WhereClause {
    LogicOp(LogicOp),
    NotOp(NotOp),
    TextComp(TextComp),
    NumComp(NumComp),
    CompOp(String),
    CompKey(String),
    TextValue(String),
    NumValue(u32),
}

impl WhereClause {
    pub fn new(input: &str) -> Result<WhereClause, String> {
        // Get the `LexerDef` for the `calc` language.
        let lexerdef = where_l::lexerdef();

        // Now we create a lexer with the `lexer` method with which we can lex an input.
        let lexer = lexerdef.lexer(input);
        // Pass the lexer to the parser and lex and parse the input.
        let (res, errs) = where_y::parse(&lexer);
        if !errs.is_empty() {
            let mut res_e: Vec<String> = vec![];
            for e in errs {
                let e = format!("{}", e.pp(&lexer, &where_y::token_epp));
                res_e.push(e);
            }
            return Err(format!("{:?}", res_e));
        }

        if let Some(Ok(r)) = res {
            eval(&lexer, r).map_err(|(span, msg)| {
                let mut res_e: Vec<String> = vec![];
                let ((line, col), _) = lexer.line_col(span);
                let err_info = format!(
                    "Evaluation error at line {} column {}, '{}' {}.",
                    line,
                    col,
                    lexer.span_str(span),
                    msg);
                res_e.push(err_info);
                format!("{:?}", res_e)
            })
        } else {
            Err("Unknown error.".to_owned())
        }
    }

    pub fn check(&self, t: &dyn ReadTag) -> Option<bool> {
        match self {
            WhereClause::LogicOp(op) => {
                let lhs = op.lhs.check(t)?;
                let rhs = op.rhs.check(t)?;
                if op.op.eq("AND") {
                    Some(lhs && rhs)
                } else if op.op.eq("OR") {
                    Some(lhs || rhs)
                } else {
                    error!("LogicOp, unsupported op: {:?}", &op.op);
                    None
                }
            }
            WhereClause::NotOp(op) => {
                match op.inner.check(t) {
                    Some(v) => Some(!v),
                    None => {
                        error!("NotOp, inner is None");
                        None
                    }
                }
            }
            WhereClause::TextComp(comp) => {
                let tag = comp.tag;
                debug!("tag: {}", &tag);

                if tag.is_text() || tag.is_date() {
                    debug!("text or date tag");
                    let op = &comp.op;
                    debug!("op: {}", op);

                    if let Some(value) = t.get_text_tag(&tag) {
                        debug!("value: {}", value);
                        if op.eq("=") {
                            Some(value.eq(&comp.value))
                        } else if op.eq("!=") || op.eq("<>") {
                            Some(value.ne(&comp.value))
                        } else if op.eq("<") {
                            Some(value.lt(&comp.value))
                        } else if op.eq(">") {
                            Some(value.gt(&comp.value))
                        } else if op.eq("<=") {
                            Some(value.le(&comp.value))
                        } else if op.eq(">=") {
                            Some(value.ge(&comp.value))
                        } else if op.eq_ignore_ascii_case("LIKE") {
                            Some(WildMatchPattern::<'%', '_'>::new(&comp.value).matches(&value))
                        } else if op.eq_ignore_ascii_case("ILIKE") {
                            Some(WildMatchPattern::<'%', '_'>::new(&comp.value.to_lowercase())
                                .matches(&value.to_lowercase()))
                        } else {
                            error!("Comp, unsupported op: {}", &op);
                            None
                        }
                    } else {
                        if op.eq("!=") || op.eq("<>") {
                            Some(true)
                        } else {
                            Some(false)
                        }
                    }
                } else {
                    error!("Comp, tag {} is NOT text or date tag", tag);
                    None
                }
            }
            WhereClause::NumComp(comp) => {
                let tag = comp.tag;
                debug!("tag: {}", &tag);

                if tag.is_numeric() {
                    debug!("numeric tag");
                    let op = &comp.op;
                    debug!("op: {}", op);

                    if let Some(value) = t.get_numeric_tag(&tag) {
                        debug!("value: {}", value);
                        if op.eq("=") {
                            Some(value.eq(&comp.value))
                        } else if op.eq("!=") || op.eq("<>") {
                            Some(value.ne(&comp.value))
                        } else if op.eq("<") {
                            Some(value.lt(&comp.value))
                        } else if op.eq(">") {
                            Some(value.gt(&comp.value))
                        } else if op.eq("<=") {
                            Some(value.le(&comp.value))
                        } else if op.eq(">=") {
                            Some(value.ge(&comp.value))
                        } else {
                            error!("Comp, unsupported op: {}", &op);
                            None
                        }
                    } else {
                        if op.eq("!=") || op.eq("<>") {
                            Some(true)
                        } else {
                            Some(false)
                        }
                    }
                } else {
                    error!("NumComp, tag {} is NOT numeric tag", tag);
                    None
                }
            }
            _ => {
                error!("Unknown where clause {:?}", self);
                None
            }
        }
    }
}

#[derive(Debug)]
pub struct LogicOp {
    op: &'static str,
    lhs: Box<WhereClause>,
    rhs: Box<WhereClause>,
}

#[derive(Debug)]
pub struct NotOp {
    inner: Box<WhereClause>,
}

#[derive(Debug)]
pub struct TextComp {
    op: String,
    tag: &'static MyTag,
    value: String,
}

#[derive(Debug)]
pub struct NumComp {
    op: String,
    tag: &'static MyTag,
    value: u32,
}

#[cfg(test)]
mod test {
    use std::path::{Path, PathBuf};
    use anyhow::Error;

    use crate::model::MyTag;
    use crate::op::ReadTag;
    use crate::where_clause::WhereClause;

    #[derive(Debug)]
    pub struct MockTagImpl<'a> {
        path: &'a Path,
        count: u32,
    }

    impl ReadTag for MockTagImpl<'_> {
        fn get_path(&self) -> &Path {
            self.path
        }

        fn get_text_tag(&self, key: &MyTag) -> Option<String> {
            match key {
                MyTag::Composer => Some("Lee's".to_owned()),
                MyTag::Copyright => Some("Disney".to_owned()),
                MyTag::Lyrics => Some("".to_owned()),
                _ => Some(format!("{}{}", key, self.count)),
            }
        }

        fn get_numeric_tag(&self, _key: &MyTag) -> Option<u32> {
            Some(self.count)
        }

        fn get_numeric_tag_string(&self, _key: &MyTag) -> Option<String> {
            Some(format!("0{}", self.count))
        }

        fn get_property_keys(&self) -> Result<Vec<String>, Error> {
            todo!()
        }

        fn get_property(&self, _key: &str) -> Result<Vec<String>, Error> {
            todo!()
        }
    }

    #[test]
    fn test_where_lyrics() {
        let path = PathBuf::from("mock_file");
        let mock_empty_lyrics = MockTagImpl { path: path.as_path(), count: 5 };

        // =
        let w = WhereClause::new("lyrics=''").expect("Error");
        assert!(w.check(&mock_empty_lyrics).unwrap());
        let w = WhereClause::new("lyrics='Some Lyrics'").expect("Error");
        assert!(!w.check(&mock_empty_lyrics).unwrap());

        // != <>
        let w = WhereClause::new("lyrics<>''").expect("Error");
        assert!(!w.check(&mock_empty_lyrics).unwrap());
        let w = WhereClause::new("lyrics!=''").expect("Error");
        assert!(!w.check(&mock_empty_lyrics).unwrap());
    }

    #[test]
    fn test_where() {
        let path = PathBuf::from("mock_file");
        let mock = MockTagImpl { path: path.as_path(), count: 5 };

        // =
        let w = WhereClause::new("title='test'").expect("Error");
        assert!(!w.check(&mock).unwrap());
        let w = WhereClause::new("title='title5'").expect("Error");
        assert!(w.check(&mock).unwrap());

        // != <>
        let w = WhereClause::new("title<>'test'").expect("Error");
        assert!(w.check(&mock).unwrap());
        let w = WhereClause::new("title!='title5'").expect("Error");
        assert!(!w.check(&mock).unwrap());

        // OR
        let w = WhereClause::new("title='title5' OR track-number=5")
            .expect("Error");
        assert!(w.check(&mock).unwrap());
        let w = WhereClause::new("title='title5' or track-number=1")
            .expect("Error");
        assert!(w.check(&mock).unwrap());
        let w = WhereClause::new("title='test' or track-number=5")
            .expect("Error");
        assert!(w.check(&mock).unwrap());
        let w = WhereClause::new("title='test' or track-number=1")
            .expect("Error");
        assert!(!w.check(&mock).unwrap());

        // AND
        let w = WhereClause::new("title='title5' AND track-number=5")
            .expect("Error");
        assert!(w.check(&mock).unwrap());
        let w = WhereClause::new("title='title5' and track-number=1")
            .expect("Error");
        assert!(!w.check(&mock).unwrap());
        let w = WhereClause::new("title='test' and track-number=5")
            .expect("Error");
        assert!(!w.check(&mock).unwrap());
        let w = WhereClause::new("title='test' and track-number=1")
            .expect("Error");
        assert!(!w.check(&mock).unwrap());

        // AND + OR
        let w =
            WhereClause::new("title='test' and artist='other' or track-number=5")
                .expect("Error");
        assert!(w.check(&mock).unwrap());
        let w =
            WhereClause::new("title='test' and (artist='other' or track-number=5)")
                .expect("Error");
        assert!(!w.check(&mock).unwrap());
        let w =
            WhereClause::new("title='test' and artist='artist5' or track-number=1")
                .expect("Error");
        assert!(!w.check(&mock).unwrap());
        let w =
            WhereClause::new("title='test' or artist='other' and track-number=5")
                .expect("Error");
        assert!(!w.check(&mock).unwrap());

        // NOT
        let w = WhereClause::new("not title='title5'")
            .expect("Error");
        assert!(!w.check(&mock).unwrap());
        let w = WhereClause::new("not title='test'")
            .expect("Error");
        assert!(w.check(&mock).unwrap());

        // AND + OR + NOT
        let w =
            WhereClause::new("not title='test' or artist='other' and track-number=5")
                .expect("Error");
        assert!(w.check(&mock).unwrap());
        let w =
            WhereClause::new("title='test' or not artist='other' and track-number=5")
                .expect("Error");
        assert!(w.check(&mock).unwrap());
        let w =
            WhereClause::new("title='test' or not artist='artist5' and track-number=5")
                .expect("Error");
        assert!(!w.check(&mock).unwrap());
        let w =
            WhereClause::new("title='test' and artist='other' or track-number<7")
                .expect("Error");
        assert!(w.check(&mock).unwrap());
        let w =
            WhereClause::new("not (title='test' and artist='other' or track-number<7)")
                .expect("Error");
        assert!(!w.check(&mock).unwrap());

        // LIKE
        let w = WhereClause::new("title like 'test'")
            .expect("Error");
        assert!(!w.check(&mock).unwrap());
        let w = WhereClause::new("title LIKE 'titl%5'")
            .expect("Error");
        assert!(w.check(&mock).unwrap());
        let w = WhereClause::new("title like 'ti%'")
            .expect("Error");
        assert!(w.check(&mock).unwrap());
        let w = WhereClause::new("title like 'TI%'")
            .expect("Error");
        assert!(!w.check(&mock).unwrap());
        let w = WhereClause::new("title like 'title_'")
            .expect("Error");
        assert!(w.check(&mock).unwrap());
        let w = WhereClause::new("title like 'Title_'")
            .expect("Error");
        assert!(!w.check(&mock).unwrap());

        // LIKE
        let w = WhereClause::new("title ilike 'ti%'")
            .expect("Error");
        assert!(w.check(&mock).unwrap());
        let w = WhereClause::new("title ilike 'TI%'")
            .expect("Error");
        assert!(w.check(&mock).unwrap());
        let w = WhereClause::new("title ilike 'title_'")
            .expect("Error");
        assert!(w.check(&mock).unwrap());
        let w = WhereClause::new("title ILIKE 'Title_'")
            .expect("Error");
        assert!(w.check(&mock).unwrap());

        // with '
        let w = WhereClause::new("composer='Lee''s'")
            .expect("Error");
        assert!(w.check(&mock).unwrap());
        let w = WhereClause::new("composer<>'Lee''s'")
            .expect("Error");
        assert!(!w.check(&mock).unwrap());
        let w = WhereClause::new("not composer<>'Lee''s'")
            .expect("Error");
        assert!(w.check(&mock).unwrap());
    }
}

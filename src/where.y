%start Expr
%avoid_insert "TEXT_VALUE"
%avoid_insert "NUM_VALUE"
%%
Expr -> Result<Expr, ()>:
      Expr 'OP_OR' TermInOr {
        Ok(Expr::Or{ span: $span, lhs: Box::new($1?), rhs: Box::new($3?) })
      }
    | TermInOr { $1 }
    ;

TermInOr -> Result<Expr, ()>:
      TermInOr 'OP_AND' TermInAnd {
        Ok(Expr::And{ span: $span, lhs: Box::new($1?), rhs: Box::new($3?) })
      }
    | TermInAnd { $1 }
    ;

TermInAnd -> Result<Expr, ()>:
      'OP_NOT' Comparator {
        Ok(Expr::Not{ span: $span, inner: Box::new($2?) })
      }
    | Comparator { $1 }
    ;

Comparator -> Result<Expr, ()>:
      '(' Expr ')' { $2 }
    | TextKey ComparatorOp TextValue {
        Ok (Expr::Comparator{
          span: $span, op: Box::new($2?), lhs: Box::new($1?), rhs: Box::new($3?)
        })
      }
    | TextKey LikeOp TextValue {
        Ok (Expr::Comparator{
          span: $span, op: Box::new($2?), lhs: Box::new($1?), rhs: Box::new($3?)
        })
      }
    | NumKey ComparatorOp NumValue {
        Ok (Expr::Comparator{
          span: $span, op: Box::new($2?), lhs: Box::new($1?), rhs: Box::new($3?)
        })
      }
    ;
ComparatorOp -> Result<Expr, ()>:
      'COMP_EQ' { Ok(Expr::ComparatorOp{ span: $span }) }
    | 'COMP_NE' { Ok(Expr::ComparatorOp{ span: $span }) }
    | 'COMP_LT' { Ok(Expr::ComparatorOp{ span: $span }) }
    | 'COMP_GT' { Ok(Expr::ComparatorOp{ span: $span }) }
    | 'COMP_LE' { Ok(Expr::ComparatorOp{ span: $span }) }
    | 'COMP_GE' { Ok(Expr::ComparatorOp{ span: $span }) }
    ;

LikeOp -> Result<Expr, ()>:
      'COMP_LIKE' { Ok(Expr::ComparatorOp{ span: $span }) }
    ;

TextKey -> Result<Expr, ()>:
      'TEXT_TAG' { Ok(Expr::TextTag{ span: $span }) };
TextValue -> Result<Expr, ()>:
      'TEXT_VALUE' { Ok(Expr::TextValue{ span: $span }) };
NumKey -> Result<Expr, ()>:
      'NUM_TAG' { Ok(Expr::NumTag{ span: $span }) };
NumValue -> Result<Expr, ()>:
      'NUM_VALUE' { Ok(Expr::NumValue{ span: $span }) };
%%

use lrpar::Span;

#[derive(Debug)]
pub enum Expr {
    Or {
        span: Span,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    And {
        span: Span,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    Not {
        span: Span,
        inner: Box<Expr>,
    },
    Comparator {
        span: Span,
        op: Box<Expr>,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    ComparatorOp {
        span: Span,
    },
    TextTag {
        span: Span,
    },
    TextValue {
        span: Span,
    },
    NumTag {
        span: Span,
    },
    NumValue {
        span: Span,
    },
}
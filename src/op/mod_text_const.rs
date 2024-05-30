use std::path::{Path, PathBuf};

use anyhow::{anyhow, Error};

use crate::model::{AddDirection, Direction, MyTag, QueryResultPosition, TEXT_TAGS, TextConst};
use crate::op::{Action, get_file_iterator, get_tags_from_args, get_where, WalkAction, WriteAction, WriteTextAction, WriteTextForCurrentAction};
use crate::op::tag_impl::ReadWriteTag;
use crate::util::str::{get_append_from_end, get_insert_from_beginning, get_replaced_any};
use crate::util::str::{get_remove_from_beginning, get_remove_from_end};
use crate::util::str::{get_replaced_beginning, get_replaced_end};
use crate::util::str::{get_replaced_first, get_replaced_last};
use crate::util::str::{rtruncate, truncate};
use crate::where_clause::WhereClause;

pub struct ModTextConstAction {
    it: Box<dyn Iterator<Item=PathBuf>>,
    dry_run: bool,
    tags: Vec<MyTag>,
    where_clause: Option<WhereClause>,
    value: TextConst,
}

impl ModTextConstAction {
    pub fn new<P>(dir: P,
                  dry_run: bool,
                  tags: &[MyTag],
                  where_string: &Option<String>,
                  value: TextConst) -> Result<Self, Error>
        where P: AsRef<Path>
    {
        let it = get_file_iterator(dir.as_ref())?;
        let where_clause = get_where(where_string)?;
        let tags = get_tags_from_args(tags, &TEXT_TAGS)?;
        Self::check(&value).map(|_|
            Self {
                it,
                dry_run,
                tags,
                where_clause,
                value,
            })
    }

    fn check(value: &TextConst) -> Result<(), Error> {
        check_text_const_args(value)?;
        Ok(())
    }
}

fn check_text_const_args(args: &TextConst) -> Result<(), Error> {
    match args {
        TextConst::Remove {
            direction: _direction,
            beginning_offset,
            end_offset
        } => {
            check_beginning_end(beginning_offset, end_offset)
        }
        _ => Ok(())
    }
}

fn check_beginning_end(beginning_offset: &usize, end_offset: &Option<usize>) -> Result<(), Error> {
    if let Some(end_offset) = end_offset {
        if beginning_offset > end_offset {
            Err(anyhow!("start-offset must less or equal end-offset"))
        } else {
            Ok(())
        }
    } else {
        Ok(())
    }
}

impl Action for ModTextConstAction {
    fn do_any(&mut self) -> Result<(), Error> {
        self.do_all()
    }
}

impl WalkAction for ModTextConstAction {
    fn get_iterator(&mut self) -> &mut dyn Iterator<Item=PathBuf> {
        &mut self.it
    }

    fn do_one_file(&mut self, path: &Path) -> Result<bool, Error> {
        self.do_one_file_write(path)
    }

    fn get_where(&self) -> &Option<WhereClause> {
        &self.where_clause
    }

    fn tags(&self) -> &Vec<MyTag> {
        &self.tags
    }
}

impl WriteAction for ModTextConstAction {
    fn is_dry_run(&self) -> bool {
        self.dry_run
    }

    fn write_tags(&self, t: &mut dyn ReadWriteTag) -> Result<bool, Error> {
        self.write_tags_impl(t)
    }
}

impl WriteTextAction for ModTextConstAction {
    fn set_text_tag(&self, t: &mut dyn ReadWriteTag, tag: &MyTag) -> bool {
        self.set_text_tag_impl(t, tag)
    }
}

impl WriteTextForCurrentAction for ModTextConstAction {
    fn get_new_text(&self, current: &Option<String>) -> Option<String> {
        if let Some(curr) = current {
            match &self.value {
                TextConst::Add {
                    add_direction,
                    offset,
                    addend
                } => {
                    get_new_value_add(curr, add_direction, offset, addend)
                }
                TextConst::Replace {
                    from,
                    position,
                    to,
                    ignore_case
                } => {
                    get_new_value_replace(curr, from, position, to, *ignore_case)
                }
                TextConst::Remove {
                    direction,
                    beginning_offset,
                    end_offset
                } => {
                    get_new_value_remove(curr,
                                         direction,
                                         beginning_offset,
                                         end_offset)
                }
                TextConst::Truncate {
                    direction,
                    limit
                } => {
                    get_truncate(curr, direction, limit)
                }
            }
        } else {
            match &self.value {
                TextConst::Add {
                    add_direction: _add_direction,
                    offset,
                    addend
                } => {
                    if *offset == 0 {
                        Some(addend.to_owned())
                    } else {
                        None
                    }
                }
                _ => None
            }
        }
    }
}

fn get_truncate(original: &str, direction: &Direction, limit: &usize) -> Option<String>
{
    let res = match direction {
        Direction::Beginning => truncate(original, *limit),
        Direction::End => rtruncate(original, *limit)
    };

    if original.len() != res.len() {
        Some(res.to_owned())
    } else {
        None
    }
}

pub fn get_new_value_add(current: &str,
                         position_direction: &AddDirection,
                         position_offset: &usize,
                         addend: &str) -> Option<String>
{
    let pos = *position_offset;

    let res = match position_direction {
        AddDirection::InsertFromBeginning => {
            get_insert_from_beginning(current, pos, addend)
        }
        AddDirection::AppendFromEnd => {
            get_append_from_end(current, pos, addend)
        }
    };

    if !none_or_some_eq(&res, current) { res } else { None }
}

pub fn get_new_value_replace(current: &str,
                             from: &str,
                             position: &QueryResultPosition,
                             to: &str,
                             ignore_case: bool) -> Option<String>
{
    let res = match position {
        QueryResultPosition::Any => {
            get_replaced_any(current, from, to, ignore_case)
        }
        QueryResultPosition::Beginning => {
            get_replaced_beginning(current, from, to, ignore_case)
        }
        QueryResultPosition::End => {
            get_replaced_end(current, from, to, ignore_case)
        }

        QueryResultPosition::First => {
            get_replaced_first(current, from, to, ignore_case)
        }
        QueryResultPosition::Last => {
            get_replaced_last(current, from, to, ignore_case)
        }
    };

    if !none_or_some_eq(&res, current) { res } else { None }
}

fn get_new_value_remove(current: &str,
                        direction: &Direction,
                        beginning_offset: &usize,
                        end_offset: &Option<usize>) -> Option<String> {
    let res = match direction {
        Direction::Beginning =>
            get_remove_from_beginning(current, *beginning_offset, end_offset),
        Direction::End =>
            get_remove_from_end(current, *beginning_offset, end_offset),
    };

    if !none_or_some_eq(&res, current) { res } else { None }
}

fn none_or_some_eq(o: &Option<String>, current: &str) -> bool {
    match o {
        Some(v) => {
            if v.eq(current) {
                true
            } else {
                false
            }
        }
        None => true
    }
}

#[cfg(test)]
mod test {
    use crate::model::{AddDirection, QueryResultPosition};

    use super::{get_new_value_add, get_new_value_replace, none_or_some_eq};

    #[test]
    fn test_get_new_value_replace() {
        let from = "01";
        let to = "";
        let res = get_new_value_replace("01",
                                        from,
                                        &QueryResultPosition::Last,
                                        to,
                                        false);
        assert_eq!(&res.unwrap(), "");
    }

    #[test]
    fn test_get_new_value_insert() {
        let res = get_new_value_add("鹿港小鎮03",
                                    &AddDirection::AppendFromEnd,
                                    &4,
                                    "_abc_");
        assert_eq!(&res.unwrap(), "鹿港_abc_小鎮03");
    }

    #[test]
    fn test_none_or_inner_eq() {
        assert!(none_or_some_eq(&None, "abc"));
        assert!(none_or_some_eq(&Some("abc".to_owned()), "abc"));

        assert_eq!(false, none_or_some_eq(&Some("abc".to_owned()), "xyz"));
    }
}
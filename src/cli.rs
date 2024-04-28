use anyhow::Error;
use clap::Parser;
use flexi_logger::{Age, Cleanup, Criterion, DeferredNow, Duplicate, FileSpec, Logger, LoggerHandle, Naming, TS_DASHES_BLANK_COLONS_DOT_BLANK, WriteMode};
use itertools::Itertools;
use log::{debug, error, Record};

use crate::args::{App, Command};
use crate::config::get_log_level;

use crate::op::Action;
use crate::op::ConvEnAction;
use crate::op::ConvUtf8Action;
use crate::op::ConvZhAction;
use crate::op::ModNumAction;
use crate::op::ModTextConstAction;
use crate::op::ModTextRegexAction;
use crate::op::SetConstAction;
use crate::op::SetSeqAction;
use crate::op::ViewAction;

pub fn direct_format(
    w: &mut dyn std::io::Write,
    _now: &mut DeferredNow,
    record: &Record,
) -> Result<(), std::io::Error> {
    write!(w, "{}", record.args())
}

pub fn my_default_format(
    w: &mut dyn std::io::Write,
    now: &mut DeferredNow,
    record: &Record,
) -> Result<(), std::io::Error> {
    write!(
        w,
        "{} {} [{}] ",
        record.level(),
        now.format(TS_DASHES_BLANK_COLONS_DOT_BLANK),
        record.module_path().unwrap_or("<unnamed>"),
    )?;

    write!(w, "{}", record.args())
}

const MAX_LOG_SIZE: u64 = 10 * 1024 * 1024;

fn init_logger() -> Result<LoggerHandle, Error> {
    let log_level = get_log_level();
    let logger = Logger::try_with_str(to_log_level(&log_level))?
        .log_to_file(FileSpec::default().directory("./logs"))
        .format_for_files(my_default_format)
        .append()
        .rotate(
            Criterion::AgeOrSize(Age::Day, MAX_LOG_SIZE),
            Naming::Timestamps,
            Cleanup::Never,
        )
        .duplicate_to_stdout(Duplicate::Info)
        .format_for_stdout(direct_format)
        .write_mode(WriteMode::BufferAndFlush)
        .start()?;
    Ok(logger)
}

fn to_log_level(i: &Option<String>) -> &str {
    match i {
        Some(s) => {
            if s.eq("trace")
                || s.eq("debug")
                || s.eq("info")
                || s.eq("warn")
                || s.eq("error") {
                s
            } else {
                "info"
            }
        }
        None => "info",
    }
}

pub fn entry() -> Result<(), Error> {
    let mut logger = init_logger()?;

    do_command(&mut logger).map_err(|e| {
        logger.flush();
        let _ = logger.adapt_duplication_to_stdout(Duplicate::None);
        error!("{:?}", e);
        e
    })
}

fn do_command(logger: &mut LoggerHandle) -> Result<(), Error> {
    let app = App::parse();
    match app.command {
        Command::ConvEn(args) => {
            debug!("args: {:?}", args);
            if args.global_opts.quiet {
                logger.adapt_duplication_to_stdout(Duplicate::Error)?;
            }
            let tags =
                args.global_opts.to_vec_my_tag().into_iter().unique()
                    .collect::<Vec<_>>();
            ConvEnAction::new(&args.global_opts.directory,
                              args.global_opts.dry_run,
                              &tags,
                              &args.global_opts.where_clause,
                              &args.profile)?.do_any()
        }
        Command::ConvZh(args) => {
            debug!("args: {:?}", args);
            if args.global_opts.quiet {
                logger.adapt_duplication_to_stdout(Duplicate::Error)?;
            }
            let tags =
                args.global_opts.to_vec_my_tag().into_iter().unique()
                    .collect::<Vec<_>>();
            ConvZhAction::new(&args.global_opts.directory,
                              args.global_opts.dry_run,
                              &tags,
                              &args.global_opts.where_clause,
                              &args.profile)?.do_any()
        }
        Command::ConvUtf8(args) => {
            debug!("args: {:?}", args);
            if args.global_opts.quiet {
                logger.adapt_duplication_to_stdout(Duplicate::Error)?;
            }
            let tags =
                args.global_opts.to_vec_my_tag().into_iter().unique()
                    .collect::<Vec<_>>();
            ConvUtf8Action::new(&args.global_opts.directory,
                                args.global_opts.dry_run,
                                &tags,
                                &args.global_opts.where_clause,
                                &args.encoding_name)?.do_any()
        }
        Command::ModNum(args) => {
            debug!("args: {:?}", args);
            if args.quiet {
                logger.adapt_duplication_to_stdout(Duplicate::Error)?;
            }
            let tags = args.to_vec_my_tag().into_iter().unique().collect::<Vec<_>>();
            ModNumAction::new(&args.directory,
                              args.dry_run,
                              &tags,
                              &args.where_clause,
                              &args.calc_method,
                              &args.operand,
                              &args.padding)?.do_any()
        }
        Command::ModTextConst(args) => {
            debug!("args: {:?}", args);
            if args.global_opts.quiet {
                logger.adapt_duplication_to_stdout(Duplicate::Error)?;
            }
            let tags =
                args.global_opts.to_vec_my_tag().into_iter().unique()
                    .collect::<Vec<_>>();
            ModTextConstAction::new(&args.global_opts.directory,
                                    args.global_opts.dry_run,
                                    &tags,
                                    &args.global_opts.where_clause,
                                    &args.value)?.do_any()
        }
        Command::ModTextRegex(args) => {
            debug!("args: {:?}", args);
            if args.global_opts.quiet {
                logger.adapt_duplication_to_stdout(Duplicate::Error)?;
            }
            let tags =
                args.global_opts.to_vec_my_tag().into_iter().unique()
                    .collect::<Vec<_>>();
            ModTextRegexAction::new(&args.global_opts.directory,
                                    args.global_opts.dry_run,
                                    &tags,
                                    &args.global_opts.where_clause,
                                    &args.from,
                                    &args.ignore_case,
                                    &args.to)?.do_any()
        }
        Command::SetConst(args) => {
            debug!("args: {:?}", args);
            if args.global_opts.quiet {
                logger.adapt_duplication_to_stdout(Duplicate::Error)?;
            }
            let tags = args.global_opts.tags.into_iter().unique()
                .collect::<Vec<_>>();
            SetConstAction::new(&args.global_opts.directory,
                                args.global_opts.dry_run,
                                &tags,
                                &args.where_clause,
                                &args.value,
                                &args.set_when,
                                &args.modify_mode)?.do_any()
        }
        Command::SetSeq(args) => {
            debug!("args: {:?}", args);
            if args.global_opts.quiet {
                logger.adapt_duplication_to_stdout(Duplicate::Error)?;
            }
            let tags = args.global_opts.tags.into_iter().unique()
                .collect::<Vec<_>>();
            SetSeqAction::new(&args.global_opts.directory,
                              args.global_opts.dry_run,
                              &tags,
                              &args.value,
                              &args.hyphen,
                              &args.modify_mode)?.do_any()
        }
        Command::View(args) => {
            debug!("args: {:?}", args);
            let tags = args.tags.into_iter().unique().collect::<Vec<_>>();
            ViewAction::new(&args.directory,
                            &tags,
                            &args.where_clause,
                            args.with_properties)?.do_any()
        }
    }
}

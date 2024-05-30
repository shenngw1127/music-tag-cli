use anyhow::Error;
use clap::Parser;
use flexi_logger::{Age, Cleanup, Criterion, DeferredNow, Duplicate, FileSpec, Logger, LoggerHandle, Naming, TS_DASHES_BLANK_COLONS_DOT_BLANK, WriteMode};
use log::{debug, error, Record};

use crate::args::{App, Command, LrcDirection};
use crate::config::get_log_level;

use crate::op::{Action, ClearAction, LrcExpAction, LrcImpAction};
use crate::op::ConvEnAction;
use crate::op::ConvUtf8Action;
use crate::op::ConvZhAction;
use crate::op::ExpAction;
use crate::op::ImpAction;
use crate::op::ModNumAction;
use crate::op::ModTextConstAction;
use crate::op::ModTextRegexAction;
use crate::op::SetConstAction;
use crate::op::SetNameAction;
use crate::op::SetSeqAction;
use crate::op::RenAction;
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
    let mut action = get_action(logger)?;
    action.do_any()
}

fn get_action(logger: &mut LoggerHandle) -> Result<Box<dyn Action>, Error> {
    let app = App::parse();
    let action: Box<dyn Action> = match app.command {
        Command::Clear(args) => {
            debug!("args: {:?}", args);
            if args.quiet {
                logger.adapt_duplication_to_stdout(Duplicate::Error)?;
            }
            Box::new(ClearAction::new(&args.directory,
                                       args.dry_run,
                                       &args.tags,
                                       &args.where_clause)?)
        }
        Command::ConvEn(args) => {
            debug!("args: {:?}", args);
            if args.global_opts.quiet {
                logger.adapt_duplication_to_stdout(Duplicate::Error)?;
            }
            let tags = args.global_opts.tags.into_iter()
                .map(|t| t.into()).collect::<Vec<_>>();
            Box::new(ConvEnAction::new(&args.global_opts.directory,
                                       args.global_opts.dry_run,
                                       &tags,
                                       &args.global_opts.where_clause,
                                       &args.profile)?)
        }
        Command::ConvZh(args) => {
            debug!("args: {:?}", args);
            if args.global_opts.quiet {
                logger.adapt_duplication_to_stdout(Duplicate::Error)?;
            }
            let tags = args.global_opts.tags.into_iter()
                .map(|t| t.into()).collect::<Vec<_>>();
            Box::new(ConvZhAction::new(&args.global_opts.directory,
                                       args.global_opts.dry_run,
                                       &tags,
                                       &args.global_opts.where_clause,
                                       &args.profile)?)
        }
        Command::ConvUtf8(args) => {
            debug!("args: {:?}", args);
            if args.global_opts.quiet {
                logger.adapt_duplication_to_stdout(Duplicate::Error)?;
            }
            let tags = args.global_opts.tags.into_iter()
                .map(|t| t.into()).collect::<Vec<_>>();
            Box::new(ConvUtf8Action::new(&args.global_opts.directory,
                                         args.global_opts.dry_run,
                                         &tags,
                                         &args.global_opts.where_clause,
                                         &args.encoding_name)?)
        }
        Command::ModNum(args) => {
            debug!("args: {:?}", args);
            if args.quiet {
                logger.adapt_duplication_to_stdout(Duplicate::Error)?;
            }
            let tags = args.tags.into_iter()
                .map(|t| t.into()).collect::<Vec<_>>();
            Box::new(ModNumAction::new(&args.directory,
                                       args.dry_run,
                                       &tags,
                                       &args.where_clause,
                                       &args.calc_method,
                                       args.operand,
                                       args.padding)?)
        }
        Command::ModTextConst(args) => {
            debug!("args: {:?}", args);
            if args.global_opts.quiet {
                logger.adapt_duplication_to_stdout(Duplicate::Error)?;
            }
            let tags = args.global_opts.tags.into_iter()
                .map(|t| t.into()).collect::<Vec<_>>();
            Box::new(ModTextConstAction::new(&args.global_opts.directory,
                                             args.global_opts.dry_run,
                                             &tags,
                                             &args.global_opts.where_clause,
                                             args.value.into())?)
        }
        Command::ModTextRegex(args) => {
            debug!("args: {:?}", args);
            if args.global_opts.quiet {
                logger.adapt_duplication_to_stdout(Duplicate::Error)?;
            }
            let tags = args.global_opts.tags.into_iter()
                .map(|t| t.into()).collect::<Vec<_>>();
            Box::new(ModTextRegexAction::new(&args.global_opts.directory,
                                             args.global_opts.dry_run,
                                             &tags,
                                             &args.global_opts.where_clause,
                                             &args.from,
                                             args.ignore_case,
                                             &args.to)?)
        }
        Command::Ren(args) => {
            debug!("args: {:?}", args);
            if args.quiet {
                logger.adapt_duplication_to_stdout(Duplicate::Error)?;
            }
            Box::new(RenAction::new(&args.directory,
                                    args.dry_run,
                                    &args.where_clause,
                                    &args.template)?)
        }
        Command::Imp(args) => {
            debug!("args: {:?}", args);
            if args.quiet {
                logger.adapt_duplication_to_stdout(Duplicate::Error)?;
            }
            Box::new(ImpAction::new(&args.source_file,
                                    &args.base_directory,
                                    args.dry_run)?)
        }
        Command::Lrc(args) => {
            debug!("args: {:?}", args);
            if args.quiet {
                logger.adapt_duplication_to_stdout(Duplicate::Error)?;
            }
            match args.direction {
                LrcDirection::Export => Box::new(LrcExpAction::new(&args.directory,
                                                                   &args.encoding_name,
                                                                   args.dry_run,
                                                                   &args.where_clause)?),
                LrcDirection::Import => Box::new(LrcImpAction::new(&args.directory,
                                                                   &args.encoding_name,
                                                                   args.dry_run,
                                                                   &args.where_clause)?)
            }
        }
        Command::SetConst(args) => {
            debug!("args: {:?}", args);
            if args.global_opts.quiet {
                logger.adapt_duplication_to_stdout(Duplicate::Error)?;
            }
            Box::new(SetConstAction::new(&args.global_opts.directory,
                                         args.global_opts.dry_run,
                                         &args.global_opts.tags,
                                         &args.where_clause,
                                         args.value.into(),
                                         &args.set_when,
                                         &args.modify_mode)?)
        }
        Command::SetName(args) => {
            debug!("args: {:?}", args);
            if args.quiet {
                logger.adapt_duplication_to_stdout(Duplicate::Error)?;
            }
            Box::new(SetNameAction::new(&args.directory,
                                        args.dry_run,
                                        &args.set_when,
                                        &args.where_clause,
                                        &args.template)?)
        }
        Command::SetSeq(args) => {
            debug!("args: {:?}", args);
            if args.global_opts.quiet {
                logger.adapt_duplication_to_stdout(Duplicate::Error)?;
            }
            Box::new(SetSeqAction::new(&args.global_opts.directory,
                                       args.global_opts.dry_run,
                                       &args.global_opts.tags,
                                       args.value.start,
                                       args.value.step,
                                       args.value.padding,
                                       &args.hyphen,
                                       &args.modify_mode)?)
        }
        Command::Exp(args) => {
            debug!("args: {:?}", args);
            logger.adapt_duplication_to_stdout(Duplicate::Error)?;
            Box::new(ExpAction::new(&args.directory,
                                    &args.tags,
                                    &args.where_clause,
                                    args.with_properties,
                                    &args.output_file)?)
        }
        Command::View(args) => {
            debug!("args: {:?}", args);
            logger.adapt_duplication_to_stdout(Duplicate::Error)?;
            Box::new(ViewAction::new(&args.directory,
                                     &args.tags,
                                     &args.where_clause,
                                     args.with_properties)?)
        }
    };
    Ok(action)
}

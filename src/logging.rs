//! Logging Options

use std::{ffi::OsString, path::Path};

// // NOTE NOT ALL OPTIONS ARE COMPATIBLE !!!!
// pub enum LoggingOptions<'a> {
//     ONLY_LOG,
//     REPORT_EXTRA,
//     VERBOSE,
//     TIME_STAMPS,
//     FULL_PATH_NAMES,
//     SIZES_BYTES,
//     DONT_LOG_SIZE,
//     DONT_LOG_CLASS,
//     DONT_LOG_FILE_NAMES,
//     DONT_LOG_DIR_NAMES,
//     NO_PROGRESS_DISPLAY,
//     SHOW_ESTIMATED_TIME_OF_ARRIVAL,
//     LOG_OUT_OVERWRITE(&'a Path),
//     LOG_OUT_APPEND(&'a Path),
//     UNICODE_OUTPUT,
//     UNICODE_LOG_OVERWRITE(&'a Path),
//     UNICODE_LOG_APPEND(&'a Path),
//     WRITE_STATUS_TO_LOG,
//     // TODO JOBS
//     _MULTIPLE([bool; 14], Option<&'a Path>, Option<&'a Path>, Option<&'a Path>, Option<&'a Path>)
// }

#[derive(Debug, Clone, Copy)]
pub struct LoggingSettings<'a> {
    pub log: &'a Path,
    pub unicode: bool,
    pub append: bool,
}

impl<'a> From<&'a LoggingSettings<'a>> for OsString {
    fn from(ls: &'a LoggingSettings<'a>) -> Self {
        OsString::from(
            String::from("/") + 
            if ls.unicode { "uni" } else { "" } + 
            "log" + if ls.append { "+" } else { "" } + 
            ":" + 
            ls.log.to_str().unwrap()
        )
    }
}
impl<'a> From<LoggingSettings<'a>> for OsString {
    fn from(ls: LoggingSettings<'a>) -> Self {
        (&ls).into()
    }
}
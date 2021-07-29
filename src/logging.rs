use std::path::Path;

// NOTE NOT ALL OPTIONS ARE COMPATIBLE !!!!
pub enum LoggingOptions<'a> {
    ONLY_LOG,
    REPORT_EXTRA,
    VERBOSE,
    TIME_STAMPS,
    FULL_PATH_NAMES,
    SIZES_BYTES,
    DONT_LOG_SIZE,
    DONT_LOG_CLASS,
    DONT_LOG_FILE_NAMES,
    DONT_LOG_DIR_NAMES,
    NO_PROGRESS_DISPLAY,
    SHOW_ESTIMATED_TIME_OF_ARRIVAL,
    LOG_OUT_OVERWRITE(&'a Path),
    LOG_OUT_APPEND(&'a Path),
    UNICODE_OUTPUT,
    UNICODE_LOG_OVERWIRTE(&'a Path),
    UNICODE_LOG_APPEND(&'a Path),
    WRITE_STATUS_TO_LOG,
    // TODO JOBS
    _MULTIPLE([bool; 14], Option<&'a Path>, Option<&'a Path>, Option<&'a Path>, Option<&'a Path>)
}


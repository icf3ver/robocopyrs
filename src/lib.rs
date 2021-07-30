pub mod filter;
pub mod performance;
pub mod logging;

use std::{convert::{TryFrom, TryInto}, ffi::OsString, ops::Add, path::Path, process::Command};
use filter::Filter;
use performance::PerformanceOptions;
use logging::LoggingSettings;

/// The file Properties
/// Default is both Data and Attributes
#[derive(Copy, Clone)]
pub enum FileProperties {
    DATA,
    ATTRIBUTES,
    TIME_STAMPS,
    NTFS_ACCESS_CONTROL_LIST,
    OWNER_INFO,
    AUDITING_INFO,
    ALL,
    _MULTIPLE([bool; 6])
}

impl Add for FileProperties {
    type Output = Self;
    
    fn add(self, rhs: Self) -> Self::Output {
        let mut result_props = match self {
            Self::_MULTIPLE(props) => props,
            Self::ALL => [true; 6],
            prop => {
                let mut val = 2_u8.pow(prop.index().unwrap() as u32) + 2_u8; 
                (0..6).map(|_| { val = val >> 1; val == 1 }).collect::<Vec<bool>>().try_into().unwrap()
            }
        };

        match rhs {
            Self::_MULTIPLE(props) => result_props = result_props.iter().zip(props.iter()).map(|(a, b)| *a && *b).collect::<Vec<bool>>().try_into().unwrap(),
            Self::ALL => (),
            prop => result_props[prop.index().unwrap()] = true
        }

        Self::_MULTIPLE(result_props)
    }
}

impl FileProperties {
    const VARIANTS: [Self; 6] = [
        Self::DATA,
        Self::ATTRIBUTES,
        Self::TIME_STAMPS,
        Self::NTFS_ACCESS_CONTROL_LIST,
        Self::OWNER_INFO,
        Self::AUDITING_INFO
    ];

    fn index(&self) -> Option<usize>{
        match self {
            Self::DATA => Some(0),
            Self::ATTRIBUTES => Some(1),
            Self::TIME_STAMPS => Some(2),
            Self::NTFS_ACCESS_CONTROL_LIST => Some(3),
            Self::OWNER_INFO => Some(4),
            Self::AUDITING_INFO => Some(5),
            _ => None,
        }
    }

    pub fn single_properties(&self) -> Vec<FileProperties> {
        match self {
            Self::_MULTIPLE(props) => {
                Self::VARIANTS.iter().zip(props.iter()).filter(|(_, exists)| **exists).into_iter().unzip::<&Self, &bool, Vec<Self>, Vec<bool>>().0
            },
            Self::ALL => Self::VARIANTS.to_vec(),
            prop => vec![*prop],
        }
    }
    
    pub fn as_os_string(&self) -> OsString {
        let full ;
        OsString::from(match self {
            Self::DATA => "/copy:D",
            Self::ATTRIBUTES => "/copy:A",
            Self::TIME_STAMPS => "/copy:T",
            Self::NTFS_ACCESS_CONTROL_LIST => "/copy:S",
            Self::OWNER_INFO => "/copy:O",
            Self::AUDITING_INFO => "/copy:U",
            Self::ALL => "/copy:DATSOU",
            Self::_MULTIPLE(props) => {
                let part = ['D', 'A', 'T', 'S', 'O', 'U'].iter().zip(props.iter()).filter(|(_, exists)| **exists).into_iter().unzip::<&char, &bool, String, Vec<bool>>().0;
                full = String::from("/copy:") + part.as_str();
                full.as_str()
            }
        })
    }
}


/// The directory Properties
/// Default is both Data and Attributes
#[derive(Copy, Clone)]
pub enum DirectoryProperties {
    DATA,
    ATTRIBUTES,
    TIME_STAMPS,
    ALL,
    _MULTIPLE([bool; 3])
}

impl Add for DirectoryProperties {
    type Output = Self;
    
    fn add(self, rhs: Self) -> Self::Output {
        let mut result_props = match self {
            Self::_MULTIPLE(props) => props,
            Self::ALL => [true; 3],
            prop => {
                let mut val = 2_u8.pow(prop.index().unwrap() as u32) + 2_u8; 
                (0..6).map(|_| { val = val >> 1; val == 1 }).collect::<Vec<bool>>().try_into().unwrap()
            }
        };

        match rhs {
            Self::_MULTIPLE(props)
             => result_props = result_props.iter().zip(props.iter()).map(|(a, b)| *a && *b).collect::<Vec<bool>>().try_into().unwrap(),
            Self::ALL => (),
            prop => result_props[prop.index().unwrap()] = true
        }

        Self::_MULTIPLE(result_props)
    }
}

impl DirectoryProperties {
    const VARIANTS: [Self; 3] = [
        Self::DATA,
        Self::ATTRIBUTES,
        Self::TIME_STAMPS,
    ];

    fn index(&self) -> Option<usize>{
        match self {
            Self::DATA => Some(0),
            Self::ATTRIBUTES => Some(1),
            Self::TIME_STAMPS => Some(2),
            _ => None,
        }
    }
    
    pub fn single_properties(&self) -> Vec<DirectoryProperties> {
        match self {
            Self::_MULTIPLE(props) => {
                Self::VARIANTS.iter().zip(props.iter()).filter(|(_, exists)| **exists).into_iter().unzip::<&Self, &bool, Vec<Self>, Vec<bool>>().0
            },
            Self::ALL => Self::VARIANTS.to_vec(),
            prop => vec![*prop],
        }
    }

    pub fn as_os_string(&self) -> OsString {
        let full ;
        OsString::from(match self {
            Self::DATA => "/dcopy:D",
            Self::ATTRIBUTES => "/dcopy:A",
            Self::TIME_STAMPS => "/dcopy:T",
            Self::ALL => "/dcopy:DAT",
            Self::_MULTIPLE(props) => {
                let part = ['D', 'A', 'T'].iter().zip(props.iter()).filter(|(_, exists)| **exists).into_iter().unzip::<&char, &bool, String, Vec<bool>>().0;
                full = String::from("/dcopy:") + part.as_str();
                full.as_str()
            }
        })
    }
}


#[derive(Copy, Clone)]
pub enum FileAttributes {
    READ_ONLY,
    ARCHIVE,
    SYSTEM,
    HIDDEN,
    COMPRESSED,
    NOT_CONTENT_INDEXED,
    ENCRYPTED,
    TEMPORARY,
    _MULTIPLE([bool; 8])
}

impl Add for FileAttributes {
    type Output = Self;
    
    fn add(self, rhs: Self) -> Self::Output {
        let mut result_attribs = match self {
            Self::_MULTIPLE(attribs) => attribs,
            attrib => {
                let mut val = 2_u8.pow(attrib.index().unwrap() as u32) * 2_u8; 
                (0..6).map(|_| { val = val >> 1; val == 1 }).collect::<Vec<bool>>().try_into().unwrap()
            }
        };

        match rhs {
            Self::_MULTIPLE(attribs)
             => result_attribs = result_attribs.iter().zip(attribs.iter()).map(|(a, b)| *a && *b).collect::<Vec<bool>>().try_into().unwrap(),
            attrib => result_attribs[attrib.index().unwrap()] = true
        }

        Self::_MULTIPLE(result_attribs)
    }
}


impl FileAttributes {
    const VARIANTS: [Self; 8] = [
        Self::READ_ONLY,
        Self::ARCHIVE,
        Self::SYSTEM,
        Self::HIDDEN,
        Self::COMPRESSED,
        Self::NOT_CONTENT_INDEXED,
        Self::ENCRYPTED,
        Self::TEMPORARY
    ];

    fn index(&self) -> Option<usize>{
        match self {
            Self::READ_ONLY => Some(0),
            Self::ARCHIVE => Some(1),
            Self::SYSTEM => Some(2),
            Self::HIDDEN => Some(3),
            Self::COMPRESSED => Some(4),
            Self::NOT_CONTENT_INDEXED => Some(5),
            Self::ENCRYPTED => Some(6),
            Self::TEMPORARY => Some(7),
            _ => None,
        }
    }
    
    pub fn single_properties(&self) -> Vec<FileAttributes> {
        match self {
            Self::_MULTIPLE(attribs) => {
                Self::VARIANTS.iter().zip(attribs.iter()).filter(|(_, exists)| **exists).into_iter().unzip::<&Self, &bool, Vec<Self>, Vec<bool>>().0
            },
            attrib => vec![*attrib],
        }
    }
    
    pub fn as_os_string(&self) -> OsString {
        let part ;
        OsString::from(match self {
            Self::READ_ONLY => "R",
            Self::ARCHIVE => "A",
            Self::SYSTEM => "S",
            Self::HIDDEN => "H",
            Self::COMPRESSED => "C",
            Self::NOT_CONTENT_INDEXED => "N",
            Self::ENCRYPTED => "E",
            Self::TEMPORARY => "T",
            Self::_MULTIPLE(props) => {
                part = ['R', 'A', 'S', 'H', 'C', 'N', 'E', 'T'].iter().zip(props.iter()).filter(|(_, exists)| **exists).into_iter().unzip::<&char, &bool, String, Vec<bool>>().0;
                part.as_str()
            }
        })
    }
}


pub enum CopyMode {
    RESTARTABLE_MODE,
    BACKUP_MODE,
    RESTARTABLE_MODE_BACKUP_MODE_FALLBACK
}

impl CopyMode {
    pub fn as_os_string(&self) -> OsString {
        match self {
            Self::RESTARTABLE_MODE => OsString::from("/z"),
            Self::BACKUP_MODE => OsString::from("/b"),
            Self::RESTARTABLE_MODE_BACKUP_MODE_FALLBACK => OsString::from("/zb"),
        }
    }
}

pub enum Move {
    FILES,
    FILES_AND_DIRS,
}

impl Move {
    pub fn as_os_string(&self) -> OsString {
        match self {
            Self::FILES => OsString::from("/mov"),
            Self::FILES_AND_DIRS => OsString::from("/move"),
        }
    }
}


pub enum PostCopyActions {
    ADD_ATTRIBS_TO_FILES(FileAttributes),
    RMV_ATTRIBS_FROM_FILES(FileAttributes),
    _MULTIPLE(FileAttributes, FileAttributes)
}


impl PostCopyActions {
    pub fn as_os_string_vec(&self) -> Vec<OsString> {
        match self {
            PostCopyActions::ADD_ATTRIBS_TO_FILES(attribs) => vec![OsString::from(String::from("/a+:") + attribs.as_os_string().to_str().unwrap())],
            PostCopyActions::RMV_ATTRIBS_FROM_FILES(attribs) => vec![OsString::from(String::from("/a-:") + attribs.as_os_string().to_str().unwrap())],
            PostCopyActions::_MULTIPLE(add_attribs, rmv_attribs) => vec![OsString::from(String::from("/a+:") + add_attribs.as_os_string().to_str().unwrap()), OsString::from(String::from("/a-:") + rmv_attribs.as_os_string().to_str().unwrap())],
        }
    }
}

pub enum FilesystemOptions {
    FAT_FILE_NAMES,
    ASSUME_FAT_FILE_TIMES,
    DISSABLE_LONG_PATHS,
    _MULTIPLE([bool; 3])
}

impl FilesystemOptions {
    pub fn as_os_string_vec(&self) -> Vec<OsString> {
        match self {
            Self::FAT_FILE_NAMES => vec![OsString::from("/fat")],
            Self::ASSUME_FAT_FILE_TIMES => vec![OsString::from("/fft")],
            Self::DISSABLE_LONG_PATHS => vec![OsString::from("/256")],
            Self::_MULTIPLE(options) => ["/fat", "/fft", "/256"].iter().zip(options.iter()).filter(|(_, exists)| **exists).map(|(option, _)| OsString::from(*option)).collect()
        }
    }
}

pub struct RetrySettings {
    pub specify_retries_failed_copies: Option<usize>, // default 1 million set in registry
    pub specify_wait_between_retries: Option<usize>, // default 30 seconds set in registry
    pub save_specifications: bool,
    
    pub await_share_names_def: bool,
}

impl RetrySettings {
    pub fn as_os_string_vec(&self) -> Vec<OsString> {
        let mut result = Vec::new();

        if let Some(specified) = self.specify_retries_failed_copies {
            result.push(OsString::from(format!("/r:{}", specified)))
        }
        if let Some(specified) = self.specify_wait_between_retries {
            result.push(OsString::from(format!("/w:{}", specified)))
        }
        if self.save_specifications {
            result.push(OsString::from("/reg"))
        }
        if self.await_share_names_def {
            result.push(OsString::from("/tbd"))
        }

        result
    }
}


#[repr(i8)]
pub enum OkExitCode{
    NO_CHANGE = 0,
    SOME_COPIES = 1,
    EXTRA_FOUND = 2,
    SOME_COPIES_EXTRA_FOUND = 3,
    MISSMATCHES = 4,
    SOME_COPIES_MISSMATCHES = 5,
    MISSMATCHES_EXTRA_FOUND = 6,
    SOME_COPIES_MISSMATCHES_EXTRA_FOUND = 7,
}

#[derive(Debug)]
#[repr(i8)]
pub enum ErrExitCode{
    FAIL = 8,
    SOME_COPIES_FAIL = 9,
    FAIL_EXTRA_FOUND = 10,
    SOME_COPIES_FAIL_EXTRA_FOUND = 11,
    FAIL_MISSMATCHES = 12,
    SOME_COPIES_FAIL_MISSMATCHES = 13,
    FAIL_MISSMATCHES_EXTRA_FOUND = 14,
    SOME_COPIES_FAIL_MISSMATCHES_EXTRA_FOUND = 15,
    NO_CHANGE_FATAL_ERROR = 16,
}

impl TryFrom<i8> for OkExitCode {
    type Error = Result<ErrExitCode, (&'static str, i8)>;

    fn try_from(n: i8) -> Result<Self, Self::Error> {
        if n < 8 {
            Ok(
                match n {
                    0 => OkExitCode::NO_CHANGE,
                    1 => OkExitCode::SOME_COPIES,
                    2 => OkExitCode::EXTRA_FOUND,
                    3 => OkExitCode::SOME_COPIES_EXTRA_FOUND,
                    4 => OkExitCode::MISSMATCHES,
                    5 => OkExitCode::SOME_COPIES_MISSMATCHES,
                    6 => OkExitCode::MISSMATCHES_EXTRA_FOUND,
                    7 => OkExitCode::SOME_COPIES_MISSMATCHES_EXTRA_FOUND,
                    _ => unreachable!(),
                }
            )
        } else {
            Err(
                match n {
                    8 => Ok(ErrExitCode::FAIL),
                    9 => Ok(ErrExitCode::SOME_COPIES_FAIL),
                    10 => Ok(ErrExitCode::FAIL_EXTRA_FOUND),
                    11 => Ok(ErrExitCode::SOME_COPIES_FAIL_EXTRA_FOUND),
                    12 => Ok(ErrExitCode::FAIL_MISSMATCHES),
                    13 => Ok(ErrExitCode::SOME_COPIES_FAIL_MISSMATCHES),
                    14 => Ok(ErrExitCode::FAIL_MISSMATCHES_EXTRA_FOUND),
                    15 => Ok(ErrExitCode::SOME_COPIES_FAIL_MISSMATCHES_EXTRA_FOUND),
                    16 => Ok(ErrExitCode::NO_CHANGE_FATAL_ERROR),
                    c => Err(("Invalid exit code", c)),
                }
            )
        }
    }
}

pub struct RobocopyCommand<'a> {
    pub source: &'a Path,
    pub destination: &'a Path,
    pub files: Vec<&'a str>, // wildcard chars are supported
    
    pub copy_mode: Option<CopyMode>,
    pub unbuffered: bool,

    pub empty_dir_copy: bool,
    pub remove_files_and_dirs_not_in_src: bool,
    pub only_copy_top_n_levels: Option<usize>,
    pub structure_and_size_zero_files_only: bool,
    
    pub copy_file_properties: Option<DirectoryProperties>,
    pub copy_dir_properties: Option<DirectoryProperties>,

    pub filter: Option<Filter<'a>>,

    pub filesystem_options: Option<FilesystemOptions>,
    pub performance_options: Option<PerformanceOptions>,
    pub retry_settings: Option<RetrySettings>,
    
    pub logging: Option<LoggingSettings<'a>>,
    
    pub mv: Option<Move>,
    pub post_copy_actions: Option<PostCopyActions>,

    /// To use this option empty_dir_copy and PostCopyAction::RMV_FILES_AND_DIRS_NOT_IN_SRC must also be in use
    pub overwrite_destination_dir_sec_settings_when_mirror: bool,
    // todo fix secfix and timfix
    // todo job options
}

impl<'a> RobocopyCommand<'a> {
    pub fn execute(&self) -> Result<OkExitCode, Result<ErrExitCode, (&'static str, i8)>>{
        let mut command = Command::new("robocopy");
        
        command
            .arg(self.source)
            .arg(self.destination);

        self.files.iter().for_each(|file| {command.arg(file);});

        if let Some(mode) = &self.copy_mode {
            command.arg(mode.as_os_string());
        }
        if self.unbuffered {
            command.arg("/j");
        }
        
        if self.empty_dir_copy && 
                self.remove_files_and_dirs_not_in_src && 
                self.overwrite_destination_dir_sec_settings_when_mirror {
            command.arg("/mir");
            command.arg("/e");
        } else {
            if self.empty_dir_copy {
                command.arg("/e");
            } else {
                command.arg("/s");
            }
            
            if self.remove_files_and_dirs_not_in_src {
                command.arg("/purge");
            }
        }

        if let Some(n) = self.only_copy_top_n_levels {
            command.arg(format!("/lev:{}", n));
        }

        if self.structure_and_size_zero_files_only {
            command.arg("/create");
        }

        if let Some(properties) = self.copy_file_properties {
            command.arg(properties.as_os_string());
        }
        if let Some(properties) = self.copy_dir_properties {
            command.arg(properties.as_os_string());
        }
        
        if let Some(filter) = &self.filter {
            filter.as_os_string_vec().into_iter().for_each(|arg| {command.arg(arg);});
        }
        if let Some(options) = &self.filesystem_options {
            options.as_os_string_vec().into_iter().for_each(|arg| {command.arg(arg);});
        }        
        if let Some(options) = &self.performance_options {
            options.as_os_string_vec().into_iter().for_each(|arg| {command.arg(arg);});
        }        
        if let Some(settings) = &self.retry_settings {
            settings.as_os_string_vec().into_iter().for_each(|arg| {command.arg(arg);});
        }

        if let Some(logging) = &self.logging {
            command.arg(logging.as_os_string());
        }

        if let Some(mv) = &self.mv {
            command.arg(mv.as_os_string());
        }
       
        if let Some(actions) = &self.post_copy_actions {
            actions.as_os_string_vec().into_iter().for_each(|arg| {command.arg(arg);});
        }

        let exit_code = command.status().expect("failed to execute robocopy")
            .code().expect("Process terminated by signal") as i8;
        
        OkExitCode::try_from(exit_code)
    }
}


pub mod filter;
pub mod performance;
pub mod logging;

use std::{convert::{TryFrom, TryInto}, ffi::OsStr, fs::File, io::{Read, Seek}, ops::{Add, AddAssign}, path::Path, process::Command};
use filter::FileAndDirectoryFilter;
use performance::PerformanceOptions;
use logging::LoggingOptions;

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

// impl AsRef<OsStr> for FileProperties {
//     fn as_ref(&self) -> &OsStr {
//         match self {
            
//         }
//     }
// }

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
}


/// The directory Properties
/// Default is both Data and Attributes
#[derive(Copy, Clone)]
pub enum DirectoryProperties {
    DATA,
    ATTRIBUTES,
    TIME_STAMPS,
    NTFS_ACCESS_CONTROL_LIST,
    OWNER_INFO,
    AUDITING_INFO,
    ALL,
    _MULTIPLE([bool; 6])
}

impl Add for DirectoryProperties {
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
            Self::_MULTIPLE(props)
             => result_props = result_props.iter().zip(props.iter()).map(|(a, b)| *a && *b).collect::<Vec<bool>>().try_into().unwrap(),
            Self::ALL => (),
            prop => result_props[prop.index().unwrap()] = true
        }

        Self::_MULTIPLE(result_props)
    }
}

impl DirectoryProperties {
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
    
    pub fn single_properties(&self) -> Vec<DirectoryProperties> {
        match self {
            Self::_MULTIPLE(props) => {
                Self::VARIANTS.iter().zip(props.iter()).filter(|(_, exists)| **exists).into_iter().unzip::<&Self, &bool, Vec<Self>, Vec<bool>>().0
            },
            Self::ALL => Self::VARIANTS.to_vec(),
            prop => vec![*prop],
        }
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
}


pub enum CopyMode {
    RESTARTABLE_MODE,
    BACKUP_MODE,
    RESTARTABLE_MODE_BACKUP_MODE_FALLBACK
}


pub enum Move {
    FILES,
    FILES_AND_DIRS,
}


pub enum PostCopyActions {
    ADD_ATTRIBS_TO_FILES(FileAttributes),
    RMV_ATTRIBS_FROM_FILES(FileAttributes),
    RMV_FILES_AND_DIRS_NOT_IN_SRC,
    _MULTIPLE(bool, FileAttributes, FileAttributes)
}


pub enum CopySubdirectory {
    All,
    N(usize),
}


pub enum FilesystemOptions {
    FAT_FILE_NAMES,
    ASSUME_FAT_FILE_TIMES,
    DISSABLE_LONG_PATHS,
    _MULTIPLE([bool; 3])
}


pub struct RetrySettings {
    pub specify_retries_failed_copies: Option<usize>, // default 1 million set in registry
    pub specify_wait_between_retries: Option<usize>, // default 30 seconds set in registry
    pub save_specifications: bool,
    
    pub await_share_names_def: bool,
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
    
    pub copy_mode: CopyMode,
    pub unbuffered: bool,

    pub empty_dir_copy: bool,
    pub subdir_copy: Option<CopySubdirectory>,
    pub structure_and_size_zero_files_only: bool,
    
    pub copy_dir_properties: Option<DirectoryProperties>,
    pub copy_file_properties: Option<DirectoryProperties>,

    pub file_and_directory_filter: Option<FileAndDirectoryFilter<'a>>,

    pub filesystem_options: Option<FilesystemOptions>,
    pub logging: Option<LoggingOptions<'a>>,
    pub performance_options: Option<PerformanceOptions>,
    pub retry_settings: Option<RetrySettings>,
    
    pub mv: Option<Move>,
    pub post_copy_actions: Option<PostCopyActions>,

    /// To use this option empty_dir_copy and PostCopyAction::RMV_FILES_AND_DIRS_NOT_IN_SRC must also be in use
    pub overwrite_destination_dir_sec_settings_when_mirror: bool,
    // todo fix secfix and timfix
    // todo job options
}

impl<'a> RobocopyCommand<'a> {
    pub fn new_quick(
        source: &'a Path, 
        destination: &'a Path, 
        files: Vec<File>, 
        mode: CopyMode, 
        buffered: bool, 
        copy_empty_dirs: bool, ){
        
    }
    
    pub fn execute(&self) -> Result<OkExitCode, Result<ErrExitCode, (&'static str, i8)>>{
        let mut command = Command::new("robocopy");
        
        command
            .arg(self.source)
            .arg(self.destination);

        self.files.iter().for_each(|file| {command.arg(file);});

        

        let exit_code = command.status().expect("failed to execute robocopy")
            .code().expect("Process terminated by signal") as i8;
        
        OkExitCode::try_from(exit_code)
    }
}


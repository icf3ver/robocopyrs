//! Robocopyrs is a wrapper for the robocopy command in Windows.
//! 
//! ```no_run
//! use robocopyrs::RobocopyCommand;
//! use robocopyrs::CopyMode;
//! use robocopyrs::FileProperties;
//! use robocopyrs::DirectoryProperties;
//! use std::path::Path;
//! 
//! let command = RobocopyCommand {
//!     source: Path::new("./source"),
//!     destination: Path::new("./destination"),
//!     copy_mode: Some(CopyMode::RESTARTABLE_MODE_BACKUP_MODE_FALLBACK),
//!     structure_and_size_zero_files_only: true,
//!     copy_file_properties: Some(FileProperties::all()),
//!     copy_dir_properties: Some(DirectoryProperties::all()),
//!     ..RobocopyCommand::default()
//! };
//! 
//! command.execute()?;
//! ```

pub mod filter;
pub mod performance;
pub mod logging;
pub mod exit_codes;

use std::{convert::{TryFrom, TryInto}, ffi::OsString, ops::Add, path::Path, process::Command};
use exit_codes::{ErrExitCode, OkExitCode};
use filter::Filter;
use performance::{PerformanceOptions, RetrySettings};
use logging::LoggingSettings;

/// For enums that allow for multiple variants to be 
/// joined into a single variant
pub trait MultipleVariant: Sized + Add<Self> {
    /// get each variant in a multiple-variant
    fn single_variants(&self) -> Vec<Self>;
}

/// The file Properties
/// Default is both Data and Attributes
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone)]
pub enum FileProperties {
    DATA,
    ATTRIBUTES,
    TIME_STAMPS,
    NTFS_ACCESS_CONTROL_LIST,
    OWNER_INFO,
    AUDITING_INFO,
    _MULTIPLE([bool; 6]),
}

impl Add for FileProperties {
    type Output = Self;
    
    #[allow(clippy::suspicious_arithmetic_impl)]
    fn add(self, rhs: Self) -> Self::Output {
        let mut result_props = match self {
            Self::_MULTIPLE(props) => props,
            prop => {
                let mut val = 2_u8.pow(prop.index_of().unwrap() as u32) + 2_u8; 
                (0..6).map(|_| { val >>= 1; val == 1 }).collect::<Vec<bool>>().try_into().unwrap()
            }
        };

        match rhs {
            Self::_MULTIPLE(props) => result_props = result_props.iter().zip(props.iter()).map(|(a, b)| *a && *b).collect::<Vec<bool>>().try_into().unwrap(),
            prop => result_props[prop.index_of().unwrap()] = true
        }

        Self::_MULTIPLE(result_props)
    }
}

impl From<&FileProperties> for OsString {
    fn from(fp: &FileProperties) -> Self {
        let full ;
        OsString::from(match fp {
            FileProperties::DATA => "/copy:D",
            FileProperties::ATTRIBUTES => "/copy:A",
            FileProperties::TIME_STAMPS => "/copy:T",
            FileProperties::NTFS_ACCESS_CONTROL_LIST => "/copy:S",
            FileProperties::OWNER_INFO => "/copy:O",
            FileProperties::AUDITING_INFO => "/copy:U",
            FileProperties::_MULTIPLE(props) => {
                let part = ['D', 'A', 'T', 'S', 'O', 'U'].iter().zip(props.iter()).filter(|(_, exists)| **exists).into_iter().unzip::<&char, &bool, String, Vec<bool>>().0;
                full = String::from("/copy:") + part.as_str();
                full.as_str()
            }
        })
    }
}
impl From<FileProperties> for OsString {
    fn from(fp: FileProperties) -> Self {
        (&fp).into()
    }
}

impl MultipleVariant for FileProperties {
    fn single_variants(&self) -> Vec<Self> {
        match self {
            Self::_MULTIPLE(props) => {
                Self::VARIANTS.iter().zip(props.iter()).filter(|(_, exists)| **exists).into_iter().unzip::<&Self, &bool, Vec<Self>, Vec<bool>>().0
            },
            prop => vec![*prop],
        }
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

    fn index_of(&self) -> Option<usize>{
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

    /// Returns a variant containing all available file properties.
    #[allow(unused)]
    pub fn all() -> Self {
        Self::_MULTIPLE([true; 6])
    }

    /// Returns a variant containing no file properties.
    #[allow(unused)]
    pub fn none() -> Self {
        Self::_MULTIPLE([false; 6])
    }
}


/// The directory Properties
/// Default is both Data and Attributes
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone)]
pub enum DirectoryProperties {
    DATA,
    ATTRIBUTES,
    TIME_STAMPS,
    _MULTIPLE([bool; 3])
}

impl Add for DirectoryProperties {
    type Output = Self;
    
    #[allow(clippy::suspicious_arithmetic_impl)]
    fn add(self, rhs: Self) -> Self::Output {
        let mut result_props = match self {
            Self::_MULTIPLE(props) => props,
            prop => {
                let mut val = 2_u8.pow(prop.index_of().unwrap() as u32) + 2_u8; 
                (0..3).map(|_| { val >>= 1; val == 1 }).collect::<Vec<bool>>().try_into().unwrap()
            }
        };

        match rhs {
            Self::_MULTIPLE(props) => result_props = result_props.iter().zip(props.iter()).map(|(a, b)| *a && *b).collect::<Vec<bool>>().try_into().unwrap(),
            prop => result_props[prop.index_of().unwrap()] = true
        }

        Self::_MULTIPLE(result_props)
    }
}

impl From<&DirectoryProperties> for OsString {
    fn from(dp: &DirectoryProperties) -> Self {
        let full ;
        OsString::from(match dp {
            DirectoryProperties::DATA => "/dcopy:D",
            DirectoryProperties::ATTRIBUTES => "/dcopy:A",
            DirectoryProperties::TIME_STAMPS => "/dcopy:T",
            DirectoryProperties::_MULTIPLE(props) => {
                let part = ['D', 'A', 'T'].iter().zip(props.iter()).filter(|(_, exists)| **exists).into_iter().unzip::<&char, &bool, String, Vec<bool>>().0;
                full = String::from("/dcopy:") + part.as_str();
                full.as_str()
            }
        })
    }
}
impl From<DirectoryProperties> for OsString {
    fn from(dp: DirectoryProperties) -> Self {
        (&dp).into()
    }
}

impl MultipleVariant for DirectoryProperties {
    fn single_variants(&self) -> Vec<Self> {
        match self {
            Self::_MULTIPLE(props) => {
                Self::VARIANTS.iter().zip(props.iter()).filter(|(_, exists)| **exists).into_iter().unzip::<&Self, &bool, Vec<Self>, Vec<bool>>().0
            },
            prop => vec![*prop],
        }
    }
}

impl DirectoryProperties {
    const VARIANTS: [Self; 3] = [
        Self::DATA,
        Self::ATTRIBUTES,
        Self::TIME_STAMPS,
    ];

    fn index_of(&self) -> Option<usize>{
        match self {
            Self::DATA => Some(0),
            Self::ATTRIBUTES => Some(1),
            Self::TIME_STAMPS => Some(2),
            _ => None,
        }
    }

    /// Returns a variant containing all available directory properties.
    #[allow(unused)]
    pub fn all() -> Self {
        Self::_MULTIPLE([true; 3])
    }

    /// Returns a variant containing no directory properties.
    #[allow(unused)]
    pub fn none() -> Self {
        Self::_MULTIPLE([false; 3])
    }
}


#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone)]
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
    
    #[allow(clippy::suspicious_arithmetic_impl)]
    fn add(self, rhs: Self) -> Self::Output {
        let mut result_attribs = match self {
            Self::_MULTIPLE(attribs) => attribs,
            attrib => {
                let mut val = 2_u8.pow(attrib.index_of().unwrap() as u32) * 2_u8; 
                (0..6).map(|_| { val >>= 1; val == 1 }).collect::<Vec<bool>>().try_into().unwrap()
            }
        };

        match rhs {
            Self::_MULTIPLE(attribs) => result_attribs = result_attribs.iter().zip(attribs.iter()).map(|(a, b)| *a && *b).collect::<Vec<bool>>().try_into().unwrap(),
            attrib => result_attribs[attrib.index_of().unwrap()] = true
        }

        Self::_MULTIPLE(result_attribs)
    }
}

impl From<&FileAttributes> for OsString {
    fn from(fa: &FileAttributes) -> Self {
        let part ;
        OsString::from(match fa {
            FileAttributes::READ_ONLY => "R",
            FileAttributes::ARCHIVE => "A",
            FileAttributes::SYSTEM => "S",
            FileAttributes::HIDDEN => "H",
            FileAttributes::COMPRESSED => "C",
            FileAttributes::NOT_CONTENT_INDEXED => "N",
            FileAttributes::ENCRYPTED => "E",
            FileAttributes::TEMPORARY => "T",
            FileAttributes::_MULTIPLE(props) => {
                part = ['R', 'A', 'S', 'H', 'C', 'N', 'E', 'T'].iter().zip(props.iter()).filter(|(_, exists)| **exists).into_iter().unzip::<&char, &bool, String, Vec<bool>>().0;
                part.as_str()
            }
        })
    }
}
impl From<FileAttributes> for OsString {
    fn from(fa: FileAttributes) -> Self {
        (&fa).into()
    }
}

impl MultipleVariant for FileAttributes {
    fn single_variants(&self) -> Vec<Self> {
        match self {
            Self::_MULTIPLE(attribs) => {
                Self::VARIANTS.iter().zip(attribs.iter()).filter(|(_, exists)| **exists).into_iter().unzip::<&Self, &bool, Vec<Self>, Vec<bool>>().0
            },
            attrib => vec![*attrib],
        }
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

    fn index_of(&self) -> Option<usize>{
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

    /// Returns a variant containing all available file attributes.
    #[allow(unused)]
    pub fn all() -> Self {
        Self::_MULTIPLE([true; 8])
    }

    /// Returns a variant containing no file attributes.
    #[allow(unused)]
    pub fn none() -> Self {
        Self::_MULTIPLE([false; 8])
    }
}


#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy)]
pub enum CopyMode {
    RESTARTABLE_MODE,
    BACKUP_MODE,
    RESTARTABLE_MODE_BACKUP_MODE_FALLBACK
}

impl From<&CopyMode> for OsString {
    fn from(cm: &CopyMode) -> OsString {
        match cm {
            CopyMode::RESTARTABLE_MODE => OsString::from("/z"),
            CopyMode::BACKUP_MODE => OsString::from("/b"),
            CopyMode::RESTARTABLE_MODE_BACKUP_MODE_FALLBACK => OsString::from("/zb"),
        }
    }
}
impl From<CopyMode> for OsString {
    fn from(cm: CopyMode) -> Self {
        (&cm).into()
    }
}


#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy)]
pub enum Move {
    FILES,
    FILES_AND_DIRS,
}

impl From<&Move> for OsString {
    fn from(mv: &Move) -> Self {
        match mv {
            Move::FILES => OsString::from("/mov"),
            Move::FILES_AND_DIRS => OsString::from("/move"),
        }
    }
}
impl From<Move> for OsString {
    fn from(mv: Move) -> Self {
        (&mv).into()
    }
}


#[derive(Debug, Copy, Clone)]
pub enum PostCopyActions {
    AddAttribsToFiles(FileAttributes),
    RmvAttribsFromFiles(FileAttributes),
    _MULTIPLE(FileAttributes, FileAttributes)
}

impl Add for PostCopyActions {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let (mut add_attribs, mut rmv_attribs) = match self {
            Self::_MULTIPLE(add, rmv) => (Some(add), Some(rmv)),
            Self::AddAttribsToFiles(attribs) => (None, Some(attribs)),
            Self::RmvAttribsFromFiles(attribs) => (Some(attribs), None)
        };

        match rhs {
            Self::_MULTIPLE(add, rmv) => {
                if let Some(attribs) = add_attribs {
                    add_attribs = Some(attribs + add);
                }
                if let Some(attribs) = rmv_attribs{
                    rmv_attribs = Some(attribs + rmv);
                }
            },
            Self::AddAttribsToFiles(add) => {
                if let Some(attribs) = add_attribs {
                    add_attribs = Some(attribs + add);
                }
            },
            Self::RmvAttribsFromFiles(rmv) => {
                if let Some(attribs) = rmv_attribs{
                    rmv_attribs = Some(attribs + rmv);
                }
            }
        }

        match (add_attribs, rmv_attribs) {
            (Some(add), Some(rmv)) => Self::_MULTIPLE(add, rmv),
            (None, Some(rmv)) => Self::RmvAttribsFromFiles(rmv),
            (Some(add), None) => Self::AddAttribsToFiles(add),
            (None, None) => panic!("use default rather than PostCopyActions::_MULTIPLE(FileAttributes::none(), FileAttributes::none())")
        }
    }
}

impl From<&PostCopyActions> for Vec<OsString> {
    fn from(pca: &PostCopyActions) -> Self {
        match pca {
            PostCopyActions::AddAttribsToFiles(attribs) => vec![OsString::from(String::from("/a+:") + Into::<OsString>::into(attribs).to_str().unwrap())],
            PostCopyActions::RmvAttribsFromFiles(attribs) => vec![OsString::from(String::from("/a-:") + Into::<OsString>::into(attribs).to_str().unwrap())],
            PostCopyActions::_MULTIPLE(add_attribs, rmv_attribs) => vec![OsString::from(String::from("/a+:") + Into::<OsString>::into(add_attribs).to_str().unwrap()), OsString::from(String::from("/a-:") + Into::<OsString>::into(rmv_attribs).to_str().unwrap())],
        }
    }
}
impl From<PostCopyActions> for Vec<OsString> {
    fn from(pca: PostCopyActions) -> Self {
        (&pca).into()
    }
}

impl MultipleVariant for PostCopyActions {
    fn single_variants(&self) -> Vec<Self> {
        match self {
            Self::_MULTIPLE(add, rmv) => vec![Self::AddAttribsToFiles(*add), Self::RmvAttribsFromFiles(*rmv)],
            variant => vec![*variant]
        }
    }
}


#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy)]
pub enum FilesystemOptions {
    FAT_FILE_NAMES,
    ASSUME_FAT_FILE_TIMES,
    DISABLE_LONG_PATHS,
    _MULTIPLE([bool; 3])
}

impl From<&FilesystemOptions> for Vec<OsString> {
    fn from(fso: &FilesystemOptions) -> Self {
        match fso {
            FilesystemOptions::FAT_FILE_NAMES => vec![OsString::from("/fat")],
            FilesystemOptions::ASSUME_FAT_FILE_TIMES => vec![OsString::from("/fft")],
            FilesystemOptions::DISABLE_LONG_PATHS => vec![OsString::from("/256")],
            FilesystemOptions::_MULTIPLE(options) => ["/fat", "/fft", "/256"].iter().zip(options.iter()).filter(|(_, exists)| **exists).map(|(option, _)| OsString::from(*option)).collect()
        }
    }
}
impl From<FilesystemOptions> for Vec<OsString> {
    fn from(fso: FilesystemOptions) -> Self {
        (&fso).into()
    }
}


/// Robocopy command Wrapper
/// 
#[derive(Debug, Clone)]
pub struct RobocopyCommand<'a> {
    pub source: &'a Path,
    pub destination: &'a Path,
    /// wildcard characters are supported
    pub files: Vec<&'a str>,
    
    pub copy_mode: Option<CopyMode>,
    pub unbuffered: bool,

    pub empty_dir_copy: bool,
    pub remove_files_and_dirs_not_in_src: bool,
    pub only_copy_top_n_levels: Option<usize>,
    pub structure_and_size_zero_files_only: bool,
    
    pub copy_file_properties: Option<FileProperties>,
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

impl<'a> Default for RobocopyCommand<'a> {
    fn default() -> Self {
        RobocopyCommand {
            source: Path::new("."),
            destination: Path::new("."),
            files: Vec::new(),
            copy_mode: None,
            unbuffered: false,
            empty_dir_copy: false,
            remove_files_and_dirs_not_in_src: false,
            only_copy_top_n_levels: None,
            structure_and_size_zero_files_only: false,
            copy_file_properties: None,
            copy_dir_properties: None,
            filter: None,
            filesystem_options: None,
            performance_options: None,
            retry_settings: None,
            logging: None,
            mv: None,
            post_copy_actions: None,
            overwrite_destination_dir_sec_settings_when_mirror: false,
        }
    }
}

impl<'a> RobocopyCommand<'a> {
    /// Execute the command
    pub fn execute(&self) -> Result<OkExitCode, Result<ErrExitCode, (&'static str, i8)>>{
        let mut command = Command::new("robocopy");
        
        command
            .arg(self.source)
            .arg(self.destination);

        self.files.iter().for_each(|file| {command.arg(file);});

        if let Some(mode) = &self.copy_mode {
            command.arg(Into::<OsString>::into(mode));
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
            command.arg(Into::<OsString>::into(properties));
        }
        if let Some(properties) = self.copy_dir_properties {
            command.arg(Into::<OsString>::into(properties));
        }
        
        if let Some(filter) = &self.filter {
            Into::<Vec<OsString>>::into(filter).into_iter().for_each(|arg| {command.arg(arg);});
        }
        if let Some(options) = &self.filesystem_options {
            Into::<Vec<OsString>>::into(options).into_iter().for_each(|arg| {command.arg(arg);});
        }        
        if let Some(options) = &self.performance_options {
            Into::<Vec<OsString>>::into(options).into_iter().for_each(|arg| {command.arg(arg);});
        }        
        if let Some(settings) = &self.retry_settings {
            Into::<Vec<OsString>>::into(settings).into_iter().for_each(|arg| {command.arg(arg);});
        }

        if let Some(logging) = &self.logging {
            command.arg(Into::<OsString>::into(logging));
        }

        if let Some(mv) = &self.mv {
            command.arg(Into::<OsString>::into(mv));
        }
       
        if let Some(actions) = &self.post_copy_actions {
            Into::<Vec<OsString>>::into(actions).into_iter().for_each(|arg| {command.arg(arg);});
        }

        let exit_code = command.status().expect("failed to execute robocopy")
            .code().expect("Process terminated by signal") as i8;
        
        OkExitCode::try_from(exit_code)
    }
}


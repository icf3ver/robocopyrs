//! Handle for Robocopy file and directory filter options
//! 
//! All filters and exceptions are handled by the Filter struct

use std::{convert::TryInto, ffi::OsString, ops::Add};
use crate::FileAttributes;
use crate::MultipleVariant;

/// Filters out files that match the variant
#[allow(non_camel_case_types)]
#[derive(Debug, Clone)]
pub enum FileExclusionFilter {
    Attributes(FileAttributes),
    PathOrName(Vec<String>),
    CHANGED,
    OLDER,
    NEWER,
    JUNCTION_POINTS,
    _MULTIPLE(Option<FileAttributes>, Vec<String>, [bool; 4])
}

impl Add for FileExclusionFilter {
    type Output = Self;
    
    fn add(self, rhs: Self) -> Self::Output {
        let (mut result_attribs, mut result_path_or_name, mut result_filters) = match self {
            Self::_MULTIPLE(attribs, path_or_name, filters) => (attribs, path_or_name, filters),
            Self::Attributes(attribs) => (Some(attribs), Vec::new(), [false; 4]),
            Self::PathOrName(path_or_name) => (None, path_or_name, [false; 4]),
            filter => {
                let mut val = 2_u8.pow(filter.index_of().unwrap() as u32) + 2_u8; 
                (None, Vec::new(), (0..6).map(|_| { val >>= 1; val == 1 }).collect::<Vec<bool>>().try_into().unwrap())
            }
        };

        match rhs {
            Self::_MULTIPLE(attribs, mut path_or_name, filters) => {
                result_filters = result_filters.iter().zip(filters.iter()).map(|(a, b)| *a && *b).collect::<Vec<bool>>().try_into().unwrap();
                if let Some(attribs) = attribs {
                    result_attribs = match result_attribs {
                        Some(res_attribs) => Some(attribs + res_attribs),
                        None => Some(attribs)
                    };
                }
                result_path_or_name.append(&mut path_or_name);
            },
            Self::Attributes(attribs) => result_attribs = match result_attribs {
                Some(res_attribs) => Some(attribs + res_attribs),
                None => Some(attribs)
            },
            Self::PathOrName(mut path_or_name) => result_path_or_name.append(&mut path_or_name),
            filter => result_filters[filter.index_of().unwrap()] = true
        }

        Self::_MULTIPLE(result_attribs, result_path_or_name, result_filters)
    }
}

impl MultipleVariant for FileExclusionFilter {
    fn single_variants(&self) -> Vec<Self> {
        match self {
            Self::_MULTIPLE(attribs, path_or_name, props) => {
                let mut filters: Vec<FileExclusionFilter> = Self::VARIANTS.iter().zip(props.iter()).filter(|(_, exists)| **exists).map(|(variant, _)| variant.clone() ).collect();
                
                if let Some(attribs) = attribs {
                    filters.push(Self::Attributes(*attribs));
                }

                if !path_or_name.is_empty() {
                    filters.push(Self::PathOrName(path_or_name.clone()))
                }

                filters
            },
            prop => vec![prop.clone()],
        }
    }
}

impl From<&FileExclusionFilter> for Vec<OsString> {
    fn from(fef: &FileExclusionFilter) -> Self {
        let mut res = Vec::new();
        fef.single_variants().iter().for_each(|filter| match filter {
            FileExclusionFilter::Attributes(file_attributes) => res.push(OsString::from(String::from("/xa:") + Into::<OsString>::into(file_attributes).to_str().unwrap())),
            FileExclusionFilter::PathOrName(path_or_name) => {
                res.push(OsString::from("/xf"));
                path_or_name.iter().for_each(|path_or_name| res.push(OsString::from(path_or_name.as_str())));
            },
            FileExclusionFilter::CHANGED => res.push(OsString::from("/xc")),
            FileExclusionFilter::OLDER => res.push(OsString::from("/xo")),
            FileExclusionFilter::NEWER => res.push(OsString::from("/xn")),
            FileExclusionFilter::JUNCTION_POINTS => res.push(OsString::from("/xjf")),
            _ => unreachable!()
        });
        res
    }
}
impl From<FileExclusionFilter> for Vec<OsString> {
    fn from(fef: FileExclusionFilter) -> Self {
        (&fef).into()
    }
}

impl FileExclusionFilter {
    const VARIANTS: [Self; 4] = [
        Self::CHANGED,
        Self::OLDER,
        Self::NEWER,
        Self::JUNCTION_POINTS
    ];

    fn index_of(&self) -> Option<usize>{
        match self {
            Self::CHANGED => Some(0),
            Self::NEWER => Some(2),
            Self::JUNCTION_POINTS => Some(3),
            _ => None,
        }
    }
}

/// Filters out directories that match the variant
#[allow(non_camel_case_types)]
#[derive(Debug, Clone)]
pub enum DirectoryExclusionFilter {
    PathOrName(Vec<String>),
    JUNCTION_POINTS,
    _BOTH(Vec<String>)
}

impl Add for DirectoryExclusionFilter {
    type Output = Self;
    
    fn add(self, rhs: Self) -> Self::Output {
        let mut junction_pts = false;

        let mut result_path_or_name = match self {
            Self::PathOrName(attribs) | Self::_BOTH(attribs) => attribs,
            Self::JUNCTION_POINTS => { junction_pts = true; Vec::new() }
        };

        match rhs {
            Self::PathOrName(mut attribs) | Self::_BOTH(mut attribs) => result_path_or_name.append(&mut attribs),
            _ => junction_pts = true
        };

        if junction_pts {
            Self::_BOTH(result_path_or_name)
        } else {
            Self::PathOrName(result_path_or_name)
        }
    }
}

impl From<&DirectoryExclusionFilter> for Vec<OsString> {
    fn from(def: &DirectoryExclusionFilter) -> Self {
        let mut res = Vec::new();
        def.single_variants().iter().for_each(|filter| match filter {
            DirectoryExclusionFilter::PathOrName(path_or_name) => {
                res.push(OsString::from("/xd"));
                path_or_name.iter().for_each(|path_or_name| res.push(OsString::from(path_or_name.as_str())));
            },
            DirectoryExclusionFilter::JUNCTION_POINTS => res.push(OsString::from("/xjd")),
            _ => unreachable!()
        });
        res
    }
}
impl From<DirectoryExclusionFilter> for Vec<OsString> {
    fn from(def: DirectoryExclusionFilter) -> Self {
        (&def).into()
    }
}

impl MultipleVariant for DirectoryExclusionFilter {
    fn single_variants(&self) -> Vec<Self> {
        match self {
            Self::_BOTH(path_or_name) => vec![Self::JUNCTION_POINTS, Self::PathOrName(path_or_name.clone())],
            Self::JUNCTION_POINTS => vec![Self::JUNCTION_POINTS],
            Self::PathOrName(path_or_name) => vec![Self::PathOrName(path_or_name.clone())]
        }
    }
}


/// Filters out files and directories that match the variant
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone)]
pub enum FileAndDirectoryExclusionFilter {
    EXTRA,
    LONELY,
    JUNCTION_POINTS,
    _MULTIPLE([bool; 3])
}

impl Add for FileAndDirectoryExclusionFilter {
    type Output = Self;
    
    #[allow(clippy::suspicious_arithmetic_impl)]
    fn add(self, rhs: Self) -> Self::Output {
        let mut result_filters = match self {
            Self::_MULTIPLE(filters) => filters,
            filter => {
                let mut val = 2_u8.pow(filter.index_of().unwrap() as u32) + 2_u8; 
                (0..6).map(|_| { val >>= 1; val == 1 }).collect::<Vec<bool>>().try_into().unwrap()
            }
        };

        match rhs {
            Self::_MULTIPLE(filters) => result_filters = result_filters.iter().zip(filters.iter()).map(|(a, b)| *a && *b).collect::<Vec<bool>>().try_into().unwrap(),
            filter => result_filters[filter.index_of().unwrap()] = true
        }

        Self::_MULTIPLE(result_filters)
    }
}

impl From<&FileAndDirectoryExclusionFilter> for Vec<OsString> {
    fn from(fadef: &FileAndDirectoryExclusionFilter) -> Self {
        let mut res = Vec::new();
        fadef.single_variants().iter().for_each(|filter| match filter {
            FileAndDirectoryExclusionFilter::EXTRA => res.push(OsString::from("/xx")),
            FileAndDirectoryExclusionFilter::LONELY => res.push(OsString::from("/xl")),
            FileAndDirectoryExclusionFilter::JUNCTION_POINTS => res.push(OsString::from("/xj")),
            _ => unreachable!()
        });
        res
    }
}
impl From<FileAndDirectoryExclusionFilter> for Vec<OsString> {
    fn from(fadef: FileAndDirectoryExclusionFilter) -> Self {
        (&fadef).into()
    }
}

impl MultipleVariant for FileAndDirectoryExclusionFilter {
    fn single_variants(&self) -> Vec<Self> {
        match self {
            Self::_MULTIPLE(filters) => {
                Self::VARIANTS.iter().zip(filters.iter()).filter(|(_, exists)| **exists).into_iter().unzip::<&Self, &bool, Vec<Self>, Vec<bool>>().0
            },
            attrib => vec![*attrib],
        }
    }
}

impl FileAndDirectoryExclusionFilter {
    const VARIANTS: [Self; 3] = [
        Self::EXTRA,
        Self::LONELY,
        Self::JUNCTION_POINTS
    ];

    fn index_of(&self) -> Option<usize>{
        match self {
            Self::EXTRA => Some(0),
            Self::LONELY => Some(1),
            Self::JUNCTION_POINTS => Some(2),
            _ => None,
        }
    }
}

/// Includes files despite the filters that match the variant
#[derive(Debug, Copy, Clone)]
pub enum FileExclusionFilterException {
    MODIFIED,
    SAME,
    TWEAKED,
    _MULTIPLE([bool; 3])
}

impl Add for FileExclusionFilterException {
    type Output = Self;
    
    #[allow(clippy::suspicious_arithmetic_impl)]
    fn add(self, rhs: Self) -> Self::Output {
        let mut result_filters = match self {
            Self::_MULTIPLE(filters) => filters,
            filter => {
                let mut val = 2_u8.pow(filter.index_of().unwrap() as u32) + 2_u8; 
                (0..6).map(|_| { val >>= 1; val == 1 }).collect::<Vec<bool>>().try_into().unwrap()
            }
        };

        match rhs {
            Self::_MULTIPLE(filters) => result_filters = result_filters.iter().zip(filters.iter()).map(|(a, b)| *a && *b).collect::<Vec<bool>>().try_into().unwrap(),
            filter => result_filters[filter.index_of().unwrap()] = true
        }

        Self::_MULTIPLE(result_filters)
    }
}

impl From<&FileExclusionFilterException> for Vec<OsString> {
    fn from(fefe: &FileExclusionFilterException) -> Self {
        let mut res = Vec::new();
        fefe.single_variants().iter().for_each(|filter| match filter {
            FileExclusionFilterException::MODIFIED => res.push(OsString::from("/im")),
            FileExclusionFilterException::SAME => res.push(OsString::from("/is")),
            FileExclusionFilterException::TWEAKED => res.push(OsString::from("/it")),
            _ => unreachable!()
        });
        res
    }
}
impl From<FileExclusionFilterException> for Vec<OsString> {
    fn from(fefe: FileExclusionFilterException) -> Self {
        (&fefe).into()
    }
}

impl MultipleVariant for FileExclusionFilterException {
    fn single_variants(&self) -> Vec<Self> {
        match self {
            Self::_MULTIPLE(filters) => {
                Self::VARIANTS.iter().zip(filters.iter()).filter(|(_, exists)| **exists).into_iter().unzip::<&Self, &bool, Vec<Self>, Vec<bool>>().0
            },
            attrib => vec![*attrib],
        }
    }
}

impl FileExclusionFilterException {
    const VARIANTS: [Self; 3] = [
        Self::MODIFIED,
        Self::SAME,
        Self::TWEAKED
    ];

    /// Returns the index of the variant in a 
    /// FileExclusionFilterException::_MULTIPLE variant
    /// and the Self::VARIANTS array
    fn index_of(&self) -> Option<usize>{
        match self {
            Self::MODIFIED => Some(0),
            Self::SAME => Some(1),
            Self::TWEAKED => Some(2),
            _ => None,
        }
    }
}

/// Handles all filter attributes supported by Robocopy
#[derive(Debug, Clone, Default)]
pub struct Filter<'a> {
    pub handle_archive_and_reset: bool,
    pub include_only_files_with_any_of_these_attribs: Option<FileAttributes>,
    
    pub file_exclusion_filter: Option<FileExclusionFilter>,
    pub directory_exclusion_filter: Option<DirectoryExclusionFilter>,
    pub file_and_directory_exclusion_filter: Option<FileAndDirectoryExclusionFilter>,

    pub file_exclusion_filter_exceptions: Option<FileExclusionFilterException>,
    
    pub max_size: Option<u128>,
    pub min_size: Option<u128>,

    pub max_age: Option<&'a str>,
    pub min_age: Option<&'a str>,
    
    pub max_last_access_date: Option<&'a str>,
    pub min_last_access_date: Option<&'a str>,
}

impl<'a> From<&'a Filter<'a>> for Vec<OsString> {
    fn from(filter: &'a Filter<'a>) -> Self {
        let mut res = Vec::new();
        
        if filter.handle_archive_and_reset {
            res.push(OsString::from("/m"));
        }
        if let Some(attribs) = filter.include_only_files_with_any_of_these_attribs {
            res.push(OsString::from(String::from("/ia:") + Into::<OsString>::into(attribs).to_str().unwrap()));
        }

        if let Some(filter) = filter.file_exclusion_filter.clone() {
            res.append(&mut filter.into());
        }
        if let Some(filter) = filter.directory_exclusion_filter.clone() {
            res.append(&mut filter.into());
        }
        if let Some(filter) = filter.file_and_directory_exclusion_filter {
            res.append(&mut filter.into());
        }

        if let Some(filter) = filter.file_exclusion_filter_exceptions {
            res.append(&mut filter.into());
        }

        if let Some(max_size) = filter.max_size {
            res.push(OsString::from(format!("/max:{}", max_size)));
        }
        if let Some(min_size) = filter.min_size {
            res.push(OsString::from(format!("/min:{}", min_size)));
        }
        
        if let Some(max_age) = filter.max_age {
            res.push(OsString::from(format!("/maxage:{}", max_age)));
        }
        if let Some(min_age) = filter.min_age {
            res.push(OsString::from(format!("/minage:{}", min_age)));
        }

        if let Some(max_lad) = filter.max_last_access_date {
            res.push(OsString::from(format!("/maxlad:{}", max_lad)));
        }
        if let Some(min_lad) = filter.min_last_access_date {
            res.push(OsString::from(format!("/minlad:{}", min_lad)));
        }

        res
    }
}
impl<'a> From<Filter<'a>> for Vec<OsString> {
    fn from(filter: Filter<'a>) -> Self {
        (&filter).into()
    }
}
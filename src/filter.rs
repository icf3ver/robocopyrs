use std::{convert::TryInto, ops::Add};
use crate::FileAttributes;

pub enum FileExclusionFilter {
    Attributes(FileAttributes),
    PATH_OR_NAME(Vec<String>),
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
            Self::PATH_OR_NAME(path_or_name) => (None, path_or_name, [false; 4]),
            filter => {
                let mut val = 2_u8.pow(filter.index().unwrap() as u32) + 2_u8; 
                (None, Vec::new(), (0..6).map(|_| { val = val >> 1; val == 1 }).collect::<Vec<bool>>().try_into().unwrap())
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
            Self::PATH_OR_NAME(mut path_or_name) => result_path_or_name.append(&mut path_or_name),
            filter => result_filters[filter.index().unwrap()] = true
        }

        Self::_MULTIPLE(result_attribs, result_path_or_name, result_filters)
    }
}

impl FileExclusionFilter {
    fn index(&self) -> Option<usize>{
        match self {
            Self::CHANGED => Some(0),
            Self::OLDER => Some(1),
            Self::NEWER => Some(2),
            Self::JUNCTION_POINTS => Some(3),
            _ => None,
        }
    }
}


pub enum DirectoryExclusionFilter {
    PATH_OR_NAME(Vec<String>),
    JUNCTION_POINTS,
    _BOTH(Vec<String>)
}


impl Add for DirectoryExclusionFilter {
    type Output = Self;
    
    fn add(self, rhs: Self) -> Self::Output {
        let mut junction_pts = false;

        let mut result_path_or_name = match self {
            Self::PATH_OR_NAME(attribs) | Self::_BOTH(attribs) => attribs,
            Self::JUNCTION_POINTS => { junction_pts = true; Vec::new() }
        };

        match rhs {
            Self::PATH_OR_NAME(mut attribs) | Self::_BOTH(mut attribs) => result_path_or_name.append(&mut attribs),
            _ => junction_pts = true
        };

        if junction_pts {
            Self::_BOTH(result_path_or_name)
        } else {
            Self::PATH_OR_NAME(result_path_or_name)
        }
    }
}


pub enum FileAndDirectoryExclusionFilter {
    EXTRA,
    LONELY,
    JUNCTION_POINTS,
    _MULTIPLE([bool; 3])
}

impl Add for FileAndDirectoryExclusionFilter {
    type Output = Self;
    
    fn add(self, rhs: Self) -> Self::Output {
        let mut result_filters = match self {
            Self::_MULTIPLE(filters) => filters,
            filter => {
                let mut val = 2_u8.pow(filter.index().unwrap() as u32) + 2_u8; 
                (0..6).map(|_| { val = val >> 1; val == 1 }).collect::<Vec<bool>>().try_into().unwrap()
            }
        };

        match rhs {
            Self::_MULTIPLE(filters) => result_filters = result_filters.iter().zip(filters.iter()).map(|(a, b)| *a && *b).collect::<Vec<bool>>().try_into().unwrap(),
            filter => result_filters[filter.index().unwrap()] = true
        }

        Self::_MULTIPLE(result_filters)
    }
}

impl FileAndDirectoryExclusionFilter {
    fn index(&self) -> Option<usize>{
        match self {
            Self::EXTRA => Some(0),
            Self::LONELY => Some(1),
            Self::JUNCTION_POINTS => Some(2),
            _ => None,
        }
    }
}


pub enum FileExclusionFilterException {
    MODIFIED,
    SAME,
    TWEAKED,
    _MULTIPLE([bool; 3])
}

impl Add for FileExclusionFilterException {
    type Output = Self;
    
    fn add(self, rhs: Self) -> Self::Output {
        let mut result_filters = match self {
            Self::_MULTIPLE(filters) => filters,
            filter => {
                let mut val = 2_u8.pow(filter.index().unwrap() as u32) + 2_u8; 
                (0..6).map(|_| { val = val >> 1; val == 1 }).collect::<Vec<bool>>().try_into().unwrap()
            }
        };

        match rhs {
            Self::_MULTIPLE(filters) => result_filters = result_filters.iter().zip(filters.iter()).map(|(a, b)| *a && *b).collect::<Vec<bool>>().try_into().unwrap(),
            filter => result_filters[filter.index().unwrap()] = true
        }

        Self::_MULTIPLE(result_filters)
    }
}

impl FileExclusionFilterException {
    fn index(&self) -> Option<usize>{
        match self {
            Self::MODIFIED => Some(0),
            Self::SAME => Some(1),
            Self::TWEAKED => Some(2),
            _ => None,
        }
    }
}


pub struct FileAndDirectoryFilter<'a> {
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
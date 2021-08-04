//! Performance options

use std::{convert::TryInto, ffi::OsString, ops::Add};

use crate::MultipleVariant;

/// Only one Performance choice can be chosen
#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum PerformanceChoice {
    Threads(u8), // max 128
    InterPacketGap(usize), // todo max
    Default, // Threads thread, how many (case None = default) or how big the gap
            // when adding this variant implies usage of the other variant
}

impl PerformanceChoice {
    fn as_os_string(&self) -> Option<OsString> {
        match self {
            Self::Threads(threads) => Some(OsString::from(format!("/MT:{}", threads))),
            Self::InterPacketGap(gap) => Some(OsString::from(format!("/ipg:{}", gap))),
            _ => None
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone)]
pub enum PerformanceOptions {
    PerformanceChoiceOnly(PerformanceChoice),
    
    DONT_OFFLOAD(PerformanceChoice),
    REQUEST_NETWORK_COMPRESSION(PerformanceChoice),
    COPY_RATHER_THAN_FOLLOW_LINK(PerformanceChoice),

    _MULTIPLE([bool; 3], PerformanceChoice),
    
    Default // Default Threads only
}

impl Add for PerformanceOptions {
    type Output = Result<Self, &'static str>;
    
    fn add(self, rhs: Self) -> Self::Output {
        let mut perf_choice ;

        let mut result_filters = match self {
            Self::_MULTIPLE(filters, choice) => {
                perf_choice = choice;
                filters
            },
            filter => {
                perf_choice = filter.performance_choice();
                if let Some(index) = filter.index_of() {
                    let mut val = 2_u8.pow(index as u32) + 2_u8; 
                    (0..6).map(|_| { val >>= 1; val == 1 }).collect::<Vec<bool>>().try_into().unwrap()    
                } else {
                    [false; 3]
                }
            }
        };

        match rhs {
            Self::_MULTIPLE(filters, choice) => {
                if choice != perf_choice {
                    if perf_choice == PerformanceChoice::Default {
                        perf_choice = choice;
                    } else if choice != PerformanceChoice::Default {
                        return Err("Performance choices do not match.");
                    }
                }
                result_filters = result_filters.iter().zip(filters.iter()).map(|(a, b)| *a && *b).collect::<Vec<bool>>().try_into().unwrap()
            },
            filter => {
                let rhs_perf_choice = filter.performance_choice();
                
                if rhs_perf_choice != perf_choice {
                    if perf_choice == PerformanceChoice::Default {
                        perf_choice = rhs_perf_choice;
                    } else if rhs_perf_choice != PerformanceChoice::Default {
                        return Err("Performance choices do not match.");
                    }
                }

                if let Some(index) = filter.index_of() {
                    result_filters[index] = true; 
                }
            }
        }

        Ok(Self::_MULTIPLE(result_filters, perf_choice))
    }
}

impl From<&PerformanceOptions> for Vec<OsString> {
    fn from(po: &PerformanceOptions) -> Self {
        let mut res = match po.performance_choice().as_os_string() {
            Some(os_string) => vec![os_string],
            None => Vec::new()
        };

        po.single_variants().iter().for_each(|filter| match filter {
            PerformanceOptions::DONT_OFFLOAD(_) => res.push(OsString::from("/nooffload")),
            PerformanceOptions::REQUEST_NETWORK_COMPRESSION(_) => res.push(OsString::from("/compress")),
            PerformanceOptions::COPY_RATHER_THAN_FOLLOW_LINK(_) => res.push(OsString::from("/sl")),
            PerformanceOptions::PerformanceChoiceOnly(_) | PerformanceOptions::Default => (),
            _ => unreachable!()
        });

        res
    }
}
impl From<PerformanceOptions> for Vec<OsString> {
    fn from(po: PerformanceOptions) -> Self {
        (&po).into()
    }
}

impl MultipleVariant for PerformanceOptions {
    fn single_variants(&self) -> Vec<Self> {
        match self {
            Self::_MULTIPLE(filters, choice) => {
                let variants: [Self; 3] = [
                    Self::DONT_OFFLOAD(*choice),
                    Self::REQUEST_NETWORK_COMPRESSION(*choice),
                    Self::COPY_RATHER_THAN_FOLLOW_LINK(*choice),
                ];

                variants.iter().zip(filters.iter()).filter(|(_, exists)| **exists).into_iter().unzip::<&Self, &bool, Vec<Self>, Vec<bool>>().0
            },
            attrib => vec![*attrib],
        }
    }
}

impl PerformanceOptions {
    fn index_of(&self) -> Option<usize>{
        match self {
            Self::DONT_OFFLOAD(_) => Some(0),
            Self::REQUEST_NETWORK_COMPRESSION(_) => Some(1),
            Self::COPY_RATHER_THAN_FOLLOW_LINK(_) => Some(2),
            _ => None,
        }
    }

    pub fn performance_choice(&self) -> PerformanceChoice {
        match self {
            Self::PerformanceChoiceOnly(choice) | 
            Self::DONT_OFFLOAD(choice) | 
            Self::REQUEST_NETWORK_COMPRESSION(choice) |
            Self::COPY_RATHER_THAN_FOLLOW_LINK(choice) |
            Self::_MULTIPLE(_, choice) => *choice,
            Self::Default => PerformanceChoice::Default
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct RetrySettings {
    pub specify_retries_failed_copies: Option<usize>, // default 1 million set in registry
    pub specify_wait_between_retries: Option<usize>, // default 30 seconds set in registry
    pub save_specifications: bool,
    
    pub await_share_names_def: bool,
}

impl From<&RetrySettings> for Vec<OsString> {
    fn from(rs: &RetrySettings) -> Self {
        let mut result = Vec::new();

        if let Some(specified) = rs.specify_retries_failed_copies {
            result.push(OsString::from(format!("/r:{}", specified)))
        }
        if let Some(specified) = rs.specify_wait_between_retries {
            result.push(OsString::from(format!("/w:{}", specified)))
        }
        if rs.save_specifications {
            result.push(OsString::from("/reg"))
        }
        if rs.await_share_names_def {
            result.push(OsString::from("/tbd"))
        }

        result
    }
}
impl From<RetrySettings> for Vec<OsString> {
    fn from(rs: RetrySettings) -> Self {
        (&rs).into()
    }
}

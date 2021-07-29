use std::{convert::TryInto, ops::Add};

/// Only one
#[derive(PartialEq, Copy, Clone)]
pub enum PerformanceChoice {
    Threads(u8), // max 128
    InterPacketGap(usize), // todo max
    Default, // Threads thread, how many (case None = default) or how big the gap
            // when adding this variant implies usage of the other variant
}

pub enum PerformanceOptions {
    PERFORMANCE_CHOICE_ONLY(PerformanceChoice),
    
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
                if let Some(index) = filter.index() {
                    let mut val = 2_u8.pow(index as u32) + 2_u8; 
                    (0..6).map(|_| { val = val >> 1; val == 1 }).collect::<Vec<bool>>().try_into().unwrap()    
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

                if let Some(index) = filter.index() {
                    result_filters[filter.index().unwrap()] = true; 
                }
            }
        }

        Ok(Self::_MULTIPLE(result_filters, perf_choice))
    }
}

impl PerformanceOptions {
    fn index(&self) -> Option<usize>{
        match self {
            Self::DONT_OFFLOAD(_) => Some(0),
            Self::REQUEST_NETWORK_COMPRESSION(_) => Some(1),
            Self::COPY_RATHER_THAN_FOLLOW_LINK(_) => Some(2),
            _ => None,
        }
    }

    fn performance_choice(&self) -> PerformanceChoice {
        match self {
            Self::PERFORMANCE_CHOICE_ONLY(choice) | 
            Self::DONT_OFFLOAD(choice) | 
            Self::REQUEST_NETWORK_COMPRESSION(choice) |
            Self::COPY_RATHER_THAN_FOLLOW_LINK(choice) |
            Self::_MULTIPLE(_, choice) => *choice,
            Self::Default => PerformanceChoice::Default
        }
    }
}
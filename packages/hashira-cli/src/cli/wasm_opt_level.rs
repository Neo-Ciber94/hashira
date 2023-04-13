use std::{fmt::{Display, self}, str::FromStr};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum WasmOptimizationLevel {
    Size,
    SizeAggressive,
    Level0,
    Level1,
    Level2,
    Level3,
    Level4,
}

impl FromStr for WasmOptimizationLevel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "s" => Ok(WasmOptimizationLevel::Size),
            "z" => Ok(WasmOptimizationLevel::SizeAggressive),
            "0" => Ok(WasmOptimizationLevel::Level0),
            "1" => Ok(WasmOptimizationLevel::Level1),
            "2" => Ok(WasmOptimizationLevel::Level2),
            "3" => Ok(WasmOptimizationLevel::Level3),
            "4" => Ok(WasmOptimizationLevel::Level4),
            _ => Err(format!("Invalid wasm optimization level: {s}")),
        }
    }
}

impl Display for WasmOptimizationLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WasmOptimizationLevel::Size => write!(f, "-Os"),
            WasmOptimizationLevel::SizeAggressive => write!(f, "-Oz"),
            WasmOptimizationLevel::Level0 => write!(f, "-O0"),
            WasmOptimizationLevel::Level1 => write!(f, "-O1"),
            WasmOptimizationLevel::Level2 => write!(f, "-O2"),
            WasmOptimizationLevel::Level3 => write!(f, "-O3"),
            WasmOptimizationLevel::Level4 => write!(f, "-O4"),
        }
    }
}

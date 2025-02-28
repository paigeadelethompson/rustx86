mod arithmetic;
mod logic;
mod control_flow;
mod data_transfer;
mod io;
mod string;
mod flags;
mod group;
mod interrupt;
mod stack;

// Re-export all instruction implementations
pub(crate) use arithmetic::*;
pub(crate) use logic::*;
pub(crate) use control_flow::*;
pub(crate) use data_transfer::*;
pub(crate) use io::*;
pub(crate) use string::*;
pub(crate) use flags::*;
pub(crate) use group::*;
pub(crate) use interrupt::*;
pub(crate) use stack::*;

use crate::cpu::CPU;
use crate::cpu::flags::Flags;

// Common traits or types used across instruction implementations can go here 
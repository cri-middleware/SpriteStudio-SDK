use flatbuffers::FlatBufferBuilder;
use ssab::ssab::*;

mod common;
mod initialization;
mod preparation;
mod frame_data;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

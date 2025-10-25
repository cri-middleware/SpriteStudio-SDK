use flatbuffers::FlatBufferBuilder;
use ssab::ssab::*;

mod common;
mod frame_data;
mod initialization;
mod preparation;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

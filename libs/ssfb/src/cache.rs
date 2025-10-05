use flatbuffers::{ForwardsUOffset, Vector, WIPOffset};
use crate::ssqb::{SequenceItem, SequenceItemArgs, SequenceItemT, SsSequenceBinaryT};

/*
fn create_cache_sequence_item<'a>(fb: &'a mut flatbuffers::FlatBufferBuilder<'a>, sequence_item_t: &SequenceItemT) -> WIPOffset<SequenceItem<'a>> {
    let mut arg = SequenceItemArgs::default();
    arg.ref_anime_pack = sequence_item_t.ref_anime_pack.as_ref().map(|s| fb.create_shared_string(s.as_ref()));
    arg.ref_anime = sequence_item_t.ref_anime.as_ref().map(|s| fb.create_shared_string(s.as_ref()));
    arg.repeat_count = sequence_item_t.repeat_count;

    SequenceItem::create(fb, &arg)
}

fn create_cache_sequence_item_lsit<'a>(fb: &'a mut flatbuffers::FlatBufferBuilder<'a>, sequence_item_t_list: &Vec<SequenceItemT>) -> WIPOffset<Vector<'a, ForwardsUOffset<SequenceItem<'a>>>> {
    let b = sequence_item_t_list.iter().map(|sequence_item_t| create_cache_sequence_item(fb, sequence_item_t)).collect::<Vec<WIPOffset<SequenceItem>>>();

    fb.create_vector(&b)
}
*/

/*
fn create_cache_sequence_binary(fb: &mut flatbuffers::FlatBufferBuilder, sequence_binary_t: &SsSequenceBinaryT) -> WIPOffset<SsSequenceBinaryT> {
    sequence_binary_t.sequence_list.unwrap().iter().map(|sequence| {
        let a = fb.create_shared_string(&sequence.name);
        sequence.list

    })
}
*/
use flatbuffers::{ForwardsUOffset, Vector, WIPOffset};
use crate::ssqb::{Sequence, SequenceArgs, SequenceItem, SequenceItemArgs, SequenceItemT, SequenceT, SsSequenceBinary, SsSequenceBinaryArgs, SsSequenceBinaryT};

pub fn create_shared_sequence_item_list<'bldr: 'args, 'args: 'mut_bldr, 'mut_bldr, A: flatbuffers::Allocator + 'bldr>(
    fb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr, A>,
    sequence_item_t_list: &Vec<SequenceItemT>
) -> WIPOffset<Vector<'bldr, ForwardsUOffset<SequenceItem<'bldr>>>> {
    let mut sequence_item_offset_vec = Vec::with_capacity(sequence_item_t_list.len());
    for sequence_item_t in sequence_item_t_list {
        let ref_anime_pack = sequence_item_t.ref_anime_pack.as_ref().map(|s| fb.create_shared_string(s.as_ref()));
        let ref_anime = sequence_item_t.ref_anime.as_ref().map(|s| fb.create_shared_string(s.as_ref()));

        let sequence_item_arg = SequenceItemArgs {
            ref_anime_pack,
            ref_anime,
            repeat_count: sequence_item_t.repeat_count,
        };
        let offset = SequenceItem::create(fb, &sequence_item_arg);
        sequence_item_offset_vec.push(offset);
    }

    fb.create_vector(&sequence_item_offset_vec)
}

pub fn create_shared_sequence_binary<'bldr: 'args, 'args: 'mut_bldr, 'mut_bldr, A: flatbuffers::Allocator + 'bldr>(
    fb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr, A>,
    sequence_binary_t: &SsSequenceBinaryT
) -> WIPOffset<SsSequenceBinary<'bldr>> {
    let mut sequence_offset_vec = Vec::new();
    if let Some(sequences) = sequence_binary_t.sequence_list.as_ref() {
        sequence_offset_vec.reserve(sequences.len());

        for sequence in sequences {
            let name = fb.create_shared_string(&sequence.name);
            let list = create_shared_sequence_item_list(fb, sequence.list.as_ref().unwrap());
            let sequence_arg = SequenceArgs {
                name: Some(name),
                index: sequence.index,
                type_: sequence.type_,
                list: Some(list),
            };
            let offset = Sequence::create(fb, &sequence_arg);
            sequence_offset_vec.push(offset);
        }
    }

    let sequence_list = fb.create_vector(&sequence_offset_vec);
    let arg = SsSequenceBinaryArgs {
        sequence_list: Some(sequence_list),
        ..Default::default()
    };

    SsSequenceBinary::create(fb, &arg)
}

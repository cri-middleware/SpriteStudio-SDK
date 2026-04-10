
pub struct SsqbCache {}
impl SsqbCache {
    pub fn new() -> Self {
        Self {}
    }

    fn create_shared_sequence_item_list<'a>(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, sequence_item_t_list: &[ssab::ssqb::ss::format::SequenceItemT]) -> flatbuffers::WIPOffset<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssqb::ss::format::SequenceItem<'a>>>> {
        let v = sequence_item_t_list.iter().map(|sequence_item_t| {
            let ref_anime_pack = sequence_item_t.ref_anime_pack.as_ref().map(|s| fb.create_shared_string(s.as_ref()));
            let ref_anime = sequence_item_t.ref_anime.as_ref().map(|s| fb.create_shared_string(s.as_ref()));

            let sequence_item_arg = ssab::ssqb::ss::format::SequenceItemArgs { ref_anime_pack, ref_anime, repeat_count: sequence_item_t.repeat_count};
            ssab::ssqb::ss::format::SequenceItem::create(fb, &sequence_item_arg)
        }).collect::<Vec<_>>();

        fb.create_vector(&v)
    }
    pub fn create_shared_sequence_binary<'a>(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, sequence_binary_t: &ssab::ssqb::ss::format::SsSequenceBinaryT) -> flatbuffers::WIPOffset<ssab::ssqb::ss::format::SsSequenceBinary<'a>> {
        let mut args = ssab::ssqb::ss::format::SsSequenceBinaryArgs::default();
        args.sequence_list = sequence_binary_t.sequence_list.as_ref().map(|sequence_list| {
            let v = sequence_list.iter().map(|sequence| {
                let mut s_args = ssab::ssqb::ss::format::SequenceArgs::default();
                s_args.name = Some(fb.create_shared_string(&sequence.name));
                s_args.index = sequence.index;
                s_args.type_ = sequence.type_;
                s_args.list = sequence.list.as_ref().map(|list| self.create_shared_sequence_item_list(fb, list));

                ssab::ssqb::ss::format::Sequence::create(fb, &s_args)
            }).collect::<Vec<_>>();
            fb.create_vector(&v)
        });

        ssab::ssqb::ss::format::SsSequenceBinary::create(fb, &args)
    }
}

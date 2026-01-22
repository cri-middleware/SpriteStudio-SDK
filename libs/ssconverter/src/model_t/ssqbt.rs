use crate::model_t::ssqbt;
use sspj::ssqe::{SsSequence, SsSequenceItem};

fn convert_sequence_t_list(sequence_list: &[SsSequence]) -> Option<Vec<ssab::ssqb::SequenceT>> {
    Some(
        sequence_list
            .iter()
            .map(|sequence| {
                let mut sequence_t = ssab::ssqb::SequenceT::default();
                sequence_t.name = sequence.name.clone();
                sequence_t.index = sequence.index;
                sequence_t.type_ = match &sequence.sequence_type {
                    sspj::sstypes::SsSequenceType::Last => ssab::ssqb::SequenceType::Last,
                    sspj::sstypes::SsSequenceType::Keep => ssab::ssqb::SequenceType::Keep,
                    sspj::sstypes::SsSequenceType::Top => ssab::ssqb::SequenceType::Top,
                };
                sequence_t.list = Some(convert_sequence_item_t_list(&sequence.list));

                sequence_t
            })
            .collect::<Vec<ssab::ssqb::SequenceT>>(),
    )
}

fn convert_sequence_item_t_list(sequence_item_list: &[SsSequenceItem], ) -> Vec<ssab::ssqb::SequenceItemT> {
    sequence_item_list
        .iter()
        .map(|sequence_item| {
            let mut sequence_item_t = ssab::ssqb::SequenceItemT::default();
            sequence_item_t.ref_anime = Some(sequence_item.ref_anime.clone());
            sequence_item_t.ref_anime_pack = Some(sequence_item.ref_anime_pack.clone());
            sequence_item_t.repeat_count = sequence_item.repeat_count;

            sequence_item_t
        })
        .collect::<Vec<ssab::ssqb::SequenceItemT>>()
}

pub fn convert_ss_sequence_binary_t(sequence_pack: &sspj::ssqe::SsSequencePack, ) -> Result<ssab::ssqb::SsSequenceBinaryT, crate::converter::converter_error::ConverterError>{
    let mut ss_sequence_binary_t = ssab::ssqb::SsSequenceBinaryT::default();
    ss_sequence_binary_t.sequence_list = ssqbt::convert_sequence_t_list(&sequence_pack.sequence_list);

    Ok(ss_sequence_binary_t)
}

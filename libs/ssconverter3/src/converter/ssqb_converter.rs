use crate::model_t::ssqbt;

pub fn convert_ss_sequence_binary_t(sequence_pack: &sspj::ssqe::SsSequencePack, ) -> ssab::ssqb::SsSequenceBinaryT {
    ssqbt::convert_ss_sequence_binary_t(sequence_pack)
}
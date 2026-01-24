use crate::model_t::ssqbt;

pub fn convert_ss_sequence_binary_t(sequence_pack: &sspj::ssqe::SsSequencePack, ) -> Result<ssab::ssqb::ss::format::SsSequenceBinaryT, crate::converter::converter_error::ConverterError> {
    ssqbt::convert_ss_sequence_binary_t(sequence_pack)
}
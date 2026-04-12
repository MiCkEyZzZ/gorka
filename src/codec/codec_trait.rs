use crate::{BitReader, GorkaError, MilliHz, RawBitWriter};

pub trait DopplerCodec {
    type State;
    type Id;

    fn encode(
        writer: &mut RawBitWriter,
        state: &mut Self::State,
        value: MilliHz,
        id: Self::Id,
    ) -> Result<(), GorkaError>;

    fn decode(
        reader: &mut BitReader,
        state: &mut Self::State,
        id: Self::Id,
    ) -> Result<MilliHz, GorkaError>;
}

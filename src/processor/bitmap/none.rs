use crate::{
    matrix::Bitmap,
    processor::{BitmapProcessor, Result},
};

pub struct NoneProcessor {}

#[allow(dead_code)]
impl NoneProcessor {
    pub fn new() -> Self {
        return NoneProcessor {};
    }
}

impl<const ROW_COUNT: usize, const COL_COUNT: usize> BitmapProcessor<ROW_COUNT, COL_COUNT>
    for NoneProcessor
{
    fn process(&mut self, _: &mut Bitmap<ROW_COUNT, COL_COUNT>) -> Result {
        return Ok(());
    }
}

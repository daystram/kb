use crate::processor::{mapper::Input, InputProcessor, Result};

pub struct NoneProcessor {}

#[allow(dead_code)]
impl NoneProcessor {
    pub fn new() -> Self {
        return NoneProcessor {};
    }
}

impl<const KEY_MATRIX_ROW_COUNT: usize, const KEY_MATRIX_COL_COUNT: usize>
    InputProcessor<KEY_MATRIX_ROW_COUNT, KEY_MATRIX_COL_COUNT> for NoneProcessor
{
    fn process(&mut self, _: &mut Input<KEY_MATRIX_ROW_COUNT, KEY_MATRIX_COL_COUNT>) -> Result {
        return Ok(());
    }
}

use ssd1306::{prelude::*, Ssd1306};

pub struct OLEDDisplay<DI>
where
    DI: WriteOnlyDataCommand,
{
    // TODO: support arbitrary modes
    display: Ssd1306<DI, ssd1306::size::DisplaySize128x32, ssd1306::mode::TerminalMode>,
}

impl<DI> OLEDDisplay<DI>
where
    DI: WriteOnlyDataCommand,
{
    pub fn new(mut display: Ssd1306<DI, DisplaySize128x32, ssd1306::mode::BasicMode>) -> Self {
        display.init().unwrap();

        let mut oled_display = OLEDDisplay {
            display: display.into_terminal_mode(),
        };
        oled_display.clear();
        oled_display
    }

    pub fn clear(&mut self) {
        self.display.clear().unwrap();
    }
}

impl<DI> core::fmt::Write for OLEDDisplay<DI>
where
    DI: WriteOnlyDataCommand,
{
    fn write_str(&mut self, s: &str) -> Result<(), core::fmt::Error> {
        self.display.write_str(s)
    }
}

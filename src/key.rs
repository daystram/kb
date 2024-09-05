use core::mem;
use defmt::Format;
use enum_map::Enum;
use usbd_human_interface_device::page::Keyboard;

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Default, Format)]
pub enum Action<L: LayerIndex> {
    #[default]
    Pass,
    None,
    Key(Key),
    ModifiedKey(ModifiedKey),
    Control(Control),
    LayerModifier(L),
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Format, PartialEq)]
#[repr(u16)]
pub enum Key {
    Escape,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    Insert,
    DeleteForward,
    Grave,
    Keyboard1,
    Keyboard2,
    Keyboard3,
    Keyboard4,
    Keyboard5,
    Keyboard6,
    Keyboard7,
    Keyboard8,
    Keyboard9,
    Keyboard0,
    Minus,
    Equal,
    DeleteBackspace,
    PageUp,
    Tab,
    Q,
    W,
    E,
    R,
    T,
    Y,
    U,
    I,
    O,
    P,
    LeftBrace,
    RightBrace,
    Backslash,
    PageDown,
    CapsLock,
    A,
    S,
    D,
    F,
    G,
    H,
    J,
    K,
    L,
    Semicolon,
    Apostrophe,
    ReturnEnter,
    Home,
    LeftShift,
    Z,
    X,
    C,
    V,
    B,
    N,
    M,
    Comma,
    Dot,
    ForwardSlash,
    RightShift,
    UpArrow,
    End,
    LeftControl,
    LeftAlt,
    LeftGUI,
    Space,
    RightGUI,
    RightAlt,
    RightControl,
    LeftArrow,
    DownArrow,
    RightArrow,
    VolumeUp,
    VolumeDown,
}

impl<L: LayerIndex> From<Key> for Action<L> {
    fn from(from: Key) -> Action<L> {
        Action::Key(from)
    }
}

impl From<Key> for Keyboard {
    fn from(from: Key) -> Keyboard {
        match from {
            Key::Escape => Keyboard::Escape,
            Key::F1 => Keyboard::F1,
            Key::F2 => Keyboard::F2,
            Key::F3 => Keyboard::F3,
            Key::F4 => Keyboard::F4,
            Key::F5 => Keyboard::F5,
            Key::F6 => Keyboard::F6,
            Key::F7 => Keyboard::F7,
            Key::F8 => Keyboard::F8,
            Key::F9 => Keyboard::F9,
            Key::F10 => Keyboard::F10,
            Key::F11 => Keyboard::F11,
            Key::F12 => Keyboard::F12,
            Key::Insert => Keyboard::Insert,
            Key::DeleteForward => Keyboard::DeleteForward,
            Key::Grave => Keyboard::Grave,
            Key::Keyboard1 => Keyboard::Keyboard1,
            Key::Keyboard2 => Keyboard::Keyboard2,
            Key::Keyboard3 => Keyboard::Keyboard3,
            Key::Keyboard4 => Keyboard::Keyboard4,
            Key::Keyboard5 => Keyboard::Keyboard5,
            Key::Keyboard6 => Keyboard::Keyboard6,
            Key::Keyboard7 => Keyboard::Keyboard7,
            Key::Keyboard8 => Keyboard::Keyboard8,
            Key::Keyboard9 => Keyboard::Keyboard9,
            Key::Keyboard0 => Keyboard::Keyboard0,
            Key::Minus => Keyboard::Minus,
            Key::Equal => Keyboard::Equal,
            Key::DeleteBackspace => Keyboard::DeleteBackspace,
            Key::PageUp => Keyboard::PageUp,
            Key::Tab => Keyboard::Tab,
            Key::Q => Keyboard::Q,
            Key::W => Keyboard::W,
            Key::E => Keyboard::E,
            Key::R => Keyboard::R,
            Key::T => Keyboard::T,
            Key::Y => Keyboard::Y,
            Key::U => Keyboard::U,
            Key::I => Keyboard::I,
            Key::O => Keyboard::O,
            Key::P => Keyboard::P,
            Key::LeftBrace => Keyboard::LeftBrace,
            Key::RightBrace => Keyboard::RightBrace,
            Key::Backslash => Keyboard::Backslash,
            Key::PageDown => Keyboard::PageDown,
            Key::CapsLock => Keyboard::CapsLock,
            Key::A => Keyboard::A,
            Key::S => Keyboard::S,
            Key::D => Keyboard::D,
            Key::F => Keyboard::F,
            Key::G => Keyboard::G,
            Key::H => Keyboard::H,
            Key::J => Keyboard::J,
            Key::K => Keyboard::K,
            Key::L => Keyboard::L,
            Key::Semicolon => Keyboard::Semicolon,
            Key::Apostrophe => Keyboard::Apostrophe,
            Key::ReturnEnter => Keyboard::ReturnEnter,
            Key::Home => Keyboard::Home,
            Key::LeftShift => Keyboard::LeftShift,
            Key::Z => Keyboard::Z,
            Key::X => Keyboard::X,
            Key::C => Keyboard::C,
            Key::V => Keyboard::V,
            Key::B => Keyboard::B,
            Key::N => Keyboard::N,
            Key::M => Keyboard::M,
            Key::Comma => Keyboard::Comma,
            Key::Dot => Keyboard::Dot,
            Key::ForwardSlash => Keyboard::ForwardSlash,
            Key::RightShift => Keyboard::RightShift,
            Key::UpArrow => Keyboard::UpArrow,
            Key::End => Keyboard::End,
            Key::LeftControl => Keyboard::LeftControl,
            Key::LeftAlt => Keyboard::LeftAlt,
            Key::LeftGUI => Keyboard::LeftGUI,
            Key::Space => Keyboard::Space,
            Key::RightGUI => Keyboard::RightGUI,
            Key::RightAlt => Keyboard::RightAlt,
            Key::RightControl => Keyboard::RightControl,
            Key::LeftArrow => Keyboard::LeftArrow,
            Key::DownArrow => Keyboard::DownArrow,
            Key::RightArrow => Keyboard::RightArrow,
            Key::VolumeUp => Keyboard::VolumeUp,
            Key::VolumeDown => Keyboard::VolumeDown,
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Default, Format, PartialEq)]
#[repr(u8)]
pub enum Modifier {
    #[default]
    None = 0,
    LeftShift = 1 << 0,
    LeftControl = 1 << 1,
    LeftAlt = 1 << 2,
    LeftGUI = 1 << 3,
    RightShift = 1 << 4,
    RightControl = 1 << 5,
    RightAlt = 1 << 6,
    RightGUI = 1 << 7,
}

impl<L: LayerIndex> From<Modifier> for Action<L> {
    fn from(from: Modifier) -> Self {
        match from {
            Modifier::LeftControl => Action::Key(Key::LeftControl),
            Modifier::LeftShift => Action::Key(Key::LeftShift),
            Modifier::LeftAlt => Action::Key(Key::LeftAlt),
            Modifier::LeftGUI => Action::Key(Key::LeftGUI),
            Modifier::RightControl => Action::Key(Key::RightControl),
            Modifier::RightShift => Action::Key(Key::RightShift),
            Modifier::RightAlt => Action::Key(Key::RightAlt),
            Modifier::RightGUI => Action::Key(Key::RightGUI),
            _ => Action::None,
        }
    }
}

#[derive(Clone, Copy, Debug, Format)]
pub struct ModifiedKey(pub u16);

impl ModifiedKey {
    pub fn get_modifiers(self) -> [Modifier; 8] {
        let mut mods = [Default::default(); 8];
        let modifier_bits = self.0 >> 8;
        mods.iter_mut().enumerate().for_each(|(i, m)| {
            let mask = 1 << i;
            if (modifier_bits & mask) != 0 {
                *m = unsafe { mem::transmute::<u8, Modifier>(mask as u8) };
            }
        });
        mods
    }

    pub fn get_key(self) -> Key {
        unsafe { mem::transmute(self.0 & 0x00FF) }
    }
}

macro_rules! LS {
    ($key:expr) => {
        ModifiedKey($key as u16 | (Modifier::LeftShift as u16) << 8)
    };
}

macro_rules! LC {
    ($key:expr) => {
        ModifiedKey($key as u16 | (Modifier::LeftControl as u16) << 8)
    };
}

macro_rules! LA {
    ($key:expr) => {
        ModifiedKey($key as u16 | (Modifier::LeftAlt as u16) << 8)
    };
}

macro_rules! LG {
    ($key:expr) => {
        ModifiedKey($key as u16 | (Modifier::LeftGUI as u16) << 8)
    };
}

macro_rules! RS {
    ($key:expr) => {
        ModifiedKey($key as u16 | (Modifier::RightShift as u16) << 8)
    };
}

macro_rules! RC {
    ($key:expr) => {
        ModifiedKey($key as u16 | (Modifier::RightControl as u16) << 8)
    };
}

macro_rules! RA {
    ($key:expr) => {
        ModifiedKey($key as u16 | (Modifier::RightAlt as u16) << 8)
    };
}

macro_rules! RG {
    ($key:expr) => {
        ModifiedKey($key as u16 | (Modifier::RightGUI as u16) << 8)
    };
}

#[allow(dead_code, clippy::enum_variant_names)]
#[derive(Clone, Copy, Debug, Format, PartialEq)]
pub enum Control {
    RGBAnimationNext,
    RGBAnimationPrevious,
    RGBSpeedUp,
    RGBSpeedDown,
    RGBBrightnessUp,
    RGBBrightnessDown,
    RGBDirectionToggle,
}

pub trait LayerIndex: Copy + Default + PartialEq + PartialOrd + Enum + Format {}

#[derive(Clone, Copy, Debug, Default, Format, PartialEq)]
pub enum Edge {
    #[default]
    None,
    Rising,
    Falling,
}

impl From<(bool, bool)> for Edge {
    fn from((from, to): (bool, bool)) -> Self {
        if !from && to {
            Edge::Rising
        } else if from && !to {
            Edge::Falling
        } else {
            Edge::None
        }
    }
}

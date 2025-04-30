use cef_dll_sys::cef_cursor_type_t;
use winit::keyboard::KeyCode;

use crate::shared::types::{Cursor, MousePosition};

impl From<MousePosition> for cef::MouseEvent {
    fn from(value: MousePosition) -> Self {
        Self {
            x: value.0,
            y: value.1,
            modifiers: 0,
        }
    }
}

impl From<cef::CursorType> for Cursor {
    fn from(value: cef::CursorType) -> Self {
        match value.as_ref() {
            cef_cursor_type_t::CT_POINTER => Cursor::Default,
            cef_cursor_type_t::CT_HAND => Cursor::Pointer,
            cef_cursor_type_t::CT_IBEAM => Cursor::Text,
            cef_cursor_type_t::CT_MOVE => Cursor::Move,
            cef_cursor_type_t::CT_ZOOMIN => Cursor::ZoomIn,
            cef_cursor_type_t::CT_ZOOMOUT => Cursor::ZoomOut,
            cef_cursor_type_t::CT_WAIT => Cursor::Wait,
            cef_cursor_type_t::CT_NONE => Cursor::None,
            _ => Cursor::Default,
        }
    }
}

pub struct WindowsKeyCode(pub i32);

impl TryFrom<KeyCode> for WindowsKeyCode {
    type Error = &'static str;

    fn try_from(value: KeyCode) -> Result<Self, Self::Error> {
        match value {
            KeyCode::Backspace => Ok(Self(8)),
            KeyCode::Tab => Ok(Self(9)),
            KeyCode::Enter => Ok(Self(13)),
            KeyCode::Space => Ok(Self(32)),
            KeyCode::ArrowLeft => Ok(Self(37)),
            KeyCode::ArrowUp => Ok(Self(38)),
            KeyCode::ArrowRight => Ok(Self(39)),
            KeyCode::ArrowDown => Ok(Self(40)),
            KeyCode::Digit0 => Ok(Self(48)),
            KeyCode::Digit1 => Ok(Self(49)),
            KeyCode::Digit2 => Ok(Self(50)),
            KeyCode::Digit3 => Ok(Self(51)),
            KeyCode::Digit4 => Ok(Self(52)),
            KeyCode::Digit5 => Ok(Self(53)),
            KeyCode::Digit6 => Ok(Self(54)),
            KeyCode::Digit7 => Ok(Self(55)),
            KeyCode::Digit8 => Ok(Self(56)),
            KeyCode::Digit9 => Ok(Self(57)),
            KeyCode::KeyA => Ok(Self(65)),
            KeyCode::KeyC => Ok(Self(67)),
            KeyCode::KeyV => Ok(Self(86)),
            KeyCode::KeyX => Ok(Self(88)),
            _ => Err("Failed to convert KeyCode to WindowsKeyCode"),
        }
    }
}

pub struct NativeKeyCode(pub i32);

impl TryFrom<KeyCode> for NativeKeyCode {
    type Error = &'static str;

    fn try_from(value: KeyCode) -> Result<Self, Self::Error> {
        match value {
            KeyCode::Backspace => Ok(Self(22)),
            KeyCode::Tab => Ok(Self(23)),
            KeyCode::Enter => Ok(Self(36)),
            KeyCode::Space => Ok(Self(65)),
            KeyCode::ArrowLeft => Ok(Self(113)),
            KeyCode::ArrowUp => Ok(Self(111)),
            KeyCode::ArrowRight => Ok(Self(114)),
            KeyCode::ArrowDown => Ok(Self(116)),
            KeyCode::Digit0 => Ok(Self(19)),
            KeyCode::Digit1 => Ok(Self(10)),
            KeyCode::Digit2 => Ok(Self(11)),
            KeyCode::Digit3 => Ok(Self(12)),
            KeyCode::Digit4 => Ok(Self(13)),
            KeyCode::Digit5 => Ok(Self(14)),
            KeyCode::Digit6 => Ok(Self(15)),
            KeyCode::Digit7 => Ok(Self(16)),
            KeyCode::Digit8 => Ok(Self(17)),
            KeyCode::Digit9 => Ok(Self(18)),
            KeyCode::KeyA => Ok(Self(38)),
            KeyCode::KeyC => Ok(Self(54)),
            KeyCode::KeyV => Ok(Self(55)),
            KeyCode::KeyX => Ok(Self(53)),
            _ => Err("Failed to convert KeyCode to NativeKeyCode"),
        }
    }
}

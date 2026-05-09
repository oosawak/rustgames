#![allow(dead_code)]

pub struct Keyboard {
    pub matrix: [u8; 11],
}

impl Keyboard {
    pub fn new() -> Self {
        Keyboard { matrix: [0xFF; 11] }
    }

    pub fn read_row(&self, col: usize) -> u8 {
        if col < 11 { self.matrix[col] } else { 0xFF }
    }

    pub fn key_down(&mut self, code: &str) {
        if let Some((row, bit)) = map_key(code) {
            self.matrix[row] &= !(1 << bit);
        }
    }

    pub fn key_up(&mut self, code: &str) {
        if let Some((row, bit)) = map_key(code) {
            self.matrix[row] |= 1 << bit;
        }
    }
}

fn map_key(code: &str) -> Option<(usize, u8)> {
    match code {
        // Row 0: 7 6 5 4 3 2 1 0
        "Digit7" => Some((0, 7)),
        "Digit6" => Some((0, 6)),
        "Digit5" => Some((0, 5)),
        "Digit4" => Some((0, 4)),
        "Digit3" => Some((0, 3)),
        "Digit2" => Some((0, 2)),
        "Digit1" => Some((0, 1)),
        "Digit0" => Some((0, 0)),
        // Row 1: symbols
        "Semicolon"   => Some((1, 7)),
        "BracketLeft" => Some((1, 6)),
        "Backquote"   => Some((1, 5)),
        "Backslash"   => Some((1, 4)),
        "Equal"       => Some((1, 3)),
        "BracketRight"=> Some((1, 2)),
        "Minus"       => Some((1, 1)),
        // Row 2
        "KeyB"   => Some((2, 7)),
        "KeyA"   => Some((2, 6)),
        "Slash"  => Some((2, 4)),
        "Period" => Some((2, 3)),
        "Comma"  => Some((2, 2)),
        "Digit9" => Some((2, 0)),
        "Digit8" => Some((2, 1)),
        // Row 3: J I H G F E D C
        "KeyJ" => Some((3, 7)),
        "KeyI" => Some((3, 6)),
        "KeyH" => Some((3, 5)),
        "KeyG" => Some((3, 4)),
        "KeyF" => Some((3, 3)),
        "KeyE" => Some((3, 2)),
        "KeyD" => Some((3, 1)),
        "KeyC" => Some((3, 0)),
        // Row 4: R Q P O N M L K
        "KeyR" => Some((4, 7)),
        "KeyQ" => Some((4, 6)),
        "KeyP" => Some((4, 5)),
        "KeyO" => Some((4, 4)),
        "KeyN" => Some((4, 3)),
        "KeyM" => Some((4, 2)),
        "KeyL" => Some((4, 1)),
        "KeyK" => Some((4, 0)),
        // Row 5: Z Y X W V U T S
        "KeyZ" => Some((5, 7)),
        "KeyY" => Some((5, 6)),
        "KeyX" => Some((5, 5)),
        "KeyW" => Some((5, 4)),
        "KeyV" => Some((5, 3)),
        "KeyU" => Some((5, 2)),
        "KeyT" => Some((5, 1)),
        "KeyS" => Some((5, 0)),
        // Row 6: F3 F2 F1 CODE CAP GRAPH CTRL SHIFT
        "F3"                             => Some((6, 7)),
        "F2"                             => Some((6, 6)),
        "F1"                             => Some((6, 5)),
        "CapsLock"                       => Some((6, 3)),
        "ControlLeft" | "ControlRight"   => Some((6, 1)),
        "ShiftLeft"   | "ShiftRight"     => Some((6, 0)),
        // Row 7: RET SEL BS STOP TAB ESC F5 F4
        "Enter"     => Some((7, 7)),
        "Backspace" => Some((7, 5)),
        "Tab"       => Some((7, 3)),
        "Escape"    => Some((7, 2)),
        "F5"        => Some((7, 1)),
        "F4"        => Some((7, 0)),
        // Row 8: RIGHT DOWN UP LEFT DEL INS HOME SPACE
        "ArrowRight" => Some((8, 7)),
        "ArrowDown"  => Some((8, 6)),
        "ArrowUp"    => Some((8, 5)),
        "ArrowLeft"  => Some((8, 4)),
        "Delete"     => Some((8, 3)),
        "Insert"     => Some((8, 2)),
        "Home"       => Some((8, 1)),
        "Space"      => Some((8, 0)),
        // Row 9: numpad
        "Numpad9" => Some((9, 7)),
        "Numpad8" => Some((9, 6)),
        "Numpad7" => Some((9, 5)),
        "Numpad6" => Some((9, 4)),
        "Numpad5" => Some((9, 3)),
        "Numpad4" => Some((9, 2)),
        "Numpad3" => Some((9, 1)),
        "Numpad2" => Some((9, 0)),
        // Row 10
        "Numpad1"        => Some((10, 7)),
        "Numpad0"        => Some((10, 6)),
        "NumpadDecimal"  => Some((10, 5)),
        "NumpadAdd"      => Some((10, 4)),
        "NumpadSubtract" => Some((10, 3)),
        "NumpadMultiply" => Some((10, 2)),
        "NumpadDivide"   => Some((10, 1)),
        _ => None,
    }
}

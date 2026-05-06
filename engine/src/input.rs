use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct MouseState {
    pub x: f32,
    pub y: f32,
    pub left_pressed: bool,
    pub right_pressed: bool,
    pub middle_pressed: bool,
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct KeyboardState {
    pub w: bool,
    pub a: bool,
    pub s: bool,
    pub d: bool,
    pub space: bool,
    pub escape: bool,
    pub enter: bool,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct InputState {
    pub mouse: MouseState,
    pub keyboard: KeyboardState,
    pub scroll_delta: f32,
}

impl InputState {
    pub fn new() -> Self {
        InputState::default()
    }
    
    pub fn update_mouse(&mut self, x: f32, y: f32) {
        self.mouse.x = x;
        self.mouse.y = y;
    }
    
    pub fn set_mouse_button(&mut self, button: MouseButton, pressed: bool) {
        match button {
            MouseButton::Left => self.mouse.left_pressed = pressed,
            MouseButton::Right => self.mouse.right_pressed = pressed,
            MouseButton::Middle => self.mouse.middle_pressed = pressed,
        }
    }
    
    pub fn set_key(&mut self, key: Key, pressed: bool) {
        match key {
            Key::W => self.keyboard.w = pressed,
            Key::A => self.keyboard.a = pressed,
            Key::S => self.keyboard.s = pressed,
            Key::D => self.keyboard.d = pressed,
            Key::Space => self.keyboard.space = pressed,
            Key::Escape => self.keyboard.escape = pressed,
            Key::Enter => self.keyboard.enter = pressed,
        }
    }
    
    pub fn reset_scroll(&mut self) {
        self.scroll_delta = 0.0;
    }
}

#[derive(Clone, Copy, Debug)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

#[derive(Clone, Copy, Debug)]
pub enum Key {
    W,
    A,
    S,
    D,
    Space,
    Escape,
    Enter,
}

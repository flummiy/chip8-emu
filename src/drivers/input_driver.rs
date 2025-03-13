use sdl3::keyboard::Keycode;

pub fn process_input(key: Keycode) -> Option<usize> {
    match key {
        Keycode::_1 => Some(0x1),
        Keycode::_2 => Some(0x2),
        Keycode::_3 => Some(0x3),
        Keycode::_4 => Some(0xC),
        Keycode::Q => Some(0x4),
        Keycode::W => Some(0x5),
        Keycode::E => Some(0x6),
        Keycode::R => Some(0xD),
        Keycode::A => Some(0x7),
        Keycode::S => Some(0x8),
        Keycode::D => Some(0x9),
        Keycode::F => Some(0xE),
        Keycode::Z => Some(0xA),
        Keycode::X => Some(0x0),
        Keycode::C => Some(0xB),
        Keycode::V => Some(0xF),
        _ => None,
    }
}

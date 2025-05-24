use raylib::prelude::KeyboardKey;

pub fn key_to_char(key: KeyboardKey, shift: bool) -> Option<char> {
    let c = match key {
        KeyboardKey::KEY_LEFT_SHIFT => return None,
        KeyboardKey::KEY_RIGHT_SHIFT => return None,
        KeyboardKey::KEY_LEFT_ALT => return None,
        KeyboardKey::KEY_RIGHT_ALT => return None,
        KeyboardKey::KEY_LEFT_CONTROL => return None,
        KeyboardKey::KEY_RIGHT_CONTROL => return None,
        KeyboardKey::KEY_LEFT_SUPER => return None,
        KeyboardKey::KEY_RIGHT_SUPER => return None,
        key if ((key as u8) >= KeyboardKey::KEY_A as u8) && ((key as u8) <= KeyboardKey::KEY_Z as u8) => {
            let base = if shift { b'A' } else { b'a' };
            (base + (key as u8 - KeyboardKey::KEY_A as u8)) as char
        }
        KeyboardKey::KEY_ZERO => if shift { ')' } else { '0' },
        KeyboardKey::KEY_ONE => if shift { '!' } else { '1' },
        KeyboardKey::KEY_TWO => if shift { '@' } else { '2' },
        KeyboardKey::KEY_THREE => if shift { '#' } else { '3' },
        KeyboardKey::KEY_FOUR => if shift { '$' } else { '4' },
        KeyboardKey::KEY_FIVE => if shift { '%' } else { '5' },
        KeyboardKey::KEY_SIX => if shift { '^' } else { '6' },
        KeyboardKey::KEY_SEVEN => if shift { '&' } else { '7' },
        KeyboardKey::KEY_EIGHT => if shift { '*' } else { '8' },
        KeyboardKey::KEY_NINE => if shift { '(' } else { '9' },

        KeyboardKey::KEY_SPACE => ' ',
        KeyboardKey::KEY_COMMA => if shift { '<' } else { ',' },
        KeyboardKey::KEY_PERIOD => if shift { '>' } else { '.' },
        KeyboardKey::KEY_SLASH => if shift { '?' } else { '/' },
        KeyboardKey::KEY_SEMICOLON => if shift { ':' } else { ';' },
        KeyboardKey::KEY_APOSTROPHE => if shift { '"' } else { '\'' },
        KeyboardKey::KEY_LEFT_BRACKET => if shift { '{' } else { '[' },
        KeyboardKey::KEY_RIGHT_BRACKET => if shift { '}' } else { ']' },
        KeyboardKey::KEY_MINUS => if shift { '_' } else { '-' },
        KeyboardKey::KEY_EQUAL => if shift { '+' } else { '=' },
        KeyboardKey::KEY_BACKSLASH => if shift { '|' } else { '\\' },
        KeyboardKey::KEY_GRAVE => if shift { '~' } else { '`' },
        
        _ => return None,
    };
    Some(c)
}

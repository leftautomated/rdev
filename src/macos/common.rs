#![allow(clippy::upper_case_acronyms)]
use crate::macos::keyboard::Keyboard;
use crate::rdev::{Button, Event, EventType, WheelDeltaUnit, WheelEvent};
use core::ptr::NonNull;
use lazy_static::lazy_static;
use objc2_core_graphics::{CGEvent, CGEventField, CGEventFlags, CGEventType};
use std::convert::TryInto;
use std::sync::Mutex;
use std::time::SystemTime;

use crate::macos::keycodes::key_from_code_or_name;

use super::keycodes::key_from_special_key;

lazy_static! {
    pub static ref LAST_FLAGS: Mutex<CGEventFlags> = Mutex::new(CGEventFlags(0));
    pub static ref KEYBOARD_STATE: Mutex<Keyboard> = Mutex::new(Keyboard::new().unwrap());
}

fn fixed_delta_to_points(fixed: i64, fallback_points: i64) -> f64 {
    if fixed == 0 {
        fallback_points as f64
    } else {
        fixed as f64 / 65_536.0
    }
}

pub fn set_is_main_thread(b: bool) {
    KEYBOARD_STATE.lock().unwrap().set_is_main_thread(b);
}

pub unsafe fn convert(
    _type: CGEventType,
    cg_event: NonNull<CGEvent>,
    keyboard_state: &mut Keyboard,
) -> Option<Event> {
    unsafe {
        let option_type = match _type {
            CGEventType::LeftMouseDown => Some(EventType::ButtonPress(Button::Left)),
            CGEventType::LeftMouseUp => Some(EventType::ButtonRelease(Button::Left)),
            CGEventType::RightMouseDown => Some(EventType::ButtonPress(Button::Right)),
            CGEventType::RightMouseUp => Some(EventType::ButtonRelease(Button::Right)),
            CGEventType::MouseMoved => {
                let point = CGEvent::location(Some(cg_event.as_ref()));
                // let point = cg_event.location();
                Some(EventType::MouseMove {
                    x: point.x,
                    y: point.y,
                })
            }
            CGEventType::LeftMouseDragged => {
                let point = CGEvent::location(Some(cg_event.as_ref()));
                Some(EventType::MouseMove {
                    x: point.x,
                    y: point.y,
                })
            }
            CGEventType::RightMouseDragged => {
                let point = CGEvent::location(Some(cg_event.as_ref()));
                Some(EventType::MouseMove {
                    x: point.x,
                    y: point.y,
                })
            }
            CGEventType::KeyDown => {
                // println!("Received {cg_event:?}");
                let code = CGEvent::integer_value_field(
                    Some(cg_event.as_ref()),
                    CGEventField::KeyboardEventKeycode,
                );
                let flags = CGEvent::flags(Some(cg_event.as_ref()));
                let name = keyboard_state.create_string_for_key(code, flags);
                let key = key_from_code_or_name(code.try_into().ok()?, name.as_deref());
                Some(EventType::KeyPress(key))
            }
            CGEventType::KeyUp => {
                let code = CGEvent::integer_value_field(
                    Some(cg_event.as_ref()),
                    CGEventField::KeyboardEventKeycode,
                );
                let flags = CGEvent::flags(Some(cg_event.as_ref()));
                let name = keyboard_state.create_string_for_key(code, flags);
                let key = key_from_code_or_name(code.try_into().ok()?, name.as_deref());
                Some(EventType::KeyRelease(key))
            }
            CGEventType::FlagsChanged => {
                let code = CGEvent::integer_value_field(
                    Some(cg_event.as_ref()),
                    CGEventField::KeyboardEventKeycode,
                );
                let code = code.try_into().ok()?;
                let flags = CGEvent::flags(Some(cg_event.as_ref()));
                let key = key_from_code_or_name(code, None);

                // Determine if this is a press or release based on flag changes
                let mut global_flags = LAST_FLAGS.lock().unwrap();
                if flags.contains(CGEventFlags::MaskShift)
                    && !global_flags.contains(CGEventFlags::MaskShift)
                {
                    *global_flags = flags;
                    Some(EventType::KeyPress(key))
                } else if !flags.contains(CGEventFlags::MaskShift)
                    && global_flags.contains(CGEventFlags::MaskShift)
                {
                    *global_flags = flags;
                    Some(EventType::KeyRelease(key))
                } else if flags.contains(CGEventFlags::MaskControl)
                    && !global_flags.contains(CGEventFlags::MaskControl)
                {
                    *global_flags = flags;
                    Some(EventType::KeyPress(key))
                } else if !flags.contains(CGEventFlags::MaskControl)
                    && global_flags.contains(CGEventFlags::MaskControl)
                {
                    *global_flags = flags;
                    Some(EventType::KeyRelease(key))
                } else if flags.contains(CGEventFlags::MaskAlternate)
                    && !global_flags.contains(CGEventFlags::MaskAlternate)
                {
                    *global_flags = flags;
                    Some(EventType::KeyPress(key))
                } else if !flags.contains(CGEventFlags::MaskAlternate)
                    && global_flags.contains(CGEventFlags::MaskAlternate)
                {
                    *global_flags = flags;
                    Some(EventType::KeyRelease(key))
                } else if flags.contains(CGEventFlags::MaskCommand)
                    && !global_flags.contains(CGEventFlags::MaskCommand)
                {
                    *global_flags = flags;
                    Some(EventType::KeyPress(key))
                } else if !flags.contains(CGEventFlags::MaskCommand)
                    && global_flags.contains(CGEventFlags::MaskCommand)
                {
                    *global_flags = flags;
                    Some(EventType::KeyRelease(key))
                } else {
                    None
                }
            }
            CGEventType::ScrollWheel => {
                let fixed_delta_y = CGEvent::integer_value_field(
                    Some(cg_event.as_ref()),
                    CGEventField::ScrollWheelEventFixedPtDeltaAxis1,
                );
                let fixed_delta_x = CGEvent::integer_value_field(
                    Some(cg_event.as_ref()),
                    CGEventField::ScrollWheelEventFixedPtDeltaAxis2,
                );
                let point_delta_y = CGEvent::integer_value_field(
                    Some(cg_event.as_ref()),
                    CGEventField::ScrollWheelEventPointDeltaAxis1,
                );
                let point_delta_x = CGEvent::integer_value_field(
                    Some(cg_event.as_ref()),
                    CGEventField::ScrollWheelEventPointDeltaAxis2,
                );
                Some(EventType::Wheel(WheelEvent {
                    delta_x: fixed_delta_to_points(fixed_delta_x, point_delta_x),
                    delta_y: fixed_delta_to_points(fixed_delta_y, point_delta_y),
                    unit: WheelDeltaUnit::Pixels,
                    phase: CGEvent::integer_value_field(
                        Some(cg_event.as_ref()),
                        CGEventField::ScrollWheelEventScrollPhase,
                    ),
                    momentum_phase: CGEvent::integer_value_field(
                        Some(cg_event.as_ref()),
                        CGEventField::ScrollWheelEventMomentumPhase,
                    ),
                    is_continuous: CGEvent::integer_value_field(
                        Some(cg_event.as_ref()),
                        CGEventField::ScrollWheelEventIsContinuous,
                    ) != 0,
                }))
            }
            CGEventType(14) => {
                // Core graphics doesnt support NX_SYSDEFINED yet

                let subtype =
                    CGEvent::integer_value_field(Some(cg_event.as_ref()), CGEventField(99));
                let data1 =
                    CGEvent::integer_value_field(Some(cg_event.as_ref()), CGEventField(149));
                let key_flags = data1 & 0x0000ffff;
                let key_pressed = ((key_flags & 0xff00) >> 8) == 0xa;
                let _key_repeat = (key_flags & 0x1) == 0x1;
                let key_code = (data1 & 0xffff0000) >> 16;

                // Mouse buttons like middle click/back/forward are subtype 7
                // Subtype 8 means keyboard event
                if subtype != 8 {
                    return None;
                }

                // println!("Received {key_code:?}");
                if let Some(code) = key_from_special_key(key_code.try_into().ok()?) {
                    if key_pressed {
                        Some(EventType::KeyPress(code))
                    } else {
                        Some(EventType::KeyRelease(code))
                    }
                } else {
                    // If we don't handle the key avoid creating an event since it can create duplicates with other keys
                    None
                }
            }
            _ev => {
                // println!("Received {_ev:?}");
                None
            }
        };
        if let Some(event_type) = option_type {
            let name = match event_type {
                EventType::KeyPress(_) => {
                    let code = CGEvent::integer_value_field(
                        Some(cg_event.as_ref()),
                        CGEventField::KeyboardEventKeycode,
                    );
                    let flags = CGEvent::flags(Some(cg_event.as_ref()));
                    keyboard_state.create_string_for_key(code, flags)
                }
                _ => None,
            };
            return Some(Event {
                event_type,
                time: SystemTime::now(),
                name,
            });
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::fixed_delta_to_points;

    #[test]
    fn fixed_delta_preserves_fractional_points() {
        assert_eq!(fixed_delta_to_points(98_304, 2), 1.5);
    }

    #[test]
    fn zero_fixed_delta_uses_integer_point_fallback() {
        assert_eq!(fixed_delta_to_points(0, -3), -3.0);
    }
}

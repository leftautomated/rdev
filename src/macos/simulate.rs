use objc2_core_foundation::{CFRetained, CGPoint};
use objc2_core_graphics::{
    CGEvent, CGEventField, CGEventFlags, CGEventSource, CGEventSourceStateID, CGEventTapLocation,
    CGEventType, CGMouseButton, CGScrollEventUnit,
};

use crate::Key;
use crate::rdev::{Button, EventType, SimulateError, WheelDeltaUnit};

use crate::macos::common::LAST_FLAGS;
use crate::macos::keycodes::code_from_key;

fn rounded_i32(value: f64) -> i32 {
    if value.is_finite() {
        value.round().clamp(i32::MIN as f64, i32::MAX as f64) as i32
    } else {
        0
    }
}

fn fixed_16_16(value: f64) -> i64 {
    if value.is_finite() {
        (value * 65_536.0)
            .round()
            .clamp(i64::MIN as f64, i64::MAX as f64) as i64
    } else {
        0
    }
}

fn convert_native_with_source(
    event_type: &EventType,
    source: CFRetained<CGEventSource>,
) -> Option<CFRetained<CGEvent>> {
    match event_type {
        EventType::KeyPress(key) => {
            let code = code_from_key(*key)?;
            // For modifier keys, we need to use FlagsChanged event type
            if is_modifier_key(*key) {
                let event = CGEvent::new(Some(&source))?;
                CGEvent::set_type(Some(&event), CGEventType::FlagsChanged);
                CGEvent::set_integer_value_field(
                    Some(&event),
                    CGEventField::KeyboardEventKeycode,
                    code as i64,
                );

                // Get current flags and update them
                let mut new_flags = LAST_FLAGS.lock().unwrap();
                match key {
                    Key::ShiftLeft | Key::ShiftRight => {
                        new_flags.insert(CGEventFlags::MaskShift);
                    }
                    Key::ControlLeft | Key::ControlRight => {
                        new_flags.insert(CGEventFlags::MaskControl);
                    }
                    Key::AltGr | Key::Alt => {
                        new_flags.insert(CGEventFlags::MaskAlternate);
                    }
                    Key::MetaLeft | Key::MetaRight => {
                        new_flags.insert(CGEventFlags::MaskCommand);
                    }
                    _ => {}
                }
                CGEvent::set_flags(Some(&event), *new_flags);
                // event.set_flags(*new_flags);
                Some(event)
            } else {
                // For non-modifier keys, use regular key events
                let event = CGEvent::new_keyboard_event(Some(&source), code, true)?;
                CGEvent::set_flags(Some(&event), *LAST_FLAGS.lock().unwrap());
                Some(event)
            }
        }
        EventType::KeyRelease(key) => {
            let code = code_from_key(*key)?;
            // For modifier keys, we need to use FlagsChanged event type
            if is_modifier_key(*key) {
                let event = CGEvent::new(Some(&source))?;
                CGEvent::set_type(Some(&event), CGEventType::FlagsChanged);
                CGEvent::set_integer_value_field(
                    Some(&event),
                    CGEventField::KeyboardEventKeycode,
                    code as i64,
                );

                // Get current flags and update them
                let mut new_flags = LAST_FLAGS.lock().unwrap();
                match key {
                    Key::ShiftLeft | Key::ShiftRight => {
                        new_flags.remove(CGEventFlags::MaskShift);
                    }
                    Key::ControlLeft | Key::ControlRight => {
                        new_flags.remove(CGEventFlags::MaskControl);
                    }
                    Key::AltGr | Key::Alt => {
                        new_flags.remove(CGEventFlags::MaskAlternate);
                    }
                    Key::MetaLeft | Key::MetaRight => {
                        new_flags.remove(CGEventFlags::MaskCommand);
                    }
                    _ => {}
                }
                CGEvent::set_flags(Some(&event), *new_flags);
                Some(event)
            } else {
                // For non-modifier keys, use regular key events
                let event = CGEvent::new_keyboard_event(Some(&source), code, false)?;
                CGEvent::set_flags(Some(&event), *LAST_FLAGS.lock().unwrap());
                Some(event)
            }
        }
        EventType::ButtonPress(button) => {
            let point = get_current_mouse_location()?;
            let event = match button {
                Button::Left => CGEventType::LeftMouseDown,
                Button::Right => CGEventType::RightMouseDown,
                _ => return None,
            };
            CGEvent::new_mouse_event(
                Some(&source),
                event,
                point,
                CGMouseButton::Left, // ignored because we don't use OtherMouse EventType
            )
        }
        EventType::ButtonRelease(button) => {
            let point = get_current_mouse_location()?;
            let event = match button {
                Button::Left => CGEventType::LeftMouseUp,
                Button::Right => CGEventType::RightMouseUp,
                _ => return None,
            };
            CGEvent::new_mouse_event(
                Some(&source),
                event,
                point,
                CGMouseButton::Left, // ignored because we don't use OtherMouse EventType
            )
        }
        EventType::MouseMove { x, y } => {
            let point = CGPoint { x: (*x), y: (*y) };
            CGEvent::new_mouse_event(
                Some(&source),
                CGEventType::MouseMoved,
                point,
                CGMouseButton::Left,
            )
        }
        EventType::Wheel(wheel) => {
            let (unit, delta_x, delta_y) = match wheel.unit {
                WheelDeltaUnit::Pixels => (CGScrollEventUnit::Pixel, wheel.delta_x, wheel.delta_y),
                WheelDeltaUnit::Lines => (CGScrollEventUnit::Line, wheel.delta_x, wheel.delta_y),
                WheelDeltaUnit::WindowsWheelDelta => (
                    CGScrollEventUnit::Line,
                    wheel.delta_x / 120.0,
                    wheel.delta_y / 120.0,
                ),
            };
            let event = CGEvent::new_scroll_wheel_event2(
                Some(&source),
                unit,
                2,
                rounded_i32(delta_y),
                rounded_i32(delta_x),
                0,
            )?;
            if wheel.unit == WheelDeltaUnit::Pixels {
                CGEvent::set_integer_value_field(
                    Some(&event),
                    CGEventField::ScrollWheelEventFixedPtDeltaAxis1,
                    fixed_16_16(delta_y),
                );
                CGEvent::set_integer_value_field(
                    Some(&event),
                    CGEventField::ScrollWheelEventFixedPtDeltaAxis2,
                    fixed_16_16(delta_x),
                );
            }
            CGEvent::set_integer_value_field(
                Some(&event),
                CGEventField::ScrollWheelEventScrollPhase,
                wheel.phase,
            );
            CGEvent::set_integer_value_field(
                Some(&event),
                CGEventField::ScrollWheelEventMomentumPhase,
                wheel.momentum_phase,
            );
            CGEvent::set_integer_value_field(
                Some(&event),
                CGEventField::ScrollWheelEventIsContinuous,
                i64::from(wheel.is_continuous),
            );
            Some(event)
        }
    }
}

fn convert_native(event_type: &EventType) -> Option<CFRetained<CGEvent>> {
    let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState)?;
    convert_native_with_source(event_type, source)
}

fn get_current_mouse_location() -> Option<CGPoint> {
    let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState)?;
    let event = CGEvent::new(Some(&source))?;
    Some(CGEvent::location(Some(&event)))
}

fn is_modifier_key(key: Key) -> bool {
    matches!(
        key,
        Key::ShiftLeft
            | Key::ShiftRight
            | Key::ControlLeft
            | Key::ControlRight
            | Key::Alt
            | Key::AltGr
            | Key::MetaLeft
            | Key::MetaRight
    )
}

#[link(name = "Cocoa", kind = "framework")]
unsafe extern "C" {}

pub fn simulate(event_type: &EventType) -> Result<(), SimulateError> {
    if let Some(cg_event) = convert_native(event_type) {
        CGEvent::post(CGEventTapLocation::HIDEventTap, Some(&cg_event));
        Ok(())
    } else {
        Err(SimulateError)
    }
}

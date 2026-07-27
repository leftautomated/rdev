#[cfg(feature = "serialize")]
use serde::{Deserialize, Serialize};
use std::time::SystemTime;
use std::{fmt, fmt::Display};

// /// Callback type to send to listen function.
// pub type Callback = dyn FnMut(Event) -> ();

/// Callback type to send to grab function.
pub type GrabCallback = fn(event: Event) -> Option<Event>;

/// Errors that occur when trying to capture OS events.
/// Be careful on Mac, not setting accessibility does not cause an error
/// it justs ignores events.
#[derive(Debug)]
#[non_exhaustive]
pub enum ListenError {
    /// MacOS
    EventTapError,
    /// MacOS
    LoopSourceError,
    /// Linux
    MissingDisplayError,
    /// Linux
    KeyboardError,
    /// Linux
    RecordContextEnablingError,
    /// Linux
    RecordContextError,
    /// Linux
    XRecordExtensionError,
    /// Windows
    KeyHookError(u32),
    /// Windows
    MouseHookError(u32),
}

/// Errors that occur when trying to grab OS events.
/// Be careful on Mac, not setting accessibility does not cause an error
/// it justs ignores events.
#[derive(Debug)]
#[non_exhaustive]
pub enum GrabError {
    /// MacOS
    EventTapError,
    /// MacOS
    LoopSourceError,
    /// Linux
    MissingDisplayError,
    /// Linux
    KeyboardError,
    /// Windows
    KeyHookError(u32),
    /// Windows
    MouseHookError(u32),
    /// All
    SimulateError,
    IoError(std::io::Error),
}
/// Errors that occur when trying to get display size.
#[non_exhaustive]
#[derive(Debug)]
pub enum DisplayError {
    NoDisplay,
    ConversionError,
}

impl From<SimulateError> for GrabError {
    fn from(_: SimulateError) -> GrabError {
        GrabError::SimulateError
    }
}

impl From<std::io::Error> for GrabError {
    fn from(err: std::io::Error) -> GrabError {
        GrabError::IoError(err)
    }
}

/// Marking an error when we tried to simulate and event
#[derive(Debug)]
pub struct SimulateError;

impl Display for SimulateError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Could not simulate event")
    }
}

impl std::error::Error for SimulateError {}

/// Key names based on physical location on the device
/// Merge Option(MacOS) and Alt(Windows, Linux) into Alt
/// Merge Windows (Windows), Meta(Linux), Command(MacOS) into Meta
/// Characters based on Qwerty layout, don't use this for characters as it WILL
/// depend on the layout. Use Event.name instead. Key modifiers gives those keys
/// a different value too.
/// Careful, on Windows KpReturn does not exist, it' s strictly equivalent to Return, also Keypad keys
/// get modified if NumLock is Off and ARE pagedown and so on.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub enum Key {
    /// Alt key on Linux and Windows (option key on macOS)
    Alt,
    AltGr,
    Backspace,
    CapsLock,
    ControlLeft,
    ControlRight,
    Delete,
    DownArrow,
    End,
    Escape,
    F1,
    F10,
    F11,
    F12,
    F13,
    F14,
    F15,
    F16,
    F17,
    F18,
    F19,
    F20,
    F21,
    F22,
    F23,
    F24,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    Home,
    LeftArrow,
    /// also known as "windows", "super", and "command"
    MetaLeft,
    /// also known as "windows", "super", and "command"
    MetaRight,
    PageDown,
    PageUp,
    Return,
    RightArrow,
    ShiftLeft,
    ShiftRight,
    Space,
    Tab,
    UpArrow,
    PrintScreen,
    ScrollLock,
    Pause,
    NumLock,
    BackQuote,
    Num1,
    Num2,
    Num3,
    Num4,
    Num5,
    Num6,
    Num7,
    Num8,
    Num9,
    Num0,
    Minus,
    Equal,
    KeyQ,
    KeyW,
    KeyE,
    KeyR,
    KeyT,
    KeyY,
    KeyU,
    KeyI,
    KeyO,
    KeyP,
    LeftBracket,
    RightBracket,
    KeyA,
    KeyS,
    KeyD,
    KeyF,
    KeyG,
    KeyH,
    KeyJ,
    KeyK,
    KeyL,
    SemiColon,
    Quote,
    BackSlash,
    IntlBackslash,
    KeyZ,
    KeyX,
    KeyC,
    KeyV,
    KeyB,
    KeyN,
    KeyM,
    Comma,
    Dot,
    Slash,
    Insert,
    KpReturn,
    KpMinus,
    KpPlus,
    KpMultiply,
    KpDivide,
    Kp0,
    Kp1,
    Kp2,
    Kp3,
    Kp4,
    Kp5,
    Kp6,
    Kp7,
    Kp8,
    Kp9,
    KpDelete,
    Function,
    VolumeUp,
    VolumeDown,
    VolumeMute,
    BrightnessUp,
    BrightnessDown,
    PreviousTrack,
    PlayPause,
    PlayCd,
    NextTrack,
    Unknown(u32),
}

/// Standard mouse buttons
/// Some mice have more than 3 buttons. These are not defined, and different
/// OSs will give different `Button::Unknown` values.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub enum Button {
    Left,
    Right,
    Middle,
    Unknown(u8),
}

/// Unit used by a [`WheelEvent`]'s deltas.
#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub enum WheelDeltaUnit {
    /// Screen-space points or pixels after platform scroll acceleration.
    Pixels,
    /// Logical lines or discrete wheel clicks.
    Lines,
    /// Native Windows wheel units, where 120 is one logical click.
    WindowsWheelDelta,
}

const WINDOWS_WHEEL_DELTA: f64 = 120.0;
const APPROXIMATE_PIXELS_PER_LINE: f64 = 10.0;

/// Additional scroll-sequence information exposed by platforms that support it.
#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub struct WheelEvent {
    /// Horizontal movement in [`Self::unit`].
    pub delta_x: f64,
    /// Vertical movement in [`Self::unit`].
    pub delta_y: f64,
    pub unit: WheelDeltaUnit,
    /// Native gesture phase. Zero means the platform supplied no phase.
    pub phase: i64,
    /// Native momentum phase. Zero means the event is not momentum scrolling.
    pub momentum_phase: i64,
    /// Whether the platform classified this as continuous scrolling.
    pub is_continuous: bool,
}

impl WheelEvent {
    /// Construct a pixel/point scroll event without gesture metadata.
    pub const fn pixels(delta_x: f64, delta_y: f64) -> Self {
        Self {
            delta_x,
            delta_y,
            unit: WheelDeltaUnit::Pixels,
            phase: 0,
            momentum_phase: 0,
            is_continuous: false,
        }
    }

    /// Construct a discrete line-based scroll event without gesture metadata.
    pub const fn lines(delta_x: f64, delta_y: f64) -> Self {
        Self {
            delta_x,
            delta_y,
            unit: WheelDeltaUnit::Lines,
            phase: 0,
            momentum_phase: 0,
            is_continuous: false,
        }
    }

    /// Construct a native Windows-v120 scroll event without gesture metadata.
    pub const fn windows_wheel_delta(delta_x: f64, delta_y: f64) -> Self {
        Self {
            delta_x,
            delta_y,
            unit: WheelDeltaUnit::WindowsWheelDelta,
            phase: 0,
            momentum_phase: 0,
            is_continuous: (delta_x != 0.0 && delta_x > -120.0 && delta_x < 120.0)
                || (delta_y != 0.0 && delta_y > -120.0 && delta_y < 120.0),
        }
    }

    /// Return the deltas converted to `unit`.
    ///
    /// Pixel/line conversion is necessarily approximate; native-unit replay on
    /// the recording platform does not use that approximation.
    pub fn deltas_in(self, unit: WheelDeltaUnit) -> (f64, f64) {
        let to_lines = |delta| match self.unit {
            WheelDeltaUnit::Pixels => delta / APPROXIMATE_PIXELS_PER_LINE,
            WheelDeltaUnit::Lines => delta,
            WheelDeltaUnit::WindowsWheelDelta => delta / WINDOWS_WHEEL_DELTA,
        };
        let from_lines = |lines| match unit {
            WheelDeltaUnit::Pixels => lines * APPROXIMATE_PIXELS_PER_LINE,
            WheelDeltaUnit::Lines => lines,
            WheelDeltaUnit::WindowsWheelDelta => lines * WINDOWS_WHEEL_DELTA,
        };
        (
            from_lines(to_lines(self.delta_x)),
            from_lines(to_lines(self.delta_y)),
        )
    }
}

/// Cross-platform keyboard and pointer event.
#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub enum EventType {
    /// The keys correspond to a standard qwerty layout, they don't correspond
    /// To the actual letter a user would use, that requires some layout logic to be added.
    KeyPress(Key),
    KeyRelease(Key),
    /// Mouse Button
    ButtonPress(Button),
    ButtonRelease(Button),
    /// Values in pixels. `EventType::MouseMove{x: 0, y: 0}` corresponds to the
    /// top left corner, with x increasing downward and y increasing rightward
    MouseMove {
        x: f64,
        y: f64,
    },
    /// Positive deltas scroll up or right; negative deltas scroll down or left.
    Wheel(WheelEvent),
}

/// When events arrive from the OS they get some additional information added from
/// EventType, which is the time when this event was received, and the name Option
/// which contains what characters should be emmitted from that event. This relies
/// on the OS layout and keyboard state machinery.
/// Caveat: Dead keys don't function on Linux(X11) yet. You will receive None for
/// a dead key, and the raw letter instead of accentuated letter.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub struct Event {
    pub time: SystemTime,
    pub name: Option<String>,
    pub event_type: EventType,
}

/// We can define a dummy Keyboard, that we will use to detect
/// what kind of EventType trigger some String. We get the currently used
/// layout for now !
/// Caveat : This is layout dependent. If your app needs to support
/// layout switching don't use this !
/// Caveat: On Linux, the dead keys mechanism is not implemented.
/// Caveat: Only shift and dead keys are implemented, Alt+unicode code on windows
/// won't work.
///
/// ```no_run
/// use rdev::{Keyboard, EventType, Key, KeyboardState};
///
/// let mut keyboard = Keyboard::new().unwrap();
/// let string = keyboard.add(&EventType::KeyPress(Key::KeyS));
/// // string == Some("s")
/// ```
pub trait KeyboardState {
    /// Changes the keyboard state as if this event happened. we don't
    /// really hit the OS here, which might come handy to test what should happen
    /// if we were to hit said key.
    fn add(&mut self, event_type: &EventType) -> Option<String>;

    /// Resets the keyboard state as if we never touched it (no shift, caps_lock and so on)
    fn reset(&mut self);
}

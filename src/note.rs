use core::fmt;
use core::ops::{Add, AddAssign, Sub, SubAssign};

use crate::{DataByte, ParseError};

#[doc = r#"
Identifies a key for some message.

Keys are interpeted as a 7-bit number.

Each value corresponds to some [`Note`] and [`Octave`].

[`Key`] `0` is `C(-1)`, and [`Key`] `127` is `G9`.

# Example
```rust
# use midix::prelude::*;

let key_byte = 63;

let key = Key::from_databyte(key_byte).unwrap(); // 63 is between 0-127

assert_eq!(key.note(), Note::DSharp);
assert_eq!(key.octave(), Octave::new(4))
```
"#]
#[derive(Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Debug, Hash)]
#[cfg_attr(
    feature = "bevy_resources",
    derive(bevy::prelude::Component, bevy::prelude::Reflect)
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Note(DataByte);

impl Note {
    /// Create a new note.
    ///
    /// Checks for correctness (leading 0 bit).
    pub fn from_databyte<B>(rep: B) -> Result<Self, ParseError>
    where
        B: TryInto<DataByte, Error = ParseError>,
    {
        rep.try_into().map(Self)
    }

    /// Creates a new note without checking for the validity of the byte
    pub const fn from_databyte_unchecked(byte: u8) -> Self {
        Self(DataByte::new_unchecked(byte))
    }

    /// Create all possible notes (128)
    pub fn all() -> [Note; 128] {
        core::array::from_fn(|i| Note(DataByte(i as u8)))
    }

    /// Create a note from a given key and octave
    ///
    /// # Panics
    /// if you pass in, on `Octave::new(9)` a Key greater than `Key::G`.
    ///
    /// this is because `Key::GSharp-Key::B` for octave 9 is not representable
    /// in midi.
    pub const fn new(key: Key, octave: Octave) -> Self {
        let octave_byte = (octave.value() + 1) as u8;

        let note_byte = key.get_mod_12();

        let octave_mult = (octave_byte) * 12;

        if octave_mult + note_byte > 127 {
            panic!("Can't make Key. See documentation for details.");
        }

        Self(DataByte(octave_mult + note_byte))
    }

    /// Identifies the key of the note pressed
    #[inline]
    pub const fn key(&self) -> Key {
        Key::from_data_byte(&self.0)
    }

    /// Returns true if the key of the note is sharp. Same as `is_flat`
    ///
    /// See [`Key::is_sharp`] for an example
    #[inline]
    pub const fn is_sharp(&self) -> bool {
        self.key().is_sharp()
    }

    /// Returns true if the key of the note is flat. Same as `is_sharp`
    /// See [`Key::is_flat`] for an example
    #[inline]
    pub const fn is_flat(&self) -> bool {
        self.key().is_flat()
    }

    /// Identifies the octave of the note pressed
    #[inline]
    pub fn octave(&self) -> Octave {
        Octave::from_data_byte(&self.0)
    }

    /// Returns the underlying byte of the note
    pub fn byte(&self) -> u8 {
        self.0.0
    }
}
/// Efficiently make a note.
///
///
/// ## Example
/// ```rust
/// # use midix::prelude::*;
/// let my_note= note!(C, 2);
/// assert_eq!(my_note, Note::new(Key::C, Octave::new(2)));
/// ```
#[macro_export]
macro_rules! note {
    ($key:ident, $oct:literal) => {
        ::midix::prelude::Note::new(
            ::midix::prelude::Key::$key,
            ::midix::prelude::Octave::new($oct),
        )
    };
}

impl fmt::Display for Note {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.key(), self.octave())
    }
}

impl Add<u8> for Note {
    type Output = Note;
    fn add(self, rhs: u8) -> Self::Output {
        let next = (self.0.0 + rhs).min(127);
        Self(DataByte(next))
    }
}

impl AddAssign<u8> for Note {
    fn add_assign(&mut self, rhs: u8) {
        if self.0.0 >= 127 {
            return;
        }
        let next = (self.0.0 + rhs).min(127);
        self.0.0 = next;
    }
}

impl Sub<u8> for Note {
    type Output = Note;
    fn sub(self, rhs: u8) -> Self::Output {
        let next = self.0.0.saturating_sub(rhs);
        Self(DataByte(next))
    }
}

impl SubAssign<u8> for Note {
    fn sub_assign(&mut self, rhs: u8) {
        let next = self.0.0.saturating_sub(rhs);
        self.0.0 = next;
    }
}

#[test]
fn add_to_note() {
    let key = Note::new(Key::C, Octave::new(9));
    let plus_one = key + 1;
    assert_eq!(plus_one, Note::new(Key::CSharp, Octave::new(9)));

    let plus_alot = key + 50;
    assert_eq!(plus_alot, Note::new(Key::G, Octave::new(9)));
}

#[test]
fn add_assign_to_note() {
    let mut key = Note::new(Key::C, Octave::new(9));
    key += 1;
    assert_eq!(key, Note::new(Key::CSharp, Octave::new(9)));

    key += 50;
    assert_eq!(key, Note::new(Key::G, Octave::new(9)));
}

#[test]
fn sub_from_note() {
    let key = Note::new(Key::C, Octave::new(9));
    let minus_one = key - 1;
    assert_eq!(minus_one, Note::new(Key::B, Octave::new(8)));

    let minus_alot = key - 220;
    assert_eq!(minus_alot, Note::new(Key::C, Octave::new(-1)));
}

#[test]
fn subassign_note() {
    let mut key = Note::new(Key::C, Octave::new(9));
    key -= 1;
    assert_eq!(key, Note::new(Key::B, Octave::new(8)));

    key -= 220;
    assert_eq!(key, Note::new(Key::C, Octave::new(-1)));
}

#[test]
fn test_note() {
    let c = Note::from_databyte(12).unwrap();

    assert_eq!(Key::C, c.key());

    let a_sharp = Note::from_databyte(94).unwrap();
    assert_eq!(Key::ASharp, a_sharp.key());
}

#[test]
fn test_octave() {
    let c = Note::from_databyte(12).unwrap();

    assert_eq!(0, c.octave().value());

    let a_sharp = Note::from_databyte(94).unwrap();
    assert_eq!(6, a_sharp.octave().value());
}

#[allow(missing_docs)]
#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
#[cfg_attr(feature = "bevy", derive(bevy::prelude::Reflect))]
#[doc = r#"
Identifies some note for a [`Key`]

# Example
```rust
# use midix::prelude::*;

let note = Note::FSharp;


let key = note.with_octave(Octave::new(4));


assert_eq!(key.octave().value(), 4);
assert_eq!(key.note(), Note::FSharp);
```
"#]
pub enum Key {
    C,
    CSharp,
    D,
    DSharp,
    E,
    F,
    FSharp,
    G,
    GSharp,
    A,
    ASharp,
    B,
}
impl Key {
    /// Returns an array beginning with [`Note::C`] to [`Note::B`]
    pub fn all() -> [Key; 12] {
        use Key::*;
        [C, CSharp, D, DSharp, E, F, FSharp, G, GSharp, A, ASharp, B]
    }
    // /// Create a new note from a byte with a leading 0
    // ///
    // /// # Errors
    // /// if the byte is > 127
    // pub fn new<K, E>(key: K) -> Result<Self, io::Error>
    // where
    //     K: TryInto<DataByte, Error = E>,
    //     E: Into<io::Error>,
    // {
    //     use Note::*;
    //     let key = key.try_into().map_err(Into::into)?;
    //     let note = key.value() % 12;

    //     Ok(match note {
    //         0 => C,
    //         1 => CSharp,
    //         2 => D,
    //         3 => DSharp,
    //         4 => E,
    //         5 => F,
    //         6 => FSharp,
    //         7 => G,
    //         8 => GSharp,
    //         9 => A,
    //         10 => ASharp,
    //         11 => B,
    //         _ => unreachable!(),
    //     })
    // }

    /// Returns true if the note type is sharp. Same as `is_flat`
    ///
    /// # Example
    /// ```rust
    /// # use midix::prelude::*;
    /// let note = Note::C;
    /// assert!(!note.is_sharp());
    /// let note = Note::FSharp;
    /// assert!(note.is_sharp());
    /// ```
    #[inline]
    pub const fn is_sharp(&self) -> bool {
        use Key::*;
        matches!(self, CSharp | DSharp | FSharp | GSharp | ASharp)
    }

    /// Returns true if the note type is flat. Same as `is_sharp`
    ///
    /// # Example
    /// ```rust
    /// # use midix::prelude::*;
    /// let note = Note::A;
    /// assert!(!note.is_flat());
    /// let note = Note::CSharp;
    /// assert!(note.is_flat());
    /// ```
    #[inline]
    pub const fn is_flat(&self) -> bool {
        self.is_sharp()
    }

    /// Identify the note from a key byte.
    #[inline]
    pub const fn from_data_byte(key: &DataByte) -> Self {
        use Key::*;
        let note = key.value() % 12;

        match note {
            0 => C,
            1 => CSharp,
            2 => D,
            3 => DSharp,
            4 => E,
            5 => F,
            6 => FSharp,
            7 => G,
            8 => GSharp,
            9 => A,
            10 => ASharp,
            11 => B,
            _ => unreachable!(),
        }
    }
    const fn get_mod_12(&self) -> u8 {
        use Key::*;
        match self {
            C => 0,
            CSharp => 1,
            D => 2,
            DSharp => 3,
            E => 4,
            F => 5,
            FSharp => 6,
            G => 7,
            GSharp => 8,
            A => 9,
            ASharp => 10,
            B => 11,
        }
    }

    /// Create a [`Key`] given this note and a provided [`Octave`]
    pub const fn with_octave(self, octave: Octave) -> Note {
        Note::new(self, octave)
    }
}
impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Key::*;
        match self {
            C => write!(f, "C"),
            CSharp => write!(f, "C#/Db"),
            D => write!(f, "D"),
            DSharp => write!(f, "D#/Eb"),
            E => write!(f, "E"),
            F => write!(f, "F"),
            FSharp => write!(f, "F#/Gb"),
            G => write!(f, "G"),
            GSharp => write!(f, "G#Ab"),
            A => write!(f, "A"),
            ASharp => write!(f, "A#/Bb"),
            B => write!(f, "B"),
        }
    }
}

#[doc = r#"
Identifies the octave for a [`Key`]. Values range from -1 to 9.

# Example

```rust
# use midix::prelude::*;

let octave = Octave::new(12); // clamps to 9

assert_eq!(octave.value(), 9);

let key = octave.with_note(Note::C);


assert_eq!(key.octave().value(), 9);
assert_eq!(key.note(), Note::C);
```
"#]
#[derive(PartialEq, Debug, Clone, Copy)]
#[cfg_attr(feature = "bevy", derive(bevy::prelude::Reflect))]
pub struct Octave(i8);

impl Octave {
    /// Identify an octave from a key byte.
    pub const fn from_data_byte(key: &DataByte) -> Self {
        let octave = key.value() / 12;

        Self(octave as i8 - 1)
    }
    /// Should be a value between [-1, 9]. Clamps between these two values.
    pub const fn new(mut octave: i8) -> Self {
        if octave < -1 {
            octave = -1
        } else if octave > 9 {
            octave = 9;
        }
        Self(octave)
    }

    /// The octave, from `[-1,9]`
    pub const fn value(&self) -> i8 {
        self.0
    }

    /// Create a [`Key`] given this octave and a provided [`Note`]
    pub fn with_note(self, note: Key) -> Note {
        Note::new(note, self)
    }
}

impl fmt::Display for Octave {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Add<i8> for Octave {
    type Output = Octave;
    fn add(self, rhs: i8) -> Self::Output {
        Self::new(self.0 + rhs)
    }
}

impl AddAssign<i8> for Octave {
    fn add_assign(&mut self, rhs: i8) {
        self.0 = (self.0 + rhs).clamp(-1, 9);
    }
}

impl Sub<i8> for Octave {
    type Output = Octave;
    fn sub(self, rhs: i8) -> Self::Output {
        Self::new(self.0 - rhs)
    }
}

impl SubAssign<i8> for Octave {
    fn sub_assign(&mut self, rhs: i8) {
        self.0 = (self.0 - rhs).clamp(-1, 9);
    }
}

#[test]
fn note_from_key_octave_pairs() {
    for key_byte in 0..128 {
        let key = Note::from_databyte(key_byte).unwrap();

        let exp_note = key.key();
        let exp_oct = key.octave();

        let made_key = Note::new(exp_note, exp_oct);

        assert_eq!(exp_oct, made_key.octave());
        assert_eq!(exp_note, made_key.key());
        assert_eq!(key, Note::new(exp_note, exp_oct));
    }
}

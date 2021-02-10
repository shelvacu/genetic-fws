use std::num::NonZeroU8;
use std::fmt;
use rand::Rng;
use std::convert::TryFrom;
use std::ops::BitAnd;

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FChar(NonZeroU8);

impl FChar {
    pub unsafe fn new_unchecked(u: u8) -> Self {
        FChar(NonZeroU8::new_unchecked(u))
    }

    pub fn new(u: u8) -> Self {
        if u <= 26 {
            FChar(NonZeroU8::new(u).unwrap())
        } else {
            panic!("u out of range");
        }
    }

    pub fn random<R: Rng + ?Sized>(rng: &mut R) -> Self {
        Self::new(rng.gen_range(1..=26))
    }
}

impl fmt::Debug for FChar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "F{}", (*self).into():char)
    }
}

impl TryFrom<char> for FChar {
    type Error = &'static str;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            'a' | 'A' => Ok(FChar(NonZeroU8::new(1).unwrap())),
            'b' | 'B' => Ok(FChar(NonZeroU8::new(2).unwrap())),
            'c' | 'C' => Ok(FChar(NonZeroU8::new(3).unwrap())),
            'd' | 'D' => Ok(FChar(NonZeroU8::new(4).unwrap())),
            'e' | 'E' => Ok(FChar(NonZeroU8::new(5).unwrap())),
            'f' | 'F' => Ok(FChar(NonZeroU8::new(6).unwrap())),
            'g' | 'G' => Ok(FChar(NonZeroU8::new(7).unwrap())),
            'h' | 'H' => Ok(FChar(NonZeroU8::new(8).unwrap())),
            'i' | 'I' => Ok(FChar(NonZeroU8::new(9).unwrap())),
            'j' | 'J' => Ok(FChar(NonZeroU8::new(10).unwrap())),
            'k' | 'K' => Ok(FChar(NonZeroU8::new(11).unwrap())),
            'l' | 'L' => Ok(FChar(NonZeroU8::new(12).unwrap())),
            'm' | 'M' => Ok(FChar(NonZeroU8::new(13).unwrap())),
            'n' | 'N' => Ok(FChar(NonZeroU8::new(14).unwrap())),
            'o' | 'O' => Ok(FChar(NonZeroU8::new(15).unwrap())),
            'p' | 'P' => Ok(FChar(NonZeroU8::new(16).unwrap())),
            'q' | 'Q' => Ok(FChar(NonZeroU8::new(17).unwrap())),
            'r' | 'R' => Ok(FChar(NonZeroU8::new(18).unwrap())),
            's' | 'S' => Ok(FChar(NonZeroU8::new(19).unwrap())),
            't' | 'T' => Ok(FChar(NonZeroU8::new(20).unwrap())),
            'u' | 'U' => Ok(FChar(NonZeroU8::new(21).unwrap())),
            'v' | 'V' => Ok(FChar(NonZeroU8::new(22).unwrap())),
            'w' | 'W' => Ok(FChar(NonZeroU8::new(23).unwrap())),
            'x' | 'X' => Ok(FChar(NonZeroU8::new(24).unwrap())),
            'y' | 'Y' => Ok(FChar(NonZeroU8::new(25).unwrap())),
            'z' | 'Z' => Ok(FChar(NonZeroU8::new(26).unwrap())),
            _ => Err("Invalid character"),
        }
    }
}

impl From<FChar> for char {
    fn from(f:FChar) -> char {
        match f.0.into():u8 {
            1..=26 => (b'`' + (f.0.into():u8)) as char,
            _ => unreachable!(),
        }
    }
}

impl From<FChar> for u8 {
    fn from(f:FChar) -> u8 {
        f.0.into()
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct CharSet(u32);

impl CharSet {
    pub fn full() -> Self {
        let mut s = Self::default();
        for i in 1..=26 {
            s = s.set(FChar(NonZeroU8::new(i).unwrap()));
        }
        s
    }

    #[must_use]
    pub fn set(self, c:FChar) -> Self {
        match c.0.into():u8 {
            1..=26 => CharSet(self.0 | (1 << c.0.into():u8)),
            _ => unreachable!(),
        }
    }

    #[must_use]
    pub fn clear(self, c:FChar) -> Self {
        match c.0.into():u8 {
            1..=26 => CharSet(self.0 & !(1 << c.0.into():u8)),
            _ => unreachable!(),
        }
    }

    pub fn check(self, c:FChar) -> bool {
        match c.0.into():u8 {
            1..=26 => (self.0 & (1 << c.0.into():u8)) > 0,
            _ => unreachable!(),
        }
    }
}

impl fmt::Debug for CharSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CharSet({})", self.into_iter().map(|f| f.into():char).collect::<String>())
    }
}

pub struct CharSetIter(CharSet,u8);

impl Iterator for CharSetIter {
    type Item = FChar;
    fn next(&mut self) -> Option<FChar> {
        while self.1 < 27 {
            let f = FChar::new(self.1);
            self.1 += 1;
            if self.0.check(f) {
                return Some(f);
            }
        }
        None
    }
}

impl IntoIterator for CharSet {
    type Item = FChar;
    type IntoIter = CharSetIter;

    fn into_iter(self) -> Self::IntoIter {
        CharSetIter(self, 1)
    }
}

impl Default for CharSet {
    fn default() -> CharSet {
        CharSet(0)
    }
}

impl BitAnd for CharSet {
    type Output = Self;

    // rhs is the "right-hand side" of the expression `a & b`
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl From<u32> for CharSet {
    fn from(a: u32) -> Self {
        CharSet(a)
    }
}

impl From<CharSet> for u32 {
    fn from(a: CharSet) -> Self {
        a.0
    }
}
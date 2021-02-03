#![feature(array_methods, exclusive_range_pattern, type_ascription)]
#![allow(unused_imports,unused_variables,dead_code)]
use std::collections::{HashMap,BTreeMap};
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::num::NonZeroU8;
use std::convert::TryFrom;
use std::hint::unreachable_unchecked;
use std::ops::BitAnd;
use std::convert::TryInto;
use std::fmt;
use std::thread::sleep;
use std::time::Duration;
use std::default::Default;
use fnv::FnvHashMap;
use indicatif::ProgressBar;

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct FChar(NonZeroU8);

impl FChar {
    pub unsafe fn new_unchecked(u: u8) -> Self {
        return FChar(NonZeroU8::new_unchecked(u))
    }

    pub fn new(u: u8) -> Self {
        if u <= 26 {
            return FChar(NonZeroU8::new(u).unwrap());
        } else {
            panic!("u out of range");
        }
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
            1..=26 => return (('`' as u8) + (f.0.into():u8)) as char,
            _ => unreachable!(),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct CharSet(u32);

impl CharSet {
    fn set(self, c:FChar) -> Self {
        match c.0.into():u8 {
            1..=26 => CharSet(self.0 | (1 << c.0.into():u8)),
            _ => unreachable!(),
        }
    }

    fn clear(self, c:FChar) -> Self {
        match c.0.into():u8 {
            1..=26 => CharSet(self.0 & !(1 << c.0.into():u8)),
            _ => unreachable!(),
        }
    }

    fn check(self, c:FChar) -> bool {
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

struct CharSetIter(CharSet,u8);

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
        return None;
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

const SIZE:usize = 10;

type Word = [FChar; SIZE];
type OWord = [Option<FChar>; SIZE];
type Square = [OWord; SIZE];
type WMap = FnvHashMap<OWord, CharSet>;

fn main() {
    dbg!(SIZE);
    let wordsfn = std::env::args().nth(1).unwrap();

    // let data = std::fs::read(&wordsfn).unwrap();

    // dbg!(&data[0..10]);

    let mut words:Vec<[FChar; SIZE]> = Vec::new();
    let mut line_no = 0;
    for maybe_line in read_lines(&wordsfn).unwrap() {
        line_no += 1;
        // println!("{}", line_no);
        // dbg!(&maybe_line);
        let line = maybe_line.unwrap();
        if line.len() == SIZE {
            if let Ok(word_vec) = line.chars().map(|c| FChar::try_from(c)).collect():Result<Vec<FChar>,_> {
                let mut word = [FChar::try_from('a').unwrap(); SIZE];
                word.as_mut_slice().copy_from_slice(word_vec.as_slice());
                words.push(word);
            }
        }
    }
    dbg!(words.len());

    let mut map:WMap = Default::default();

    for word in &words {
        let mut partial_word:[Option<FChar>; SIZE] = [None; SIZE];
        for i in 0..SIZE {
            partial_word[i] = Some(word[i]);
        }
        for i in (0..SIZE).rev() {
            let c = partial_word[i].unwrap();
            partial_word[i] = None;
            let set = map.entry(partial_word).or_default();
            *set = set.set(c);
        }
    }

    dbg!(map.len());
    // let pb = ProgressBar::new(words.len().try_into().unwrap():u64);
    // pb.inc(0);
    let mut square:[[Option<FChar>; SIZE]; SIZE] = [[None; SIZE]; SIZE];

    let mut num_squares = 0;
    let start = std::time::Instant::now();
    for word in &words[0..10] {
        for i in 0..SIZE {
            square[0][i] = Some(word[i]);
        }
        recurse(square,0,1,&map,&mut num_squares);
        // pb.inc(1);
    }
    dbg!(start.elapsed());
    dbg!(&num_squares);

    // let thing = [
    //     Some(FChar::try_from('b').unwrap()),
    //     Some(FChar::try_from('e').unwrap()),
    //     Some(FChar::try_from('g').unwrap()),
    //     Some(FChar::try_from('i').unwrap()),
    //     Some(FChar::try_from('n').unwrap()),
    //     Some(FChar::try_from('n').unwrap()),
    //     Some(FChar::try_from('i').unwrap()),
    //     None, //Some(FChar::try_from('n').unwrap()),
    //     None, //Some(FChar::try_from('g').unwrap()),
    // ];
    //dbg!(map.get(&[None; SIZE]));
    //dbg!(map.get(&thing).unwrap().clone().into_iter().map(|a| a.into():char).collect():Vec<_>);

    //count 3 levels deep
    //dbg!(count(SIZE-1,[None; SIZE],0,&map));
}

fn recurse(s:Square,col:usize,row:usize,map:&WMap,count:&mut u64) {
    //dbg!(col, row, s);
    if row == SIZE {
        for i in 0..SIZE {
            for j in 0..SIZE {
                print!("{} ", s[i][j].unwrap().into():char);
            }
            println!();
        }
        println!();
        // sleep(Duration::from_millis(1000));
        *count += 1;
    }
    let mut col_key:OWord = [None; SIZE];
    let mut row_key:OWord = [None; SIZE];
    for i in 0..col {
        row_key[i] = s[row][i];
    }
    for i in 0..row {
        col_key[i] = s[i][col];
    }
    //dbg!(col_key, row_key);
    let col_set = map.get(&col_key).map(|a| *a).unwrap_or_default();
    let row_set = map.get(&row_key).map(|a| *a).unwrap_or_default();
    let and_set = col_set & row_set;
    let new_col = (col+1) % SIZE;
    let new_row = if col == SIZE-1 {
        row + 1
    } else { row };
    //dbg!(col_set, row_set, and_set);
    //confirm();
    for c in and_set.into_iter() {
        //if col == 0 && row > 0 && s[col][row].unwrap() > c { continue; }
        let mut new_s = s;
        new_s[row][col] = Some(c);
        recurse(new_s,new_col,new_row,map,count);
    }
}

fn count(levels:usize,thing:OWord,index:usize,map:&WMap) -> usize {
    let set = map.get(&thing).map(|a| *a).unwrap_or_default();
    assert!(thing[index].is_none());
    set.into_iter().map(|f| {
        if index < levels {
            let mut new_thing = thing;
            new_thing[index] = Some(f);
            return count(levels, new_thing, index + 1,map);
        } else {
            return 1;
        }
    }).sum()
}

fn confirm() -> String {
    loop {
        let mut answer = String::new();

        io::stdin().read_line(&mut answer)
                   .ok()
                   .expect("Failed to read line");

        if !answer.is_empty() && answer != "\n" && answer != "\r\n" {
            return answer
        }
    }
}
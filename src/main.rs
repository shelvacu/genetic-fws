#![feature(array_methods, exclusive_range_pattern, type_ascription, total_cmp)]
#![allow(clippy::needless_range_loop)]
//#![allow(unused_imports,unused_variables,dead_code)]
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::convert::TryFrom;
use std::default::Default;
use std::cmp::PartialOrd;
use std::fmt;
use fnv::FnvHashMap;
use rand::Rng;
use serde_derive::{Serialize,Deserialize};
use libflate::gzip::Encoder;

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

mod fchar;
use fchar::{FChar, CharSet};

mod svm;
use svm::*;

mod genetics;
use genetics::*;

const SIZE:usize = 10;

type Word = [FChar; SIZE];
type OWord = [Option<FChar>; SIZE];
type Square = [OWord; SIZE];
type WMap = FnvHashMap<OWord, CharSet>;


const INSTRUCTION_SIZE_MAX:usize = 1024;
const INSTRUCTION_SIZE_INIT:usize = 32;
const GENEPOOL_SIZE:usize = 1000;

const WEIGHT_FALSE_POSITIVE:f64 = 1000.0;
const WEIGHT_FALSE_NEGATIVE:f64 = 4000.0;
const WEIGHT_INSTRUCTION:f64 = 0.1;

const NUM_TRIALS:usize = 1000;



// Fitness function, lower is better
fn fitness<R: Rng + ?Sized, I: Iterator<Item=SvmInstruction> + std::iter::ExactSizeIterator + fmt::Debug + Clone>(
    instructions: I,
    map: &WMap,
    rng: &mut R,
    trials: usize
) -> f64 {
    let mut sum = 0.0;
    for _ in 0..trials {
        let num_to_fill = rng.gen_range(2..SIZE); //purposefully NOT inclusive of completely filled
        let mut word:OWord = [None; SIZE];
        let mut set = CharSet::full();
        let mut num_filled = 0;
        while num_filled < num_to_fill {
            //dbg!(num_filled, num_to_fill, word, set);
            let options:Vec<_> = set.into_iter().collect();
            if options.is_empty() { panic!(); }
            word[num_filled] = Some(options[rng.gen_range(0..options.len())]);
            num_filled += 1;
            set = *map.get(&word).unwrap();
        }
        // for i in 0..num_to_fill {
        //     word[i] = Some(FChar::random(rng));
        // }
        sum += fitness_single(instructions.clone(), map, word, false);
    }
    sum/(trials as f64) + (instructions.len() as f64) * WEIGHT_INSTRUCTION
}

// f full
// g guess
// r real

// f|g|r|&|^|description
// 0|0|0|0|0|doesnt matter
// 0|0|1|0|1|doesnt matter
// 0|1|0|0|1|doesnt matter
// 0|1|1|1|0|doesnt matter
// 1|0|0|0|0|accurate
// 1|0|1|0|1|false negative
// 1|1|0|0|1|false positive
// 1|1|1|1|0|accurate

// false positives = (g^r)&g
// false negatives = (g^r)&r


fn fitness_single<I: Iterator<Item=SvmInstruction> + fmt::Debug>(
    instructions: I,
    map: &WMap,
    word: OWord,
    debug: bool,
) -> f64 {
    let mut state = SvmState::new(instructions);
    
    for i in 0..SIZE {
        state.memory_mut()[i+1] = word[i].map(|f| CharSet::default().set(f).into():u32).unwrap_or_default();
    }
    //Default to outputting all 0's (the worst default) to discourage empty programs
    state.memory_mut()[0] = 0;
    loop {
        let res = state.step();
        if res == StepResult::Finish { break; }
    }
    let full:u32 = CharSet::full().into();
    let guess:u32 = state.memory_mut()[0];
    let real:u32 = map.get(&word).copied().unwrap_or_default().into();
    let xor = guess^real;
    let false_positives:f64 = (full&xor&guess).count_ones().into();
    let false_negatives:f64 = (full&xor&real ).count_ones().into();
    if debug {
        dbg!(
            guess,
            real,
            xor,
            (full&guess).count_ones(),
            false_positives,
            ((!full)| guess).count_zeros(),
            false_negatives,
        );
    }
    ((false_positives * WEIGHT_FALSE_POSITIVE) + (false_negatives * WEIGHT_FALSE_NEGATIVE)).powf(2.0)
}

fn main() {
    dbg!(SIZE);
    let wordsfn = std::env::args().nth(1).unwrap_or_else(|| String::from("/home/shelvacu/words/uncompressible.txt"));

    // let data = std::fs::read(&wordsfn).unwrap();

    // dbg!(&data[0..10]);

    let mut words:Vec<Word> = Vec::new();
    for maybe_line in read_lines(&wordsfn).unwrap() {
        let line = maybe_line.unwrap();
        if line.len() == SIZE {
            if let Ok(word_vec) = line.chars().map(FChar::try_from).collect():Result<Vec<FChar>,_> {
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

    if false {
        find_squares(&words, &map);
        std::process::exit(0);
    }

    let mut rng = rand::thread_rng();
    #[derive(Debug,Serialize,Deserialize)]
    struct Genome {
        instructions: Vec<Gene>,
        mutation_rate: f64,
        fitness: f64,
    }

    #[derive(Debug,Serialize,Deserialize)]
    struct State {
        round: usize,
        pool: Vec<Genome>,
    }
    //let mut pool:Vec<Genome> = Vec::with_capacity(GENEPOOL_SIZE);
    let mut state = State{
        round: 1,
        pool: Vec::with_capacity(GENEPOOL_SIZE),
    };

    for _ in 0..GENEPOOL_SIZE {
        let mut instructions:Vec<Gene> = Vec::with_capacity(INSTRUCTION_SIZE_INIT);

        for _ in 0..INSTRUCTION_SIZE_INIT {
            instructions.push(Gene{order: rng.gen(), ins: SvmInstruction::random(&mut rng)});
        }
        instructions.sort_unstable_by(|a,b| a.order.total_cmp(&b.order));
        state.pool.push(Genome{
            instructions,
            mutation_rate: 0.5,
            fitness: 0.0,
        });
    }

    //let mut round = 1;
    loop {
        for g in &mut state.pool {
            g.fitness = fitness(g.instructions.iter().map(|g| g.ins), &map, &mut rng, NUM_TRIALS);
        }
        state.pool.sort_by(|a,b| a.fitness.partial_cmp(&b.fitness).unwrap());
        if state.round % 16 == 0 {
            let filename = format!("round{}.json.gz",state.round);
            println!("Wrote {:?}", filename);
            let f = std::fs::File::create(&filename).unwrap();
            let mut encoder = Encoder::new(f).unwrap();
            serde_json::to_writer(&mut encoder, &state).unwrap();
            encoder.finish().unwrap().0.sync_all().unwrap();
        }
        let first = state.pool.first().unwrap();
        for ins in &first.instructions {
            println!("{:.5}: {}", ins.order, ins.ins)
        }
        println!("First mutation rate {}", first.mutation_rate);
        fitness_single(
            first.instructions.iter().map(|a| a.ins),
            &map, 
            [
                Some(FChar::try_from('a').unwrap()),
                Some(FChar::try_from('b').unwrap()),
                Some(FChar::try_from('a').unwrap()),
                Some(FChar::try_from('c').unwrap()),
                Some(FChar::try_from('a').unwrap()),
                None,
                None,
                None,
                None,
                None,
            ],
            true,
        );
        fitness_single(
            first.instructions.iter().map(|a| a.ins),
            &map, 
            [
                Some(FChar::try_from('a').unwrap()),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            ],
            true,
        );
        println!(
            "Round {}, best/med/worst {:.5}/{:.5}/{:.5}",
            state.round,
            state.pool.first().unwrap().fitness,
            state.pool[state.pool.len()/2].fitness,
            state.pool.last().unwrap().fitness,
        );
        // for _ in 0..((pool.len()/4)*3) {
        //     pool.pop();
        // }
        let prev_len = state.pool.len() as f64;
        let mut i = 0;
        #[allow(clippy::eval_order_dependence)]
        state.pool.retain(|_| (rng.gen::<f64>() > ((i as f64)/prev_len), i += 1).0);
        let parents_end = state.pool.len();
        dbg!(parents_end);
        for _ in parents_end..GENEPOOL_SIZE {
            let parents = (
                &state.pool[rng.gen_range(0..parents_end)],
                &state.pool[rng.gen_range(0..parents_end)],
            );
            let mut genes = Vec::new();
            for mut b in diff(parents.0.instructions.iter().copied(), parents.1.instructions.iter().copied()) {
                match b.ty {
                    DiffTy::Both => genes.append(&mut b.block),
                    _ => if rng.gen():bool { genes.append(&mut b.block) },
                }
            }
            let mut mutation_rates = [parents.0.mutation_rate, parents.1.mutation_rate];
            mutation_rates.sort_unstable_by(|a,b| a.partial_cmp(&b).unwrap());
            let child_mutation_rate;
            if mutation_rates[0].eq(&mutation_rates[1]) {
                child_mutation_rate = mutation_rates[0];
            } else {
                child_mutation_rate = rng.gen_range(mutation_rates[0]..mutation_rates[1]);
            }
            let mut child = Genome{
                instructions: genes,
                mutation_rate: child_mutation_rate,
                fitness: -1.0,
            };
            child.mutation_rate += (rng.gen::<f64>() - 0.5)*child.mutation_rate*0.1;
            if child.mutation_rate > 0.9 {
                child.mutation_rate = 0.9;
            }
            if child.mutation_rate < 0.01 {
                child.mutation_rate = 0.01;
            }
            while rng.gen::<f64>() < child.mutation_rate {
                if rng.gen::<bool>() && child.instructions.len() < INSTRUCTION_SIZE_MAX {
                    //add a random gene
                    child.instructions.push(Gene{order: rng.gen(), ins: SvmInstruction::random(&mut rng)});
                    child.instructions.sort();
                } else {
                    //remove a random gene
                    if !child.instructions.is_empty() {
                        child.instructions.remove(rng.gen_range(0..child.instructions.len()));
                    }
                }
            }
            state.pool.push(child);
        }

        state.round += 1;
    }
}

fn find_squares(words: &[Word], map: &WMap) {
    let mut square:[[Option<FChar>; SIZE]; SIZE] = [[None; SIZE]; SIZE];

    let mut num_squares = 0;
    let start = std::time::Instant::now();
    for word in &words[0..10] {
        for i in 0..SIZE {
            square[0][i] = Some(word[i]);
        }
        recurse(square,0,1,&map,&mut num_squares);
    }
    dbg!(start.elapsed());
    dbg!(&num_squares);
}

fn recurse(s:Square,col:usize,row:usize,map:&WMap,count:&mut u64) {
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
    row_key.copy_from_slice(&s[row]);
    // for i in 0..col {
    //     row_key[i] = s[row][i];
    // }
    for i in 0..row {
        col_key[i] = s[i][col];
    }
    //dbg!(col_key, row_key);
    let col_set = map.get(&col_key).copied().unwrap_or_default();
    let row_set = map.get(&row_key).copied().unwrap_or_default();
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

#[allow(dead_code)]
fn count(
    levels:usize,
    thing:OWord,
    index:usize,
    map:&WMap
) -> usize {
    let set = map.get(&thing).copied().unwrap_or_default();
    assert!(thing[index].is_none());
    set.into_iter().map(|f| {
        if index < levels {
            let mut new_thing = thing;
            new_thing[index] = Some(f);
            count(levels, new_thing, index + 1,map)
        } else {
            1
        }
    }).sum()
}
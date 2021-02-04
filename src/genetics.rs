use std::cmp::Ordering;

use super::svm::*;

#[derive(Debug,Clone,Copy)]
pub struct Gene(pub f64, pub SvmInstruction);

impl PartialEq for Gene {
    fn eq(&self, other: &Self) -> bool {
        self.0.partial_cmp(&other.0).unwrap() == Ordering::Equal && self.1 == other.1
    }
}

impl Eq for Gene {}

impl PartialOrd for Gene {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Gene {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.0.partial_cmp(&other.0).unwrap() {
            Ordering::Equal => self.1.cmp(&other.1),
            a => a,
        }
    }
}

#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum DiffTy {
    A,
    B,
    Both,
}

#[derive(Debug,Clone,PartialEq,Eq)]
pub struct DiffBlock {
    pub ty: DiffTy,
    pub block: Vec<Gene>,
}

pub fn diff(mut a_it:impl Iterator<Item=Gene>, mut b_it:impl Iterator<Item=Gene>) -> Vec<DiffBlock> {
    let mut res:Vec<DiffBlock> = Vec::new();
    let mut cur_block = DiffBlock{ty: DiffTy::Both, block: vec![]};
    let mut cur_a = a_it.next();
    let mut cur_b = b_it.next();
    let mut add_block = |ty, gene| {
        if cur_block.ty != ty {
            let mut block = DiffBlock{ty, block: vec![]};
            std::mem::swap(&mut cur_block, &mut block);
            if !block.block.is_empty() {
                res.push(block);
            }
        }
        cur_block.block.push(gene);
    };
    loop {
        match (cur_a, cur_b, cur_a.cmp(&cur_b)) {
            (None, None, _) => break,
            (Some(a), Some(_), Ordering::Equal) => {
                add_block(DiffTy::Both, a);
                cur_a = a_it.next();
                cur_b = b_it.next();
            },
            (Some(a), None, _)|(Some(a), Some(_), Ordering::Less) => {
                add_block(DiffTy::A, a);
                cur_a = a_it.next();
            },
            (None, Some(b), _)|(Some(_), Some(b), Ordering::Greater) => {
                add_block(DiffTy::B, b);
                cur_b = b_it.next();
            },
        }
    }
    std::mem::drop(add_block);
    res.push(cur_block);
    res
}

#[cfg(test)]
mod diff_test {
    use super::*;
    fn convenient_diff(a: &[f64], b: &[f64], expected: &[(DiffTy, Vec<f64>)]) {
        let ins = SvmInstruction{ty: SvmInstructionTy::Xor, dest: 0, src: 0};
        let res = diff(
            a.iter().map(|n| Gene(*n, ins)),
            b.iter().map(|n| Gene(*n, ins))
        );
        let nums:Vec<_> = res.iter().map(|g| (g.ty, g.block.iter().map(|n| n.0).collect():Vec<_>)).collect();
        assert_eq!(nums.as_slice(), expected);
    }
    #[test]
    fn both_empty() {
        convenient_diff(&[], &[], &[]);
    }

    #[test]
    fn a_only() {
        convenient_diff(
            &[0.0,0.1],
            &[],
            &[
                (DiffTy::A, vec![0.0, 0.1]),
            ],
        );
    }

    #[test]
    fn b_only() {
        convenient_diff(
            &[],
            &[0.0,0.1],
            &[
                (DiffTy::B, vec![0.0, 0.1]),
            ],
        );
    }

    #[test]
    fn mix_1() {
        convenient_diff(
            &[0.0,0.2],
            &[0.1],
            &[
                (DiffTy::A, vec![0.0,]),
                (DiffTy::B, vec![0.1,]),
                (DiffTy::A, vec![0.2,]),
            ],
        );
    }

    #[test]
    fn mix_2() {
        convenient_diff(
            &[0.0,0.1,0.2,0.3,0.4,0.6,0.7],
            &[0.0,0.1,0.4,0.5],
            &[
                (DiffTy::Both, vec![0.0, 0.1]),
                (DiffTy::A, vec![0.2, 0.3]),
                (DiffTy::Both, vec![0.4]),
                (DiffTy::B, vec![0.5]),
                (DiffTy::A, vec![0.6, 0.7]),
            ],
        );
    }
}
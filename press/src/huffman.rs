
use std::collections::HashMap;
use std::collections::BinaryHeap;
use core::hash::Hash;
use bitvec::prelude::*;
use bimap::BiMap;
use bitvec::vec::BitVec;


#[allow(dead_code)]
#[derive(Debug)]
pub enum HuffmanError {
    String(&'static str)
}

pub enum HuffmanTree {
    Terminal {
        freq: u64,
        symbol: u8,
    },
    Node{
        freq: u64,
        left: Box<HuffmanTree>,
        right: Box<HuffmanTree>
    }
}

impl std::fmt::Debug for HuffmanTree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Terminal { freq, symbol } => write!(f, "T({}, {})", *symbol as char, freq),
            Self::Node { freq, left, right } => write!(f, "N({:?},{:?})[{}]", left, right, freq),
        }
    }
}

impl HuffmanTree {
    pub fn freq(&self) -> u64 {
        match self {
            HuffmanTree::Node{freq, ..} => *freq,
            HuffmanTree::Terminal{freq, ..} => *freq
        }
    } 

    pub fn from_frequencies(frequencies: &HashMap<u8, u64>) -> HuffmanTree {
        let mut heap: BinaryHeap<HuffmanTree> = BinaryHeap::new();
        
        for (c, freq) in frequencies.iter(){
            heap.push(HuffmanTree::Terminal{
                symbol: *c,
                freq: *freq
            })
        }
    
        while heap.len() > 1 {
            let left = heap.pop().unwrap();
            let right = heap.pop().unwrap();
            heap.push(HuffmanTree::Node { 
                freq: left.freq()+right.freq(), 
                left: Box::new(left), 
                right: Box::new(right)
            })
        }
        
        heap.pop().unwrap()
    }
}

impl Ord for HuffmanTree {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.freq().cmp(&self.freq())
    }
}
impl PartialOrd for HuffmanTree {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl PartialEq for HuffmanTree {
    fn eq(&self, other: &Self) -> bool {
        self.freq() == other.freq()
    }
}
impl Eq for HuffmanTree {}


trait Merge {
    fn merge(self, other: Self) -> Self;
}

impl<L,R> Merge for BiMap<L,R> 
where 
    L: Hash + PartialEq + Eq + PartialOrd + Ord,
    R: Hash + PartialEq + Eq + PartialOrd + Ord,
{   
    fn merge(self, other: Self) -> Self {
        self.into_iter().chain(other).collect()
    }
}

pub struct HuffmanEncoding{
    pub encoding: BiMap<u8, BitVec<u8, Lsb0>>
}

impl PartialEq for HuffmanEncoding {
    fn eq(&self, other: &Self) -> bool {
        self.encoding == other.encoding
    }
}
impl Eq for HuffmanEncoding {}

fn descend(t: &HuffmanTree, seq: BitVec<u8, Lsb0>) -> BiMap<u8, BitVec<u8, Lsb0>> {
    match t {
        HuffmanTree::Node {left, right, .. } => {
            let mut lpath = seq.clone();
            lpath.push(false);
            let l = descend(&left, lpath);
            
            let mut rpath = seq.clone();
            rpath.push(true);
            let r = descend(&right, rpath);
            
            l.merge(r)
        },
        HuffmanTree::Terminal { symbol, .. } => {
            return BiMap::from_iter(vec![(*symbol, seq)])
        }
    }
}

fn frequency_map(input: &Vec<u8>) -> HashMap<u8, u64> {
    let mut hm: HashMap<u8, u64> = HashMap::new();
    for &b in input{
        *hm.entry(b).or_insert(0) += 1;
    }
    hm
}

impl HuffmanEncoding{
    pub fn from_data_vec(input: &Vec<u8>) -> HuffmanEncoding {
        let frequencies = frequency_map(input);
        HuffmanEncoding::from_frequencies(frequencies)
    }

    pub fn from_frequencies(frequencies: HashMap<u8, u64>) -> HuffmanEncoding {
        let tree = HuffmanTree::from_frequencies(&frequencies);
        HuffmanEncoding::from_tree(&tree)
    }

    pub fn from_tree(tree: &HuffmanTree) -> HuffmanEncoding {
        HuffmanEncoding{
            encoding: descend(tree, BitVec::new())
        }
    }

    pub fn save(&mut self) -> Vec<u8> {
        let mut out: Vec<u8> = Vec::new();
        // let max_bitfield_size = self.encoding.right_values().map(|f| f.len()).max().unwrap();

        for (&l, r) in self.encoding.iter() {
            let len = r.len();
            if len > std::u8::MAX.into() {
                panic!("Cannot encode bitfield length in 8 bits");
            }
            r.to_owned().set_uninitialized(false);
            let bits = r.clone().into_vec();
            let _bytes = bits.len();
            
            // { symbol: u8, len: u8, bits: [u8, len]}
            out.push(l);
            out.push(len as u8);
            out.append(&mut bits.clone());

            // println!("encoding {} as {} with len {} in {} bytes", l as char, r, len, bytes);
        }
        return out;
    }

    pub fn restore(d: &Vec<u8>) -> Result<HuffmanEncoding, HuffmanError> {
        let mut encoding: BiMap<u8, BitVec<u8,Lsb0>> = BiMap::new(); 

        let mut index: usize = 0;
        loop {
            let c = *d.get(index).expect("Error deserialising input file");
            let len = *d.get(index+1).expect("Error deserialising input file");
            let size = (len as f64 / 8.0).ceil() as usize;
            let bits: &[u8] = &d[index+2..index+size+2];
            let mut v: BitVec<u8, Lsb0> = BitVec::from_slice(bits);
            v.truncate(len as usize);

            // println!("decoding {} to {} in {} bytes with {} bits", c as char, v, size, len);

            encoding.insert(c, v);

            index += size+2;

            if index >= d.len() { break }
        }        

        return Ok(HuffmanEncoding{
            encoding: encoding
        });
    }

    pub fn encode(&self,input: &Vec<u8>) -> BitVec<u8, Lsb0> {
        let mut filestream = bitvec![u8, Lsb0;];
        
        for c in input{
            let mut code = self.encoding.get_by_left(&c).unwrap().clone();
            filestream.append(&mut code);
        }
       
       filestream
    }

    pub fn decode(self, _input: &Vec<u8>) -> Vec<u8> {
        // &BitVec<u8, Lsb0>
        vec![]
    }

    pub fn diff(&self, other: &Self) -> Vec<(u8, BitVec<u8,Lsb0>, BitVec<u8,Lsb0>)> {
        let mut diffs = Vec::new();
        for (l,r) in self.encoding.iter(){
            let sr = self.encoding.get_by_left(l).unwrap();
            let or = other.encoding.get_by_left(l).unwrap();
            if  sr != or {
                diffs.push((
                    *l,
                    r.clone(),
                    or.clone()
                ))
            }
        }
        return diffs;
    }
}


// impl Serialize for HuffmanEncoding {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: serde::Serializer 
//     {
//         todo!()
//     }
// }

// impl<'de> Deserialize<'de> for HuffmanEncoding {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: serde::Deserializer<'de> 
//     {
//         todo!()
//     }
// }

fn _print_char_table(hm: &HashMap<u8, u64>){
    for (k, v) in hm.iter(){
        println!("{} -> {}", *k as char , v)
    }
}

fn _show_encoding(freq: &HashMap<u8, u64>, en: &BiMap<u8, BitVec>){
    for (c, v) in en.iter(){
        println!("{} [{}] <> {}", *c as char, freq.get(c).unwrap(), v)
    }
}
impl std::fmt::Debug for HuffmanEncoding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (c, v) in self.encoding.iter(){
            write!(f, "{} <> {}\n", *c as char, v)?
        }
        write!(f, "")
    }
}
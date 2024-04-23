use crate::proto::{
    piston_server::Piston, CompressRequest, CompressResponse, DecompressRequest, DecompressResponse,
};

use bit_vec::BitVec;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, VecDeque};
use std::fmt;
use std::ops::Deref;
use tonic::{Request, Response, Status};

#[derive(Default)]
pub struct PistonService;

#[tonic::async_trait]
impl Piston for PistonService {
    async fn compress(
        &self,
        request: Request<CompressRequest>,
    ) -> Result<Response<CompressResponse>, Status> {
        let request = request.into_inner();
        println!("\nReceived: {:#?}", request);

        let text = request
            .data
            .into_iter()
            .map(|b| b as char)
            .collect::<String>();
        let (data, count, tree) = compress(&text);
        let response = CompressResponse {
            data,
            count: count as u32,
            tree,
        };

        Ok(Response::new(response))
    }

    async fn decompress(
        &self,
        request: Request<DecompressRequest>,
    ) -> Result<Response<DecompressResponse>, Status> {
        let request = request.into_inner();
        println!("\nReceived: {:#?}", request);

        let text = decompress(&request.data, request.count as usize, request.tree);
        let response = DecompressResponse {
            data: text.bytes().collect(),
        };

        Ok(Response::new(response))
    }
}

#[derive(Debug, Clone)]
enum HuffmanNode {
    Leaf {
        frequency: usize,
        character: char,
    },
    Internal {
        frequency: usize,
        left: Box<HuffmanNode>,
        right: Box<HuffmanNode>,
    },
}

impl HuffmanNode {
    fn frequency(&self) -> usize {
        match *self {
            HuffmanNode::Leaf { frequency, .. } => frequency,
            HuffmanNode::Internal { frequency, .. } => frequency,
        }
    }

    fn contains(&self, character: char) -> bool {
        match self {
            HuffmanNode::Leaf { character: c, .. } => *c == character,
            HuffmanNode::Internal { left, right, .. } => {
                left.contains(character) || right.contains(character)
            }
        }
    }
}

impl PartialEq for HuffmanNode {
    fn eq(&self, other: &Self) -> bool {
        self.frequency() == other.frequency()
    }
}

impl Eq for HuffmanNode {}

impl Ord for HuffmanNode {
    fn cmp(&self, other: &Self) -> Ordering {
        other.frequency().cmp(&self.frequency())
    }
}

impl PartialOrd for HuffmanNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Display for HuffmanNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            HuffmanNode::Leaf {
                frequency,
                character,
            } => write!(f, "(Leaf {} {})", frequency, character),
            HuffmanNode::Internal {
                frequency,
                left,
                right,
            } => write!(f, "(Internal {} {} {})", frequency, left, right),
        }
    }
}

fn build_huffman_tree(text: &str) -> Box<HuffmanNode> {
    let mut counts = HashMap::new();
    for ch in text.chars() {
        *counts.entry(ch).or_insert(0) += 1;
    }

    let mut heap: BinaryHeap<HuffmanNode> = counts
        .into_iter()
        .map(|(character, frequency)| HuffmanNode::Leaf {
            frequency,
            character,
        })
        .collect();

    while heap.len() > 1 {
        let left = Box::new(heap.pop().unwrap());
        let right = Box::new(heap.pop().unwrap());
        let frequency = left.frequency() + right.frequency();
        heap.push(HuffmanNode::Internal {
            frequency,
            left,
            right,
        });
    }

    let tree = heap.pop().unwrap();
    Box::new(tree)
}

impl Into<Vec<i64>> for Box<HuffmanNode> {
    fn into(self) -> Vec<i64> {
        match *self {
            HuffmanNode::Leaf {
                character,
                frequency,
            } => {
                vec![0x80000000 | ((character as i64) << 32) | frequency as i64]
            }
            HuffmanNode::Internal {
                frequency,
                left,
                right,
            } => {
                let mut result = vec![frequency as i64];
                let mut left_vec: Vec<i64> = left.into();
                let mut right_vec: Vec<i64> = right.into();
                result.append(&mut left_vec);
                result.append(&mut right_vec);
                result
            }
        }
    }
}

impl From<Vec<i64>> for Box<HuffmanNode> {
    fn from(mut list: Vec<i64>) -> Self {
        let mut stack = Vec::new();
        let mut iter = list.into_iter().rev();

        while let Some(value) = iter.next() {
            if value & 0x80000000 != 0 {
                // Check if it's a leaf node
                let character = ((value >> 32) & 0xFF) as u8 as char;
                let frequency = (value & 0xFFFFFFFF) as usize;
                stack.push(Box::new(HuffmanNode::Leaf {
                    character,
                    frequency,
                }));
            } else {
                // It's an internal node
                let frequency = value as usize;
                let right = stack.pop().expect("No right child in stack");
                let left = stack.pop().expect("No left child in stack");
                stack.push(Box::new(HuffmanNode::Internal {
                    frequency,
                    left,
                    right,
                }));
            }
        }

        stack.pop().expect("No node in stack")
    }
}

fn encode_char(char: char, node: &HuffmanNode) -> BitVec {
    let mut result = BitVec::new();
    let mut stack = VecDeque::new();
    stack.push_back((node, BitVec::new()));

    while let Some((current, mut path)) = stack.pop_front() {
        match current {
            HuffmanNode::Leaf { character, .. } => {
                if *character == char {
                    result = path;
                    break;
                }
            }
            HuffmanNode::Internal { left, right, .. } => {
                let mut left_path = path.clone();
                left_path.push(false);
                stack.push_back((left.deref(), left_path));

                let mut right_path = path;
                right_path.push(true);
                stack.push_back((right.deref(), right_path));
            }
        }
    }

    result
}

fn encode(text: &str) -> (BitVec, usize, Box<HuffmanNode>) {
    let tree = build_huffman_tree(text);
    let mut result = BitVec::new();
    let mut count = 0;

    for ch in text.chars() {
        let encoded = encode_char(ch, &tree);
        count += encoded.len();
        result.extend(encoded);
    }

    (result, count, tree)
}

fn decode(encoded: &BitVec, tree: &HuffmanNode) -> String {
    let mut result = String::new();
    let mut current = tree;

    for bit in encoded.iter() {
        match current {
            HuffmanNode::Leaf { character, .. } => {
                result.push(*character);
                current = tree;
            }
            HuffmanNode::Internal { left, right, .. } => {
                current = if bit { right.deref() } else { left.deref() };
            }
        }
    }

    result
}

fn compress(text: &str) -> (Vec<u8>, usize, Vec<i64>) {
    let (encoded, count, tree) = encode(text);
    (encoded.to_bytes(), count, tree.into())
}

fn decompress(data: &[u8], count: usize, tree: Vec<i64>) -> String {
    let tree = Box::from(tree);
    let encoded = BitVec::from_bytes(data)
        .into_iter()
        .take(count * 8)
        .collect();
    decode(&encoded, &tree)
}

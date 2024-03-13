use std::collections::{BinaryHeap, HashMap};

use crate::proto::{
    piston_server::Piston, ByteMapping, CompressRequest, CompressResponse, DecompressRequest,
    DecompressResponse,
};

use bit_vec::BitVec;
use tonic::{Request, Response, Status};

#[derive(Debug, Clone)]
struct Node {
    value: Option<u8>,
    frequency: u32,
    left: Option<Box<Node>>,
    right: Option<Box<Node>>,
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.frequency.eq(&other.frequency)
    }
}

impl Eq for Node {}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.frequency.cmp(&self.frequency)
    }
}

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

        let byte_mappings = generate_byte_mappings(&request.data);
        let (data, count) = encode_data(&request.data, &byte_mappings);

        let byte_mappings = byte_mappings
            .into_iter()
            .map(|(byte, bits)| ByteMapping {
                byte: byte as u32,
                bits: bits.into_iter().collect::<Vec<_>>(),
            })
            .collect();

        Ok(Response::new(CompressResponse {
            data,
            byte_mappings,
            count,
        }))
    }

    async fn decompress(
        &self,
        request: Request<DecompressRequest>,
    ) -> Result<Response<DecompressResponse>, Status> {
        let request = request.into_inner();
        println!("\nReceived: {:#?}", request);

        if request.byte_mappings.is_empty() {
            return Err(Status::invalid_argument(
                "byte mappings required for decompression",
            ));
        }

        let bit_mappings = request
            .byte_mappings
            .into_iter()
            .map(|mapping| {
                (
                    mapping.bits.into_iter().collect::<BitVec>(),
                    mapping.byte as u8,
                )
            })
            .collect();
        let data = decode_data(&request.data, &bit_mappings, request.count);

        Ok(Response::new(DecompressResponse { data }))
    }
}

fn encode_data(data: &[u8], byte_mappings: &HashMap<u8, BitVec>) -> (Vec<u8>, u32) {
    let mut bit_vec = BitVec::new();
    let mut count = 0;

    for byte in data {
        match byte_mappings.get(byte) {
            Some(mapping) => {
                bit_vec.extend(mapping);
                count += 1;
            }
            None => panic!("Byte not found in mappings"),
        }
    }

    (bit_vec.to_bytes(), count)
}

fn decode_data(data: &[u8], bit_mappings: &HashMap<BitVec, u8>, count: u32) -> Vec<u8> {
    let mut bit_vec = BitVec::from_bytes(data).into_iter();
    let mut curr_bits = BitVec::new();
    let mut result = Vec::new();
    let mut i = 0;

    while i < count {
        match bit_mappings.get(&curr_bits) {
            Some(&byte) => {
                result.push(byte);
                curr_bits = BitVec::new();
                i += 1;
            }
            None => match bit_vec.next() {
                Some(bit) => curr_bits.push(bit),
                None => panic!("Unexpected end of bit vector"),
            },
        }
    }

    result
}

fn generate_byte_mappings(data: &[u8]) -> HashMap<u8, BitVec> {
    let frequency_map = data.iter().fold(HashMap::new(), |mut acc, &byte| {
        *acc.entry(byte).or_insert(0) += 1;
        acc
    });

    let mut pq = frequency_map
        .clone()
        .into_iter()
        .map(|(value, frequency)| Node {
            value: Some(value),
            frequency,
            left: None,
            right: None,
        })
        .collect::<BinaryHeap<_>>();

    while pq.len() > 1 {
        let right = pq.pop().unwrap();
        let left = pq.pop().unwrap();

        let parent = Node {
            value: None,
            frequency: left.frequency + right.frequency,
            left: Some(Box::new(left)),
            right: Some(Box::new(right)),
        };

        pq.push(parent);
    }

    let tree = pq.pop().unwrap();
    let mut paths = HashMap::new();
    let mut path = BitVec::new();

    extract_paths(&tree, &mut path, &mut paths);

    paths
}

fn extract_paths(node: &Node, path: &mut BitVec, paths: &mut HashMap<u8, BitVec>) {
    if let Some(value) = node.value {
        paths.insert(value, path.clone());
    } else {
        if let Some(ref left) = node.left {
            path.push(false);
            extract_paths(left, path, paths);
            path.pop();
        }

        if let Some(ref right) = node.right {
            path.push(true);
            extract_paths(right, path, paths);
            path.pop();
        }
    }
}

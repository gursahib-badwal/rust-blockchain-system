use crate::queue::{Task, WorkQueue};
use digest::consts::U32;
use sha2::digest::generic_array::GenericArray;
use sha2::{Digest, Sha256};
use std::fmt::Write;
use std::sync;

pub type Hash = GenericArray<u8, U32>;

#[derive(Debug, Clone)]
pub struct Block {
    pub prev_hash: Hash,
    pub generation: u64,
    pub difficulty: u8,
    pub data: String,
    pub proof: Option<u64>,
}

impl Block {
    pub fn initial(difficulty: u8) -> Block {
        // TODO: create and return a new initial block
        // unimplemented!()
        Block {
            prev_hash: Hash::default(),
            generation: 0,
            difficulty: difficulty,
            data: String::new(),
            proof: None,
        }
    }

    pub fn next(previous: &Block, data: String) -> Block {
        // TODO: create and return a block that could follow `previous` in the chain
        // unimplemented!()
        Block {
            prev_hash: previous.hash_for_proof(previous.proof.unwrap()),
            generation: previous.generation + 1,
            difficulty: previous.difficulty,
            data: data,
            proof: None
        }
    }
 
    pub fn hash_string_for_proof(&self, proof: u64) -> String {
        // TODO: return the hash string this block would have if we set the proof to `proof`.
        // unimplemented!()
        let str = String::new() + 
        &format!("{:02x}", self.prev_hash) + ":" +
        &format!("{:x}", self.generation) + ":" +
        &format!("{}", self.difficulty) + ":" +
        self.data.as_str() + ":" +
        &format!("{}", proof);

        return str;
    }

    pub fn hash_string(&self) -> String {
        // self.proof.unwrap() panics if block not mined
        let p = self.proof.unwrap();
        self.hash_string_for_proof(p)
    }

    pub fn hash_for_proof(&self, proof: u64) -> Hash {
        // TODO: return the block's hash as it would be if we set the proof to `proof`.
        // unimplemented!()
        let str = self.hash_string_for_proof(proof);
        
        let mut new_hash = Sha256::new();
        new_hash.update(str);
        return new_hash.finalize();
    }

    pub fn hash(&self) -> Hash {
        // self.proof.unwrap() panics if block not mined
        let p = self.proof.unwrap();
        self.hash_for_proof(p)
    }

    pub fn set_proof(self: &mut Block, proof: u64) {
        self.proof = Some(proof);
    }

    pub fn hash_satisfies_difficulty(difficulty:u8,hash:Hash) -> bool {
        // TODO: does the hash `hash` have `difficulty` trailing 0s
        // unimplemented!()
    let n_bytes = (difficulty / 8) as usize;
    let n_bits = (difficulty % 8) as usize;
    let mut start_index = hash.len() - n_bytes;

    while start_index < hash.len() {
        if hash[start_index] != 0u8 {
            return false;
        }
        start_index += 1;
    }
    //here hash.len() - n_bytes - 1 is the index for the one before the last n_byte
    let last_hash_byte = hash[hash.len() - n_bytes - 1];
    if last_hash_byte % (1 << n_bits) != 0 { 
        return false; //returning false because the byte one before the last n_byte is not divisible by (1 << n_bits)
        } 

    return true;
    }

    pub fn is_valid_for_proof(&self, proof: u64) -> bool {
        Self::hash_satisfies_difficulty(self.difficulty,self.hash_for_proof(proof))
    }

    pub fn is_valid(&self) -> bool {
        if self.proof.is_none() {
            return false;
        }
        self.is_valid_for_proof(self.proof.unwrap())
    }

    // Mine in a very simple way: check sequentially until a valid hash is found.
    // This doesn't *need* to be used in any way, but could be used to do some mining
    // before your .mine is complete. Results should be the same as .mine (but slower).
    pub fn mine_serial(self: &mut Block) {
        let mut p = 0u64;
        while !self.is_valid_for_proof(p) {
            p += 1;
        }
        self.proof = Some(p);
    }

    pub fn mine_range(self: &Block, workers: usize, start: u64, end: u64, chunks: u64) -> u64 {
        // TODO: with `workers` threads, check proof values in the given range, breaking up
	// into `chunks` tasks in a work queue. Return the first valid proof found.
        // HINTS:
        // - Create and use a queue::WorkQueue.
        // - Use sync::Arc to wrap a clone of self for sharing.
        // unimplemented!()
        let mut work_queue = WorkQueue::new(workers);
        let arc_block = sync::Arc::new(self.clone());
        let mut i = 0; 

        let chunk_size= (end - start)/chunks;
        while i < chunks {

            let chunk_start = i * chunk_size;
            let chunk_end;

            if end < (i+1)* chunk_size - 1 {
                chunk_end = end;
            }
            else {
                chunk_end = (i+1)* chunk_size - 1;
            }

            work_queue.enqueue(MiningTask {block: sync::Arc::clone(&arc_block), start: chunk_start, end: chunk_end}).unwrap();
            i+=1;
        }

        loop {
            let result = work_queue.recv();
            if self.is_valid_for_proof(result) {
                work_queue.shutdown(); //created the work_queue in this function so shutting it down in the same function as its no longer needed 
                return result;
            } 
        }
    }

    pub fn mine_for_proof(self: &Block, workers: usize) -> u64 {
        let range_start: u64 = 0;
        let range_end: u64 = 8 * (1 << self.difficulty); // 8 * 2^(bits that must be zero)
        let chunks: u64 = 2345;
        self.mine_range(workers, range_start, range_end, chunks)
    }

    pub fn mine(self: &mut Block, workers: usize) {
        self.proof = Some(self.mine_for_proof(workers));
    }
}

struct MiningTask {
    block: sync::Arc<Block>,
    // TODO: more fields as needed
    start: u64,
    end: u64
}

impl Task for MiningTask {
    type Output = u64;

    fn run(&self) -> Option<u64> {
        // TODO: what does it mean to .run?
        // unimplemented!()
        for proof in self.start..self.end {
            if self.block.is_valid_for_proof(proof) {
                return Some(proof);
            }
        }
        return None;
    }
}
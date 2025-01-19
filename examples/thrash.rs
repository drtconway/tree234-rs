use clap::Parser;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use tree234_rs::Tree234;

/// Insert, Remove, Fetch items from a 234-tree
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Random number seed
    #[arg(short, long, default_value_t = 17)]
    seed: u64,

    /// Number of iterations to perform
    #[arg(short, long, default_value_t = 1024*1024)]
    count: u64,

    /// Number of bits for keys
    #[arg(short, long, default_value_t = 20)]
    bits: usize,
}

#[derive(Debug)]
struct Counts {
    search_count: u64,
    search_successes: u64,
    insert_count: u64,
    insert_replacements: u64,
    remove_count: u64,
    remove_successes: u64,
}

pub fn main() {
    let args = Args::parse();

    let mut rng = StdRng::seed_from_u64(args.seed);
    let n = args.count;
    let m: u64 = (1 << args.bits) - 1;
    let mut tree: Tree234<u64, u64> = Tree234::new();
    let mut counts = Counts {
        search_count: 0,
        search_successes: 0,
        insert_count: 0,
        insert_replacements: 0,
        remove_count: 0,
        remove_successes: 0,
    };
    for i in 0..n {
        let u = rng.gen::<u64>() & m;
        let s = tree.get(&u);
        counts.search_count += 1;
        if s.is_some() {
            counts.search_successes += 1;
        }
        if rng.gen::<bool>() {
            let t = tree.insert(u, i);
            counts.insert_count += 1;
            if t.is_some() {
                counts.insert_replacements += 1;
            }
        } else {
            let r = tree.remove(&u);
            counts.remove_count += 1;
            if r.is_some() {
                counts.remove_successes += 1;
            }
        }
    }
    let z = tree.size();
    let mut s = 0;
    for item in tree.iter() {
        s += n - item.1;
    }
    println!("{:?}", counts);
    println!("mean age = {}", (s as f64)/(z as f64));
}

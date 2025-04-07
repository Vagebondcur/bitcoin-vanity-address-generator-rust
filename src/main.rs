use bitcoin::address::Address;
use bitcoin::key::{KeyPair, PublicKey};
use bitcoin::Network;
use bitcoin::secp256k1::{Secp256k1, rand};
use clap::Parser;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Bitcoin Vanity Address Generator specifically for bc1q addresses
#[derive(Parser, Debug)]
#[clap(version, about, long_about = None)]
struct Args {
    /// Pattern to search for after the bc1q prefix
    #[clap(short, long)]
    pattern: String,

    /// Pattern that the address should end with
    #[clap(short = 'x', long)]
    suffix: Option<String>,

    /// Number of threads to use (defaults to all available)
    #[clap(short, long)]
    threads: Option<usize>,

    /// Print stats every N seconds
    #[clap(short, long, default_value = "5")]
    stats_interval: u64,
}

// Stats structure to track the progress
struct Stats {
    attempts: u64,
    started_at: Instant,
}

impl Stats {
    fn new() -> Self {
        Stats {
            attempts: 0,
            started_at: Instant::now(),
        }
    }

    fn increment(&mut self, count: u64) {
        self.attempts += count;
    }

    fn print(&self) {
        let elapsed = self.started_at.elapsed().as_secs();
        if elapsed > 0 {
            let rate = self.attempts as f64 / elapsed as f64;
            println!(
                "Attempts: {}, Time: {}s, Rate: {:.2} addr/s",
                self.attempts, elapsed, rate
            );
        }
    }
}

fn generate_p2wpkh_address(secp: &Secp256k1<bitcoin::secp256k1::All>) -> (KeyPair, String) {
    // Generate a key pair
    let key_pair = KeyPair::new(secp, &mut rand::thread_rng());
    let public_key = PublicKey::new(key_pair.public_key());
    
    // Create a P2WPKH address (bc1q format)
    let address = Address::p2wpkh(&public_key, Network::Bitcoin)
        .expect("Failed to create P2WPKH address");
    
    // Return the address string representation
    (key_pair, address.to_string())
}

fn check_address(address: &str, prefix_pattern: &str, suffix_pattern: Option<&str>) -> bool {
    // bc1q addresses are more than 14 characters
    if address.len() <= 4 || !address.starts_with("bc1q") {
        return false;
    }

    // Check if the prefix pattern appears right after bc1q
    let prefix_match = address[4..].starts_with(prefix_pattern);

    // If there's no suffix pattern, just return the prefix match result
    if let Some(suffix) = suffix_pattern {
        // Check both prefix and suffix
        prefix_match && address.ends_with(suffix)
    } else {
        // Only check prefix
        prefix_match
    }
}

fn main() {
    let args = Args::parse();
    
    // Prepare the patterns
    let prefix_pattern = Arc::new(args.pattern.to_lowercase());
    let suffix_pattern = Arc::new(args.suffix.map(|s| s.to_lowercase()));
    
    // Set the number of threads to use
    if let Some(threads) = args.threads {
        rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build_global()
            .unwrap();
    }
    
    println!("Starting Bitcoin bc1q vanity address generator");
    println!("Looking for pattern: '{}' (after bc1q)", prefix_pattern);
    if let Some(suffix) = &*suffix_pattern {
        println!("And ending with: '{}'", suffix);
    }
    println!("Press Ctrl+C to stop...");
    
    // Initialize statistics
    let stats = Arc::new(Mutex::new(Stats::new()));
    let found: Arc<Mutex<Option<(String, String)>>> = Arc::new(Mutex::new(None));
    let start_time = Instant::now();
    let stats_interval = Duration::from_secs(args.stats_interval);
    let last_stats_print = Arc::new(Mutex::new(Instant::now()));
    
    // Start the search in parallel
    rayon::scope(|s| {
        for thread_id in 0..rayon::current_num_threads() {
            let stats = Arc::clone(&stats);
            let found = Arc::clone(&found);
            let prefix_pattern = Arc::clone(&prefix_pattern);
            let suffix_pattern = Arc::clone(&suffix_pattern);
            let last_stats_print = Arc::clone(&last_stats_print);
            
            s.spawn(move |_| {
                let secp = Secp256k1::new();
                let batch_size = 1000; // Update stats after checking this many addresses
                
                while found.lock().unwrap().is_none() {
                    // Generate address in batches for better performance
                    for _ in 0..batch_size {
                        let (key_pair, address) = generate_p2wpkh_address(&secp);
                        
                        if check_address(&address, &prefix_pattern, suffix_pattern.as_deref()) {
                            let private_key = key_pair.secret_key().display_secret().to_string();
                            let result = (private_key, address.clone());
                            
                            let mut found_guard = found.lock().unwrap();
                            *found_guard = Some(result);
                            break;
                        }
                    }
                    
                    // Update global stats occasionally
                    let mut stats_guard = stats.lock().unwrap();
                    stats_guard.increment(batch_size);
                    drop(stats_guard);
                    
                    // Print stats at regular intervals
                    let mut last_print = last_stats_print.lock().unwrap();
                    if last_print.elapsed() >= stats_interval {
                        stats.lock().unwrap().print();
                        *last_print = Instant::now();
                    }
                    
                    // Check if we need to stop
                    if found.lock().unwrap().is_some() {
                        break;
                    }
                }
                
                println!("Thread {} finished", thread_id);
            });
        }
    });
    
    // Print the result
    let result = found.lock().unwrap().clone();
    if let Some((private_key, address)) = result {
        let elapsed = start_time.elapsed();
        let attempts = stats.lock().unwrap().attempts;
        
        println!("\n🎉 Found matching address after {} attempts in {:.2?}!", attempts, elapsed);
        println!("Address:     {}", address);
        println!("Private key: {}", private_key);
    }
}

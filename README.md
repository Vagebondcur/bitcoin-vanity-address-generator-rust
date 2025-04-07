# Bitcoin Vanity Address Generator

A Rust program that generates Bitcoin vanity addresses with the "bc1q" prefix (Bech32 segwit addresses) followed by a custom pattern and/or ending with a specific suffix.

## Features

- Generates Bitcoin segwit (bc1q) addresses
- Customize both the beginning (after bc1q) and end of the address
- Multi-threaded processing for maximum performance
- Real-time statistics (addresses per second)
- Automatic saving of the private key when a match is found

## Requirements

- Rust and Cargo (latest stable version recommended)

## Installation

Clone the repository and build the project:

```bash
git clone https://github.com/Vagebondcur/bitcoin-vanity-address-generator-rust
cd bitcoin-vanity-address-generator-rust
cargo build --release
```

The compiled binary will be located at `./target/release/vanity-address-rust`.

## Usage

```bash
# Generate an address with "coffee" after the bc1q prefix
./target/release/vanity-address-rust --pattern coffee

# Use 8 threads and print stats every 2 seconds
./target/release/vanity-address-rust --pattern coffee --threads 8 --stats-interval 2

# Generate a simple address with "a" after the bc1q prefix
./target/release/vanity-address-rust --pattern a

# Generate an address with a specific suffix
./target/release/vanity-address-rust --pattern a --suffix xyz

# Generate an address with both specific beginning and ending
./target/release/vanity-address-rust --pattern coffee --suffix 1337
```

### Command-line Options

- `--pattern, -p`: Pattern to search for after the bc1q prefix
- `--suffix, -x`: Pattern that the address should end with (optional)
- `--threads, -t`: Number of threads to use (defaults to all available)
- `--stats-interval, -s`: Print stats every N seconds (default: 5)

## Performance Notes

- On a modern CPU, this tool can check hundreds of thousands of addresses per second
- The search time increases exponentially with the pattern length
- Short patterns (1-3 characters) typically complete within seconds
- Longer patterns (4+ characters) may take minutes, hours, or longer depending on length
- Combining both prefix and suffix patterns will significantly increase search time
- Performance sample: ~300,000 addresses/second on a modern multi-core CPU

Example output:
```
Starting Bitcoin bc1q vanity address generator
Looking for pattern: 'a' (after bc1q)
And ending with: 'z'
Press Ctrl+C to stop...

🎉 Found matching address after 14000 attempts in 34.97ms!
Address:     bc1qacxa60q3n93gtjedj9wrl8mrt9e0pku89agguz
Private key: 33442b362cb3cfaac9cc76adfc9f4a8d78093446d0268807b0cd48cde4c82bb6
```
Note: do not use the private key from above, it is for demonstration purposes only, consider it compromised.

## Security Note

Always store your private keys securely. The private key is displayed once a matching address is found. You should immediately secure this information if you plan to use the address.

## How It Works

1. Generates random private/public key pairs
2. Converts the public key to a Bitcoin P2WPKH segwit (bc1q) address
3. Checks if the generated address contains the desired pattern after the bc1q prefix
4. If specified, also checks if the address ends with the desired suffix
5. Uses Rayon for parallel processing across multiple CPU cores
6. Provides real-time statistics on address generation rate

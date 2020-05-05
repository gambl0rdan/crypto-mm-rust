# crypto-mm-rust

Connects to the Blockchain exchange via their websockets API.
This implementation tries to use the Standard Library as much as possible but makes use of many I/O and serialisation crates such as tokio, websocket and serde_json. See Cargo.toml for full dependency list.

**Full API Description** https://exchange.blockchain.com/api/


## Features covered
* Secured TCP connection on a synchronous channel, with authentication using a user's API secret (See section "Getting and Reading your API Key")
* Checks balances for user via authenticated chanels in chosen local currency
* Subscribe to a variety of market data using the anonymous channels, e.g., "l2" (Level 2 order book) "tickers" showing high, low, close etc for a symbol; and "prices" showing last traded price for a symbol
* Simple example of a 'strategy' whereby the application will submit a Limit Order if the last price < low price (up to a limit per session)
* Cancels any open/non-completely filled orders on startup automatically


## How to Run
1. If you're new to Rust, great! Take a look at the getting starting guide at: https://www.rust-lang.org/learn/get-started
2. Most Rust developers use Cargo as a build and dependency manager, make sure you have this...
3. Check out the code and ensure your shell is in the root directory
4. Run: cargo build
5. Run: cargo run

## Getting and Reading your API Key
1. Follow the instructions on the API documentation or go to: https://exchange.blockchain.com/settings/api
2. Copy and paste your API secret into a file called .API_SECRET in your root (i.e., ABOVE src) directory. You will need to create this file yourself. ALWAYS check you do not commit your .API_SECRET file to a code repository in chosen local currency
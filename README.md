# crypto-mm-rust

Connects to the Blockchain exchange via their websockets API.
This implementation tries to use the Standard Library as much as possible but makes use of many I/O and serialisation crates such as tokio, websocket and serde_json.

**Full API Description** https://exchange.blockchain.com/api/


## Features covered
* Secured TCP connection on a synchronous channel, with authentication using a user's API secret (See bottom of MD)
* Checks balances for user via authenticated chanels in chosen local currency
* Subscribe to a variety of market data using the anonymous channels, e.g., "l2" (Level 2 order book) "tickers" showing high, low, close etc for a symbol; and "prices" showing last traded price for a symbol
* Simple example of a 'strategy' whereby the application will submit a Limit Order if the last price < low price.
* Cancels any open/non-completely filled orders on startup automatically

## Getting and Reading your API Key
1. Follow the instructions on the API documentation or go to: https://exchange.blockchain.com/settings/api
2. Copy and paste your API secret into a file called .API_SECRET in your root (src) directory. You will need to create this file yourself. ALWAYS check you do not commit your .API_SECRET file to a code repository in chosen local currency
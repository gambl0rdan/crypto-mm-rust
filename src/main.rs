extern crate futures;
extern crate serde_json;
extern crate uuid;
extern crate tokio;
extern crate websocket;

use serde_json::json;
use uuid::Uuid;
use websocket::{ClientBuilder, OwnedMessage, Message, sync::Client, sync::stream::TlsStream};

use std::collections::HashMap;
#[macro_use]
use std::format;
use std::net::TcpStream;

pub mod orders;
use orders::{ExchangeOrderBuilder, ExchangeOrder, OrderEngine, Task, TickPrice, CcyPair, USD_BTC, GBP_BTC};

fn main(){
        run_for_blockchain(String::from("GBP"));
}

fn connect_sub() -> String {
    let secret = std::fs::read_to_string(".API_SECRET")
        .expect("Something went wrong reading the API secret file");

    let sub = json!({
        "token": secret, 
        "action" : "subscribe", 
        "channel": "auth"
    });
    sub.to_string()
}

fn subscribe_to_channel(channel : &str, 
    subscriptions : &HashMap<&'static str, String>,
    client: &mut Client<TlsStream<TcpStream>>

) {
    println!("Attempt to subscribe to channel {}", channel);
    let message = Message::text(subscriptions.get(channel).unwrap());
    client.send_message(&message).unwrap();
}

fn build_subscriptions(base_ccy : String) -> HashMap<&'static str, String> {
    let mut scores = HashMap::new();
    let sub_prices : serde_json::Value = json!({
        "action": "subscribe",
        "channel": "prices",
        "symbol": format!("BTC-{}", base_ccy),
        "granularity":  3600 //60 //86400
    });

    let sub_ticker : serde_json::Value = json!({
        "action": "subscribe",
        "channel": "ticker",
        "symbol": format!("BTC-{}", base_ccy)
    });

    let sub_balances : serde_json::Value = json!({
        "action": "subscribe",
        "channel": "balances",
        "local_currency": format!("{}", base_ccy)
    });

    let sub_l2_order_book : serde_json::Value = json!({
        "action": "subscribe",
        "channel": "l2",
        "symbol": format!("BTC-{}", base_ccy)
    });
        
    let sub_trading : serde_json::Value = json!({
        "action": "subscribe",
        "channel": "trading"
    });

    scores.insert("prices", sub_prices.to_string());
    scores.insert("ticker", sub_ticker.to_string());
    scores.insert("balances", sub_balances.to_string());
    scores.insert("l2", sub_l2_order_book.to_string());
    scores.insert("trading", sub_trading.to_string());
    scores
}

fn run_for_blockchain(base_ccy : String) {
    let ccy_pair = if "GBP" == base_ccy {GBP_BTC} else {USD_BTC};
    let mut engine : OrderEngine::<CcyPair> = OrderEngine::new(ccy_pair); 
    let subscriptions : HashMap<&str, String> = build_subscriptions(base_ccy);

    let url = "wss://ws.prod.blockchain.info/mercury-gateway/v1/ws".to_string();
    let mut builder = ClientBuilder::new(&url)
        .unwrap()
        .origin("https://exchange.blockchain.com".to_owned());
    
    println!("Try connect syncronously to {}", &url);
    
    let mut client = builder.connect_secure(None).unwrap();
    let mut client_in_loop = Box::new(builder.connect_secure(None).unwrap());

    println!("{}", "Connection successful");
    
    let message = Message::text(connect_sub());
    client.send_message(&message).unwrap();
    
    subscribe_to_channel("prices", &subscriptions, &mut client); 
    subscribe_to_channel("ticker", &subscriptions, &mut client);
    subscribe_to_channel("balances", &subscriptions, &mut client);
    subscribe_to_channel("l2", &subscriptions, &mut client);
    subscribe_to_channel("trading", &subscriptions, &mut client);

    client_in_loop.send_message(&message).unwrap();
    subscribe_to_channel("trading", &subscriptions, &mut client_in_loop);
    
    for message in client.incoming_messages() { 
		let message = match message {
			Ok(message) => message,
			Err(e) => {
				println!("Error: {:?}", e);
				break;
			}
		};
		match message {
			OwnedMessage::Text(txt) => {
                match handle_api_response(&txt) {
                    Some(tasks) => {
                        for input_task in tasks{
                            
                            let task = engine.decide(input_task);
                            // println!("Task is {}", task.action);
                            
                            if task.is_some(){
                                let task = task.unwrap();
                                match task.action {
                                    "cancel" => {
                                        for order_id in task.order_ids.unwrap() {
                                            cancel_order(order_id, &mut client_in_loop).unwrap();
                                        }
                                    },
                                    "new_order" => {
                                        // max_orders
                                        engine.increment_order_count();
                                        create_and_submit_order(task.tick_last_price.unwrap(), &mut client_in_loop).unwrap();
                                    },
                                    _ => break 
                                };
                            } else{
                                break
                            }
                        }
                    },
                    None => (),
                }
			}
			OwnedMessage::Close(_) => {
				// let _ = sender.send_message(&Message::close());
				break;
			}
			OwnedMessage::Ping(data) => {
				// sender.send_message(&OwnedMessage::Pong(data)).unwrap();
			}
			_ => (),
		}
    }
}

fn handle_api_response(resp : &String) -> Option<Vec<Task>> {
    let parsed : serde_json::Value = match serde_json::from_str(resp) {
        Ok(val) => val,
        Err(e) => {
            println!("Conversion failed: {:?}", e);
            return None;
        }
    };

    let channel = parsed["channel"].as_str().unwrap();  
    let event = parsed["event"].as_str().unwrap();
    if "subscribed" == event{
        println!("{}", parsed);
        return None;
    } 

    if "ticker" == channel {
        return handle_ticker(parsed);
    }
    else  if "prices" == channel {
        return handle_prices(parsed);
    }
    else if channel == "balances"{
        println!("Latest balances: {}", parsed);
    }
    else if channel == "l2"{
        handle_l2(parsed).unwrap();
    }
    else if channel == "trading" {
        return handle_trading(parsed);
    }
    else{
        println!("Other channel: {}", parsed);
    }

    None
}

fn create_order(side : &str, price : f64, order_qty : f64, symbol : &str) -> ExchangeOrder { 
    let order_id = Uuid::new_v4().to_string();
    let order_id = &order_id[0..20];

    ExchangeOrderBuilder::new()
        .side(side)
        .price(price)
        .orderQty(order_qty)
        .symbol(symbol)
        .clOrdID(order_id)
        .finalize()
}

fn create_and_submit_order(price : f64, client: &mut Client<TlsStream<TcpStream>>) -> serde_json::Result<()> { 
    let order : ExchangeOrder = create_order("buy", price, 0.010, "BTC-GBP");
    let order_json = serde_json::to_string(&order)?;

    println!("Submitting order {}", order_json);

    let message = Message::text(order_json);
    client.send_message(&message).unwrap();

    Ok(())
}

fn cancel_order(order_id :String, client: &mut Client<TlsStream<TcpStream>>) -> serde_json::Result<()> {
    let cancel = json! ({
        "action": "CancelOrderRequest",
        "channel": "trading",
        "orderID": order_id
      });

      let cancel_json = serde_json::to_string(&cancel)?;

      println!("Cancelling order {}", cancel_json);
      
      let message = Message::text(cancel_json);
      client.send_message(&message).unwrap();
      
      Ok(())
}

fn handle_trading(value : serde_json::Value, ) -> Option<Vec<Task>>{
    let mut order_ids : Vec<String> = vec![];

    let event = value["event"].as_str().unwrap();  
    
    if "snapshot" == event {
        println!("Open orders: {}", value["orders"]);
        match value["orders"].as_array() {
            Some(orders) => {
                for order in orders {
                    // println!("Open order: {}", order);
                    let order_id : String = order["orderID"].as_str().unwrap().to_owned();
                    order_ids.push(order_id); 
                }

            },
            None => println!("No active orders")
        };
    }

    if order_ids.len() > 0 {Some(vec![Task{action:"cancel", order_ids: Some(order_ids), tick_price: None, tick_l2: None, tick_last_price:None}])} else {None}   
}

fn handle_l2(value : serde_json::Value) -> serde_json::Result<()>{
    let tick_l2 : orders::OrderL2 = serde_json::from_value(value)?;
    
    //I've commented these out as the ticks are too frequent. Can do something smarter than just iterating though.
    // for b in tick_l2.bids{
    //     println!("{} {}", b.px, b.qty);
    // }

    // for a in tick_l2.asks{
    //     println!("{} {}", a.px, a.qty);
    // }

    Ok(())
}

fn handle_ticker(value : serde_json::Value) -> Option<Vec<Task>>{
    println!("Latest ticker tick is {}", value);
    match value["last_trade_price"].as_f64() {
        Some(px) => Some(vec![Task{action:"market_data", order_ids:None, tick_price:None, tick_l2:None, tick_last_price: Some(px)}]),
        None => None
    }
}

fn handle_prices(value : serde_json::Value) -> Option<Vec<Task>>{
    match value["price"].as_array() {
        Some(raw_prices) => {

            //timestamp, open, high, low, close, volume
            let price = TickPrice {
                timestamp: raw_prices[0].as_f64().unwrap(), 
                open: raw_prices[1].as_f64().unwrap(), 
                high: raw_prices[2].as_f64().unwrap(),
                low: raw_prices[3].as_f64().unwrap(), 
                close: raw_prices[4].as_f64().unwrap(), 
                volume: raw_prices[5].as_f64().unwrap()
            };
            
            println!("Price tick has following: high:{} low:{} open:{} close{}", price.high,price.low, price.open, price.close);
            
        },
        None => ()
    }

    Some(vec![Task{action:"market_data", order_ids:None, tick_price:None, tick_l2:None, tick_last_price:None}])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_api_secret_present() {
        connect_sub();
    }
}

#[macro_use]
use serde_json;
use serde::{Deserialize, Serialize};

pub const BTC_USD : CcyPair = CcyPair{ base: "USD", quoted: "BTC"};
pub const BTC_GBP : CcyPair = CcyPair{ base: "GBP", quoted: "BTC"};
    
pub struct TickPrice {
    pub timestamp : f64, 
    pub open : f64, 
    pub high : f64,
    pub low : f64, 
    pub close : f64, 
    pub volume : f64
}

pub struct Task {
    pub action : & 'static str,
    pub order_ids : Option<Vec<String>>,
    pub tick_price : Option<TickPrice>,
    pub tick_l2 : Option<OrderL2>,
    pub tick_last_price : Option<f64> 
}

#[derive(Serialize, Deserialize)]
pub struct OrderRow {
    num : i32,
    pub px : f64,
    pub qty : f64
}

#[derive(Serialize, Deserialize)]
pub struct OrderL2 {
    seqnum: i32,
    event: String,
    channel : String,
    symbol : String,
    pub bids: Vec<OrderRow>,
    pub asks: Vec<OrderRow>,
}

#[derive(Serialize, Deserialize)]
pub struct ExchangeOrder {
    action      : String,
    channel     : String,
    ordType     : String,
    timeInForce : String,
    orderQty    : f64,
    side        : String,
    price       : f64,
    symbol      : String,
    clOrdID     : String,
}

pub struct ExchangeOrderBuilder {
    action      : String,
    channel     : String,
    ordType     : String,
    timeInForce : String,
    orderQty    : f64,
    side        : String,
    price       : f64,
    symbol      : String,
    clOrdID     : String,
}

impl ExchangeOrderBuilder {
    pub fn new() -> ExchangeOrderBuilder {
        ExchangeOrderBuilder {
            action: String::from("NewOrderSingle"),
            channel: String::from("trading"), 
            ordType: String::from("limit"),
            timeInForce :String::from("GTC"),
            orderQty : 0.0,
            side: String::from("buy"),
            price : 0.0,
            symbol : String::from("BTC-USD"),
            clOrdID: String::from(""),
        }
    }

    pub fn ordType(&mut self, ordType: &str) -> &mut ExchangeOrderBuilder {
        self.ordType = String::from(ordType);
        self
    }

    pub fn orderQty(&mut self, orderQty: f64) -> &mut ExchangeOrderBuilder {
        self.orderQty = orderQty;
        self
    }

    pub fn side(&mut self, side: &str) -> &mut ExchangeOrderBuilder {
        self.side = String::from(side);
        self
    }

    pub fn clOrdID(&mut self, clOrdID: &str) -> &mut ExchangeOrderBuilder {
        self.clOrdID = String::from(clOrdID);
        self
    }

    pub fn price(&mut self, price: f64) -> &mut ExchangeOrderBuilder {
        self.price = price;
        self
    }

    pub fn symbol(&mut self, symbol: &str) -> &mut ExchangeOrderBuilder {
        self.symbol = String::from(symbol);
        self
    }

    pub fn finalize(&self) -> ExchangeOrder {
        ExchangeOrder {
            action: self.action.to_string(),
            channel: self.channel.to_string(), 
            ordType: self.ordType.to_string(),
            timeInForce : self.timeInForce.to_string(),
            orderQty : self.orderQty,
            side: self.side.to_string(),
            price : self.price,
            symbol : self.symbol.to_string(),
            clOrdID: self.clOrdID.to_string()}
    }
}

pub struct CcyPair<'a> {
    pub base: &'a str,
    pub quoted : &'a str,
}

pub struct OrderEngine<'a, CcyPair> {
    pub series_l2 : Vec<OrderL2>,
    pub series_prices : Vec<TickPrice>,
    pub serices_last_price : Vec<f64>,
    pub ccy_pair : &'a CcyPair,
    max_orders : u32,
    submitted_orders : u32
}

impl <'a, CcyPair> OrderEngine<'a, CcyPair> {
    
    pub fn new(ccy_pair: &'static CcyPair) -> OrderEngine<CcyPair> {
        OrderEngine {series_l2 : vec![],
            series_prices : vec![],
            serices_last_price : vec![],
            ccy_pair : ccy_pair,
            max_orders : 2,
            submitted_orders : 0
        }
    }

    pub fn increment_order_count(&mut self){
        self.submitted_orders  = &self.submitted_orders  + 1;
    }

    pub fn decide(&mut self, input : Task) -> Option<Task> {
        match input.action {
            "cancel" => return Some(input),
            "market_data" => {
                if input.tick_price.is_some() {self.series_prices.push(input.tick_price.unwrap())} else {()}; 
                if input.tick_l2.is_some() {self.series_l2.push(input.tick_l2.unwrap())} else {()}; 
                
                self.serices_last_price.last().and_then(|px| self.series_prices.last().and_then(|cls_px|  {
                    println!("Checking prices to decide to generate a new order for last px: {} and low px: {}", px, cls_px.low);
                    if px < &cls_px.low && self.submitted_orders  < self.max_orders  { 
                        return Some(Task{action:"new_order", order_ids:None, tick_price:None, tick_l2:None, tick_last_price:Some(px * 0.995)});
                    } else{
                        return None;
                    }
                }));
            },
            _ => return None 
        };
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_currencies() {
        const BTC_USD : CcyPair = CcyPair{ base: "USD", quoted: "BTC"};
        const BTC_GBP : CcyPair = CcyPair{ base: "GBP", quoted: "BTC"};

        assert_eq!(BTC_USD.base, "USD");
        assert_eq!(BTC_GBP.base, "GBP");
        assert_eq!(BTC_USD.quoted, "BTC");
        assert_eq!(BTC_GBP.quoted, "BTC");
    }
}
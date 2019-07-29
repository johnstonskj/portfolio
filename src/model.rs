use std::fs::File;
use std::io;
use std::io::prelude::*;

use serde::{Deserialize, Serialize};
use shellexpand;
use steel_cent::currency::{Currency, with_code};
use toml;

use fin_model::prelude::*;

// ------------------------------------------------------------------------------------------------
// Public Types
// ------------------------------------------------------------------------------------------------

#[derive(Debug)]
pub enum ModelError {
    PathError(String),
    FileError(io::Error),
    ParseError(toml::de::Error),
    WriteError(toml::ser::Error),
}

pub struct Portfolio {
    pub default_currency: Option<Currency>,
    pub items: Vec<Item>,
}

#[derive(Clone, Debug)]
pub enum Item {
    Watch(Symbol),
    Price(Symbol, Holding),
}

#[derive(Clone, Debug)]
pub struct Holding {
    pub quantity: u32,
    pub purchase_price: Money,
}

// ------------------------------------------------------------------------------------------------
// Private Types (serialization format)
// ------------------------------------------------------------------------------------------------

#[derive(Deserialize, Serialize, Clone)]
struct SerializedMoney {
    pub major: i32,
    pub minor: i32,
    pub currency_code: String,
}

#[derive(Deserialize, Serialize, Clone)]
struct SerializedHolding {
    pub symbol: Symbol,
    pub watch_only: bool,
    pub quantity: Option<u32>,
    pub purchase_price: Option<SerializedMoney>,
}

#[derive(Deserialize, Serialize, Clone)]
struct SerializedPortfolio {
    pub default_currency: Option<String>,
    holdings: Vec<SerializedHolding>,
}

// ------------------------------------------------------------------------------------------------
// Public Functions
// ------------------------------------------------------------------------------------------------

pub fn default_file_name() -> String {
    shellexpand::tilde("~/portfolio.toml").to_string()
}

pub fn read_file(file_name: Option<String>) -> Result<Portfolio, ModelError> {
    let file_name = file_name.unwrap_or(default_file_name());
    info!("model::read_file {}", file_name);

    let mut f = match File::open(file_name.to_string()) {
        Ok(handle) => handle,
        Err(_) => return Err(ModelError::PathError(file_name)),
    };

    let mut buffer = String::new();
    match f.read_to_string(&mut buffer) {
        Ok(_) => (),
        Err(err) => return Err(ModelError::FileError(err)),
    };

    let serialized: SerializedPortfolio = match toml::from_str(&buffer.as_str()) {
        Ok(portfolio) => portfolio,
        Err(parse_err) => return Err(ModelError::ParseError(parse_err)),
    };

    Ok(Portfolio {
        default_currency: match serialized.default_currency {
            Some(c) => Some(with_code(&c).unwrap()),
            None => None
        },
        items: serialized
            .holdings
            .iter()
            .map(|holding| {
                if holding.watch_only {
                    Item::Watch(holding.symbol.to_string())
                } else {
                    let purchase_price = holding.clone().purchase_price.unwrap();
                    let currency: Currency = with_code(&purchase_price.currency_code).unwrap();
                    let price =
                        Money::of_major_minor(currency, purchase_price.major, purchase_price.minor);
                    Item::Price(
                        holding.symbol.to_string(),
                        Holding {
                            quantity: holding.quantity.unwrap(),
                            purchase_price: price,
                        },
                    )
                }
            })
            .collect(),
    })
}

pub fn write_file(file_name: Option<String>, portfolio: &Portfolio) -> Result<(), ModelError> {
    let file_name = file_name.unwrap_or(default_file_name());
    info!("model::write_file {}", file_name);

    let serializable = SerializedPortfolio {
        default_currency: match portfolio.default_currency {
            Some(c) => Some(c.code()),
            None => None
        },
        holdings: portfolio
            .items
            .iter()
            .map(|item| match item {
                Item::Watch(symbol) => SerializedHolding {
                    symbol: symbol.to_string(),
                    watch_only: true,
                    quantity: None,
                    purchase_price: None,
                },
                Item::Price(symbol, holding) => SerializedHolding {
                    symbol: symbol.to_string(),
                    watch_only: false,
                    quantity: Some(holding.quantity),
                    purchase_price: Some(SerializedMoney {
                        major: holding.purchase_price.major_part(),
                        minor: holding.purchase_price.minor_part(),
                        currency_code: holding.purchase_price.currency.code(),
                    }),
                },
            })
            .collect(),
    };
    let toml = match toml::to_string(&serializable) {
        Ok(data) => data,
        Err(err) => return Err(ModelError::WriteError(err)),
    };

    let mut f = match File::create(file_name.to_string()) {
        Ok(handle) => handle,
        Err(_) => return Err(ModelError::PathError(file_name)),
    };

    match f.write_all(toml.as_bytes()) {
        Ok(()) => Ok(()),
        Err(err) => Err(ModelError::FileError(err)),
    }
}

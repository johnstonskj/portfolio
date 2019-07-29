#[macro_use]
extern crate log;
extern crate flexi_logger;

use chrono::Local;
use fin_model::prelude::*;
use fin_model::provider::Provider;
use fin_iex::IEXProvider;
use steel_cent::currency::with_code;

use portfolio::display::DATE_FMT;
use portfolio::holdings::show_holdings;
use portfolio::model;
use portfolio::model::{Holding, Item, ModelError, Portfolio};
use portfolio::show::show_portfolio;
use portfolio::watch::watch_portfolio;

#[derive(Debug)]
enum Command {
    Show,
    Watch,

    Holdings,
    Add(Symbol, Option<String>, Option<String>, Option<String>),
    Remove(Symbol),

    None,
}

fn main() {
    flexi_logger::Logger::with_env().start().unwrap();
    info!("folio::main started");

    let cmd = handle_args();

    if let Command::None = cmd {
        println!("Pick a [valid] command");
    } else {
        let default_currency = with_code("USD").unwrap();

        if let Some(portfolio) = get_portfolio() {
            match cmd {
                Command::Show | Command::Watch => {
                    let provider = match IEXProvider::new() {
                        Ok(provider) => provider,
                        Err(RequestError::ConfigurationError(err)) => {
                            println!("Error configuring provider: {}", err);
                            return ();
                        }
                        Err(err) => {
                            println!("Unknown error from provider: {:?}", err);
                            return ();
                        }
                    };

                    match cmd {
                        Command::Show => show_portfolio(portfolio, provider),
                        Command::Watch => watch_portfolio(portfolio),
                        _ => (),
                    }
                },
                Command::Holdings | Command::Add(_, _, _, _) | Command::Remove(_) => {
                    match cmd {
                        Command::Holdings =>
                            show_holdings(portfolio),
                        Command::Add(s, p, q, d) => {
                            let p = match p {
                                Some(p) => {
                                    let ps: Vec<&str> = p.split(".").collect();
                                    if ps.len() == 1 {
                                        Money::of_major_minor(
                                            portfolio.default_currency.unwrap_or(default_currency),
                                            ps.get(0).unwrap().parse::<i32>().unwrap(),
                                            0)
                                    } else if ps.len() == 2 {
                                        Money::of_major_minor(
                                            portfolio.default_currency.unwrap_or(default_currency),
                                            ps.get(0).unwrap().parse::<i32>().unwrap(),
                                            ps.get(1).unwrap().parse::<i32>().unwrap())
                                    } else {
                                        warn!("Could not parse price {}, using zero", p);
                                        Money::zero(default_currency)
                                    }
                                },
                                None => Money::zero(default_currency)
                            };
                            let q = match q {
                                Some(q) => match q.parse::<u32>() {
                                    Ok(n) => n,
                                    Err(_) => {
                                        warn!("Could not parse quantity {}, using zero", q);
                                        0
                                    },
                                }
                                None => 0,
                            };
                            let d = match d {
                                Some(d) => match Date::parse_from_str(&d, DATE_FMT) {
                                    Ok(d) => Some(d),
                                    Err(_) => {
                                        warn!("Could not parse date {}, using today", d);
                                        Some(Local::today().naive_local())
                                    },
                                }
                                None => None,
                            };
                            let new_item = Item::Price(
                                s,
                                Holding {
                                    quantity: q,
                                    purchase_price: p,
                                    purchase_date: d,
                                }
                            );
                            let new_portfolio = Portfolio {
                                default_currency: portfolio.default_currency,
                                items: portfolio.items.into_iter().chain(vec![new_item]).collect()
                            };
                            match model::write_file(None, &new_portfolio) {
                                Err(err) => {
                                    println!("Failed to save portfolio file, error: {:?}", err);
                                    ()
                                },
                                Ok(_) => (),
                            }
                        },
                        Command::Remove(symbol) => {
                            let new_portfolio = Portfolio {
                                default_currency: portfolio.default_currency,
                                items: portfolio.items.iter().filter(|item|
                                    match item {
                                        Item::Watch(s) | Item::Price(s, _) => *s != symbol,
                                    }
                                ).cloned().collect()
                            };
                            match model::write_file(None, &new_portfolio) {
                                Err(err) => {
                                    println!("Failed to save portfolio file, error: {:?}", err);
                                    ()
                                },
                                Ok(_) => (),
                            }
                        },
                        _ => (),
                    }
                },
                Command::None => (),
            }
        }
    }
}

// ------------------------------------------------------------------------------------------------

extern crate clap;

use clap::{App, Arg, SubCommand};

fn handle_args() -> Command {
    let matches = App::new("folio")
        .about("Portfolio Manager")
        .version("v1.0-pre")
        .subcommand(
            SubCommand::with_name("show")
                .about("Show quotes for all portfolio symbols")
        )
        .subcommand(
            SubCommand::with_name("watch")
                .about("Watch quotes for portfolio symbols")
                .arg(
                    Arg::with_name("delay")
                        .short("d")
                        .long("refresh-delay")
                        .takes_value(true)
                        .help("Delay between refreshes"),
                )
        )
        .subcommand(
            SubCommand::with_name("holdings")
                .about("Show all holdings in current portfolio")
        )
        .subcommand(
            SubCommand::with_name("add")
                .about("Add a symbol to the portfolio")
                .arg(
                    Arg::with_name("quantity")
                        .short("q")
                        .long("quantity")
                        .takes_value(true)
                        .help("The quantity of this security you hold"),
                )
                .arg(
                    Arg::with_name("price")
                        .short("p")
                        .long("purchase-price")
                        .takes_value(true)
                        .help("The purchase price of the security"),
                )
                .arg(
                    Arg::with_name("date")
                        .short("d")
                        .long("purchase-date")
                        .takes_value(true)
                        .help("The purchase date of the security (YYYY-MM-DD)"),
                )
                .arg(
                    Arg::with_name("symbol")
                        .help("The security symbol")
                        .required(true)
                        .index(1),
                ),
        )
        .subcommand(
            SubCommand::with_name("delete")
                .about("Delete a symbol from the portfolio")
                .arg(
                    Arg::with_name("symbol")
                        .help("The security symbol")
                        .required(true)
                        .index(1),
                )
        )
        .get_matches();

    match matches.subcommand() {
        ("show", Some(_)) => Command::Show,
        ("watch", Some(_)) => Command::Watch,

        ("holdings", Some(_)) => Command::Holdings,
        ("add", Some(matches)) => Command::Add(
            matches.value_of("symbol").unwrap().to_string(),
            match matches.value_of("price") {
                Some(s) => Some(s.to_string()),
                None => None,
            },
            match matches.value_of("quantity") {
                Some(s) => Some(s.to_string()),
                None => None,
            },
            match matches.value_of("date") {
                Some(s) => Some(s.to_string()),
                None => None,
            },
        ),
        ("delete", Some(matches)) => Command::Remove(
            matches.value_of("symbol").unwrap().to_string()
        ),

        _ => {
            Command::None
        }
    }
}

// ------------------------------------------------------------------------------------------------

fn get_portfolio() -> Option<Portfolio> {
    let result = model::read_file(None);
    match result {
        Ok(portfolio) => Some(portfolio),
        Err(ModelError::PathError(_)) => {
            println!(
                "No portfolio file exists, creating an example in {}",
                model::default_file_name()
            );
            let example = Portfolio {
                default_currency: with_code("USD"),
                items: vec![
                    Item::Watch("AAPL".to_string()),
                    Item::Watch("AMZN".to_string()),
                    Item::Watch("MSFT".to_string()),
                ],
            };
            match model::write_file(None, &example) {
                Err(err) => {
                    println!("Failed to create example portfolio file, error: {:?}", err);
                    None
                },
                Ok(_) => Some(example),
            }
        }
        Err(err) => {
            println!(":( {:?}", err);
            None
        },
    }
}
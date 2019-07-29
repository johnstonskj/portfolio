use std::collections::HashMap;

use fin_model::prelude::*;
use fin_model::quote::{FetchPriceQuote, Quote};
use num_format::SystemLocale;
use prettytable::{Attr, Table};

use crate::display::*;
use crate::model::{Item, Portfolio};

pub fn show_portfolio<T: FetchPriceQuote>(portfolio: Portfolio, provider: T) {
    let locale: SystemLocale = SystemLocale::default().unwrap();
    let mut table = Table::new();
    let mut quote_cache: HashMap<Symbol, Quote> = HashMap::new();
    table.set_titles(row!["Symbol", "Price", "Change", "Open", "Low", "High", "Close", "Volume", "Purchased", "Quantity", "Value"]);
    for item in portfolio.items {
        let symbol = item_symbol(&item).to_string();
        if !quote_cache.contains_key(&symbol) {
            let quote = provider.real_time(symbol.to_string());
            match quote {
                Ok(quote) => {
                    quote_cache.insert(symbol.to_string(), quote);
                },
                Err(err) => {
                    println!("Error retrieving quote for {}: {:?}", symbol, err);
                    return;
                }
            }
        };
        add_item(&mut table, &item, &quote_cache.get(&symbol).unwrap(), &locale);
    }
    table.printstd();
}


fn add_item(table: &mut Table, item: &Item, quote: &Quote, locale: &SystemLocale) {
    match item {
        Item::Watch(s) =>
            table.add_row(row![
            s,
            // The following from Quote
            price_cell(quote.data.latest.price),
            change_cell(quote),
            if let Some(range) = &quote.data.range { price_cell(range.open) } else { default_cell() },
            if let Some(range) = &quote.data.range { price_cell(range.low) } else { default_cell() },
            if let Some(range) = &quote.data.range { price_cell(range.high) } else { default_cell() },
            if let Some(range) = &quote.data.range { price_cell(range.close) } else { default_cell() },
            if let Some(range) = &quote.data.range { number_cell_or(range.volume, &locale, default_cell()) } else { default_cell() },
            // The following from Holding
            default_cell(), default_cell(), default_cell()]),
        Item::Price(s, h) =>
            table.add_row(row![
            s,
            // The following from Quote
            price_cell(quote.data.latest.price),
            change_cell(quote),
            if let Some(range) = &quote.data.range { price_cell(range.open) } else { default_cell() },
            if let Some(range) = &quote.data.range { price_cell(range.low) } else { default_cell() },
            if let Some(range) = &quote.data.range { price_cell(range.high) } else { default_cell() },
            if let Some(range) = &quote.data.range { price_cell(range.close) } else { default_cell() },
            if let Some(range) = &quote.data.range { number_cell_or(range.volume, &locale, default_cell()) } else { default_cell() },
            // The following from Holding
            price_cell(h.purchase_price).with_style(Attr::Bold),
            number_cell(h.quantity as i64, &locale).with_style(Attr::Bold),
            // (quote.data.latest.price - h.purchase_price) * h.quantity
            price_cell((quote.data.latest.price - h.purchase_price) * h.quantity as i32).with_style(Attr::Bold)]),
    };
}

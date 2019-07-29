use std::collections::HashMap;

use fin_model::prelude::*;
use fin_model::quote::{FetchPriceQuote, Quote};
use num_format::{SystemLocale, ToFormattedString};
use prettytable::{Attr, Cell, color, Table};
use prettytable::format::Alignment;
use steel_cent::formatting::us_style;

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

fn item_symbol(item: &Item) -> String {
    match item {
        Item::Watch(s) | Item::Price(s, _) => s.to_string(),
    }
}

fn default_cell() -> Cell {
    Cell::new_align("-", Alignment::CENTER)
}

fn change_cell(quote: &Quote) -> Cell {
    match (quote.data.latest.change, quote.data.latest.percentage) {
        (Some(change), Some(percent)) => {
            let change_str = if change.major_part().is_positive() {
                format!("{}", us_style().display_for(&change))
            } else {
                format!("{}", us_style().display_for(&change))
            };
            let value = format!(
                "{} {}{}%",
                change_str,
                if change.minor_amount().is_positive() {
                    "↑"
                } else {
                    "↓"
                },
                percent.abs(),
            );
            if change.minor_amount().is_positive() {
                Cell::new_align(&value, Alignment::RIGHT)
                    .with_style(Attr::ForegroundColor(color::GREEN))
            } else {
                Cell::new_align(&value, Alignment::RIGHT)
                    .with_style(Attr::ForegroundColor(color::RED))
            }
        },
        (_, _) => default_cell()
    }
}

fn price_cell(m: Money) -> Cell {
    Cell::new_align(&format!("{}", us_style().display_for(&m)), Alignment::RIGHT)
}

fn number_cell(i: i64, locale: &SystemLocale) -> Cell {
    Cell::new_align(&i.to_formatted_string(locale), Alignment::RIGHT)
}

fn price_cell_or(m: Option<Money>, default: Cell) -> Cell {
    match m {
        Some(v) => price_cell(v),
        None => default,
    }
}
fn number_cell_or(i: Option<u64>, locale: &SystemLocale, default: Cell) -> Cell {
    match i {
        Some(v) => number_cell(v as i64, locale),
        None => default,
    }
}


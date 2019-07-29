use fin_model::prelude::*;
use fin_model::quote::Quote;

use num_format::{SystemLocale, ToFormattedString};
use prettytable::{Attr, Cell, color};
use prettytable::format::Alignment;
use steel_cent::formatting::{format, us_style};

use crate::model::Item;

pub const DATE_FMT: &'static str = "%Y-%m-%d";

pub fn bold(cell: Cell) -> Cell {
    cell.with_style(Attr::Bold)
}

pub fn default_cell() -> Cell {
    Cell::new_align("-", Alignment::CENTER)
}

pub fn price_cell(value: Money) -> Cell {
    Cell::new_align(&format(us_style(), &value), Alignment::RIGHT)
}

pub fn price_cell_or(value: Option<Money>, default: Cell) -> Cell {
    match value {
        Some(value) => price_cell(value),
        None => default,
    }
}

pub fn number_cell(value: i64, locale: &SystemLocale) -> Cell {
    Cell::new_align(&value.to_formatted_string(locale), Alignment::RIGHT)
}

pub fn number_cell_or(value: Option<u64>, locale: &SystemLocale, default: Cell) -> Cell {
    match value {
        Some(value) => number_cell(value as i64, locale),
        None => default,
    }
}

pub fn item_symbol(item: &Item) -> String {
    match item {
        Item::Watch(s) | Item::Price(s, _) => s.to_string(),
    }
}

pub fn change_string(change: &Money, percentage: &f64) -> String {
    let change_str= format(us_style(), change);
    format!(
        "{} {}{}%",
        change_str,
        if change.minor_amount().is_positive() {
            "↑"
        } else {
            "↓"
        },
        percentage.abs(),
    )
}

pub fn change_cell(quote: &Quote) -> Cell {
    match (quote.data.latest.change, quote.data.latest.percentage) {
        (Some(change), Some(percentage)) => {
            let value = change_string(&change, &percentage);
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


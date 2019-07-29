use num_format::SystemLocale;
use prettytable::Table;

use crate::display::*;
use crate::model::*;

pub fn show_holdings(portfolio : Portfolio) {
    let locale: SystemLocale = SystemLocale::default().unwrap();
    let mut table = Table::new();
    table.set_titles(row!["Symbol", "Purchase Date", "Purchase Price", "Quantity"]);
    for item in &portfolio.items {
        match item {
            Item::Price(s, h) => {
                table.add_row(row![
                    s,
                    default_cell(),
                    price_cell(h.purchase_price),
                    number_cell(h.quantity as i64, &locale),
                ]);
                ()
            },
            _ => (),
        }
    }
    table.printstd();

    let watching: Vec<String> = portfolio.items
        .iter()
        .filter(|item| match item { Item::Watch(_) => true, _ => false } )
        .map(|item| match item {
            Item::Watch(s) => s.to_string(),
            _ => "#error#".to_string()
        })
        .collect();

    println!("Also watching: {}", watching.join(", "));
}


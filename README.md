# Crate portfolio

This crate provides a single binary, `folio`, which uses the 
[rust-financial](https://github.com/johnstonskj/rust-financial)
crates to fetch stock (and other securities) data for a local
portfolio file.

## Usage

set `IEX_TOKEN`

```bash
~/ $ IEX_TOKEN=pk_...26be folio --help
folio v1.0-pre
Portfolio Manager

USAGE:
    folio [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    add       Add a symbol to the portfolio
    delete    Delete a symbol from the portfolio
    help      Prints this message or the help of the given subcommand(s)
    show      Show quotes for all portfolio
    watch     Watch quotes for portfolio
```

Local portfolio file commands:

* *add* - add a new holding to the local portfolio file.
* *delete* - remove a holding from the local portfolio file.

Portfolio data commands:

* *show* - show, once, the current details for your portfolio.
* *watch* - TBD

```
+--------+-----------+----------------------+------+-----+------+-------+--------+-----------+----------+------------+
| Symbol | Price     | Change               | Open | Low | High | Close | Volume | Purchased | Quantity | Value      |
+========+===========+======================+======+=====+======+=======+========+===========+==========+============+
| AAPL   | $210.29   | $2.55 (↑0.01227%)    | -    | -   | -    | -     | -      | -         | -        | -          |
+--------+-----------+----------------------+------+-----+------+-------+--------+-----------+----------+------------+
| MSFT   | $141.05   | $0.29 (↑0.00205%)    | -    | -   | -    | -     | -      | -         | -        | -          |
+--------+-----------+----------------------+------+-----+------+-------+--------+-----------+----------+------------+
| AMZN   | $1,907.08 | ($34.75) (↓0.01814%) | -    | -   | -    | -     | -      | $1,786.00 | 104      | $12,592.32 |
+--------+-----------+----------------------+------+-----+------+-------+--------+-----------+----------+------------+
```

## The portfolio file

```toml
default_currency = "USD"

[[holdings]]
symbol = "AAPL"
watch_only = true

[[holdings]]
symbol = "MSFT"
watch_only = true

[[holdings]]
symbol = "AMZN"
watch_only = false
quantity = 104

[holdings.purchase_price]
major = 1786
minor = 0
currency_code = "USD"
```

## Troubleshooting

```bash
export RUST_BACKTRACE=1
export RUST_LOG='fin_iex=debug'
```

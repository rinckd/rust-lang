 

use chrono::prelude::*;
use clap::Clap;
use yahoo_finance_api as yahoo;

#[derive(Clap)]
#[clap(
    version = "1.0",
    author = "Claus Matzinger",
    about = "Milestone 1: a simple tracker"
)]
struct Opts {
    #[clap(short, long, default_value = "AAPL,MSFT,UBER,GOOG")]
    symbols: String,
    #[clap(short, long, default_value="2021-05-01T12:00:09Z")]
    from: String,
}

fn price_diff(a: &[f64]) -> Option<(f64, f64)> {
    if !a.is_empty() {
        // unwrap is safe here even if first == last
        let (first, last) = (a.first().unwrap(), a.last().unwrap());
        let abs_diff = last - first;
        let first = if *first == 0.0 { 1.0 } else { *first };
        let rel_diff = abs_diff / first;
        Some((abs_diff, rel_diff))
    } else {
        None
    }
}

fn n_window_sma(n: usize, series: &[f64]) -> Option<Vec<f64>> {
    if !series.is_empty() && n > 1 {
        Some(
            series
                .windows(n)
                .map(|w| w.iter().sum::<f64>() / w.len() as f64)
                .collect(),
        )
    } else {
        None
    }
}

fn max(series: &[f64]) -> Option<f64> {
    if series.is_empty() {
        None
    } else {
        Some(series.iter().fold(f64::MIN, |acc, q| acc.max(*q)))
    }
}

fn min(series: &[f64]) -> Option<f64> {
    if series.is_empty() {
        None
    } else {
        Some(series.iter().fold(f64::MAX, |acc, q| acc.min(*q)))
    }
}

fn main() -> std::io::Result<()> {
    let opts = Opts::parse();
    let from: DateTime<Utc> = opts.from.parse().expect("Couldn't parse 'from' date");
    let provider = yahoo::YahooConnector::new();

    println!("period start,symbol,price,change %,min,max,30d avg");

    for symbol in opts.symbols.split(',') {
        if let Ok(response) = provider.get_quote_history(symbol, from, Utc::now()) {
            match response.quotes() {
                Ok(mut quotes) => {
                    if !quotes.is_empty() {
                        quotes.sort_by_cached_key(|k| k.timestamp);
                        let closes: Vec<f64> = quotes.iter().map(|q| q.adjclose as f64).collect();
                        if !closes.is_empty() {
                            // min/max of the period
                            let period_max: f64 = max(&closes).unwrap();
                            let period_min: f64 = min(&closes).unwrap();
                            let last_price = *closes.last().unwrap_or(&0.0);
                            let (_, pct_change) = price_diff(&closes).unwrap_or((0.0, 0.0));
                            let sma = n_window_sma(30, &closes).unwrap_or_default();
                            println!(
                                "{},{},${:.2},{:.2}%,${:.2},${:.2},${:.2}",
                                from.to_rfc3339(),
                                symbol,
                                last_price,
                                pct_change * 100.0,
                                period_min,
                                period_max,
                                sma.last().unwrap_or(&0.0)
                            );
                        }
                    }
                }
                _ => {
                    eprint!("No quotes found for symbol '{}'", symbol);
                }
            }
        } else {
            eprint!("No quotes found for symbol '{}'", symbol);
        }
    }
    Ok(())
}

 

# ninjabook

[![Crates.io][crates-badge]][crates-url]
[![Documentation][doc-badge]][doc-url]
[![MIT licensed][mit-badge]][mit-url]
[![Rust][rust-badge]][rust-url]

[crates-badge]: https://img.shields.io/crates/v/ninjabook.svg
[crates-url]: https://crates.io/crates/ninjabook
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: https://github.com/ninja-quant/ninjabook/blob/main/LICENSE
[doc-badge]: https://docs.rs/ninjabook/badge.svg
[doc-url]: https://docs.rs/ninjabook
[rust-badge]: https://shields.io/badge/rust-1.77.2%2B-blue.svg
[rust-url]: https://github.com/ninja-quant/ninjabook

A lightweight and high-performance order-book designed to process level 2 and trades data.

# Performance
Ran a couple of benchmarks showcasing real case scenarios against a naive `Vec` implementation and orderbook with fixed sizes of 100 and 500 levels.

The benchmarks are run with [300,000 events of level 2 orderbook data](https://github.com/ninja-quant/ninjabook/blob/main/data/norm_book_data_300k.csv) . This data is split in 2 chunks:
- First 200,000 events are used to warm up.
- Last 100,000 for the actual benchmark.

The scenarios tested are:
- Process events and stream best bid and ask
- Process events and stream top5 bids and asks

Here are the results:
|bench| iterations | time | ns/iter |
|--|--|--|--|
| ninjabook_bbo | 100,000 | 9.1093 ms | 91.093 ns | 
| fixed_100_bbo | 100,000 | 14.253 ms | 142.53 ns | 
| fixed_500_bbo | 100,000 | 72.636 ms | 726.36 ns | 
| naive_bbo | 100,000 | 263.77 ms | 2,637.7 ns | 
| ninjabook_top5 | 100,000 | 16.729 ms | 167.29 ns | 
| fixed_100_top5 | 100,000 | 23.532 ms | 235.32 ns | 
| fixed_100_top5 | 100,000 | 81.112 ms | 811.12 ns | 
| naive_top5 | 100,000 | 275.40 ms | 2,754 ns | 

# Contributing
To add a better version, create a new file, implementing the same methods as `orderbook.rs` (including tests) and add the improved orderbook to the bench `optimal_vs_naive.rs`. Only order books with a better performance than `orderbook.rs` will be considered. Lastly, add performance logs to the Pull Request, can just copy paste what `cargo bench` outputs.


Any issues, refactoring, docs and tests are also welcomed. Feel free to reach out [here](https://twitter.com/ninjaquant_) if you have any questions.


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
Ran a couple of benchmarks showcasing real case scenarios against a naive `Vec` implementation.

The benchmarks are run with [300,000 events of level 2 orderbook data](https://github.com/ninja-quant/ninjabook/blob/main/data/norm_book_data_300k.csv) . This data is split in 2 chunks:
- First 200,000 events are used to warm up.
- Last 100,000 for the actual benchmark.

The scenarios are tested are:
- Process event and stream best bid and ask
- Process event and stream top5 bids and asks

Here are the results:
|bench| iterations | time | ns/iter |
|--|--|--|--|
| ninjabook_bbo | 100,000 | 3.9380 ms | 39.38 ns | 
| naive_bbo | 100,000 | 263.77 ms | 2,637.7 ns | 
| ninjabook_top5 | 100,000 | 12.310 ms | 123.1 ns | 
| naive_top5 | 100,000 | 275.40 ms | 2,754.7 ns | 

# Contributing
To add a better version, create a new file, implementing the same methods as `orderbook.rs` (including tests) and adding to the benchmark `optimal_vs_naive`. Only order books with better performance than `orderbook.rs` will be considered. Lastly, add performance logs to the Pull Request, can just copy paste what `cargo bench` outputs.


Any issues and tests are also welcomed. Feel free to reach out at  [https://twitter.com/ninjaquant_](https://twitter.com/ninjaquant_) if you have any questions.

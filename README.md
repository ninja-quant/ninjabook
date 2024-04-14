
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
Ran a couple of benchmarks showcasing real case scenarios against a naive `Vec` implementation and an orderbook with a fixed size of 500 levels.

The benchmarks are run with [300,000 events of level 2 orderbook data](https://github.com/ninja-quant/ninjabook/blob/main/data/norm_book_data_300k.csv) . This data is split in 2 chunks:
- First 200,000 events are used to warm up and verify all orderbook versions publish the same BBO and top5 levels.
- Last 100,000 for the actual benchmark.

The scenarios tested are:
- Process events and stream best bid and ask
- Process events and stream top5 bids and asks

Here are the results:
|bench| iterations | time | ns/iter |
|--|--|--|--|
| ninjabook_bbo | 100,000 | 5.0108 ms | 50.108 ns | 
| fixed_500_bbo | 100,000 | 49.018 ms | 490.18 ns | 
| naive_bbo | 100,000 | 90.552 ms | 905.52 ns | 
| ninjabook_top5 | 100,000 | 11.797 ms | 117.97 ns | 
| fixed_500_top5 | 100,000 | 54.693 ms | 546.93 ns | 
| naive_top5 | 100,000 | 95.644 ms | 956.44 ns | 

# Contributing
To add a better version, create a new file, implementing the same methods as `orderbook.rs` (including tests) and add the improved orderbook to the bench `optimal_vs_naive.rs`. Only order books with a better performance than `orderbook.rs` will be considered. Lastly, add performance logs to the Pull Request, can just copy paste what `cargo bench` outputs.


Any issues, refactoring, docs and tests are also welcomed. Feel free to reach out [here](https://twitter.com/ninjaquant_) if you have any questions.

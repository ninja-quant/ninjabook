use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ninjabook::{
    event::Event, fixed_orderbook::Orderbook as FixedOrderbook,
    naive_orderbook::Orderbook as NaiveOrderbook, orderbook::Orderbook,
};

#[inline]
fn process_and_bbo(mut ob: Orderbook, data: Vec<Event>) {
    data.into_iter().for_each(|event| {
        ob.process_stream_bbo(event);
    });
}

#[inline]
fn process_and_top5(mut ob: Orderbook, data: Vec<Event>) {
    data.into_iter().for_each(|event| {
        ob.process(event);
        ob.top_bids(5);
        ob.top_asks(5);
    });
}

#[inline]
fn naive_process_and_bbo(mut ob: NaiveOrderbook, data: Vec<Event>) {
    data.into_iter().for_each(|event| {
        ob.process_stream_bbo(event);
    });
}

#[inline]
fn naive_process_and_top5(mut ob: NaiveOrderbook, data: Vec<Event>) {
    data.into_iter().for_each(|event| {
        ob.process(event);
        ob.top_bids(5);
        ob.top_asks(5);
    });
}

#[inline]
fn fixed_process_and_bbo(mut ob: FixedOrderbook, data: Vec<Event>) {
    data.into_iter().for_each(|event| {
        ob.process_stream_bbo(event);
    });
}

#[inline]
fn fixed_process_and_top5(mut ob: FixedOrderbook, data: Vec<Event>) {
    data.into_iter().for_each(|event| {
        ob.process(event);
        ob.top_bids(5);
        ob.top_asks(5);
    });
}

fn bench_group(c: &mut Criterion) {
    let mut reader = csv::Reader::from_path("./data/norm_book_data_300k.csv").unwrap();

    let mut data = Vec::new();

    let tick_size = 0.01;

    let mut ob = Orderbook::new(tick_size);
    let mut naive_ob = NaiveOrderbook::new();
    let mut fixed_ob = FixedOrderbook::new();

    for (i, result) in reader.deserialize::<Event>().enumerate() {
        let event = result.unwrap();
        match i {
            0..=199_999 => {
                ob.process(event);
                naive_ob.process(event);
                fixed_ob.process(event);

                assert_eq!(ob.top_asks(5), fixed_ob.top_asks(5));
                assert_eq!(ob.top_bids(5), fixed_ob.top_bids(5));
                assert_eq!(ob.top_bids(5), naive_ob.top_bids(5));
                assert_eq!(ob.top_asks(5), naive_ob.top_asks(5));

                assert_eq!(ob.best_bid(), fixed_ob.best_bid());
                assert_eq!(ob.best_ask(), fixed_ob.best_ask());
                assert_eq!(ob.best_bid(), naive_ob.best_bid());
                assert_eq!(ob.best_ask(), naive_ob.best_ask());
            }
            200_000..=299_999 => data.push(event),
            _ => break,
        }
    }

    assert_eq!(data.len(), 100_000);

    let mut group = c.benchmark_group("bench");

    group.bench_function("process_and_bbo", |b| {
        b.iter(|| process_and_bbo(black_box(ob.clone()), black_box(data.clone())))
    });

    group.bench_function("process_and_top5", |b| {
        b.iter(|| process_and_top5(black_box(ob.clone()), black_box(data.clone())))
    });

    group.bench_function("naive_process_and_bbo", |b| {
        b.iter(|| naive_process_and_bbo(black_box(naive_ob.clone()), black_box(data.clone())))
    });

    group.bench_function("naive_process_and_top5", |b| {
        b.iter(|| naive_process_and_top5(black_box(naive_ob.clone()), black_box(data.clone())))
    });

    group.bench_function("fixed_process_and_bbo", |b| {
        b.iter(|| fixed_process_and_bbo(black_box(fixed_ob.clone()), black_box(data.clone())))
    });

    group.bench_function("fixed_process_and_top5", |b| {
        b.iter(|| fixed_process_and_top5(black_box(fixed_ob.clone()), black_box(data.clone())))
    });

    group.finish()
}

criterion_group!(benches, bench_group);
criterion_main!(benches);

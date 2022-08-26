#![feature(async_closure)]
#![feature(generators)]
#![feature(iter_from_generator)]
#![feature(test)]

extern crate test;
use std::future::Future;
use stream_future::*;
use test::{black_box, Bencher};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver};
use tokio_stream::{wrappers::UnboundedReceiverStream, StreamExt};

fn async_bench<F: Future>(b: &mut Bencher, mut f: impl FnMut() -> F) {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    b.iter(|| runtime.block_on(f()))
}

const ITER_MAX: i32 = 10000;

#[bench]
fn this(b: &mut Bencher) {
    #[stream(i32)]
    async fn foo() {
        for i in 0..ITER_MAX {
            yield i;
        }
    }

    async_bench(b, async move || {
        let s = foo();
        tokio::pin!(s);
        while let Some(v) = s.next().await {
            black_box(v);
        }
    })
}

#[bench]
fn async_stream(b: &mut Bencher) {
    fn foo() -> impl Stream<Item = i32> {
        async_stream::stream! {
            for i in 0..ITER_MAX {
                yield i;
            }
        }
    }

    async_bench(b, async move || {
        let s = foo();
        tokio::pin!(s);
        while let Some(v) = s.next().await {
            black_box(v);
        }
    })
}

#[bench]
fn mpsc_channel(b: &mut Bencher) {
    fn foo() -> UnboundedReceiver<i32> {
        let (tx, rx) = unbounded_channel();
        tokio::spawn(async move {
            for i in 0..ITER_MAX {
                tx.send(i).unwrap();
            }
        });
        rx
    }

    async_bench(b, async move || {
        let s = UnboundedReceiverStream::new(foo());
        tokio::pin!(s);
        while let Some(v) = s.next().await {
            black_box(v);
        }
    })
}

#[bench]
fn iter_stream(b: &mut Bencher) {
    async_bench(b, async move || {
        let mut s = tokio_stream::iter(0..ITER_MAX);
        while let Some(v) = s.next().await {
            black_box(v);
        }
    })
}

#[bench]
fn iter(b: &mut Bencher) {
    async_bench(b, async move || {
        for i in 0..ITER_MAX {
            black_box(i);
        }
    })
}

#[bench]
fn generator(b: &mut Bencher) {
    async_bench(b, async move || {
        let s = std::iter::from_generator(|| {
            for i in 0..ITER_MAX {
                yield i;
            }
        });
        for v in s {
            black_box(v);
        }
    })
}

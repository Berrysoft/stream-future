# stream-future

[![crates.io](https://img.shields.io/crates/v/stream-future)](https://crates.io/crates/stream-future)
[![docs.rs](https://img.shields.io/badge/docs.rs-stream--future-latest)](https://docs.rs/stream-future)

This is a `no_std` compatible library to author a `Future` with `Stream` implemented.
You can author simply with `await` and `yield`.

A nightly feature `generators` is required.

``` rust
#![feature(generators)]

use stream_future::stream;

#[derive(Debug)]
enum Prog {
    Stage1,
    Stage2,
    End,
}

#[stream(Prog)]
async fn foo() -> Result<i32> {
    yield Prog::Stage1;
    // some works...
    yield Prog::Stage2;
    // some other works...
    yield Prog::End;
    Ok(0)
}

use tokio_stream::StreamExt;

let bar = foo();
tokio::pin!(bar);
while let Some(prog) = bar.next().await {
    println!("{:?}", prog);
}
let bar = bar.await?;
assert_eq!(bar, 0);
```

You can specify the yield type in the attribute. Either the yield type or return type could be `()`.
You can simply `await` other futures, and the macro will handle that.

## Compare with `async-stream`
You can return any value you like! The caller can simply `await` and get the value without iterate the stream.

This library is 7x faster than `async-stream`, according to our benchmark.

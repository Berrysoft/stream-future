# stream-future

This is a `no_std` compatible library to author a `Future` with `Stream` implemented.
You can author simply with `await` and `yield`.

A nightly feature `generator` is required.

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

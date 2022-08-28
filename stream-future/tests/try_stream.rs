#![feature(generators)]

use anyhow::Result;
use futures_util::TryStreamExt;
use std::future::ready;
use stream_future_impl::try_stream;

#[tokio::test]
async fn basic() {
    #[try_stream(i32)]
    async fn foo() -> Result<bool> {
        yield 0;
        yield 1;
        yield (ready(2).await);
        Ok(true)
    }

    let gf = foo();
    tokio::pin!(gf);
    assert_eq!((&mut gf).try_collect::<Vec<_>>().await.unwrap(), [0, 1, 2]);
    assert_eq!(gf.await.unwrap(), true);
}

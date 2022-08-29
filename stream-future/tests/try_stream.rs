#![feature(generators)]

use anyhow::Result;
use futures_util::{StreamExt, TryStreamExt};
use std::future::ready;
use stream_future::try_stream;

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

#[tokio::test]
async fn err() {
    #[try_stream(i32)]
    async fn foo(i: i32) -> Result<()> {
        yield 0;
        if i == 0 {
            anyhow::bail!("error");
        }
        yield 1;
        Ok(())
    }

    {
        let gf = foo(0);
        tokio::pin!(gf);
        assert!(gf.try_collect::<Vec<_>>().await.is_err());
    }
    {
        let gf = foo(1);
        tokio::pin!(gf);
        assert_eq!(gf.try_collect::<Vec<_>>().await.unwrap(), [0, 1]);
    }
}

#[tokio::test]
async fn option() {
    #[try_stream(i32)]
    async fn foo(i: i32) -> Option<bool> {
        yield 0;
        if i == 0 {
            return None;
        }
        yield 1;
        Some(true)
    }

    {
        let gf = foo(0);
        tokio::pin!(gf);
        assert_eq!(gf.next().await, Some(Some(0)));
        assert_eq!(gf.next().await, Some(None));
    }
    {
        let gf = foo(1);
        tokio::pin!(gf);
        assert_eq!(gf.next().await, Some(Some(0)));
        assert_eq!(gf.next().await, Some(Some(1)));
        assert_eq!(gf.next().await, None);
        assert_eq!(gf.await, Some(true));
    }
}

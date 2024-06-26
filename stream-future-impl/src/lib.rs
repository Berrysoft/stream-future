use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, Parser},
    parse_macro_input,
    visit_mut::{visit_expr_mut, visit_stmt_mut, VisitMut},
    Block, Expr, ItemFn, Lifetime, ReturnType, Type,
};

/// See the crate document of [`stream-future`].
fn stream_impl(attr: TokenStream, input: TokenStream, gen_ty: &str, is_try: bool) -> TokenStream {
    let gen_ty = Ident::parse.parse_str(gen_ty).unwrap();
    let mut p_type = Type::parse.parse2(quote!(())).unwrap();
    let mut lifetime = Lifetime::parse.parse2(quote!('static)).unwrap();
    if !attr.is_empty() {
        let args_parser = syn::meta::parser(|meta| {
            if meta.path.is_ident("lifetime") {
                lifetime = meta.value()?.parse()?;
            } else {
                p_type = Type::parse.parse2(meta.path.into_token_stream())?;
            }
            Ok(())
        });
        parse_macro_input!(attr with args_parser);
    }
    let mut func = parse_macro_input!(input as ItemFn);
    func.sig.asyncness = None;
    let future_return_type = match func.sig.output {
        ReturnType::Default => Box::new(Type::parse.parse2(quote!(())).unwrap()),
        ReturnType::Type(_, t) => t,
    };
    func.sig.output = ReturnType::parse
        .parse2(if !is_try {
            quote! {
                -> impl ::core::future::Future<Output = #future_return_type> + ::stream_future::Stream<Item = #p_type> + #lifetime
            }
        } else {
            quote! {
                -> impl ::core::future::Future<Output = #future_return_type> + ::stream_future::Stream<Item = ::stream_future::TryStreamItemType<#future_return_type, #p_type>> + #lifetime
            }
        })
        .unwrap();
    let mut old_block = func.block;
    for stmt in old_block.stmts.iter_mut() {
        visit_stmt_mut(&mut AwaitYieldVisitor, stmt);
    }
    func.block = Box::new(
        Block::parse
            .parse2(quote! {{
                ::stream_future::#gen_ty::<#p_type, _>::new(#[coroutine] static move |#[allow(unused_mut)] mut __cx: ::stream_future::ResumeTy| {
                    #old_block
                })
            }})
            .unwrap(),
    );

    func.to_token_stream().into()
}

struct AwaitYieldVisitor;

impl VisitMut for AwaitYieldVisitor {
    fn visit_expr_mut(&mut self, i: &mut Expr) {
        match i {
            Expr::Await(expr_await) => {
                let attrs = &expr_await.attrs;
                let mut inner_expr = expr_await.base.clone();
                self.visit_expr_mut(&mut inner_expr);
                *i = Expr::parse
                    .parse2(quote! {
                        #(#attrs)*
                        {
                            let mut __future = #inner_expr;
                            loop {
                                #[allow(unsafe_code)]
                                let mut __future = unsafe { ::core::pin::Pin::new_unchecked(&mut __future) };
                                match __cx.poll_future(__future) {
                                    ::core::task::Poll::Ready(__ret) => {
                                        break __ret;
                                    }
                                    ::core::task::Poll::Pending => {
                                        yield ::core::task::Poll::Pending;
                                    }
                                }
                            }
                        }
                    })
                    .unwrap();
            }
            Expr::Yield(expr_yield) => {
                let mut inner_expr = expr_yield
                    .expr
                    .take()
                    .unwrap_or_else(|| Box::new(Expr::parse.parse2(quote!(())).unwrap()));
                self.visit_expr_mut(&mut inner_expr);
                expr_yield.expr = Some(Box::new(
                    Expr::parse
                        .parse2(quote!(::core::task::Poll::Ready(
                            #[allow(unused_parens)]
                            #inner_expr
                        )))
                        .unwrap(),
                ));
                *i = Expr::parse
                    .parse2(quote! {
                        __cx = #expr_yield
                    })
                    .unwrap()
            }
            _ => visit_expr_mut(self, i),
        }
    }

    fn visit_expr_async_mut(&mut self, _i: &mut syn::ExprAsync) {}
}

#[proc_macro_attribute]
pub fn stream(attr: TokenStream, input: TokenStream) -> TokenStream {
    stream_impl(attr, input, "GenStreamFuture", false)
}

#[proc_macro_attribute]
pub fn try_stream(attr: TokenStream, input: TokenStream) -> TokenStream {
    stream_impl(attr, input, "GenTryStreamFuture", true)
}

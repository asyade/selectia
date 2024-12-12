pub(crate) mod prelude;

use proc_macro2::Span;
use quote::ToTokens;
use syn::{DeriveInput, GenericArgument, Ident, PatType, PathArguments};

use crate::prelude::*;

/// Helper macro to generate required struct and impl for a Service
#[proc_macro_error]
#[proc_macro_derive(Task)]
pub fn task_derive(input: TokenStream) -> TokenStream {
    let input = TokenStream2::from(input);
    let Ok(ast) = syn::parse2::<DeriveInput>(input.clone()) else {
        abort!(input, "Failed to parse input");
    };

    let name = &ast.ident;

    let output = quote! {
        impl Task for #name {
        }
    };

    output.into()
}

macro_rules! extract_proc_macro_ident {
    (TY => $e:expr, $item:expr) => {
        match $e {
            syn::Type::Path(syn::TypePath { path, .. }) => {
                path.segments.clone().into_iter().last().unwrap().ident
            }
            _ => {
                abort!($item, "Only single-identifier patterns are supported");
            }
        }
    };
    (GENERIC_0 => $e:expr, $item:expr) => {
        match $e {
            syn::Type::Path(syn::TypePath { path, .. }) => {
                let args = path
                    .segments
                    .clone()
                    .into_iter()
                    .last()
                    .unwrap()
                    .arguments
                    .clone();
                let PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
                    args, ..
                }) = args
                else {
                    abort!($item, "Expected one generic argument");
                };
                let GenericArgument::Type(syn::Type::Path(syn::TypePath { path, .. })) =
                    args.iter().next().unwrap().clone()
                else {
                    abort!($item, "Expected one generic argument");
                };
                path.get_ident().unwrap().clone()
            }
            _ => {
                abort!($item, "Only single-identifier patterns are supported");
            }
        }
    };
}

#[proc_macro_error]
#[proc_macro_attribute]
pub fn singleton_service(input: TokenStream, item: TokenStream) -> TokenStream {
    let input = TokenStream2::from(input);
    let item = TokenStream2::from(item);

    let Some(service_name) = input.clone().into_iter().next().map(|e| e.to_string()) else {
        abort!(
            input,
            "Input must contain a service name, e.g. #[singleton_service(ServiceA)]"
        );
    };

    let Ok(item_ast) = syn::parse2::<syn::ItemFn>(item.clone()) else {
        abort!(item, "`singleton_service` can only be used on functions");
    };

    let mut ctx_param = None;
    let mut rx_param = None;
    let mut dispatcher_param = vec![];
    let mut extra_param = vec![];

    for param in item_ast.sig.inputs.iter() {
        match param {
            syn::FnArg::Typed(PatType { ty, .. }) => {
                let type_ident = extract_proc_macro_ident!(TY => &**ty, item);

                match type_ident.to_string().as_str() {
                    "ServiceContext" if rx_param.is_none() => {
                        if ctx_param.replace(param.clone()).is_some() {
                            abort!(item, "Only one `ServiceContext` parameter is allowed");
                        }
                    }
                    "ServiceContext" => {
                        abort!(item, "First parameter must be of type `ServiceContext`");
                    }
                    "ServiceReceiver" if ctx_param.is_some() && dispatcher_param.len() == 0 => {
                        let type_ident = extract_proc_macro_ident!(GENERIC_0 => &**ty, item);
                        if rx_param.replace(type_ident).is_some() {
                            abort!(item, "Only one `ServiceReceiver` parameter is allowed");
                        }
                    }
                    "ServiceReceiver" => {
                        abort!(
                            item,
                            "The second parameter must be of type `ServiceReceiver`"
                        );
                    }
                    "EventDispatcher" if extra_param.len() == 0 => {
                        let type_ident = extract_proc_macro_ident!(GENERIC_0 => &**ty, item);
                        dispatcher_param.push(type_ident);
                    }
                    "EventDispatcher" => {
                        abort!(
                            item,
                            "Event dispatcher must be placed before any extra parameter"
                        );
                    }
                    _ if ctx_param.is_some() && rx_param.is_some() => {
                        extra_param.push(param.clone());
                    }
                    _ => {
                        abort!(
                            item,
                            "Extra parametter are only allowed after all special parametter !"
                        );
                    }
                }
            }
            _ => {}
        }
    }

    let service_entrypoint_name = item_ast.sig.ident.clone();

    let Some(task_type_name) = rx_param else {
        abort!(item, "`singleton_service` must have a `ServiceReceiver` parameter i.e `rx: ServiceReceiver<Task>`");
    };

    let service_name = Ident::new(&service_name, Span::call_site());

    let dispatcher_registration = dispatcher_param.iter().map(|type_ident| {
        quote! {
            ctx.register_singleton(SingletonServiceDispatcher::<#task_type_name, #type_ident, #service_name>::new()).await?;
        }.to_token_stream()
    }).collect::<Vec<_>>();

    let dispatcher_args_def = dispatcher_param.iter().enumerate().map(|(i, type_ident)| {
        let dispatcher_ident = Ident::new(&format!("dispatcher_{}", i), Span::call_site());
        quote! {
            let #dispatcher_ident = ctx.get_singleton::<SingletonServiceDispatcher<#task_type_name, #type_ident, #service_name>>().await.expect("Failed to get dispatcher");
        }.to_token_stream()
    }).collect::<Vec<_>>();

    let dispatcher_args_use = dispatcher_param
        .iter()
        .enumerate()
        .map(|(i, _type_ident)| {
            let dispatcher_ident = Ident::new(&format!("dispatcher_{}", i), Span::call_site());
            quote! {
                #dispatcher_ident.into(),
            }
            .to_token_stream()
        })
        .collect::<Vec<_>>();

    let extra_args_params = extra_param
        .iter()
        .map(|param| {
            quote! {
                #param
            }
            .to_token_stream()
        })
        .collect::<Vec<_>>();

    let extra_args_use = extra_param
        .iter()
        .map(|param| {
            let param_ident = match param {
                syn::FnArg::Typed(PatType { pat, .. }) => pat.clone(),
                _ => {
                    abort!(item, "Extra parameter must be a typed parameter");
                }
            };
            quote! {
                #param_ident
            }
            .to_token_stream()
        })
        .collect::<Vec<_>>();

    let quoted = quote! {
        #item_ast

        pub struct #service_name {
            service: theater::service::AddressableService<#task_type_name>,
        }

        impl SingletonService for #service_name {
            type Task = #task_type_name;
            fn address(&self) -> AddressableService<#task_type_name> {
                self.service.clone()
            }
        }

        impl #service_name {
            pub async fn spawn<T: GlobalTheaterContext + 'static>(parent_ctx: &T, #(#extra_args_params),*) -> TheaterResult<()> {
                let ctx = ServiceContext::new::<#service_name>(parent_ctx.as_global().clone());
                let service = AddressableService::<#task_type_name>::new(&ctx, |ctx, rx| async move {

                    #(#dispatcher_args_def)*

                    #service_entrypoint_name(ctx, rx, #(#dispatcher_args_use)* #(#extra_args_use)*).await
                })
                .await;

                let singleton = Self { service };

                ctx.register_singleton(singleton).await?;

                #(#dispatcher_registration)*

                Ok(())
            }
        }
    };

    quoted.into()
}

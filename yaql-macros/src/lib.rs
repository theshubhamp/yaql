use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::{parse_macro_input, Expr, FnArg, Ident, ItemFn, LitBool, LitStr, Result};

struct YaqlArgs {
    name: LitStr,
    argspec: Option<Expr>,
    types: Option<Expr>,
    kwargs: Option<LitBool>,
}

impl Parse for YaqlArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let name = input.parse()?;
        let mut argspec = None;
        let mut types = None;
        let mut kwargs = None;
        while input.peek(syn::Token![,]) {
            let _: syn::Token![,] = input.parse()?;
            if argspec.is_none() {
                argspec = Some(input.parse()?);
            } else if types.is_none() {
                types = Some(input.parse()?);
            } else if kwargs.is_none() {
                kwargs = Some(input.parse()?);
            } else {
                return Err(input.error("too many arguments to yaql_function"));
            }
        }
        Ok(YaqlArgs { name, argspec, types, kwargs })
    }
}

fn is_raw_signature(func: &ItemFn) -> bool {
    let mut params = func.sig.inputs.iter();
    let first = match params.next() {
        Some(FnArg::Typed(p)) => p,
        _ => return false,
    };
    let second = match params.next() {
        Some(FnArg::Typed(p)) => p,
        _ => return false,
    };
    if params.next().is_some() {
        return false;
    }
    let ty_str = |t: &syn::Type| -> String {
        quote::ToTokens::to_token_stream(t).to_string().replace(' ', "")
    };
    ty_str(&first.ty) == "Vec<Primitive>"
        && ty_str(&second.ty) == "Vec<(Primitive,Primitive)>"
        && matches!(func.sig.output, syn::ReturnType::Default)
}

/// Attribute macro that registers a stdlib function.
///
/// Two forms are supported, dispatched automatically:
///
/// 1. **Typed** — the spec (name, arity, arg types, kwargs) is inferred from the
///    Rust signature. Each param must implement `FromPrimitive` and the return
///    type must implement `IntoPrimitive`.
///    ```ignore
///    #[yaql_function("abs")]
///    fn abs_int(n: i64) -> i64 { n.abs() }
///    ```
///
/// 2. **Raw** — provide `argspec`, `arg_types`, and `kwargs` explicitly and give
///    the function a raw signature:
///    `fn(Vec<Primitive>, Vec<(Primitive, Primitive)>) -> Result<Primitive, EvalError>`.
///    ```ignore
///    #[yaql_function("list", ArgSpec::Varargs, [], false)]
///    pub fn list_fn(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Result<Primitive, EvalError> {
///        Ok(Primitive::Array(args))
///    }
///    ```
#[proc_macro_attribute]
pub fn yaql_function(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as YaqlArgs);
    let func = parse_macro_input!(input as ItemFn);

    if args.argspec.is_some() || is_raw_signature(&func) {
        raw_impl(args, func)
    } else {
        typed_impl(args, func)
    }
}

fn raw_impl(args: YaqlArgs, func: ItemFn) -> TokenStream {
    let name_lit = args.name;
    let argspec = args
        .argspec
        .expect("raw yaql_function requires an ArgSpec expression");
    let types = args.types.unwrap_or_else(|| syn::parse_quote! { [] });
    let kwargs = args.kwargs.unwrap_or_else(|| LitBool::new(false, proc_macro2::Span::call_site()));
    let fn_name = func.sig.ident.clone();

    let expanded = quote! {
        #func
        inventory::submit! {
            yaql_core::lang::functions::Spec::new(#name_lit, #fn_name, #argspec, &#types, #kwargs)
        }
    };

    expanded.into()
}

fn typed_impl(args: YaqlArgs, func: ItemFn) -> TokenStream {
    let name_lit = args.name;
    let fn_name = func.sig.ident.clone();
    let mod_name = format_ident!("__yaql_fn_{}", fn_name);

    let ret_ty = match &func.sig.output {
        syn::ReturnType::Type(_, ty) => ty.clone(),
        syn::ReturnType::Default => {
            return syn::Error::new(func.sig.span(), "yaql_function requires a return type")
                .to_compile_error()
                .into();
        }
    };

    let mut arg_idents: Vec<Ident> = Vec::new();
    let mut arg_tys: Vec<syn::Type> = Vec::new();
    for input in &func.sig.inputs {
        match input {
            FnArg::Receiver(_) => {
                return syn::Error::new(
                    func.sig.span(),
                    "yaql_function cannot take a `self` receiver",
                )
                .to_compile_error()
                .into();
            }
            FnArg::Typed(pat_ty) => {
                let ident = match pat_ty.pat.as_ref() {
                    syn::Pat::Ident(pat_ident) => pat_ident.ident.clone(),
                    _ => {
                        return syn::Error::new(
                            pat_ty.span(),
                            "yaql_function params must be plain identifiers",
                        )
                        .to_compile_error()
                        .into();
                    }
                };
                arg_idents.push(ident);
                arg_tys.push((*pat_ty.ty).clone());
            }
        }
    }

    let arity = arg_idents.len();
    let body = &func.block;
    let type_array = arg_tys
        .iter()
        .map(|ty| quote! { <#ty as FromPrimitive>::TYPE });
    let unpack = arg_idents
        .iter()
        .zip(arg_tys.iter())
        .enumerate()
        .map(|(i, (ident, ty))| {
            let i = syn::Index::from(i);
            quote! {
                let #ident = match <#ty as FromPrimitive>::from_primitive(&args[#i]) {
                    Some(v) => v,
                    None => return Ok(Primitive::Null),
                };
            }
        })
        .collect::<Vec<_>>();

    let expanded = quote! {
        #[allow(non_camel_case_types)]
        mod #mod_name {
            use yaql_core::lang::functions::*;
            pub const SPEC: Spec = Spec::new(#name_lit, func, ArgSpec::Exact(#arity), &[#(#type_array),*], false);
            pub fn func(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Result<Primitive, EvalError> {
                #(#unpack)*
                let __ret: #ret_ty = #body;
                Ok(IntoPrimitive::into_primitive(__ret))
            }
        }
        inventory::submit! { #mod_name::SPEC }
    };

    expanded.into()
}

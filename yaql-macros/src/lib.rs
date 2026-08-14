use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::{parse_macro_input, FnArg, Ident, ItemFn, LitStr, Result};

struct YaqlArgs {
    name: LitStr,
}

impl Parse for YaqlArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(YaqlArgs {
            name: input.parse()?,
        })
    }
}

/// Attribute macro that registers a typed stdlib function.
///
/// Usage:
/// ```ignore
/// #[yaql_function("abs")]
/// fn abs_int(n: i64) -> i64 { n.abs() }
/// ```
#[proc_macro_attribute]
pub fn yaql_function(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as YaqlArgs);
    let func = parse_macro_input!(input as ItemFn);

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

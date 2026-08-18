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
        let name = input.parse()?;
        Ok(YaqlArgs { name })
    }
}

fn ty_str(t: &syn::Type) -> String {
    quote::ToTokens::to_token_stream(t).to_string().replace(' ', "")
}

/// If `t` is `Varargs<N>`, return `Some(N)`; otherwise `None`.
fn varargs_min(t: &syn::Type) -> Option<usize> {
    if let syn::Type::Path(tp) = t {
        let seg = tp.path.segments.last()?;
        if seg.ident != "Varargs" {
            return None;
        }
        if let syn::PathArguments::AngleBracketed(args) = &seg.arguments {
            if let Some(syn::GenericArgument::Const(syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Int(n),
                ..
            }))) = args.args.first()
            {
                return n.base10_parse::<usize>().ok();
            }
        }
    }
    None
}

fn is_result_type(ty: &syn::Type) -> bool {
    if let syn::Type::Path(tp) = ty {
        if let Some(seg) = tp.path.segments.last() {
            return seg.ident == "Result";
        }
    }
    false
}

/// Attribute macro that registers a stdlib function.
///
/// The spec (name, arity, arg types, kwargs) is inferred entirely from the Rust
/// signature. Each fixed param must implement `FromPrimitive` and the return
/// type must implement `IntoPrimitive` (or be `Result<T, EvalError>`).
///
/// Special trailing params:
/// - `Varargs<N>` — accept extra args, requiring at least `N` of them (yields
///   `ArgSpec::Varargs` for `Varargs<0>` with no fixed params, otherwise
///   `ArgSpec::Min(fixed_count + N)`).
/// - `Kwargs` — accept keyword arguments (sets `kwargs: true`).
///
/// ```ignore
/// #[yaql_function("abs")]
/// fn abs_int(n: i64) -> i64 { n.abs() }
///
/// #[yaql_function("list")]
/// fn list_fn(args: Varargs<0>) -> Vec<Primitive> { args.0 }
///
/// #[yaql_function("dict")]
/// fn dict_fn(args: Varargs<0>, kwargs: Kwargs) -> Result<Primitive, EvalError> { ... }
/// ```
#[proc_macro_attribute]
pub fn yaql_function(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as YaqlArgs);
    let func = parse_macro_input!(input as ItemFn);
    typed_impl(args, func)
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
    let mut rest_min: Option<usize> = None;
    let mut has_kwargs = false;

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
                let ty = (*pat_ty.ty).clone();
                if let Some(min) = varargs_min(&ty) {
                    if rest_min.is_some() {
                        return syn::Error::new(
                            pat_ty.span(),
                            "yaql_function cannot have more than one Varargs param",
                        )
                        .to_compile_error()
                        .into();
                    }
                    rest_min = Some(min);
                    arg_idents.push(ident);
                    arg_tys.push(ty);
                } else if ty_str(&ty) == "Kwargs" {
                    if has_kwargs {
                        return syn::Error::new(
                            pat_ty.span(),
                            "yaql_function cannot have more than one Kwargs param",
                        )
                        .to_compile_error()
                        .into();
                    }
                    has_kwargs = true;
                    arg_idents.push(ident);
                    arg_tys.push(ty);
                } else {
                    arg_idents.push(ident);
                    arg_tys.push(ty);
                }
            }
        }
    }

    let fixed_count = arg_tys
        .iter()
        .filter(|t| varargs_min(t).is_none() && ty_str(t) != "Kwargs")
        .count();

    let argspec = match rest_min {
        Some(min) => {
            let n = syn::Index::from(fixed_count + min);
            if fixed_count == 0 && min == 0 {
                quote! { ArgSpec::Varargs }
            } else {
                quote! { ArgSpec::Min(#n) }
            }
        }
        _ => {
            let n = syn::Index::from(fixed_count);
            quote! { ArgSpec::Exact(#n) }
        }
    };

    let type_array = arg_tys
        .iter()
        .filter(|t| varargs_min(t).is_none() && ty_str(t) != "Kwargs")
        .map(|ty| quote! { <#ty as FromPrimitive>::TYPE });

    let mut unpack = Vec::new();
    let mut fixed_idx = 0usize;
    for (ident, ty) in arg_idents.iter().zip(arg_tys.iter()) {
        if let Some(min) = varargs_min(ty) {
            let i = syn::Index::from(fixed_count);
            let min = syn::Index::from(min);
            unpack.push(quote! {
                let #ident = Varargs::<#min>(args[#i..].to_vec());
            });
        } else if ty_str(ty) == "Kwargs" {
            unpack.push(quote! {
                let #ident = Kwargs(kwargs);
            });
        } else {
            let i = syn::Index::from(fixed_idx);
            unpack.push(quote! {
                let #ident = match <#ty as FromPrimitive>::from_primitive(&args[#i]) {
                    Some(v) => v,
                    None => return Ok(Primitive::Null),
                };
            });
            fixed_idx += 1;
        }
    }

    let kwargs_lit = has_kwargs;

    let call = if is_result_type(&ret_ty) {
        quote! {
            let __ret: #ret_ty = super::#fn_name(#(#arg_idents),*);
            match __ret {
                Ok(v) => Ok(IntoPrimitive::into_primitive(v)),
                Err(e) => Err(e),
            }
        }
    } else {
        quote! {
            let __ret: #ret_ty = super::#fn_name(#(#arg_idents),*);
            Ok(IntoPrimitive::into_primitive(__ret))
        }
    };

    let expanded = quote! {
        #func
        #[allow(non_camel_case_types)]
        mod #mod_name {
            use super::*;
            use yaql_core::lang::functions::*;
            use yaql_core::lang::*;
            use yaql_core::interpreter::*;
            pub const SPEC: Spec = Spec::new(#name_lit, func, #argspec, &[#(#type_array),*], #kwargs_lit);
            pub fn func(args: Vec<Primitive>, kwargs: Vec<(Primitive, Primitive)>) -> Result<Primitive, EvalError> {
                #(#unpack)*
                #call
            }
        }
        inventory::submit! { #mod_name::SPEC }
    };

    expanded.into()
}

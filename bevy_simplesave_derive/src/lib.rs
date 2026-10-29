use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Attribute, DeriveInput, Error, Ident, LitStr, parse_macro_input};

#[proc_macro_derive(SaveResource, attributes(save))]
pub fn derive_save_resource(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    into_output(expand_resource(&input))
}

#[proc_macro_derive(SaveMember, attributes(save))]
pub fn derive_save_member(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    into_output(expand_member(&input))
}

fn into_output(result: syn::Result<TokenStream2>) -> TokenStream {
    match result {
        Ok(tokens) => tokens.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

// SaveResource ========================================================

fn expand_resource(input: &DeriveInput) -> syn::Result<TokenStream2> {
    reject_generics(input, "SaveResource")?;
    let save_attr = find_save_attribute(input, "#[save(timing = auto | manual)]")?;
    let timing = parse_timing(save_attr)?;
    let name = &input.ident;

    Ok(quote! {
        impl ::bevy_simplesave::Saveable for #name {
            const TIMING: ::bevy_simplesave::SaveTiming = #timing;
        }
    })
}

fn parse_timing(attr: &Attribute) -> syn::Result<TokenStream2> {
    let mut timing: Option<Ident> = None;

    attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("timing") {
            timing = Some(meta.value()?.parse()?);
            Ok(())
        } else {
            Err(meta.error("unsupported key in `#[save(...)]`; expected `timing`"))
        }
    })?;

    let timing = timing.ok_or_else(|| {
        Error::new_spanned(attr, "`#[save(...)]` is missing the required `timing` key")
    })?;

    match timing.to_string().as_str() {
        "auto" => Ok(quote! { ::bevy_simplesave::SaveTiming::Auto }),
        "manual" => Ok(quote! { ::bevy_simplesave::SaveTiming::Manual }),
        _ => Err(Error::new_spanned(
            &timing,
            "expected `timing` to be `auto` or `manual`",
        )),
    }
}

// SaveMember ==========================================================

fn expand_member(input: &DeriveInput) -> syn::Result<TokenStream2> {
    reject_generics(input, "SaveMember")?;
    let save_attr = find_save_attribute(input, "#[save(key = \"...\")]")?;
    let key = parse_key(save_attr)?;
    let name = &input.ident;

    Ok(quote! {
        impl ::bevy_simplesave::SaveMember for #name {
            const KEY: &'static str = #key;
        }
    })
}

fn parse_key(attr: &Attribute) -> syn::Result<LitStr> {
    let mut key: Option<LitStr> = None;

    attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("key") {
            key = Some(meta.value()?.parse()?);
            Ok(())
        } else {
            Err(meta.error("unsupported key in `#[save(...)]`; expected `key`"))
        }
    })?;

    let key = key.ok_or_else(|| {
        Error::new_spanned(attr, "`#[save(...)]` is missing the required `key` key")
    })?;

    if key.value().is_empty() {
        return Err(Error::new_spanned(&key, "`key` must not be empty"));
    }
    Ok(key)
}

// Shared ==============================================================

fn reject_generics(input: &DeriveInput, derive: &str) -> syn::Result<()> {
    if input.generics.params.is_empty() {
        return Ok(());
    }
    Err(Error::new_spanned(
        &input.generics,
        format!("`#[derive({derive})]` does not support generic types"),
    ))
}

fn find_save_attribute<'a>(input: &'a DeriveInput, expected: &str) -> syn::Result<&'a Attribute> {
    input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("save"))
        .ok_or_else(|| {
            Error::new_spanned(
                &input.ident,
                format!("missing required `{expected}` attribute"),
            )
        })
}

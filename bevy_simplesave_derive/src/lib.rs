use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Attribute, DeriveInput, Error, LitStr, parse_macro_input};

#[proc_macro_derive(SaveMember, attributes(save))]
pub fn derive_save_member(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match expand(&input) {
        Ok(tokens) => tokens.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

fn expand(input: &DeriveInput) -> syn::Result<TokenStream2> {
    reject_generics(input)?;
    let key = parse_key(find_save_attribute(input)?)?;
    let name = &input.ident;

    Ok(quote! {
        impl ::bevy_simplesave::SaveMember for #name {
            const KEY: &'static str = #key;
        }
    })
}

fn reject_generics(input: &DeriveInput) -> syn::Result<()> {
    if input.generics.params.is_empty() {
        return Ok(());
    }
    Err(Error::new_spanned(
        &input.generics,
        "`#[derive(SaveMember)]` does not support generic types",
    ))
}

fn find_save_attribute(input: &DeriveInput) -> syn::Result<&Attribute> {
    input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("save"))
        .ok_or_else(|| {
            Error::new_spanned(
                &input.ident,
                "missing required `#[save(key = \"...\")]` attribute",
            )
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

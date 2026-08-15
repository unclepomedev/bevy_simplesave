use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Attribute, DeriveInput, Error, Ident, parse_macro_input};

#[proc_macro_derive(SaveResource, attributes(save))]
pub fn derive_save_resource(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match expand(&input) {
        Ok(tokens) => tokens.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

fn expand(input: &DeriveInput) -> syn::Result<TokenStream2> {
    if !input.generics.params.is_empty() {
        return Err(Error::new_spanned(
            &input.generics,
            "`#[derive(SaveResource)]` does not support generic types",
        ));
    }

    let save_attr = find_save_attribute(input)?;
    let timing = parse_timing(save_attr)?;
    let name = &input.ident;

    Ok(quote! {
        impl ::bevy_simplesave::Saveable for #name {
            const TIMING: ::bevy_simplesave::SaveTiming = #timing;
        }
    })
}

fn find_save_attribute(input: &DeriveInput) -> syn::Result<&Attribute> {
    input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("save"))
        .ok_or_else(|| {
            Error::new_spanned(
                &input.ident,
                "missing required `#[save(timing = auto | manual)]` attribute",
            )
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

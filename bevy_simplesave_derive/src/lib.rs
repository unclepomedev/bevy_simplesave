use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{DeriveInput, Error, Ident, parse_macro_input};

#[proc_macro_derive(SaveResource, attributes(save))]
pub fn derive_save_resource(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let timing = match extract_timing(&input) {
        Ok(timing) => timing,
        Err(e) => return e.to_compile_error().into(),
    };

    let name = &input.ident;
    let expanded = quote! {
        impl ::bevy_simplesave::Saveable for #name {
            const TIMING: ::bevy_simplesave::SaveTiming = #timing;
        }
    };
    expanded.into()
}

fn extract_timing(input: &DeriveInput) -> syn::Result<TokenStream2> {
    for attr in &input.attrs {
        if !attr.path().is_ident("save") {
            continue;
        }

        let mut timing_ident: Option<Ident> = None;
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("timing") {
                timing_ident = Some(meta.value()?.parse()?);
                Ok(())
            } else {
                Err(meta.error("unsupported key in `#[save(...)]`; expected `timing`"))
            }
        })?;

        return match timing_ident {
            Some(ident) if ident == "auto" => Ok(quote! { ::bevy_simplesave::SaveTiming::Auto }),
            Some(ident) if ident == "manual" => {
                Ok(quote! { ::bevy_simplesave::SaveTiming::Manual })
            }
            Some(ident) => Err(Error::new_spanned(
                &ident,
                "expected `timing` to be `auto` or `manual`",
            )),
            None => Err(Error::new_spanned(
                attr,
                "`#[save(...)]` is missing the required `timing` key",
            )),
        };
    }

    Err(Error::new_spanned(
        &input.ident,
        "missing required `#[save(timing = auto | manual)]` attribute",
    ))
}

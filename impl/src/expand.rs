use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    DeriveInput,
    Result,
};

pub fn derive(input: &DeriveInput) -> TokenStream {
    match try_expand(input) {
        Ok(expanded) => expanded,
        Err(error) => panic!("{error}"),
    }
}

fn try_expand(_input: &DeriveInput) -> Result<TokenStream> {
    Ok(quote!())
}

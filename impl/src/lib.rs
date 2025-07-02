mod expand;

use proc_macro::TokenStream;
use syn::{
    DeriveInput,
    parse_macro_input,
};

#[proc_macro_derive(BizError, attributes(bizerror, from))]
pub fn derive_error(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand::derive_only_impl(&input).into()
}

#[proc_macro_attribute]
pub fn bizerror(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand::derive_complete(&input).into()
}

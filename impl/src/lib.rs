mod expand;

use proc_macro::TokenStream;
use syn::{
    DeriveInput,
    parse_macro_input,
};

#[proc_macro_derive(BizError, attributes(bizcode))]
pub fn derive_error(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand::derive(&input).into()
}

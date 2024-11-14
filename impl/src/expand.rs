use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    DeriveInput,
    Result,
};

pub fn derive(input: &DeriveInput) -> TokenStream {
    match try_expand(input) {
        Ok(expanded) => expanded,
        // If there are invalid attributes in the input, expand to an Error impl
        // anyway to minimize spurious knock-on errors in other code that uses
        // this type as an Error.
        Err(error) => fallback(input, error),
    }
}

fn try_expand(_input: &DeriveInput) -> Result<TokenStream> {
    Ok(quote!())
}

fn fallback(input: &DeriveInput, error: syn::Error) -> TokenStream {
    let ty = &input.ident;
    let (impl_generics, ty_generics, where_clause) =
        input.generics.split_for_impl();

    let error = error.to_compile_error();

    quote! {
        #error

        #[allow(unused_qualifications)]
        #[automatically_derived]
        impl #impl_generics ::thiserror::__private::Error for #ty #ty_generics #where_clause
        where
            // Work around trivial bounds being unstable.
            // https://github.com/rust-lang/rust/issues/48214
            for<'workaround> #ty #ty_generics: ::core::fmt::Debug,
        {}

        #[allow(unused_qualifications)]
        #[automatically_derived]
        impl #impl_generics ::core::fmt::Display for #ty #ty_generics #where_clause {
            fn fmt(&self, __formatter: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::unreachable!()
            }
        }
    }
}

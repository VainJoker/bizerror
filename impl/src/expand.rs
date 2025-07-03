use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    Attribute,
    Data,
    DataEnum,
    DeriveInput,
    Error,
    Fields,
    Ident,
    Lit,
    Meta,
    Result,
    Variant,
};

pub fn derive(input: &DeriveInput) -> TokenStream {
    match try_expand(input) {
        Ok(expanded) => expanded,
        Err(error) => error.to_compile_error(),
    }
}

fn try_expand(input: &DeriveInput) -> Result<TokenStream> {
    match &input.data {
        Data::Enum(data_enum) => expand_enum(input, data_enum),
        _ => Err(Error::new_spanned(
            input,
            "BizError can only be derived for enums",
        )),
    }
}

fn expand_enum(
    input: &DeriveInput,
    data_enum: &DataEnum,
) -> Result<TokenStream> {
    let enum_name = &input.ident;
    let variants = parse_variants(&data_enum.variants)?;

    if variants.is_empty() {
        return Err(Error::new_spanned(
            input,
            "Enum must have at least one variant with #[bizcode()] attribute",
        ));
    }

    let biz_error_impl = generate_biz_error_impl(enum_name, &variants);
    let debug_impl = generate_debug_impl(enum_name, &variants);

    Ok(quote! {
        #biz_error_impl
        #debug_impl
    })
}

struct VariantInfo {
    name:   Ident,
    code:   u16,
    fields: Fields,
}

fn parse_variants(
    variants: &syn::punctuated::Punctuated<Variant, syn::token::Comma>,
) -> Result<Vec<VariantInfo>> {
    let mut variant_infos = Vec::new();
    let mut used_codes = std::collections::HashSet::new();

    for variant in variants {
        if let Some(code) = extract_bizcode_attr(&variant.attrs)? {
            // Check for duplicate error codes
            if used_codes.contains(&code) {
                return Err(Error::new_spanned(
                    variant,
                    format!("Duplicate error code: {}", code),
                ));
            }
            used_codes.insert(code);

            variant_infos.push(VariantInfo {
                name: variant.ident.clone(),
                code,
                fields: variant.fields.clone(),
            });
        }
    }

    Ok(variant_infos)
}

fn extract_bizcode_attr(attrs: &[Attribute]) -> Result<Option<u16>> {
    for attr in attrs {
        if attr.path().is_ident("bizcode") {
            return parse_bizcode_value(attr).map(Some);
        }
    }
    Ok(None)
}

fn parse_bizcode_value(attr: &Attribute) -> Result<u16> {
    match &attr.meta {
        Meta::List(meta_list) => {
            // Parse tokens manually for syn 2.0
            let tokens = &meta_list.tokens;
            let parsed: Lit = syn::parse2(tokens.clone())?;

            match parsed {
                Lit::Int(lit_int) => {
                    let code: u16 = lit_int.base10_parse()?;
                    if code == 0 {
                        return Err(Error::new_spanned(
                            attr,
                            "Error code cannot be 0",
                        ));
                    }
                    Ok(code)
                }
                _ => Err(Error::new_spanned(
                    attr,
                    "bizcode attribute must contain an integer: \
                     #[bizcode(1234)]",
                )),
            }
        }
        _ => Err(Error::new_spanned(
            attr,
            "bizcode attribute must be a list: #[bizcode(1234)]",
        )),
    }
}

fn generate_biz_error_impl(
    enum_name: &Ident,
    variants: &[VariantInfo],
) -> TokenStream {
    let code_arms = variants.iter().map(|v| {
        let variant_name = &v.name;
        let code = v.code;
        let pattern = make_pattern(&v.fields);

        quote! {
            Self::#variant_name #pattern => #code,
        }
    });

    let name_arms = variants.iter().map(|v| {
        let variant_name = &v.name;
        let name_str = variant_name.to_string();
        let pattern = make_pattern(&v.fields);

        quote! {
            Self::#variant_name #pattern => #name_str,
        }
    });

    quote! {
        impl bizerror::BizError for #enum_name {
            fn code(&self) -> u16 {
                match self {
                    #(#code_arms)*
                }
            }

            fn name(&self) -> &str {
                match self {
                    #(#name_arms)*
                }
            }

            // msg() uses default implementation: self.to_string()
        }
    }
}

fn generate_debug_impl(
    enum_name: &Ident,
    variants: &[VariantInfo],
) -> TokenStream {
    let enum_name_str = enum_name.to_string();

    let debug_arms = variants.iter().map(|v| {
        let variant_name = &v.name;
        let variant_name_str = variant_name.to_string();
        let code = v.code;
        let pattern = make_pattern(&v.fields);

        quote! {
            Self::#variant_name #pattern => {
                let mut debug_struct = f.debug_struct(#enum_name_str);
                debug_struct.field("variant", &#variant_name_str);
                debug_struct.field("code", &#code);
                debug_struct.field("message", &self.to_string());
                if let Some(source) = std::error::Error::source(self) {
                    debug_struct.field("source", &source);
                }
                debug_struct.finish()
            }
        }
    });

    quote! {
        impl std::fmt::Debug for #enum_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    #(#debug_arms)*
                }
            }
        }
    }
}

fn make_pattern(fields: &Fields) -> TokenStream {
    match fields {
        // Unit variant: Timeout
        Fields::Unit => quote! {},

        // Tuple variant: RequestBuild(#[from] std::io::Error)
        Fields::Unnamed(_) => quote! { (..) },

        // Struct variant: RequestFailed { status: u16, body: String }
        Fields::Named(_) => quote! { { .. } },
    }
}

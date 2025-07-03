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

    // Parse configuration from #[bizconfig(...)] attribute
    let config = parse_bizconfig(&input.attrs)?;

    // Assign codes to all variants (explicit and automatic)
    let variants = assign_codes(&data_enum.variants, &config)?;

    let biz_error_impl = generate_biz_error_impl(enum_name, &variants, &config);
    let debug_impl = generate_debug_impl(enum_name, &variants, &config);

    Ok(quote! {
        #biz_error_impl
        #debug_impl
    })
}

#[derive(Debug)]
struct BizConfig {
    code_type:      String,
    auto_start:     i64,
    auto_increment: i64,
}

impl Default for BizConfig {
    fn default() -> Self {
        Self {
            code_type:      "u32".to_string(),
            auto_start:     0,
            auto_increment: 1,
        }
    }
}

struct VariantInfo {
    name:   Ident,
    code:   VariantCode,
    fields: Fields,
}

#[derive(Debug)]
enum VariantCode {
    Explicit(TokenStream), // User-specified code
    Auto(usize),           // Auto-assigned index
}

fn parse_bizconfig(attrs: &[Attribute]) -> Result<BizConfig> {
    let mut config = BizConfig::default();

    for attr in attrs {
        if attr.path().is_ident("bizconfig") {
            parse_bizconfig_content(attr, &mut config)?;
        }
    }

    Ok(config)
}

fn parse_bizconfig_content(
    attr: &Attribute,
    config: &mut BizConfig,
) -> Result<()> {
    match &attr.meta {
        Meta::List(meta_list) => {
            // Parse the content inside #[bizconfig(...)]
            let content = &meta_list.tokens;

            // For simplicity, we'll parse manually but handle spaces around =
            // Expected format: key = "value", key = value
            let content_str = content.to_string();

            for part in content_str.split(',') {
                let part = part.trim();
                if let Some((key, value)) = part.split_once('=') {
                    let key = key.trim();
                    let value = value.trim();

                    // Remove quotes if present
                    let value =
                        if value.starts_with('"') && value.ends_with('"') {
                            &value[1..value.len() - 1]
                        } else {
                            value
                        };

                    match key {
                        "code_type" => {
                            config.code_type = value.to_string();
                        }
                        "auto_start" => {
                            config.auto_start =
                                value.parse().map_err(|_| {
                                    Error::new_spanned(
                                        attr,
                                        format!(
                                            "auto_start must be a valid \
                                             integer, got: '{}'",
                                            value
                                        ),
                                    )
                                })?;
                        }
                        "auto_increment" => {
                            config.auto_increment =
                                value.parse().map_err(|_| {
                                    Error::new_spanned(
                                        attr,
                                        format!(
                                            "auto_increment must be a valid \
                                             integer, got: '{}'",
                                            value
                                        ),
                                    )
                                })?;
                        }
                        _ => {
                            return Err(Error::new_spanned(
                                attr,
                                format!("Unknown bizconfig parameter: {}", key),
                            ));
                        }
                    }
                }
            }
        }
        Meta::Path(_) => {
            // #[bizconfig] without parameters - use defaults
        }
        _ => {
            return Err(Error::new_spanned(
                attr,
                "bizconfig must be either #[bizconfig] or #[bizconfig(...)]",
            ));
        }
    }

    Ok(())
}

fn assign_codes(
    variants: &syn::punctuated::Punctuated<Variant, syn::token::Comma>,
    _config: &BizConfig,
) -> Result<Vec<VariantInfo>> {
    let mut result = Vec::new();
    let mut auto_counter = 0usize;

    for variant in variants {
        let code = if let Some(explicit_code) =
            extract_bizcode_attr(&variant.attrs)?
        {
            VariantCode::Explicit(explicit_code)
        } else {
            let auto_code = VariantCode::Auto(auto_counter);
            auto_counter += 1;
            auto_code
        };

        result.push(VariantInfo {
            name: variant.ident.clone(),
            code,
            fields: variant.fields.clone(),
        });
    }

    Ok(result)
}

fn extract_bizcode_attr(attrs: &[Attribute]) -> Result<Option<TokenStream>> {
    for attr in attrs {
        if attr.path().is_ident("bizcode") {
            return parse_bizcode_value(attr).map(Some);
        }
    }
    Ok(None)
}

fn parse_bizcode_value(attr: &Attribute) -> Result<TokenStream> {
    match &attr.meta {
        Meta::List(meta_list) => {
            // Return the tokens as-is, let the compiler handle type checking
            Ok(meta_list.tokens.clone())
        }
        _ => Err(Error::new_spanned(
            attr,
            "bizcode attribute must be a list: #[bizcode(value)]",
        )),
    }
}

fn generate_biz_error_impl(
    enum_name: &Ident,
    variants: &[VariantInfo],
    config: &BizConfig,
) -> TokenStream {
    let code_type = parse_code_type(&config.code_type);

    let code_arms = variants.iter().map(|v| {
        let variant_name = &v.name;
        let code_value = generate_code_value(&v.code, config);
        let pattern = make_pattern(&v.fields);

        quote! {
            Self::#variant_name #pattern => #code_value,
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
            type CodeType = #code_type;

            fn code(&self) -> Self::CodeType {
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
    _config: &BizConfig,
) -> TokenStream {
    let enum_name_str = enum_name.to_string();

    let debug_arms = variants.iter().map(|v| {
        let variant_name = &v.name;
        let variant_name_str = variant_name.to_string();
        let pattern = make_pattern(&v.fields);

        quote! {
            Self::#variant_name #pattern => {
                let mut debug_struct = f.debug_struct(#enum_name_str);
                debug_struct.field("variant", &#variant_name_str);
                debug_struct.field("code", &self.code());
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

fn generate_code_value(code: &VariantCode, config: &BizConfig) -> TokenStream {
    match code {
        VariantCode::Explicit(tokens) => tokens.clone(),
        VariantCode::Auto(index) => {
            let value =
                config.auto_start + (*index as i64 * config.auto_increment);

            if config.code_type == "String" {
                quote! { #value.to_string() }
            } else if config.code_type.contains("str") {
                let value_str = value.to_string();
                quote! { #value_str }
            } else {
                // For numeric types, generate the literal value directly
                match config.code_type.as_str() {
                    "u8" => quote! { #value as u8 },
                    "u16" => quote! { #value as u16 },
                    "u32" => quote! { #value as u32 },
                    "u64" => quote! { #value as u64 },
                    "u128" => quote! { #value as u128 },
                    "i8" => quote! { #value as i8 },
                    "i16" => quote! { #value as i16 },
                    "i32" => quote! { #value as i32 },
                    "i64" => quote! { #value },
                    "i128" => quote! { #value as i128 },
                    _ => quote! { #value as u32 }, // Default fallback
                }
            }
        }
    }
}

fn parse_code_type(type_str: &str) -> TokenStream {
    match type_str {
        "u8" => quote! { u8 },
        "u16" => quote! { u16 },
        "u32" => quote! { u32 },
        "u64" => quote! { u64 },
        "u128" => quote! { u128 },
        "i8" => quote! { i8 },
        "i16" => quote! { i16 },
        "i32" => quote! { i32 },
        "i64" => quote! { i64 },
        "i128" => quote! { i128 },
        "String" => quote! { String },
        "&'static str" => quote! { &'static str },
        _ => {
            // For unknown types, assume it's a valid Rust type
            let ident: TokenStream =
                type_str.parse().unwrap_or_else(|_| quote! { u32 });
            ident
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

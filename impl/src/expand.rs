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
    LitInt,
    LitStr,
    Meta,
    Result,
    Token,
    Variant,
    parse::{
        Parse,
        ParseStream,
    },
    punctuated::Punctuated,
    token::Comma,
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

// Add parsing structure for bizconfig attributes
#[derive(Debug)]
enum BizConfigParam {
    CodeType(String),
    AutoStart(i64),
    AutoIncrement(i64),
}

impl Parse for BizConfigParam {
    fn parse(input: ParseStream) -> Result<Self> {
        let key: Ident = input.parse()?;
        let _: Token![=] = input.parse()?;

        let key_str = key.to_string();
        match key_str.as_str() {
            "code_type" => {
                let value: LitStr = input.parse()?;
                Ok(BizConfigParam::CodeType(value.value()))
            }
            "auto_start" => {
                let value: LitInt = input.parse()?;
                Ok(BizConfigParam::AutoStart(value.base10_parse()?))
            }
            "auto_increment" => {
                let value: LitInt = input.parse()?;
                Ok(BizConfigParam::AutoIncrement(value.base10_parse()?))
            }
            _ => Err(Error::new_spanned(
                key,
                format!("Unknown bizconfig parameter: {}", key_str),
            )),
        }
    }
}

struct BizConfigParams {
    params: Punctuated<BizConfigParam, Comma>,
}

impl Parse for BizConfigParams {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(BizConfigParams {
            params: input.parse_terminated(BizConfigParam::parse, Comma)?,
        })
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
            // Parse using syn::parse for more robust parsing
            let params: BizConfigParams =
                syn::parse2(meta_list.tokens.clone())?;

            for param in params.params {
                match param {
                    BizConfigParam::CodeType(value) => {
                        config.code_type = value;
                    }
                    BizConfigParam::AutoStart(value) => {
                        config.auto_start = value;
                    }
                    BizConfigParam::AutoIncrement(value) => {
                        config.auto_increment = value;
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
    let code_type = config.code_type.parse().unwrap_or_else(|_| quote! { u32 });

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
        VariantCode::Explicit(tokens) => {
            // For explicit codes, use user's literal directly
            // Let the compiler handle type checking
            if config.code_type == "String" {
                quote! { (#tokens).to_string() }
            } else {
                // Use user's literal directly - they can write 100u8, 100u32,
                // "AUTH_ERROR", etc.
                tokens.clone()
            }
        }
        VariantCode::Auto(index) => {
            // For auto-generated codes, we need to generate the appropriate
            // literal
            let value =
                config.auto_start + (*index as i64 * config.auto_increment);

            match config.code_type.as_str() {
                "String" => quote! { #value.to_string() },
                t if t.contains("str") => {
                    let value_str = value.to_string();
                    quote! { #value_str }
                }
                "i64" => quote! { #value }, /* i64 is the native type, no
                                              * cast needed */
                _ => {
                    // For all other numeric types, cast to the target type
                    // This handles u8, u16, u32, u64, u128, i8, i16, i32, i128,
                    // etc.
                    let target_type = config
                        .code_type
                        .parse()
                        .unwrap_or_else(|_| quote! { u32 });
                    quote! { #value as #target_type }
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

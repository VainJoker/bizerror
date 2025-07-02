use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    Attribute,
    Data,
    DeriveInput,
    Error,
    Fields,
    Result,
    Variant,
};

/// derive macro 模式：只添加 impl 块（向后兼容）
pub fn derive_only_impl(input: &DeriveInput) -> TokenStream {
    match try_expand_impl_only(input) {
        Ok(expanded) => expanded,
        Err(error) => error.to_compile_error(),
    }
}

/// attribute macro 模式：完全替换 enum（理想方案）
pub fn derive_complete(input: &DeriveInput) -> TokenStream {
    match try_expand_complete(input) {
        Ok(expanded) => expanded,
        Err(error) => error.to_compile_error(),
    }
}

fn try_expand_impl_only(input: &DeriveInput) -> Result<TokenStream> {
    match &input.data {
        Data::Enum(_) => generate_bizerror_methods(input),
        _ => Err(Error::new_spanned(
            input,
            "BizError can only be used with enum types"
        )),
    }
}

fn try_expand_complete(input: &DeriveInput) -> Result<TokenStream> {
    match &input.data {
        Data::Enum(_) => generate_complete_implementation(input),
        _ => Err(Error::new_spanned(
            input,
            "bizerror attribute can only be used with enum types"
        )),
    }
}

/// 生成完整的实现：替换版 enum + BizError 方法
fn generate_complete_implementation(input: &DeriveInput) -> Result<TokenStream> {
    let enhanced_enum = generate_enhanced_enum(input)?;
    let bizerror_impl = generate_bizerror_methods(input)?;
    
    Ok(quote! {
        #enhanced_enum
        #bizerror_impl
    })
}

/// 生成增强版的 enum（自动添加 derives + 转换属性）
fn generate_enhanced_enum(input: &DeriveInput) -> Result<TokenStream> {
    let name = &input.ident;
    let vis = &input.vis;
    let generics = &input.generics;
    
    // 过滤原有属性，保留除 derive 之外的所有属性
    let filtered_attrs: Vec<_> = input.attrs.iter()
        .filter(|attr| !attr.path().is_ident("derive"))
        .collect();
    
    // 转换所有变体（将 #[bizerror] 转换为 #[error]）
    let transformed_variants = if let Data::Enum(data_enum) = &input.data {
        data_enum.variants.iter()
            .map(transform_variant)
            .collect::<Result<Vec<_>>>()?
    } else {
        return Err(Error::new_spanned(input, "Expected enum"));
    };
    
    Ok(quote! {
        #(#filtered_attrs)*
        #[derive(thiserror::Error, Debug)]
        #vis enum #name #generics {
            #(#transformed_variants,)*
        }
    })
}

/// 转换单个变体（将 #[bizerror] 转换为 #[error]，保留其他属性）
fn transform_variant(variant: &Variant) -> Result<Variant> {
    let mut new_variant = variant.clone();
    let mut new_attrs = Vec::new();
    
    for attr in &variant.attrs {
        if attr.path().is_ident("bizerror") {
            // 转换 #[bizerror] 为 #[error]
            let (_code, message) = parse_bizerror_attr(attr)?;
            let error_attr: Attribute = syn::parse_quote! {
                #[error(#message)]
            };
            new_attrs.push(error_attr);
        } else {
            // 保留其他所有属性（如 #[from], #[source], 文档注释等）
            new_attrs.push(attr.clone());
        }
    }
    
    new_variant.attrs = new_attrs;
    Ok(new_variant)
}

/// 生成 BizError 业务方法
fn generate_bizerror_methods(input: &DeriveInput) -> Result<TokenStream> {
    let name = &input.ident;
    let generics = &input.generics;
    let code_arms = generate_code_arms(input)?;
    
    Ok(quote! {
        #[automatically_derived]
        impl #generics #name #generics {
            /// Get business error code
            pub fn code(&self) -> u16 {
                match self {
                    #(#code_arms,)*
                }
            }
        }
    })
}

fn generate_code_arms(input: &DeriveInput) -> Result<Vec<TokenStream>> {
    let mut code_arms = Vec::new();
    
    if let Data::Enum(data_enum) = &input.data {
        for variant in &data_enum.variants {
            let variant_name = &variant.ident;
            let (code, _) = parse_bizerror_attr_from_variant(variant)?;
            
            let arm = match &variant.fields {
                Fields::Unnamed(fields) if fields.unnamed.len() == 1 => quote! {
                    Self::#variant_name(_) => #code
                },
                Fields::Unnamed(_) => quote! {
                    Self::#variant_name(..) => #code
                },
                Fields::Named(_) => quote! {
                    Self::#variant_name { .. } => #code
                },
                Fields::Unit => quote! {
                    Self::#variant_name => #code
                },
            };
            
            code_arms.push(arm);
        }
    }
    
    Ok(code_arms)
}

/// 从变体中解析 bizerror 属性
fn parse_bizerror_attr_from_variant(variant: &syn::Variant) -> Result<(u16, String)> {
    for attr in &variant.attrs {
        if attr.path().is_ident("bizerror") {
            return parse_bizerror_attr(attr);
        }
    }
    
    Err(Error::new_spanned(
        variant,
        "All enum variants must have #[bizerror(code, \"message\")] attribute"
    ))
}

/// 解析 bizerror 属性
fn parse_bizerror_attr(attr: &Attribute) -> Result<(u16, String)> {
    if let Ok(list) = attr.meta.require_list() {
        let tokens_str = list.tokens.to_string();
        
        // 更智能的解析：找到第一个逗号（不在字符串内）
        let mut in_string = false;
        let mut escaped = false;
        let mut first_comma_pos = None;
        
        let chars: Vec<char> = tokens_str.chars().collect();
        for (i, &ch) in chars.iter().enumerate() {
            if escaped {
                escaped = false;
                continue;
            }
            
            match ch {
                '\\' => escaped = true,
                '"' => in_string = !in_string,
                ',' if !in_string => {
                    first_comma_pos = Some(i);
                    break;
                }
                _ => {}
            }
        }
        
        let (code_str, message_str) = if let Some(comma_pos) = first_comma_pos {
            (tokens_str[..comma_pos].trim(), tokens_str[comma_pos + 1..].trim())
        } else {
            return Err(Error::new_spanned(attr, "bizerror attribute requires both code and message: #[bizerror(code, \"message\")]"));
        };
        
        // 解析错误码
        let code = code_str.parse::<u16>().map_err(|_| {
            Error::new_spanned(attr, "bizerror attribute requires valid u16 error code as first argument")
        })?;
        
        // 解析错误消息
        let message = if message_str.starts_with('"') && message_str.ends_with('"') {
            message_str[1..message_str.len()-1].to_string()
        } else {
            return Err(Error::new_spanned(attr, "bizerror error message must be a string literal"));
        };
        
        Ok((code, message))
    } else {
        Err(Error::new_spanned(attr, "bizerror attribute must be in list form: #[bizerror(code, \"message\")]"))
    }
}

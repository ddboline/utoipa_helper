use proc_macro::TokenStream;
use quote::quote;
use syn::{
    spanned::Spanned, Data, DeriveInput, Expr, Fields, Lit, Meta, PathArguments, Token, Type,
    TypePath,
};

#[proc_macro_derive(UtoipaResponse, attributes(response))]
pub fn derive_utoipa_response_fn(input: TokenStream) -> TokenStream {
    #[derive(Default, Debug)]
    struct UtoipaResponse {
        description: Option<String>,
        content: Option<String>,
        status: Option<String>,
        error: Option<String>,
    }
    let mut utoipa_response = UtoipaResponse::default();
    let input: DeriveInput = syn::parse(input).expect("Failed to parse");
    let DeriveInput {
        attrs, ident, data, ..
    } = input;
    for attr in &attrs {
        if attr.meta.path().is_ident("response") {
            if let Meta::List(metalist) = &attr.meta {
                metalist
                    .parse_nested_meta(|meta| {
                        if let Some(ident) = meta.path.get_ident() {
                            let ident = ident.to_string();
                            if let Expr::Lit(lit) = meta.value()?.parse::<Expr>()? {
                                if let Lit::Str(lit) = lit.lit {
                                    let lit = Some(lit.value());
                                    match ident.as_str() {
                                        "description" => utoipa_response.description = lit,
                                        "content" => utoipa_response.content = lit,
                                        "status" => utoipa_response.status = lit,
                                        "error" => utoipa_response.error = lit,
                                        id => panic!("{} is not a valid key", id),
                                    }
                                }
                            }
                        }
                        Ok(())
                    })
                    .map_err(|e| panic!("encountered error {}", e))
                    .unwrap();
            }
        }
    }
    let mut inner_type: Option<TypePath> = None;
    if let Data::Struct(data_struct) = data {
        if let Fields::Unnamed(fields) = data_struct.fields {
            if let Some(first) = fields.unnamed.first() {
                if let Type::Path(typath) = &first.ty {
                    inner_type = Some(typath.clone());
                }
            }
        }
    }
    let inner_type = inner_type.expect("No inner type");
    let mut inner_type_mod = inner_type.clone();
    if let Some(first) = inner_type_mod.path.segments.first_mut() {
        if let PathArguments::AngleBracketed(args) = &mut first.arguments {
            args.colon2_token = Some(Token![::](args.span()));
        }
    }
    let from_impl = quote! {
        impl From<#inner_type> for #ident {
            fn from(item: #inner_type) -> Self {
                Self(item)
            }
        }
    };
    let content = match utoipa_response.content.as_deref() {
        Some("text/html") => Some(quote! {utoipa_helper::content_type_trait::ContentTypeHtml}),
        Some("text/css") => Some(quote! {utoipa_helper::content_type_trait::ContentTypeCss}),
        Some("application/json") => Some(quote! {utoipa_helper::content_type_trait::ContentTypeJs}),
        Some(val) => panic!("{} is not a valid content type", val),
        None => None,
    };
    let status = match utoipa_response.status.as_deref() {
        Some("OK") => Some(quote! {utoipa_helper::status_code_trait::StatusCodeOk}),
        Some("CREATED") => Some(quote! {utoipa_helper::status_code_trait::StatusCodeCreated}),
        Some("NO_CONTENT") => Some(quote!(utoipa_helper::status_code_trait::StatusCodeNoContent)),
        Some(s) => s
            .parse::<u16>()
            .ok()
            .map(|c| quote!(utoipa_helper::status_code_trait::StatusCodeValue::<#c>)),
        _ => None,
    };
    let content_reply = if let Some(content) = &content {
        quote! {
            use utoipa_helper::content_type_trait::ContentTypeTrait;
            res.headers_mut().insert(
                axum::http::header::CONTENT_TYPE ,
                axum::http::HeaderValue::from_static( #content::content_type_header() )
            );
        }
    } else {
        quote! {}
    };
    let status_reply = if let Some(status) = &status {
        quote! {
            use utoipa_helper::status_code_trait::StatusCodeTrait;
            *res.status_mut() = #status::status_code();
        }
    } else {
        quote! {}
    };
    let axum_into_response_impl = quote! {
        impl axum::response::IntoResponse for #ident {
            fn into_response(self) -> axum::response::Response {
                let mut res = self.0.into_response();
                #content_reply
                #status_reply
                res
            }
        }
    };
    let content_response_entity = if let Some(content) = &content {
        quote! {
            use utoipa_helper::content_type_trait::ContentTypeTrait;
            content_type = #content::content_type().into();
        }
    } else {
        quote! {}
    };
    let description_response_entity = if let Some(description) = &utoipa_response.description {
        quote! {
            resp = resp.description(#description);
        }
    } else {
        quote! {}
    };
    let status_response_entity = if let Some(status) = &status {
        quote! {
            use utoipa_helper::status_code_trait::StatusCodeTrait;
            code = #status::status_code().as_u16().to_string().into();
        }
    } else {
        quote! {}
    };
    let utoipa_into_responses_impl = quote! {
        impl utoipa::IntoResponses for #ident {
            fn responses() -> std::collections::BTreeMap<String, utoipa::openapi::RefOr<utoipa::openapi::Response>> {
                let mut responses = utoipa::openapi::ResponsesBuilder::new();
                let mut resp = utoipa::openapi::ResponseBuilder::new();
                let mut code = std::borrow::Cow::Borrowed("200");
                let mut content_type = std::borrow::Cow::Borrowed("text/html");
                #status_response_entity
                #content_response_entity
                let content = utoipa::openapi::content::ContentBuilder::new().schema(Some(#inner_type::schema())).build();
                resp = resp.content(content_type, content);    
                #description_response_entity
                responses.response(code, resp).build().into()
            }
        }
    };
    let tokens = quote! {
        #from_impl
        #axum_into_response_impl
        #utoipa_into_responses_impl
    };
    tokens.into()
}


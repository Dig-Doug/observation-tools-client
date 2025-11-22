use proc_macro::TokenStream;
use quote::quote;
use syn::parse::Parse;
use syn::parse::ParseStream;
use syn::parse_macro_input;
use syn::spanned::Spanned;
use syn::Expr;
use syn::Ident;
use syn::LitStr;
use syn::Result;
use syn::Token;

/// Parse the observe! macro arguments
enum ObserveArg {
  /// Just a variable or expression (auto-capture name)
  Simple(Expr),
  /// name = "..." or name = CONST
  Name(Expr),
  /// label = "..." or label = expr
  Label(Expr),
  /// payload = expr
  Payload(Expr),
  /// custom = true/false or custom_serialization = true/false
  Custom(bool),
  /// metadata { key: value, ... }
  Metadata(Vec<(Expr, Expr)>),
}

struct ObserveArgs {
  args: Vec<ObserveArg>,
}

impl Parse for ObserveArgs {
  fn parse(input: ParseStream) -> Result<Self> {
    let mut args = Vec::new();

    // Check if this is a simple case (just an expression)
    if input.peek(Ident) && input.peek2(Token![=]) {
      // Named arguments
      while !input.is_empty() {
        let key: Ident = input.parse()?;
        let key_str = key.to_string();

        match key_str.as_str() {
          "name" => {
            input.parse::<Token![=]>()?;
            let value: Expr = input.parse()?;
            args.push(ObserveArg::Name(value));
          }
          "label" => {
            input.parse::<Token![=]>()?;
            let value: Expr = input.parse()?;
            args.push(ObserveArg::Label(value));
          }
          "payload" => {
            input.parse::<Token![=]>()?;
            let value: Expr = input.parse()?;
            args.push(ObserveArg::Payload(value));
          }
          "custom" | "custom_serialization" => {
            input.parse::<Token![=]>()?;
            let value: syn::LitBool = input.parse()?;
            args.push(ObserveArg::Custom(value.value));
          }
          "metadata" => {
            let content;
            syn::braced!(content in input);
            let mut metadata = Vec::new();

            while !content.is_empty() {
              // Parse key as identifier or expression
              let key_expr: Expr = content.parse()?;

              // Convert identifier to string literal
              let key_string = match &key_expr {
                Expr::Path(expr_path)
                  if expr_path.path.segments.len() == 1
                    && expr_path.path.segments[0].arguments.is_empty() =>
                {
                  // Simple identifier - convert to string
                  let ident = &expr_path.path.segments[0].ident;
                  Expr::Lit(syn::ExprLit {
                    attrs: vec![],
                    lit: syn::Lit::Str(LitStr::new(&ident.to_string(), ident.span())),
                  })
                }
                _ => key_expr, // Use expression as-is
              };

              content.parse::<Token![:]>()?;
              let value_expr: Expr = content.parse()?;
              metadata.push((key_string, value_expr));

              if !content.is_empty() {
                content.parse::<Token![,]>()?;
              }
            }

            args.push(ObserveArg::Metadata(metadata));
          }
          _ => {
            return Err(syn::Error::new_spanned(
              key,
              format!("Unknown argument: {}", key_str),
            ));
          }
        }

        if !input.is_empty() {
          input.parse::<Token![,]>()?;
        }
      }
    } else {
      // Simple case: just an expression or name, value
      let first_expr: Expr = input.parse()?;

      if input.is_empty() {
        // observe!(value) - auto-capture variable name
        args.push(ObserveArg::Simple(first_expr));
      } else {
        // observe!(name_expr, value) - explicit name (can be string literal, const, or
        // expression)
        input.parse::<Token![,]>()?;
        let second_expr: Expr = input.parse()?;

        args.push(ObserveArg::Name(first_expr));
        args.push(ObserveArg::Payload(second_expr));

        // Check for custom flag
        if !input.is_empty() {
          input.parse::<Token![,]>()?;
          let third: Ident = input.parse()?;
          if third == "custom" || third == "custom_serialization" {
            input.parse::<Token![=]>()?;
            let value: syn::LitBool = input.parse()?;
            args.push(ObserveArg::Custom(value.value));
          }
        }
      }
    }

    Ok(ObserveArgs { args })
  }
}

/// The observe! procedural macro
///
/// Supports multiple syntaxes:
/// - `observe!(variable)` - Auto-captures variable name
/// - `observe!("name", value)` - Explicit name
/// - `observe!(name = "...", payload = expr)` - Structured syntax
/// - `observe!(name = "...", payload = expr, label = "...")` - With label
/// - `observe!(name = "...", payload = expr, metadata { key: value, ... })` -
///   With metadata
/// - `observe!(name = "...", payload = expr, custom = true)` - Use custom
///   serialization
#[proc_macro]
pub fn observe(input: TokenStream) -> TokenStream {
  let args = parse_macro_input!(input as ObserveArgs);

  // Extract arguments
  let mut name: Option<Expr> = None;
  let mut label: Option<Expr> = None;
  let mut payload: Option<Expr> = None;
  let mut custom = false;
  let mut metadata: Vec<(Expr, Expr)> = Vec::new();
  let mut simple_expr: Option<Expr> = None;

  for arg in args.args {
    match arg {
      ObserveArg::Simple(expr) => simple_expr = Some(expr),
      ObserveArg::Name(n) => name = Some(n),
      ObserveArg::Label(l) => label = Some(l),
      ObserveArg::Payload(p) => payload = Some(p),
      ObserveArg::Custom(c) => custom = c,
      ObserveArg::Metadata(m) => metadata = m,
    }
  }

  // Handle simple case: observe!(variable)
  if let Some(simple) = simple_expr {
    // Try to extract variable name from expression
    let auto_name = match &simple {
      Expr::Path(expr_path) => {
        if expr_path.path.segments.len() == 1 {
          Some(expr_path.path.segments[0].ident.to_string())
        } else {
          None
        }
      }
      _ => None,
    };

    if let Some(var_name) = auto_name {
      name = Some(Expr::Lit(syn::ExprLit {
        attrs: vec![],
        lit: syn::Lit::Str(LitStr::new(&var_name, simple.span())),
      }));
      payload = Some(simple);
    } else {
      return syn::Error::new_spanned(
        simple,
        "Cannot auto-capture name from complex expression. Use observe!(\"name\", value) instead.",
      )
      .to_compile_error()
      .into();
    }
  }

  // Validate required fields
  let name = match name {
    Some(n) => n,
    None => {
      return syn::Error::new(proc_macro2::Span::call_site(), "Missing observation name")
        .to_compile_error()
        .into()
    }
  };

  let payload = match payload {
    Some(p) => p,
    None => {
      return syn::Error::new(
        proc_macro2::Span::call_site(),
        "Missing observation payload",
      )
      .to_compile_error()
      .into()
    }
  };

  // Validate label format (if provided and is a string literal)
  if let Some(Expr::Lit(ref expr_lit)) = label {
    if let syn::Lit::Str(ref label_lit) = expr_lit.lit {
      let label_value = label_lit.value();
      // Basic validation: labels should use path convention
      if label_value.contains("//") || label_value.starts_with('/') || label_value.ends_with('/') {
        return syn::Error::new_spanned(
                    label_lit,
                    "Label should use path convention (e.g., 'api/request/headers'), not start/end with '/' or contain '//'",
                )
                .to_compile_error()
                .into();
      }
    }
  }

  // Build the observation
  let file = file!();
  let line = line!();

  let payload_method = if custom {
    quote! { custom_payload }
  } else {
    quote! { payload }
  };

  let label_call = if let Some(label_expr) = label {
    quote! { .label(#label_expr) }
  } else {
    quote! {}
  };

  let metadata_calls = metadata.iter().map(|(key, value)| {
    quote! {
        .metadata(#key, #value)
    }
  });

  let expanded = quote! {
      {
          ::observation_tools_client::ObservationBuilder::new(#name)
              .#payload_method(&#payload)
              .source(#file, #line)
              #label_call
              #(#metadata_calls)*
              .build()
      }
  };

  TokenStream::from(expanded)
}

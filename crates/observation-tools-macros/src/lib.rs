use proc_macro::TokenStream;
use quote::quote;
use syn::parse::Parse;
use syn::parse::ParseStream;
use syn::parse_macro_input;
use syn::spanned::Spanned;
use syn::Expr;
use syn::LitStr;
use syn::Result;

/// Parse the observe! macro argument - just a single expression for the name
struct ObserveArg {
  name_expr: Expr,
}

impl Parse for ObserveArg {
  fn parse(input: ParseStream) -> Result<Self> {
    let name_expr: Expr = input.parse()?;
    Ok(ObserveArg { name_expr })
  }
}

/// Returns true if the identifier looks like a local variable (snake_case
/// starting with lowercase) Returns false for constants (SCREAMING_SNAKE_CASE)
/// or other patterns
fn is_variable_name(name: &str) -> bool {
  // If it starts with lowercase, treat it as a variable name to auto-capture
  // Constants are typically SCREAMING_SNAKE_CASE (all uppercase with underscores)
  name.chars().next().is_some_and(|c| c.is_lowercase())
}

/// The observe! procedural macro
///
/// Creates an ObservationBuilder with the given name and source info
/// automatically set. The caller then chains builder methods to set payload and
/// other fields.
///
/// Supports:
/// - `observe!("name")` - String literal name
/// - `observe!(CONST_NAME)` - Constant or expression for name
///   (SCREAMING_SNAKE_CASE uses value)
/// - `observe!(variable)` - Auto-captures variable name as the observation name
///   (snake_case)
///
/// Examples:
/// ```ignore
/// // Simple observation with serde serialization (fire-and-forget)
/// observe!("request").serde(&data);
///
/// // With label and metadata
/// observe!("response")
///     .label("api/users")
///     .metadata("status", "200")
///     .serde(&response);
///
/// // Using Debug trait instead of serde
/// observe!("debug_data").debug(&my_struct);
///
/// // Auto-capture variable name (snake_case identifiers)
/// let my_data = get_data();
/// observe!(my_data).serde(&my_data);
///
/// // Use constant value (SCREAMING_SNAKE_CASE identifiers)
/// const OBS_NAME: &str = "my_observation";
/// observe!(OBS_NAME).serde(&data);
///
/// // Wait for upload completion
/// observe!("important").serde(&data).wait_for_upload().await?;
///
/// // Get observation handle
/// let handle = observe!("tracked").serde(&data).handle();
/// ```
#[proc_macro]
pub fn observe(input: TokenStream) -> TokenStream {
  let args = parse_macro_input!(input as ObserveArg);

  // Determine the name - either from a string literal, expression, or
  // auto-captured variable name
  let name_expr = match &args.name_expr {
    // If it's a simple identifier, check if it looks like a variable name (snake_case)
    // Constants (SCREAMING_SNAKE_CASE) should use their value, not their name
    Expr::Path(expr_path) if expr_path.path.segments.len() == 1 => {
      let ident_name = expr_path.path.segments[0].ident.to_string();
      if is_variable_name(&ident_name) {
        // snake_case: auto-capture the variable name as the observation name
        Expr::Lit(syn::ExprLit {
          attrs: vec![],
          lit: syn::Lit::Str(LitStr::new(&ident_name, args.name_expr.span())),
        })
      } else {
        // SCREAMING_SNAKE_CASE or other: use the constant/expression value
        args.name_expr
      }
    }
    // Otherwise use the expression as-is (string literal, etc.)
    _ => args.name_expr,
  };

  // Build the observation builder with name and source info
  let expanded = quote! {
      ::observation_tools::ObservationBuilder::new(#name_expr)
          .source(file!(), line!())
  };

  TokenStream::from(expanded)
}

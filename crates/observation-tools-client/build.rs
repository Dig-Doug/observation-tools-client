use progenitor::GenerationSettings;
use progenitor::InterfaceStyle;
use syn::__private::quote::quote;

fn main() {
  napi_build::setup();

  run_progenitor(
    "openapi.json",
    "observation_tools_openapi.rs",
    GenerationSettings::new()
      .with_interface(InterfaceStyle::Builder)
      .with_inner_type(quote! {crate::server_client::ObservationToolsServerClientOpts})
      .with_pre_hook_async(quote! {crate::server_client::pre_hook_async}),
  );
}

fn run_progenitor(src: &str, output_name: &str, settings: &mut GenerationSettings) {
  println!("cargo:rerun-if-changed={}", src);

  let file = std::fs::File::open(src).unwrap();
  let spec = serde_json::from_reader(file).unwrap();
  let mut generator = progenitor::Generator::new(settings);

  let tokens = match generator.generate_tokens(&spec) {
    Ok(tokens) => tokens,
    Err(e) => {
      panic!("Error generating code from {}: {}", src, e);
    }
  };
  let ast = syn::parse2(tokens).unwrap();
  let content = prettyplease::unparse(&ast);
  let content = content.replace(
    "pub fn ",
    "#[allow(dead_code)] #[allow(mismatched_lifetime_syntaxes)] pub fn ",
  );
  let content = content.replace("pub async fn ", "#[allow(dead_code)] pub async fn ");

  let mut out_file = std::path::Path::new(&std::env::var("OUT_DIR").unwrap()).to_path_buf();
  out_file.push(output_name);

  std::fs::write(out_file, content).unwrap();
}

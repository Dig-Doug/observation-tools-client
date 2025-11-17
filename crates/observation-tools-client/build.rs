use observation_tools_server::api::ApiDoc;
use progenitor::GenerationSettings;
use progenitor::InterfaceStyle;
use syn::__private::quote::quote;
use utoipa::OpenApi;

fn main() {
  napi_build::setup();

  // Generate OpenAPI spec from server code
  let openapi = ApiDoc::openapi();
  let mut spec = serde_json::to_value(&openapi).expect("Failed to serialize OpenAPI spec");

  // Convert OpenAPI 3.1 to 3.0 for progenitor compatibility
  convert_openapi_31_to_30(&mut spec);

  // Optionally write the spec to file for reference
  let spec_path = std::path::Path::new("openapi.json");
  let spec_json = serde_json::to_string_pretty(&spec).expect("Failed to serialize spec");
  std::fs::write(spec_path, spec_json).expect("Failed to write OpenAPI spec");

  run_progenitor(
    spec,
    "observation_tools_openapi.rs",
    GenerationSettings::new()
      .with_interface(InterfaceStyle::Builder)
      .with_inner_type(quote! {crate::server_client::ObservationToolsServerClientOpts})
      .with_pre_hook_async(quote! {crate::server_client::pre_hook_async}),
  );
}

fn run_progenitor(spec: serde_json::Value, output_name: &str, settings: &mut GenerationSettings) {
  // Trigger rebuild if the server crate changes
  println!("cargo:rerun-if-changed=../observation-tools-server/src");

  // Convert the JSON value back to OpenAPI spec type
  let openapi_spec: openapiv3::OpenAPI =
    serde_json::from_value(spec).expect("Failed to deserialize OpenAPI spec");

  let mut generator = progenitor::Generator::new(settings);

  let tokens = match generator.generate_tokens(&openapi_spec) {
    Ok(tokens) => tokens,
    Err(e) => {
      panic!("Error generating code: {}", e);
    }
  };
  let ast = syn::parse2(tokens).expect("Failed to parse generated tokens");
  let content = prettyplease::unparse(&ast);
  let content = content.replace(
    "pub fn ",
    "#[allow(dead_code)] #[allow(mismatched_lifetime_syntaxes)] pub fn ",
  );
  let content = content.replace("pub async fn ", "#[allow(dead_code)] pub async fn ");

  let mut out_file =
    std::path::Path::new(&std::env::var("OUT_DIR").expect("OUT_DIR not set")).to_path_buf();
  out_file.push(output_name);

  std::fs::write(out_file, content).expect("Failed to write generated code");
}

/// Convert OpenAPI 3.1 spec to 3.0 for progenitor compatibility
fn convert_openapi_31_to_30(value: &mut serde_json::Value) {
  // Change openapi version from 3.1.x to 3.0.3
  if let Some(obj) = value.as_object_mut() {
    if let Some(openapi_version) = obj.get_mut("openapi") {
      *openapi_version = serde_json::Value::String("3.0.3".to_string());
    }
  }

  // Recursively process the entire document
  convert_nullable_types(value);
}

/// Recursively convert array-based nullable types to nullable property
fn convert_nullable_types(value: &mut serde_json::Value) {
  match value {
    serde_json::Value::Object(map) => {
      // Check if this is a schema with type as an array
      if let Some(type_value) = map.get("type") {
        if let Some(type_array) = type_value.as_array() {
          // Check if it's a nullable type array like ["string", "null"]
          if type_array.len() == 2 {
            let has_null = type_array.iter().any(|v| v.as_str() == Some("null"));
            if has_null {
              // Find the non-null type
              if let Some(actual_type) = type_array.iter().find(|v| v.as_str() != Some("null")) {
                // Replace type array with single type and add nullable
                map.insert("type".to_string(), actual_type.clone());
                map.insert("nullable".to_string(), serde_json::Value::Bool(true));
              }
            }
          }
        }
      }

      // Handle oneOf with null (OpenAPI 3.1 feature)
      if let Some(one_of) = map.get("oneOf").cloned() {
        if let Some(one_of_array) = one_of.as_array() {
          // Check if oneOf contains {"type": "null"}
          let has_null_variant = one_of_array.iter().any(|v| {
            if let Some(obj) = v.as_object() {
              obj.get("type").and_then(|t| t.as_str()) == Some("null")
            } else {
              false
            }
          });

          if has_null_variant && one_of_array.len() == 2 {
            // Find the non-null schema
            if let Some(actual_schema) = one_of_array
              .iter()
              .find(|v| {
                if let Some(obj) = v.as_object() {
                  obj.get("type").and_then(|t| t.as_str()) != Some("null")
                } else {
                  true
                }
              })
              .cloned()
            {
              // Replace oneOf with the actual schema and add nullable
              map.remove("oneOf");
              if let Some(schema_obj) = actual_schema.as_object() {
                for (k, v) in schema_obj {
                  map.insert(k.clone(), v.clone());
                }
              }
              map.insert("nullable".to_string(), serde_json::Value::Bool(true));
            }
          }
        }
      }

      // Recursively process all values
      for (_, v) in map.iter_mut() {
        convert_nullable_types(v);
      }
    }
    serde_json::Value::Array(arr) => {
      // Recursively process array elements
      for item in arr.iter_mut() {
        convert_nullable_types(item);
      }
    }
    _ => {}
  }
}

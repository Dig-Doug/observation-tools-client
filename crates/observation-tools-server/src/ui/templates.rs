//! Template initialization and filters

use minijinja::path_loader;
use minijinja::Environment;
use minijinja::Value;
use minijinja_autoreload::AutoReloader;
use pulldown_cmark::Options;
use pulldown_cmark::Parser;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::error;

pub fn items_filter(value: Value) -> Value {
  if value.as_object().is_some() {
    let mut items = Vec::new();
    let Ok(values) = value.try_iter() else {
      error!("Failed to iterate over items in items filter");
      return Value::from(Vec::<Value>::new());
    };
    for key in values {
      if let Ok(val) = value.get_item(&key) {
        items.push(Value::from(vec![
          Value::from(key.as_str().unwrap_or("")),
          val,
        ]));
      }
    }
    Value::from(items)
  } else {
    Value::from(Vec::<Value>::new())
  }
}

pub fn render_markdown(value: String) -> String {
  let mut options = Options::empty();
  options.insert(Options::ENABLE_STRIKETHROUGH);
  options.insert(Options::ENABLE_TABLES);
  options.insert(Options::ENABLE_FOOTNOTES);
  options.insert(Options::ENABLE_TASKLISTS);

  let parser = Parser::new_ext(&value, options);
  let mut html_output = String::new();
  pulldown_cmark::html::push_html(&mut html_output, parser);

  ammonia::clean(&html_output)
}

/// Parse a JSON string and return a result object for template rendering.
/// Returns { ok: true, value: <parsed> } on success, { ok: false } on failure.
pub fn parse_json(value: String) -> Value {
  match serde_json::from_str::<serde_json::Value>(&value) {
    Ok(json) => {
      let mut map = std::collections::BTreeMap::new();
      map.insert("ok".to_string(), Value::from(true));
      map.insert("value".to_string(), Value::from_serialize(&json));
      Value::from_iter(map)
    }
    Err(_) => {
      let mut map = std::collections::BTreeMap::new();
      map.insert("ok".to_string(), Value::from(false));
      Value::from_iter(map)
    }
  }
}

/// Initialize the template auto-reloader
pub fn init_templates() -> Arc<AutoReloader> {
  Arc::new(AutoReloader::new(move |notifier| {
    let mut env = Environment::new();

    // Add custom filter to unescape common escape sequences
    env.add_filter("unescape", |value: String| -> String {
      value
        .replace("\\n", "\n")
        .replace("\\r", "\r")
        .replace("\\t", "\t")
        .replace("\\\\", "\\")
    });

    // Add items filter to convert maps to iterable key-value pairs
    env.add_filter("items", items_filter);

    // Add render_markdown filter to convert markdown to sanitized HTML
    env.add_filter("render_markdown", render_markdown);

    // Add parse_json filter to parse JSON strings for template rendering
    env.add_filter("parse_json", parse_json);

    if cfg!(debug_assertions) {
      tracing::info!("Running in local development mode, enabling autoreload for templates");
      let template_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("templates");
      env.set_loader(path_loader(&template_path));
      notifier.watch_path(&template_path, true);
    } else {
      tracing::info!("Using embedded templates");
      minijinja_embed::load_templates!(&mut env);
    }
    Ok(env)
  }))
}

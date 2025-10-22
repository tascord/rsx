use std::{
    env,
    fs::{self, File},
    io::Write,
    path::Path,
    process::Command,
};

fn main() {
    println!("cargo:rerun-if-changed=mdn/attributes.json");

    generate_attr_map();
}

fn generate_attr_map() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("generated_attrs.rs");

    // Read the JSON file
    let json_path = "mdn/attributes.json";
    let json_content = fs::read_to_string(json_path).expect("Failed to read attributes.json");

    // Execute the transformation using Node.js
    let node_script = format!(
        "const attrs = {json_content}; const result = attrs.map(v => `(\"${{v.attr}}\", &[${{v.tags.map(v => `\"${{v}}\"`).join(\", \
         \")}}])`); console.log(result.join(',\\n'));"
    );

    let output = Command::new("node").args(["-e", &node_script]).output().expect("Failed to execute Node.js");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        panic!("Node.js execution failed: {stderr}");
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let generated_entries = stdout.trim();

    // Generate the Rust code
    let rust_code = format!(
        r#"/// Generated attribute map from MDN data
pub const ATTR_MAP: &[(&str, &[&str])] = &[
    {generated_entries}
];
"#
    );

    let mut file = File::create(&dest_path).unwrap();
    file.write_all(rust_code.as_bytes()).unwrap();

    println!("Generated attribute map at: {}", dest_path.display());
}

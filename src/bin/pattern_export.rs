/// Pattern export utility - generates JSON catalog from code patterns
///
/// This utility exports all code-based patterns to the JSON catalog format,
/// ensuring external tools have access to the complete pattern library while
/// maintaining code as the primary source of truth.
use polyphonica::patterns::{
    ElectronicPatterns, FunkPatterns, JazzPatterns, LatinPatterns, PatternCatalog, PatternLibrary,
    PopPatterns, RockPatterns,
};
use std::env;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get output path from command line or use default
    let args: Vec<String> = env::args().collect();
    let output_path = if args.len() > 1 {
        &args[1]
    } else {
        "drum_samples_catalog_exported.json"
    };

    println!("Exporting patterns from code to JSON catalog...");

    // Collect all patterns from library and collections
    let mut all_patterns = Vec::new();

    // Add patterns from library
    let library = PatternLibrary::with_defaults();
    all_patterns.extend(library.all_patterns().into_iter().cloned());

    // Add patterns from genre collections
    all_patterns.extend(RockPatterns::all());
    all_patterns.extend(JazzPatterns::all());
    all_patterns.extend(LatinPatterns::all());
    all_patterns.extend(FunkPatterns::all());
    all_patterns.extend(PopPatterns::all());
    all_patterns.extend(ElectronicPatterns::all());

    println!("Found {} patterns in code library:", all_patterns.len());
    for pattern in &all_patterns {
        println!("  - {} ({})", pattern.display_name, pattern.name);
    }

    // Create JSON catalog from patterns
    let catalog = PatternCatalog::from_patterns(&all_patterns);

    // Export to JSON
    let json_output = catalog.to_json()?;
    fs::write(output_path, json_output)?;

    println!("Successfully exported patterns to: {}", output_path);
    println!("Catalog version: {}", catalog.catalog_version);
    println!("Total patterns exported: {}", catalog.drum_patterns.len());

    Ok(())
}

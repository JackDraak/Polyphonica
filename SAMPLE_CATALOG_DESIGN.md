# Sample Catalog System Design

## Overview

A comprehensive JSON-based catalog system for organizing, discovering, and managing sample collections within the Polyphonica ecosystem. This system provides curation tools for sample libraries and intuitive browsing for users.

## Core Concepts

### Sample Collections
- **Sample Sets**: Grouped collections (e.g., "808 Drum Kit", "Piano Samples C1-C8")
- **Categories**: High-level organization (drums, melodic, percussion, vocals, fx)
- **Tags**: Flexible metadata for search and filtering
- **Quality Ratings**: Community or curator ratings

### Directory Structure
```
samples/
├── catalog.json                 # Master catalog
├── drums/
│   ├── acoustic/
│   │   ├── kit_01/
│   │   │   ├── kick.wav
│   │   │   ├── snare.wav
│   │   │   └── manifest.json
│   │   └── kit_02/
│   └── electronic/
├── melodic/
│   ├── piano/
│   ├── guitar/
│   └── synth/
└── fx/
    ├── reverb/
    └── ambient/
```

## JSON Schema Design

### Master Catalog (`catalog.json`)
```json
{
  "version": "1.0.0",
  "updated": "2025-09-21T12:00:00Z",
  "collections": [
    {
      "id": "acoustic_kit_01",
      "name": "Acoustic Drum Kit 01",
      "description": "Professional studio-recorded acoustic drum kit",
      "category": "drums",
      "subcategory": "acoustic",
      "path": "drums/acoustic/kit_01",
      "tags": ["acoustic", "studio", "rock", "versatile"],
      "bpm": null,
      "key": null,
      "quality_rating": 4.5,
      "sample_count": 12,
      "total_duration_secs": 8.4,
      "created": "2025-09-21T10:00:00Z",
      "curator": "studio_samples",
      "license": "royalty_free",
      "manifest": "drums/acoustic/kit_01/manifest.json"
    }
  ],
  "categories": {
    "drums": {
      "name": "Drums & Percussion",
      "description": "Kick, snare, hi-hat, toms, cymbals, and percussion",
      "subcategories": ["acoustic", "electronic", "ethnic", "fx"]
    },
    "melodic": {
      "name": "Melodic Instruments",
      "description": "Piano, guitar, synth, strings, and tonal instruments",
      "subcategories": ["piano", "guitar", "synth", "strings", "brass", "woodwind"]
    },
    "fx": {
      "name": "Sound Effects",
      "description": "Ambient, noise, impact, transition effects",
      "subcategories": ["ambient", "impact", "sweep", "glitch", "noise"]
    }
  },
  "tags": {
    "style": ["rock", "jazz", "electronic", "hip-hop", "classical", "ambient"],
    "mood": ["aggressive", "mellow", "dark", "bright", "mysterious", "energetic"],
    "production": ["studio", "live", "vintage", "modern", "lo-fi", "hi-fi"],
    "tempo": ["slow", "medium", "fast", "variable"]
  }
}
```

### Collection Manifest (`manifest.json`)
```json
{
  "collection_id": "acoustic_kit_01",
  "name": "Acoustic Drum Kit 01",
  "version": "1.0.0",
  "description": "Professional studio-recorded acoustic drum kit with multiple velocity layers",
  "samples": [
    {
      "id": "kick_01",
      "filename": "kick.wav",
      "name": "Kick Drum",
      "description": "22-inch bass drum, felt beater",
      "base_frequency": 60.0,
      "suggested_frequencies": [55.0, 60.0, 65.0],
      "duration_secs": 1.2,
      "peak_amplitude": 0.95,
      "rms_amplitude": 0.42,
      "spectral_centroid": 80.5,
      "tags": ["kick", "bass", "punchy"],
      "velocity_layer": "medium",
      "sample_rate": 44100,
      "bit_depth": 16,
      "file_size_bytes": 105840,
      "loop_points": null,
      "recommended_envelope": {
        "attack_secs": 0.001,
        "decay_secs": 0.15,
        "sustain_level": 0.3,
        "release_secs": 0.8
      }
    },
    {
      "id": "snare_01",
      "filename": "snare.wav",
      "name": "Snare Drum",
      "description": "14x5.5 steel snare, medium tension",
      "base_frequency": 200.0,
      "suggested_frequencies": [180.0, 200.0, 220.0, 250.0],
      "duration_secs": 0.8,
      "peak_amplitude": 0.88,
      "rms_amplitude": 0.35,
      "spectral_centroid": 2400.0,
      "tags": ["snare", "crisp", "bright"],
      "velocity_layer": "medium",
      "sample_rate": 44100,
      "bit_depth": 16,
      "file_size_bytes": 70560,
      "loop_points": null,
      "recommended_envelope": {
        "attack_secs": 0.001,
        "decay_secs": 0.08,
        "sustain_level": 0.2,
        "release_secs": 0.4
      }
    }
  ],
  "relationships": {
    "complementary": ["electronic_kit_01", "percussion_ethnic_01"],
    "alternatives": ["acoustic_kit_02", "acoustic_kit_vintage"],
    "components_of": null
  },
  "usage_examples": [
    {
      "name": "Basic Rock Beat",
      "description": "4/4 rock pattern with kick and snare",
      "pattern": [
        {"sample_id": "kick_01", "time": 0.0, "velocity": 0.8},
        {"sample_id": "snare_01", "time": 0.5, "velocity": 0.7},
        {"sample_id": "kick_01", "time": 1.0, "velocity": 0.8},
        {"sample_id": "snare_01", "time": 1.5, "velocity": 0.7}
      ]
    }
  ],
  "metadata": {
    "recorded_by": "Studio Engineer",
    "recording_location": "Abbey Road Studios",
    "microphones": ["AKG D112", "Shure SM57"],
    "processing": "minimal EQ, no compression",
    "tempo_range": "60-180 BPM",
    "genre_suitability": ["rock", "pop", "alternative"]
  }
}
```

## Features & Capabilities

### Discovery & Browsing
1. **Category Navigation**: Browse by drums, melodic, fx
2. **Tag-based Search**: Find samples by style, mood, production quality
3. **Quality Filtering**: Filter by rating, duration, file size
4. **Relationship Mapping**: Discover complementary or alternative samples

### Curation Tools
1. **Automatic Analysis**: Extract frequency, amplitude, spectral features
2. **Manual Metadata**: Add descriptions, tags, relationships
3. **Quality Scoring**: Rate samples for community curation
4. **Usage Tracking**: Monitor popular samples and combinations

### Integration Features
1. **Test Tool Integration**: Browse and audition samples directly
2. **Pattern Suggestions**: Recommend sample combinations
3. **Envelope Presets**: Suggested ADSR settings per sample
4. **Frequency Mapping**: Optimal pitch ranges for musical use

## CLI Commands (Test Tool Extension)

### Sample Discovery
```bash
# Browse categories
cargo run --bin polyphonica-test catalog list-categories

# List collections in category
cargo run --bin polyphonica-test catalog list-collections --category drums

# Show collection details
cargo run --bin polyphonica-test catalog show acoustic_kit_01

# Search by tags
cargo run --bin polyphonica-test catalog search --tags "rock,studio" --category drums
```

### Sample Audition
```bash
# Quick audition of sample
cargo run --bin polyphonica-test catalog audition acoustic_kit_01/kick_01 --play

# Audition with suggested frequencies
cargo run --bin polyphonica-test catalog audition acoustic_kit_01/snare_01 --frequencies 180,200,220 --play

# Play usage pattern example
cargo run --bin polyphonica-test catalog play-pattern acoustic_kit_01 "Basic Rock Beat" --play
```

### Catalog Management
```bash
# Scan directory and build catalog
cargo run --bin polyphonica-test catalog scan samples/ --output catalog.json

# Add new collection
cargo run --bin polyphonica-test catalog add-collection samples/drums/new_kit --name "Custom Kit"

# Update sample metadata
cargo run --bin polyphonica-test catalog analyze samples/drums/acoustic/kit_01/kick.wav --update-manifest
```

## Implementation Strategy

### Phase 1: Core Infrastructure
1. **JSON Schema**: Define catalog and manifest structures
2. **File Scanner**: Automatic sample discovery and analysis
3. **Metadata Extraction**: Basic audio feature analysis
4. **CLI Integration**: Basic browse and search commands

### Phase 2: Enhanced Features
1. **Advanced Analysis**: Spectral features, tempo detection
2. **Relationship Engine**: Suggest complementary samples
3. **Pattern Builder**: Create and save sample patterns
4. **Community Features**: Ratings, sharing, curation

### Phase 3: Advanced Tools
1. **Web Interface**: Browser-based catalog management
2. **AI Recommendations**: Machine learning for sample suggestions
3. **Live Performance**: Real-time sample triggering
4. **Cloud Sync**: Share catalogs across devices

## Benefits

### For Library Curators
- **Organized Structure**: Clear hierarchy and metadata
- **Quality Control**: Rating and review system
- **Batch Operations**: Efficient catalog management
- **Standard Format**: Interoperable JSON schema

### For Musicians/Producers
- **Quick Discovery**: Find samples by mood, style, or technical specs
- **Auditioning**: Preview samples with different pitches/envelopes
- **Pattern Building**: Combine samples into musical patterns
- **Educational**: Learn optimal usage for each sample

### For Developers
- **Extensible Schema**: Easy to add new metadata fields
- **Tool Integration**: Seamless CLI and API access
- **Performance**: Efficient search and filtering
- **Standards-based**: JSON schema validation and documentation

This catalog system transforms the `/samples/` directory from a simple file collection into a curated, searchable, and intelligently organized sample library that enhances the creative workflow.
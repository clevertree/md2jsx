# md2jsx Copilot Instructions

## Project Overview
High-performance Markdown to JSX AST transpiler for Web (WASM) and Android (JNI).
Powered by Rust and \`pulldown-cmark\`.

## Architecture

### Key Components
1. **Markdown Parser** - Uses \`pulldown-cmark\` for GFM-compatible parsing.
2. **HTML Filter** - Custom regex-based filter for allowed HTML tags.
3. **AST Generator** - Converts Markdown events into a JSON-serializable AST.
4. **Platform Bindings** - WASM for Web and JNI for Android.

## Implementation Details

### AST Structure
The output is a list of nodes:
\`\`\`json
[
  {
    "type": "element",
    "tag": "h1",
    "props": {},
    "children": [
      { "type": "text", "content": "Hello" }
    ]
  }
]
\`\`\`

### Allowed Tags
The parser accepts an \`allowed_tags\` list to filter which HTML tags are rendered.

## Build & Test

### Rust Tests
\`\`\`bash
cargo test
\`\`\`

### WASM Build
\`\`\`bash
wasm-pack build --release --target web
\`\`\`

### Android Build
\`\`\`bash
bash scripts/build-android.sh
\`\`\`

## Development Workflow

### Modifying AST Generation
- Update the \`Node\` enum and serialization logic in \`src/lib.rs\`.
- Ensure consistency between WASM and JNI outputs.

### Adding GFM Features
- Enable relevant features in \`pulldown-cmark\` options in \`src/lib.rs\`.

## Key Files
- \`src/lib.rs\` - Main implementation and platform bindings.
- \`Cargo.toml\` - Dependencies and crate configuration.
- \`scripts/build-android.sh\` - Android build script.

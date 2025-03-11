# Debugging Mermaid Diagrams

This guide helps you diagnose and fix issues with Mermaid diagram rendering in the Crabwalk web visualizer.

## Common Error: "Cannot read properties of null (reading 'firstChild')"

This error typically occurs when:
1. The Mermaid library cannot parse the diagram content
2. The DOM element for rendering isn't properly set up
3. There's a race condition in the rendering process

## How to Debug

### 1. Use the Test Page

We've created a standalone test page to isolate and debug Mermaid rendering:

```bash
# Run the Mermaid test page
cd /Users/mritchie712/blackbird/yato-main/crabwalk/crabwalk-web
./scripts/debug_mermaid.sh
```

This will open a browser with a test page that:
- Shows multiple test cases for Mermaid diagrams
- Displays detailed error messages
- Allows you to test both valid and invalid content

### 2. Check Your Diagram Content

If you're seeing errors with a specific diagram:

1. Copy the problematic diagram content
2. Start the test page (as shown above)
3. Add a new test case with your content
4. Look for syntax errors in the Mermaid content

### 3. Fix Options

The most reliable way to fix Mermaid rendering issues is to:

1. Import Mermaid directly rather than dynamically loading it
2. Use the render method with a unique ID
3. Directly use the returned SVG content
4. Add robust error handling

## Current Implementation

The current implementation in `src/components/MermaidDiagram.tsx` has been updated to:

1. Use a proper render loop with state management
2. Properly handle errors and display them
3. Use unique IDs for each rendering
4. Show a loading state during processing

## Testing Your Own Diagrams

To test your specific diagrams:

1. Edit `src/test/MermaidTest.tsx`
2. Add your diagram content to the `samples` array
3. Run the test script
4. Check the output and error messages

## Getting Additional Help

If you continue to have issues:

1. Check Mermaid's official syntax guide: https://mermaid.js.org/intro/
2. Look at Mermaid's live editor: https://mermaid.live/
3. Try simplifying your diagram to identify problem areas

## Known Limitations

- Very complex diagrams might be slow to render
- Some advanced features may not be supported
- Auto-generated connections work best with standard naming conventions
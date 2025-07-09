// Mock implementation for typstyle-wasm for testing purposes

export function format(code, config) {
  // Mock formatter - just adds some basic formatting
  return code
    .split('\n')
    .map(line => line.trim())
    .join('\n')
    .replace(/\s+/g, ' ')
    .replace(/\s*=\s*/g, ' = ')
    .replace(/\s*\+\s*/g, ' + ')
    .replace(/\s*\-\s*/g, ' - ')
    .replace(/\s*\*\s*/g, ' * ')
    .replace(/\s*\/\s*/g, ' / ');
}

export function parse(code) {
  // Mock parser - return a simple AST-like JSON
  return JSON.stringify({
    type: "document",
    content: code.split('\n').map((line, i) => ({
      type: "line",
      number: i + 1,
      content: line
    }))
  }, null, 2);
}

export function format_ir(code, config) {
  // Mock IR formatter - return a simple representation
  return `# Intermediate Representation for:\n# ${code.split('\n')[0]}\n\nformat_line(content="${code.replace(/"/g, '\\"')}", config=${JSON.stringify(config)})`;
}
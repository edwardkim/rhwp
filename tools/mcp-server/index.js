#!/usr/bin/env node

/**
 * rhwp MCP Server
 *
 * Model Context Protocol server that exposes rhwp CLI functionality to AI agents.
 * Enables AI assistants to read, analyze, edit, and convert HWP/HWPX documents.
 *
 * @see https://modelcontextprotocol.io/
 * @see https://github.com/edwardkim/rhwp
 */

import { Server } from '@modelcontextprotocol/sdk/server/index.js';
import { StdioServerTransport } from '@modelcontextprotocol/sdk/server/stdio.js';
import {
  CallToolRequestSchema,
  ListResourcesRequestSchema,
  ListToolsRequestSchema,
  ReadResourceRequestSchema,
} from '@modelcontextprotocol/sdk/types.js';
import { spawn } from 'child_process';
import { readFile, access } from 'fs/promises';
import { resolve, dirname, join } from 'path';
import { fileURLToPath } from 'url';
import { constants } from 'fs';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

// Configuration
const CONFIG = {
  rhwpBinaryPath: process.env.RHWP_BINARY_PATH || 'rhwp',
  timeout: parseInt(process.env.RHWP_TIMEOUT || '60000', 10),
  maxOutputSize: parseInt(process.env.RHWP_MAX_OUTPUT_SIZE || '10485760', 10), // 10MB
};

/**
 * Execute rhwp CLI command
 * @param {string[]} args - Command arguments
 * @param {string|null} stdin - Optional stdin input
 * @returns {Promise<{stdout: string, stderr: string, exitCode: number}>}
 */
async function executeRhwp(args, stdin = null) {
  return new Promise((resolve, reject) => {
    const child = spawn(CONFIG.rhwpBinaryPath, args, {
      timeout: CONFIG.timeout,
      maxBuffer: CONFIG.maxOutputSize,
    });

    let stdout = '';
    let stderr = '';

    child.stdout.on('data', (data) => {
      stdout += data.toString();
    });

    child.stderr.on('data', (data) => {
      stderr += data.toString();
    });

    if (stdin) {
      child.stdin.write(stdin);
      child.stdin.end();
    }

    child.on('close', (code) => {
      resolve({
        stdout: stdout.trim(),
        stderr: stderr.trim(),
        exitCode: code || 0,
      });
    });

    child.on('error', (error) => {
      reject(new Error(`Failed to execute rhwp: ${error.message}`));
    });
  });
}

/**
 * Parse JSON output from rhwp, with error handling
 * @param {string} output - Raw output from rhwp
 * @param {number} exitCode - Exit code from rhwp
 * @returns {Object} Parsed JSON or error object
 */
function parseRhwpJson(output, exitCode) {
  if (exitCode !== 0 && exitCode !== 3 && exitCode !== 4) {
    return {
      error: true,
      exitCode,
      message: output || 'Command failed',
    };
  }

  try {
    return JSON.parse(output);
  } catch (e) {
    return {
      error: true,
      message: `Failed to parse JSON: ${e.message}`,
      rawOutput: output,
    };
  }
}

/**
 * Validate file path exists
 * @param {string} path - File path to validate
 * @returns {Promise<boolean>}
 */
async function validateFilePath(path) {
  try {
    await access(path, constants.R_OK);
    return true;
  } catch {
    return false;
  }
}

// MCP Server Implementation
const server = new Server(
  {
    name: 'rhwp-mcp-server',
    version: '0.1.0',
  },
  {
    capabilities: {
      resources: {},
      tools: {},
    },
  }
);

/**
 * MCP Tools Definition
 * Each tool maps to rhwp CLI commands with --json output
 */
const TOOLS = [
  {
    name: 'hwp_info',
    description: 'Get HWP/HWPX document information (format, version, sections, page count, fonts, etc.)',
    inputSchema: {
      type: 'object',
      properties: {
        path: {
          type: 'string',
          description: 'Path to HWP/HWPX file',
        },
      },
      required: ['path'],
    },
  },
  {
    name: 'hwp_fields',
    description: 'List all form fields (누름틀) in HWP/HWPX document with their properties',
    inputSchema: {
      type: 'object',
      properties: {
        path: {
          type: 'string',
          description: 'Path to HWP/HWPX file',
        },
      },
      required: ['path'],
    },
  },
  {
    name: 'hwp_search',
    description: 'Search for text in HWP/HWPX document and return matches with page numbers and context',
    inputSchema: {
      type: 'object',
      properties: {
        path: {
          type: 'string',
          description: 'Path to HWP/HWPX file',
        },
        query: {
          type: 'string',
          description: 'Search query text',
        },
        ignoreCase: {
          type: 'boolean',
          description: 'Ignore case when searching',
          default: false,
        },
        limit: {
          type: 'number',
          description: 'Maximum number of matches to return',
        },
      },
      required: ['path', 'query'],
    },
  },
  {
    name: 'hwp_export_text',
    description: 'Extract plain text content from HWP/HWPX document by page',
    inputSchema: {
      type: 'object',
      properties: {
        path: {
          type: 'string',
          description: 'Path to HWP/HWPX file',
        },
        page: {
          type: 'number',
          description: 'Specific page number to extract (0-based). Omit to extract all pages',
        },
      },
      required: ['path'],
    },
  },
  {
    name: 'hwp_export_tables',
    description: 'Extract tables from HWP/HWPX document as structured JSON with cell merging preserved',
    inputSchema: {
      type: 'object',
      properties: {
        path: {
          type: 'string',
          description: 'Path to HWP/HWPX file',
        },
      },
      required: ['path'],
    },
  },
  {
    name: 'hwp_export_structure',
    description: 'Extract document outline/clause structure as nested JSON tree',
    inputSchema: {
      type: 'object',
      properties: {
        path: {
          type: 'string',
          description: 'Path to HWP/HWPX file',
        },
        mode: {
          type: 'string',
          enum: ['auto', 'outline', 'clause'],
          description: 'Extraction mode: auto (default), outline (IR-based), or clause (pattern-based)',
          default: 'auto',
        },
      },
      required: ['path'],
    },
  },
  {
    name: 'hwp_fill_fields',
    description: 'Fill form fields (누름틀) in HWP/HWPX document with provided values',
    inputSchema: {
      type: 'object',
      properties: {
        path: {
          type: 'string',
          description: 'Path to source HWP/HWPX file',
        },
        data: {
          type: 'object',
          description: 'Field values as key-value pairs (field name -> value)',
          additionalProperties: { type: 'string' },
        },
        output: {
          type: 'string',
          description: 'Output file path. Defaults to <input>_filled.hwp',
        },
        dryRun: {
          type: 'boolean',
          description: 'Preview changes without saving',
          default: false,
        },
      },
      required: ['path', 'data'],
    },
  },
  {
    name: 'hwp_replace_text',
    description: 'Replace all occurrences of text in HWP/HWPX document',
    inputSchema: {
      type: 'object',
      properties: {
        path: {
          type: 'string',
          description: 'Path to source HWP/HWPX file',
        },
        find: {
          type: 'string',
          description: 'Text to find',
        },
        replace: {
          type: 'string',
          description: 'Replacement text',
        },
        ignoreCase: {
          type: 'boolean',
          description: 'Ignore case when searching',
          default: false,
        },
        output: {
          type: 'string',
          description: 'Output file path. Defaults to <input>_replaced.hwp',
        },
        dryRun: {
          type: 'boolean',
          description: 'Preview changes without saving',
          default: false,
        },
      },
      required: ['path', 'find', 'replace'],
    },
  },
  {
    name: 'hwp_set_cell',
    description: 'Set table cell value at specific coordinates in HWP/HWPX document',
    inputSchema: {
      type: 'object',
      properties: {
        path: {
          type: 'string',
          description: 'Path to source HWP/HWPX file',
        },
        table: {
          type: 'number',
          description: 'Table index (0-based)',
        },
        row: {
          type: 'number',
          description: 'Row index (0-based)',
        },
        col: {
          type: 'number',
          description: 'Column index (0-based)',
        },
        text: {
          type: 'string',
          description: 'Cell content',
        },
        keepStyle: {
          type: 'boolean',
          description: 'Preserve cell text style',
          default: false,
        },
        output: {
          type: 'string',
          description: 'Output file path. Defaults to <input>_cell.hwp',
        },
        dryRun: {
          type: 'boolean',
          description: 'Preview changes without saving',
          default: false,
        },
      },
      required: ['path', 'table', 'row', 'col', 'text'],
    },
  },
  {
    name: 'hwp_batch',
    description: 'Process multiple HWP/HWPX files in parallel with a specified command (info, export-text, export-structure, export-tables, fields, search)',
    inputSchema: {
      type: 'object',
      properties: {
        paths: {
          type: 'array',
          items: { type: 'string' },
          description: 'Array of file paths to process',
        },
        command: {
          type: 'string',
          enum: ['info', 'export-text', 'export-structure', 'export-tables', 'fields', 'search'],
          description: 'Command to execute on each file',
        },
        query: {
          type: 'string',
          description: 'Search query (required for search command)',
        },
        mode: {
          type: 'string',
          enum: ['auto', 'outline', 'clause'],
          description: 'Mode for export-structure command',
        },
        threads: {
          type: 'number',
          description: 'Number of parallel threads',
        },
      },
      required: ['paths', 'command'],
    },
  },
  {
    name: 'hwp_export_svg',
    description: 'Export HWP/HWPX pages to SVG format',
    inputSchema: {
      type: 'object',
      properties: {
        path: {
          type: 'string',
          description: 'Path to HWP/HWPX file',
        },
        outputDir: {
          type: 'string',
          description: 'Output directory for SVG files',
        },
        page: {
          type: 'number',
          description: 'Specific page number to export (0-based)',
        },
        profile: {
          type: 'string',
          enum: ['screen', 'print', 'high-quality', 'fast-preview'],
          description: 'Output profile',
        },
      },
      required: ['path'],
    },
  },
  {
    name: 'hwp_export_png',
    description: 'Export HWP/HWPX pages to PNG format (requires native-skia feature)',
    inputSchema: {
      type: 'object',
      properties: {
        path: {
          type: 'string',
          description: 'Path to HWP/HWPX file',
        },
        outputDir: {
          type: 'string',
          description: 'Output directory for PNG files',
        },
        page: {
          type: 'number',
          description: 'Specific page number to export (0-based)',
        },
        scale: {
          type: 'number',
          description: 'Scale factor',
        },
        dpi: {
          type: 'number',
          description: 'DPI for output',
        },
        vlmTarget: {
          type: 'string',
          enum: ['claude', 'gpt4v-low', 'gpt4v-high', 'gemini', 'qwen-vl', 'llava'],
          description: 'VLM target preset',
        },
      },
      required: ['path'],
    },
  },
  {
    name: 'hwp_export_pdf',
    description: 'Export HWP/HWPX document to PDF format',
    inputSchema: {
      type: 'object',
      properties: {
        path: {
          type: 'string',
          description: 'Path to HWP/HWPX file',
        },
        output: {
          type: 'string',
          description: 'Output PDF file path',
        },
        page: {
          type: 'number',
          description: 'Specific page number to export (0-based)',
        },
        backend: {
          type: 'string',
          enum: ['svg', 'direct'],
          description: 'PDF backend',
        },
      },
      required: ['path'],
    },
  },
  {
    name: 'hwp_convert',
    description: 'Convert HWP/HWPX to editable HWP format',
    inputSchema: {
      type: 'object',
      properties: {
        input: {
          type: 'string',
          description: 'Input HWP/HWPX file path',
        },
        output: {
          type: 'string',
          description: 'Output HWP file path',
        },
        verify: {
          type: 'boolean',
          description: 'Verify IR consistency after conversion',
          default: false,
        },
        verifyPages: {
          type: 'boolean',
          description: 'Verify page count consistency',
          default: false,
        },
      },
      required: ['input', 'output'],
    },
  },
  {
    name: 'hwp_export_hwpx',
    description: 'Convert HWP to HWPX format (ZIP+XML)',
    inputSchema: {
      type: 'object',
      properties: {
        input: {
          type: 'string',
          description: 'Input HWP file path',
        },
        output: {
          type: 'string',
          description: 'Output HWPX file path',
        },
        verify: {
          type: 'boolean',
          description: 'Verify IR consistency after conversion',
          default: false,
        },
        verifyPages: {
          type: 'boolean',
          description: 'Verify page count consistency',
          default: false,
        },
      },
      required: ['input'],
    },
  },
];

// Tool handlers
const TOOL_HANDLERS = {
  async hwp_info({ path }) {
    const exists = await validateFilePath(path);
    if (!exists) {
      return { error: true, message: `File not found: ${path}` };
    }

    const result = await executeRhwp(['info', path, '--json']);
    return parseRhwpJson(result.stdout, result.exitCode);
  },

  async hwp_fields({ path }) {
    const exists = await validateFilePath(path);
    if (!exists) {
      return { error: true, message: `File not found: ${path}` };
    }

    const result = await executeRhwp(['fields', path, '--json']);
    return parseRhwpJson(result.stdout, result.exitCode);
  },

  async hwp_search({ path, query, ignoreCase, limit }) {
    const exists = await validateFilePath(path);
    if (!exists) {
      return { error: true, message: `File not found: ${path}` };
    }

    const args = ['search', path, query, '--json'];
    if (ignoreCase) args.push('--ignore-case');
    if (limit) args.push('--limit', limit.toString());

    const result = await executeRhwp(args);
    return parseRhwpJson(result.stdout, result.exitCode);
  },

  async hwp_export_text({ path, page }) {
    const exists = await validateFilePath(path);
    if (!exists) {
      return { error: true, message: `File not found: ${path}` };
    }

    const args = ['export-text', path, '--json'];
    if (page !== undefined) args.push('-p', page.toString());

    const result = await executeRhwp(args);
    return parseRhwpJson(result.stdout, result.exitCode);
  },

  async hwp_export_tables({ path }) {
    const exists = await validateFilePath(path);
    if (!exists) {
      return { error: true, message: `File not found: ${path}` };
    }

    const result = await executeRhwp(['export-tables', path, '--json']);
    return parseRhwpJson(result.stdout, result.exitCode);
  },

  async hwp_export_structure({ path, mode = 'auto' }) {
    const exists = await validateFilePath(path);
    if (!exists) {
      return { error: true, message: `File not found: ${path}` };
    }

    const args = ['export-structure', path, '--json'];
    if (mode && mode !== 'auto') args.push('--mode', mode);

    const result = await executeRhwp(args);
    return parseRhwpJson(result.stdout, result.exitCode);
  },

  async hwp_fill_fields({ path, data, output, dryRun = false }) {
    const exists = await validateFilePath(path);
    if (!exists) {
      return { error: true, message: `File not found: ${path}` };
    }

    const args = ['edit', 'fill-fields', path, '--data', JSON.stringify(data), '--json'];
    if (output) args.push('-o', output);
    if (dryRun) args.push('--dry-run');

    const result = await executeRhwp(args);
    return parseRhwpJson(result.stdout, result.exitCode);
  },

  async hwp_replace_text({ path, find, replace, ignoreCase = false, output, dryRun = false }) {
    const exists = await validateFilePath(path);
    if (!exists) {
      return { error: true, message: `File not found: ${path}` };
    }

    const args = ['edit', 'replace-text', path, '--find', find, '--replace', replace, '--json'];
    if (ignoreCase) args.push('--ignore-case');
    if (output) args.push('-o', output);
    if (dryRun) args.push('--dry-run');

    const result = await executeRhwp(args);
    return parseRhwpJson(result.stdout, result.exitCode);
  },

  async hwp_set_cell({ path, table, row, col, text, keepStyle = false, output, dryRun = false }) {
    const exists = await validateFilePath(path);
    if (!exists) {
      return { error: true, message: `File not found: ${path}` };
    }

    const args = [
      'edit',
      'set-cell',
      path,
      '--table',
      table.toString(),
      '--row',
      row.toString(),
      '--col',
      col.toString(),
      '--text',
      text,
      '--json',
    ];
    if (keepStyle) args.push('--keep-style');
    if (output) args.push('-o', output);
    if (dryRun) args.push('--dry-run');

    const result = await executeRhwp(args);
    return parseRhwpJson(result.stdout, result.exitCode);
  },

  async hwp_batch({ paths, command, query, mode, threads }) {
    const args = ['batch', command, '--json'];
    if (command === 'search' && !query) {
      return { error: true, message: 'query is required for search command' };
    }
    if (query) args.push('--query', query);
    if (mode) args.push('--mode', mode);
    if (threads) args.push('--threads', threads.toString());

    const stdin = paths.join('\n');
    const result = await executeRhwp(args, stdin);

    // Parse NDJSON output
    const lines = result.stdout.split('\n').filter((line) => line.trim());
    const records = lines.map((line) => {
      try {
        return JSON.parse(line);
      } catch (e) {
        return { error: true, message: `Failed to parse NDJSON line: ${e.message}`, rawLine: line };
      }
    });

    return {
      schemaVersion: '1.0',
      command,
      processedCount: records.length,
      records,
      exitCode: result.exitCode,
    };
  },

  async hwp_export_svg({ path, outputDir, page, profile }) {
    const exists = await validateFilePath(path);
    if (!exists) {
      return { error: true, message: `File not found: ${path}` };
    }

    const args = ['export-svg', path, '--json'];
    if (outputDir) args.push('-o', outputDir);
    if (page !== undefined) args.push('-p', page.toString());
    if (profile) args.push('--profile', profile);

    const result = await executeRhwp(args);
    return parseRhwpJson(result.stdout, result.exitCode);
  },

  async hwp_export_png({ path, outputDir, page, scale, dpi, vlmTarget }) {
    const exists = await validateFilePath(path);
    if (!exists) {
      return { error: true, message: `File not found: ${path}` };
    }

    const args = ['export-png', path];
    if (outputDir) args.push('-o', outputDir);
    if (page !== undefined) args.push('-p', page.toString());
    if (scale) args.push('--scale', scale.toString());
    if (dpi) args.push('--dpi', dpi.toString());
    if (vlmTarget) args.push('--vlm-target', vlmTarget);

    const result = await executeRhwp(args);

    if (result.exitCode === 0) {
      return { success: true, stderr: result.stderr };
    } else {
      return { error: true, exitCode: result.exitCode, message: result.stderr || result.stdout };
    }
  },

  async hwp_export_pdf({ path, output, page, backend }) {
    const exists = await validateFilePath(path);
    if (!exists) {
      return { error: true, message: `File not found: ${path}` };
    }

    const args = ['export-pdf', path];
    if (output) args.push('-o', output);
    if (page !== undefined) args.push('-p', page.toString());
    if (backend) args.push('--backend', backend);

    const result = await executeRhwp(args);

    if (result.exitCode === 0) {
      return { success: true, output: output || 'default path', stderr: result.stderr };
    } else {
      return { error: true, exitCode: result.exitCode, message: result.stderr || result.stdout };
    }
  },

  async hwp_convert({ input, output, verify = false, verifyPages = false }) {
    const exists = await validateFilePath(input);
    if (!exists) {
      return { error: true, message: `File not found: ${input}` };
    }

    const args = ['convert', input, output];
    if (verify) args.push('--verify');
    if (verifyPages) args.push('--verify-pages');

    const result = await executeRhwp(args);

    if (result.exitCode === 0) {
      return { success: true, input, output, stderr: result.stderr };
    } else {
      return { error: true, exitCode: result.exitCode, message: result.stderr || result.stdout };
    }
  },

  async hwp_export_hwpx({ input, output, verify = false, verifyPages = false }) {
    const exists = await validateFilePath(input);
    if (!exists) {
      return { error: true, message: `File not found: ${input}` };
    }

    const args = ['export-hwpx', input];
    if (output) args.push(output);
    if (verify) args.push('--verify');
    if (verifyPages) args.push('--verify-pages');

    const result = await executeRhwp(args);

    if (result.exitCode === 0) {
      return { success: true, input, output: output || 'default path', stderr: result.stderr };
    } else {
      return { error: true, exitCode: result.exitCode, message: result.stderr || result.stdout };
    }
  },
};

// Register list tools handler
server.setRequestHandler(ListToolsRequestSchema, async () => {
  return {
    tools: TOOLS,
  };
});

// Register call tool handler
server.setRequestHandler(CallToolRequestSchema, async (request) => {
  const { name, arguments: args } = request.params;

  const handler = TOOL_HANDLERS[name];
  if (!handler) {
    throw new Error(`Unknown tool: ${name}`);
  }

  try {
    const result = await handler(args || {});
    return {
      content: [
        {
          type: 'text',
          text: JSON.stringify(result, null, 2),
        },
      ],
    };
  } catch (error) {
    return {
      content: [
        {
          type: 'text',
          text: JSON.stringify(
            {
              error: true,
              message: error.message,
              stack: error.stack,
            },
            null,
            2
          ),
        },
      ],
      isError: true,
    };
  }
});

// MCP Resources - expose documentation and examples
const RESOURCES = [
  {
    uri: 'rhwp://docs/cli-commands',
    name: 'rhwp CLI Commands Manual',
    description: 'Complete CLI commands reference',
    mimeType: 'text/markdown',
  },
  {
    uri: 'rhwp://docs/agents',
    name: 'rhwp Agent Task Playbook',
    description: 'Guide for AI agents working with rhwp',
    mimeType: 'text/markdown',
  },
  {
    uri: 'rhwp://docs/json-pipeline',
    name: 'CLI JSON Pipeline Guide',
    description: 'Guide for using rhwp JSON pipeline',
    mimeType: 'text/markdown',
  },
];

server.setRequestHandler(ListResourcesRequestSchema, async () => {
  return {
    resources: RESOURCES,
  };
});

server.setRequestHandler(ReadResourceRequestSchema, async (request) => {
  const { uri } = request.params;

  const resourceMap = {
    'rhwp://docs/cli-commands': join(__dirname, '../../mydocs/manual/cli_commands.md'),
    'rhwp://docs/agents': join(__dirname, '../../mydocs/manual/agent_task_playbook.md'),
    'rhwp://docs/json-pipeline': join(__dirname, '../../mydocs/manual/cli_json_pipeline_guide.md'),
  };

  const filePath = resourceMap[uri];
  if (!filePath) {
    throw new Error(`Unknown resource: ${uri}`);
  }

  try {
    const content = await readFile(filePath, 'utf-8');
    return {
      contents: [
        {
          uri,
          mimeType: 'text/markdown',
          text: content,
        },
      ],
    };
  } catch (error) {
    throw new Error(`Failed to read resource ${uri}: ${error.message}`);
  }
});

// Start server
async function main() {
  const transport = new StdioServerTransport();
  await server.connect(transport);

  // Log to stderr (stdout is reserved for MCP protocol)
  console.error('rhwp MCP server started');
  console.error(`rhwp binary: ${CONFIG.rhwpBinaryPath}`);
  console.error(`timeout: ${CONFIG.timeout}ms`);
}

main().catch((error) => {
  console.error('Fatal error:', error);
  process.exit(1);
});

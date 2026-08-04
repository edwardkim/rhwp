#!/usr/bin/env node

/**
 * Test suite for rhwp MCP server
 *
 * Validates basic functionality without requiring Claude Desktop:
 * - rhwp binary accessibility
 * - Command execution
 * - JSON parsing
 * - Error handling
 */

import { spawn } from 'child_process';
import { writeFile, unlink } from 'fs/promises';
import { tmpdir } from 'os';
import { join } from 'path';

const CONFIG = {
  rhwpBinaryPath: process.env.RHWP_BINARY_PATH || 'rhwp',
};

let testsPassed = 0;
let testsFailed = 0;

function log(message) {
  console.log(`[TEST] ${message}`);
}

function success(message) {
  console.log(`✓ ${message}`);
  testsPassed++;
}

function failure(message, error) {
  console.error(`✗ ${message}`);
  if (error) {
    console.error(`  Error: ${error.message || error}`);
  }
  testsFailed++;
}

async function executeRhwp(args) {
  return new Promise((resolve, reject) => {
    const child = spawn(CONFIG.rhwpBinaryPath, args, {
      timeout: 10000,
    });

    let stdout = '';
    let stderr = '';

    child.stdout.on('data', (data) => {
      stdout += data.toString();
    });

    child.stderr.on('data', (data) => {
      stderr += data.toString();
    });

    child.on('close', (code) => {
      resolve({
        stdout: stdout.trim(),
        stderr: stderr.trim(),
        exitCode: code || 0,
      });
    });

    child.on('error', (error) => {
      reject(error);
    });
  });
}

async function test1_rhwpVersion() {
  log('Test 1: Check rhwp binary version');
  try {
    const result = await executeRhwp(['--version']);
    if (result.exitCode === 0 && result.stdout) {
      success(`rhwp version: ${result.stdout}`);
    } else {
      failure('rhwp --version failed');
    }
  } catch (error) {
    failure('rhwp binary not accessible', error);
  }
}

async function test2_capabilities() {
  log('Test 2: Check rhwp capabilities');
  try {
    const result = await executeRhwp(['capabilities', '--json']);
    if (result.exitCode === 0) {
      const caps = JSON.parse(result.stdout);
      if (caps.schemaVersion && caps.commands) {
        success(`rhwp capabilities: ${caps.commands.length} commands available`);
      } else {
        failure('capabilities JSON missing expected fields');
      }
    } else {
      failure('capabilities command failed');
    }
  } catch (error) {
    failure('capabilities test failed', error);
  }
}

async function test3_capabilitiesMcp() {
  log('Test 3: Check rhwp MCP capabilities');
  try {
    const result = await executeRhwp(['capabilities', '--mcp']);
    if (result.exitCode === 0) {
      const mcpCaps = JSON.parse(result.stdout);
      if (mcpCaps.schemaVersion && mcpCaps.tools) {
        success(`rhwp MCP capabilities: ${mcpCaps.tools.length} tools defined`);
      } else {
        failure('MCP capabilities JSON missing expected fields');
      }
    } else {
      failure('capabilities --mcp command failed');
    }
  } catch (error) {
    failure('MCP capabilities test failed', error);
  }
}

async function test4_createTestDocument() {
  log('Test 4: Create test document');

  // Create a minimal test HWP document using gen-table if available
  try {
    const result = await executeRhwp(['gen-table']);
    if (result.exitCode === 0) {
      success('Test document generation capability verified');
      return true;
    } else {
      log('gen-table not available, skipping document creation tests');
      return false;
    }
  } catch (error) {
    log('gen-table not available, skipping document creation tests');
    return false;
  }
}

async function test5_infoCommand() {
  log('Test 5: Test info command with JSON output');

  // This test would require a sample file
  // For now, we'll just verify the command syntax is correct
  try {
    // Test with non-existent file to verify error handling
    const result = await executeRhwp(['info', 'nonexistent.hwp', '--json']);

    // Exit code should be 1 (runtime failure) for missing file
    if (result.exitCode === 1) {
      success('Error handling works correctly (missing file returns exit code 1)');
    } else {
      failure(`Unexpected exit code for missing file: ${result.exitCode}`);
    }
  } catch (error) {
    failure('info command test failed', error);
  }
}

async function test6_mcpServerImport() {
  log('Test 6: Verify MCP server module can be imported');

  try {
    // Try to import the module
    const serverModule = await import('./index.js');
    success('MCP server module imports successfully');
  } catch (error) {
    failure('MCP server module import failed', error);
  }
}

async function runTests() {
  console.log('=== rhwp MCP Server Test Suite ===\n');

  log(`Using rhwp binary: ${CONFIG.rhwpBinaryPath}`);
  console.log('');

  await test1_rhwpVersion();
  await test2_capabilities();
  await test3_capabilitiesMcp();
  const hasTestDoc = await test4_createTestDocument();
  await test5_infoCommand();
  await test6_mcpServerImport();

  console.log('\n=== Test Results ===');
  console.log(`Passed: ${testsPassed}`);
  console.log(`Failed: ${testsFailed}`);
  console.log(`Total:  ${testsPassed + testsFailed}`);

  if (testsFailed > 0) {
    console.log('\n⚠ Some tests failed. Please check the errors above.');
    process.exit(1);
  } else {
    console.log('\n✓ All tests passed!');
    process.exit(0);
  }
}

// Run tests
runTests().catch((error) => {
  console.error('Fatal error running tests:', error);
  process.exit(1);
});

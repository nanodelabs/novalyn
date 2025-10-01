/**
 * Basic smoke tests for changelogen npm package
 * 
 * Run with: node --test test/basic.test.js
 */

const { test, describe } = require('node:test');
const assert = require('node:assert');

describe('changelogen npm package', () => {
  test('exports expected functions', () => {
    const changelogen = require('..');
    
    assert.ok(typeof changelogen.generate === 'function', 'exports generate function');
    assert.ok(typeof changelogen.release === 'function', 'exports release function');
    assert.ok(typeof changelogen.showVersion === 'function', 'exports showVersion function');
  });

  test('generate returns a promise', () => {
    const changelogen = require('..');
    const result = changelogen.generate();
    
    assert.ok(result instanceof Promise, 'generate returns a Promise');
  });

  test('release returns a promise', () => {
    const changelogen = require('..');
    const result = changelogen.release();
    
    assert.ok(result instanceof Promise, 'release returns a Promise');
  });

  test('showVersion returns a promise', () => {
    const changelogen = require('..');
    const result = changelogen.showVersion();
    
    assert.ok(result instanceof Promise, 'showVersion returns a Promise');
  });
});

describe('TypeScript definitions', () => {
  test('index.d.ts exists', async () => {
    const fs = require('node:fs/promises');
    const path = require('node:path');
    
    const defsPath = path.join(__dirname, '..', 'index.d.ts');
    const stats = await fs.stat(defsPath);
    
    assert.ok(stats.isFile(), 'index.d.ts exists and is a file');
  });
});

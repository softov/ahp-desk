import { describe, expect, it } from 'vitest';
import { hostId, normalizeUrl, shortUrl, withToken } from './url.js';

describe('normalizeUrl', () => {
  it('adds ws:// to a bare address', () => {
    expect(normalizeUrl('127.0.0.1:9187')).toBe('ws://127.0.0.1:9187');
    expect(normalizeUrl('  host.example:9187 ')).toBe('ws://host.example:9187');
  });

  it('keeps ws:// and wss:// as typed', () => {
    expect(normalizeUrl('wss://host.example')).toBe('wss://host.example');
    expect(normalizeUrl('WS://host')).toBe('WS://host');
  });

  it('leaves an empty address empty', () => {
    expect(normalizeUrl('   ')).toBe('');
  });
});

describe('withToken', () => {
  it('puts the token in the query, escaped', () => {
    expect(withToken('ws://h:1', 'a b&c')).toBe('ws://h:1?tkn=a%20b%26c');
    expect(withToken('ws://h:1/?x=1', 't')).toBe('ws://h:1/?x=1&tkn=t');
  });

  it('adds nothing with no token', () => {
    expect(withToken('ws://h:1', null)).toBe('ws://h:1');
    expect(withToken('ws://h:1', '')).toBe('ws://h:1');
  });
});

describe('shortUrl', () => {
  it('drops a plain ws:// only', () => {
    expect(shortUrl('ws://127.0.0.1:9187')).toBe('127.0.0.1:9187');
    expect(shortUrl('wss://host.example')).toBe('wss://host.example');
  });
});

describe('hostId', () => {
  it('makes a slug from the name, unique among the ids taken', () => {
    expect(hostId('My Office', [])).toBe('my-office');
    expect(hostId('My Office', ['my-office'])).toBe('my-office-2');
    expect(hostId('Café!', [])).toBe('cafe');
    expect(hostId('***', [])).toBe('host');
  });
});

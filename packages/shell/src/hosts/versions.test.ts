import { describe, expect, it } from 'vitest';
import { newer } from './versions.js';

// The same table as ahpd's test/update.test.ts, in the same order, so the two never disagree.
describe('newer', () => {
  it.each([
    ['0.6.0', '0.5.0', true],
    ['0.10.0', '0.9.0', true],
    ['1.0.0', '0.99.99', true],
    ['0.5.0', '0.5.0', false],
    ['0.5.0', '0.6.0', false],
    ['0.5.0', '0.5.0-beta.1', true],
    ['0.5.0-beta.1', '0.5.0', false],
    ['0.5.0-beta.2', '0.5.0-beta.1', false],
    ['0.6.0', 'unknown', false],
    ['unknown', '0.5.0', false],
    ['', '0.5.0', false],
    ['1.2', '0.5.0', false],
    ['0.6.0', '1.2', false],
  ])('%s over %s is %s', (latest, current, expected) => {
    expect(newer(latest, current)).toBe(expected);
  });
});

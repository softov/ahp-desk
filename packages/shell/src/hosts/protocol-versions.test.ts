import { describe, expect, it } from 'vitest';
import { OFFERED_PROTOCOL_VERSIONS, offeringEveryVersion } from './protocol-versions.js';

describe('offeringEveryVersion', () => {
  it('offers every version in an initialize, as text or as an object', () => {
    const asText = offeringEveryVersion(
      JSON.stringify({ jsonrpc: '2.0', id: 1, method: 'initialize', params: { clientId: 'c', protocolVersions: ['1.0.0'] } }),
    );
    expect(JSON.parse(asText).params).toEqual({ clientId: 'c', protocolVersions: [...OFFERED_PROTOCOL_VERSIONS] });
    const asObject = offeringEveryVersion({ method: 'initialize', params: {} });
    expect(asObject.params).toEqual({ protocolVersions: [...OFFERED_PROTOCOL_VERSIONS] });
  });

  it('offers 1.0.0 first, then 0.10.0 and 0.9.0', () => {
    expect(OFFERED_PROTOCOL_VERSIONS.slice(0, 2)).toEqual(['1.0.0', '0.10.0']);
    expect(OFFERED_PROTOCOL_VERSIONS).toContain('0.9.0');
  });

  it('leaves every other frame as it came', () => {
    const other = { method: 'subscribe', params: { uri: 'x' } };
    expect(offeringEveryVersion(other)).toBe(other);
    expect(offeringEveryVersion('not json')).toBe('not json');
  });
});

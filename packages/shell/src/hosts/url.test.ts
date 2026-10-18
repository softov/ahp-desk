import { describe, expect, it } from 'vitest';
import { ahpdPort, hostId, normalizeUrl, serverFolder, shortUrl, splitToken, sshTarget, withToken } from './url.js';

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

describe('splitToken', () => {
  it('takes the token out of ?tkn= and ?token=, with or without the slash', () => {
    for (const pasted of [
      'ws://10.0.0.5:9137?tkn=tkn_123',
      'ws://10.0.0.5:9137/?tkn=tkn_123',
      'ws://10.0.0.5:9137/?token=tkn_123',
      '  ws://10.0.0.5:9137/?tkn=tkn_123  ',
    ])
      expect(splitToken(pasted), pasted).toEqual({ address: 'ws://10.0.0.5:9137', token: 'tkn_123' });
  });

  it('keeps a path, other parameters and a fragment', () => {
    expect(splitToken('wss://host.example/ahp/?a=1&tkn=x%2By&b=2#f')).toEqual({
      address: 'wss://host.example/ahp/?a=1&b=2#f',
      token: 'x+y',
    });
    expect(splitToken('host.example:9187/?tkn=x')).toEqual({ address: 'host.example:9187', token: 'x' });
  });

  it('drops an empty token from the address, and leaves an address with none as it is', () => {
    expect(splitToken('ws://host?tkn=')).toEqual({ address: 'ws://host', token: null });
    expect(splitToken('ws://host:9187/?a=1')).toEqual({ address: 'ws://host:9187/?a=1', token: null });
    expect(splitToken('ws://host:91')).toEqual({ address: 'ws://host:91', token: null });
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

describe('sshTarget', () => {
  it('takes a name, an IPv4 and an IPv6 address, trimmed', () => {
    expect(sshTarget(' server.example ', '22', ' me ')).toEqual({ ok: true, host: 'server.example', port: 22, user: 'me' });
    expect(sshTarget('10.0.0.5', '2222', 'root')).toEqual({ ok: true, host: '10.0.0.5', port: 2222, user: 'root' });
    expect(sshTarget('fe80::1', '22', 'me')).toEqual({ ok: true, host: 'fe80::1', port: 22, user: 'me' });
  });

  it('names the field that is wrong', () => {
    expect(sshTarget('', '22', 'me')).toEqual({ ok: false, field: 'host' });
    expect(sshTarget('dev 02', '22', 'me')).toEqual({ ok: false, field: 'host' });
    expect(sshTarget('server.example', '0', 'me')).toEqual({ ok: false, field: 'port' });
    expect(sshTarget('server.example', '65536', 'me')).toEqual({ ok: false, field: 'port' });
    expect(sshTarget('server.example', '22', ' ')).toEqual({ ok: false, field: 'user' });
  });
});

describe('ahpdPort', () => {
  it('takes 1 to 65535 and refuses the rest', () => {
    expect([ahpdPort('9187'), ahpdPort('1'), ahpdPort('65535')]).toEqual([9187, 1, 65535]);
    expect([ahpdPort('0'), ahpdPort('65536'), ahpdPort(''), ahpdPort('port')]).toEqual([null, null, null, null]);
  });
});

describe('serverFolder', () => {
  it('reads ~ and ~/ from the home, and takes an absolute path as typed', () => {
    expect(serverFolder('~', '/home/ana')).toBe('/home/ana');
    expect(serverFolder(' ~/ ', '/home/ana')).toBe('/home/ana');
    expect(serverFolder('~/work/app/', '/home/ana/')).toBe('/home/ana/work/app');
    expect(serverFolder('/srv/code', '/home/ana')).toBe('/srv/code');
    expect(serverFolder('/', '/home/ana')).toBe('/');
  });

  it('refuses a relative folder and an empty one', () => {
    expect([serverFolder('work', '/home/ana'), serverFolder('', '/home/ana'), serverFolder('~ana', '/home/ana')]).toEqual([
      null,
      null,
      null,
    ]);
  });
});

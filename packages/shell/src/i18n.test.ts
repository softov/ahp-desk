import { describe, expect, it } from 'vitest';
import { DICTIONARIES, fromOs, LANGS, translate } from './i18n.js';

describe('the dictionaries', () => {
  const keys = (lang: (typeof LANGS)[number]) => Object.keys(DICTIONARIES[lang]).sort();

  it.each(LANGS)('%s has every key and no other', (lang) => {
    expect(keys(lang)).toEqual(keys('en'));
  });

  it.each(LANGS)('%s has no empty word', (lang) => {
    for (const [key, word] of Object.entries(DICTIONARIES[lang])) expect(word.trim(), key).not.toBe('');
  });

  it.each(LANGS)('%s keeps the placeholders of each English word', (lang) => {
    const names = (word: string) => [...word.matchAll(/\{(\w+)\}/g)].map((match) => match[1]).sort();
    for (const [key, word] of Object.entries(DICTIONARIES.en))
      expect(names(DICTIONARIES[lang][key as keyof typeof DICTIONARIES.en]), key).toEqual(names(word));
  });
});

describe('translate', () => {
  it('fills each placeholder', () => {
    expect(translate('pt-BR', 'hosts.removeQuestion', { name: 'Office' })).toBe('Remover Office?');
  });

  it('leaves a placeholder with no value as it is', () => {
    expect(translate('en', 'form.editTitle')).toBe('Edit {name}');
  });
});

describe('fromOs', () => {
  it('maps any Portuguese to Portuguese (Brasil) and any Spanish to Spanish', () => {
    expect(fromOs(['pt-PT'])).toBe('pt-BR');
    expect(fromOs(['es-MX', 'en-US'])).toBe('es');
  });

  it('takes the first language it knows', () => {
    expect(fromOs(['de-DE', 'pt-BR'])).toBe('pt-BR');
  });

  it('falls back to English', () => {
    expect(fromOs(['de-DE'])).toBe('en');
    expect(fromOs([])).toBe('en');
  });
});

/**
 * The desk's words in English, Portuguese (Brasil) and Spanish (decision: the
 * desk speaks English, Brazilian Portuguese and Spanish). The language follows
 * the OS until one is picked; the pick is kept in `localStorage`.
 */

import { useSyncExternalStore } from 'react';

export type Lang = 'en' | 'pt-BR' | 'es';

export const LANGS: readonly Lang[] = ['en', 'pt-BR', 'es'];

/** Each language's name in itself, as the picker lists them. */
export const LANG_NAMES: Record<Lang, string> = {
  en: 'English',
  'pt-BR': 'Português (Brasil)',
  es: 'Español',
};

const en = {
  'window.minimize': 'Minimize',
  'window.maximize': 'Maximize',
  'window.restore': 'Restore',
  'window.close': 'Close',
  'window.language': 'Language',

  'hosts.title': 'Hosts',
  'hosts.add': '+ Add host',
  'hosts.empty': 'No hosts yet. Add an ahpd, or any AHP server, by its address.',
  'hosts.more': 'More',
  'hosts.edit': 'Edit',
  'hosts.remove': 'Remove',
  'hosts.removeQuestion': 'Remove {name}?',
  'hosts.open': 'Open',
  'hosts.retry': 'Retry',
  'hosts.tokenInFile': 'The token is in a plain file.',
  'hosts.noKeyring': 'No OS keyring was available.',

  'status.connected': 'connected',
  'status.connecting': 'connecting',
  'status.down': 'down',
  'status.noCommonVersion': 'This host speaks none of the protocol versions {versions}. Update its server.',
  'status.noAnswer': 'No answer in 8 seconds.',
  'status.couldNotConnect': 'Could not connect. Check the address and the token.',

  'form.addTitle': 'Add host',
  'form.editTitle': 'Edit {name}',
  'form.name': 'Name',
  'form.namePlaceholder': 'Office',
  'form.address': 'Address',
  'form.token': 'Token',
  'form.tokenUnchanged': 'unchanged',
  'form.tokenHint': 'The token is kept in the OS keyring.',
  'form.connected': 'Connected: {server}',
  'form.cancel': 'Cancel',
  'form.test': 'Test',
  'form.testing': 'Testing…',
  'form.save': 'Save',
};

export type Key = keyof typeof en;

const ptBR: Record<Key, string> = {
  'window.minimize': 'Minimizar',
  'window.maximize': 'Maximizar',
  'window.restore': 'Restaurar',
  'window.close': 'Fechar',
  'window.language': 'Idioma',

  'hosts.title': 'Hosts',
  'hosts.add': '+ Adicionar host',
  'hosts.empty': 'Nenhum host ainda. Adicione um ahpd, ou qualquer servidor AHP, pelo endereço.',
  'hosts.more': 'Mais',
  'hosts.edit': 'Editar',
  'hosts.remove': 'Remover',
  'hosts.removeQuestion': 'Remover {name}?',
  'hosts.open': 'Abrir',
  'hosts.retry': 'Tentar de novo',
  'hosts.tokenInFile': 'O token está em um arquivo comum.',
  'hosts.noKeyring': 'Nenhum chaveiro do sistema estava disponível.',

  'status.connected': 'conectado',
  'status.connecting': 'conectando',
  'status.down': 'sem conexão',
  'status.noCommonVersion': 'Este host não fala nenhuma das versões do protocolo {versions}. Atualize o servidor dele.',
  'status.noAnswer': 'Sem resposta em 8 segundos.',
  'status.couldNotConnect': 'Não foi possível conectar. Confira o endereço e o token.',

  'form.addTitle': 'Adicionar host',
  'form.editTitle': 'Editar {name}',
  'form.name': 'Nome',
  'form.namePlaceholder': 'Escritório',
  'form.address': 'Endereço',
  'form.token': 'Token',
  'form.tokenUnchanged': 'sem alteração',
  'form.tokenHint': 'O token fica no chaveiro do sistema.',
  'form.connected': 'Conectado: {server}',
  'form.cancel': 'Cancelar',
  'form.test': 'Testar',
  'form.testing': 'Testando…',
  'form.save': 'Salvar',
};

const es: Record<Key, string> = {
  'window.minimize': 'Minimizar',
  'window.maximize': 'Maximizar',
  'window.restore': 'Restaurar',
  'window.close': 'Cerrar',
  'window.language': 'Idioma',

  'hosts.title': 'Hosts',
  'hosts.add': '+ Añadir host',
  'hosts.empty': 'Aún no hay hosts. Añade un ahpd, o cualquier servidor AHP, por su dirección.',
  'hosts.more': 'Más',
  'hosts.edit': 'Editar',
  'hosts.remove': 'Quitar',
  'hosts.removeQuestion': '¿Quitar {name}?',
  'hosts.open': 'Abrir',
  'hosts.retry': 'Reintentar',
  'hosts.tokenInFile': 'El token está en un archivo sin cifrar.',
  'hosts.noKeyring': 'No había un llavero del sistema disponible.',

  'status.connected': 'conectado',
  'status.connecting': 'conectando',
  'status.down': 'sin conexión',
  'status.noCommonVersion': 'Este host no habla ninguna de las versiones del protocolo {versions}. Actualiza su servidor.',
  'status.noAnswer': 'Sin respuesta en 8 segundos.',
  'status.couldNotConnect': 'No se pudo conectar. Revisa la dirección y el token.',

  'form.addTitle': 'Añadir host',
  'form.editTitle': 'Editar {name}',
  'form.name': 'Nombre',
  'form.namePlaceholder': 'Oficina',
  'form.address': 'Dirección',
  'form.token': 'Token',
  'form.tokenUnchanged': 'sin cambios',
  'form.tokenHint': 'El token se guarda en el llavero del sistema.',
  'form.connected': 'Conectado: {server}',
  'form.cancel': 'Cancelar',
  'form.test': 'Probar',
  'form.testing': 'Probando…',
  'form.save': 'Guardar',
};

export const DICTIONARIES: Record<Lang, Record<Key, string>> = { en, 'pt-BR': ptBR, es };

const STORAGE_KEY = 'ahp-desk.language';

/** The OS's language as one of ours: Portuguese and Spanish by their prefix, English otherwise. */
export function fromOs(languages: readonly string[]): Lang {
  for (const tag of languages) {
    const lower = tag.toLowerCase();
    if (lower.startsWith('pt')) return 'pt-BR';
    if (lower.startsWith('es')) return 'es';
    if (lower.startsWith('en')) return 'en';
  }
  return 'en';
}

function picked(): Lang | null {
  const value = localStorage.getItem(STORAGE_KEY);
  return LANGS.find((lang) => lang === value) ?? null;
}

// Read on first use, so the dictionaries import without a browser, as in a test.
let current: Lang | null = null;
const listeners = new Set<() => void>();

export function language(): Lang {
  if (current === null) {
    current = picked() ?? fromOs(navigator.languages);
    document.documentElement.lang = current;
  }
  return current;
}

/** Picks `lang`, keeps the pick, and redraws whoever reads the language. */
export function setLanguage(lang: Lang): void {
  localStorage.setItem(STORAGE_KEY, lang);
  current = lang;
  document.documentElement.lang = lang;
  for (const listener of listeners) listener();
}

/** The language, for a component that redraws when it changes. */
export function useLanguage(): Lang {
  return useSyncExternalStore((listener) => {
    listeners.add(listener);
    return () => listeners.delete(listener);
  }, language);
}

/** `key` in `lang`, each `{name}` replaced from `vars`. */
export function translate(lang: Lang, key: Key, vars: Record<string, string> = {}): string {
  return DICTIONARIES[lang][key].replace(/\{(\w+)\}/g, (all, name: string) => vars[name] ?? all);
}

/** `key` in the current language. */
export function t(key: Key, vars?: Record<string, string>): string {
  return translate(language(), key, vars);
}

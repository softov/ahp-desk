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
  'form.startOnLaunch': 'Start when the desktop app starts',
  'form.stopOnQuit': 'Stop when the desktop app quits',
  'form.stopOnQuitHint': 'Only an ahpd the desktop app started. A stop ends the turns that run.',
  'add.kind': 'Kind',
  'add.url': 'URL',
  'local.ahpd': 'ahpd',
  'local.looking': 'Looking for ahpd…',
  'local.foundPath': 'ahpd {version}, on PATH',
  'local.foundSet': 'ahpd {version}, the one set here',
  'local.foundInstalled': 'ahpd {version}, installed by the desktop app',
  'local.none': 'Not on this computer. The desktop app installs it with its own Node.',
  'local.tooOld': 'ahpd {version} is too old: the desktop app needs {needed} or later. Update it, then check again:',
  'local.checkAgain': 'Check again',
  'local.change': 'Change',
  'local.pickTitle': 'The ahpd to use',
  'local.folder': 'Folder',
  'local.pickFolderTitle': 'The folder ahpd serves',
  'local.backends': 'Backends',
  'local.bot': 'Bots',
  'local.installAndStart': 'Install and start',
  'local.saveAndStart': 'Save and start',
  'local.installing': 'Installing ahpd…',
  'local.starting': 'Starting ahpd…',
  'local.oneOnly': 'This computer is in the list already.',
  'local.toList': 'Back to the list',
  'local.installed': 'installed',
  'local.adding': 'Adding the backends…',
  'form.connected': 'Connected: {server}',
  'form.cancel': 'Cancel',
  'form.test': 'Test',
  'form.testing': 'Testing…',
  'form.save': 'Save',
  'form.back': 'Back',
  'form.saveAndOpen': 'Save and open',

  'firstRun.welcomeTitle': 'Welcome to ahp-desk',
  'firstRun.welcomeText': 'ahp-desk opens your Agent Host Protocol hosts, each in a window of its own. Start by adding one.',
  'firstRun.start': 'Get started',
  'firstRun.whereTitle': 'Where is your host?',
  'firstRun.remote': 'On the internet or my network',
  'firstRun.remoteHint': 'An ahpd, or any AHP server, at an address you know.',
  'firstRun.local': 'This computer',
  'firstRun.localHint': 'A host that runs here, with no address to type.',
  'firstRun.soon': 'Coming soon',
  'firstRun.opening': 'Opening {name}…',
  'firstRun.toList': 'Host list',
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
  'form.startOnLaunch': 'Iniciar quando o app de desktop iniciar',
  'form.stopOnQuit': 'Parar quando o app de desktop fechar',
  'form.stopOnQuitHint': 'Só um ahpd que o app de desktop iniciou. Parar encerra os turnos em andamento.',
  'add.kind': 'Tipo',
  'add.url': 'URL',
  'local.ahpd': 'ahpd',
  'local.looking': 'Procurando o ahpd…',
  'local.foundPath': 'ahpd {version}, no PATH',
  'local.foundSet': 'ahpd {version}, o definido aqui',
  'local.foundInstalled': 'ahpd {version}, instalado pelo app de desktop',
  'local.none': 'Não está neste computador. O app de desktop o instala com o próprio Node.',
  'local.tooOld': 'O ahpd {version} é antigo demais: o app de desktop precisa do {needed} ou mais novo. Atualize e verifique de novo:',
  'local.checkAgain': 'Verificar de novo',
  'local.change': 'Trocar',
  'local.pickTitle': 'O ahpd a usar',
  'local.folder': 'Pasta',
  'local.pickFolderTitle': 'A pasta que o ahpd serve',
  'local.backends': 'Backends',
  'local.bot': 'Bots',
  'local.installAndStart': 'Instalar e iniciar',
  'local.saveAndStart': 'Salvar e iniciar',
  'local.installing': 'Instalando o ahpd…',
  'local.starting': 'Iniciando o ahpd…',
  'local.oneOnly': 'Este computador já está na lista.',
  'local.toList': 'Voltar à lista',
  'local.installed': 'instalado',
  'local.adding': 'Adicionando os backends…',
  'form.connected': 'Conectado: {server}',
  'form.cancel': 'Cancelar',
  'form.test': 'Testar',
  'form.testing': 'Testando…',
  'form.save': 'Salvar',
  'form.back': 'Voltar',
  'form.saveAndOpen': 'Salvar e abrir',

  'firstRun.welcomeTitle': 'Boas-vindas ao ahp-desk',
  'firstRun.welcomeText': 'O ahp-desk abre seus hosts do Agent Host Protocol, cada um em uma janela própria. Comece adicionando um.',
  'firstRun.start': 'Começar',
  'firstRun.whereTitle': 'Onde está o seu host?',
  'firstRun.remote': 'Na internet ou na minha rede',
  'firstRun.remoteHint': 'Um ahpd, ou qualquer servidor AHP, em um endereço que você conhece.',
  'firstRun.local': 'Este computador',
  'firstRun.localHint': 'Um host que roda aqui, sem endereço para digitar.',
  'firstRun.soon': 'Em breve',
  'firstRun.opening': 'Abrindo {name}…',
  'firstRun.toList': 'Lista de hosts',
};

const es: Record<Key, string> = {
  'window.minimize': 'Minimizar',
  'window.maximize': 'Maximizar',
  'window.restore': 'Restaurar',
  'window.close': 'Cerrar',
  'window.language': 'Idioma',

  'hosts.title': 'Hosts',
  'hosts.add': '+ Añadir host',
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
  'form.startOnLaunch': 'Iniciar cuando se inicie la app de escritorio',
  'form.stopOnQuit': 'Detener cuando se cierre la app de escritorio',
  'form.stopOnQuitHint': 'Solo un ahpd que la app de escritorio inició. Detenerlo termina los turnos en curso.',
  'add.kind': 'Tipo',
  'add.url': 'URL',
  'local.ahpd': 'ahpd',
  'local.looking': 'Buscando ahpd…',
  'local.foundPath': 'ahpd {version}, en el PATH',
  'local.foundSet': 'ahpd {version}, el indicado aquí',
  'local.foundInstalled': 'ahpd {version}, instalado por la app de escritorio',
  'local.none': 'No está en este equipo. La app de escritorio lo instala con su propio Node.',
  'local.tooOld': 'ahpd {version} es demasiado antiguo: la app de escritorio necesita {needed} o posterior. Actualízalo y vuelve a comprobar:',
  'local.checkAgain': 'Comprobar de nuevo',
  'local.change': 'Cambiar',
  'local.pickTitle': 'El ahpd que usar',
  'local.folder': 'Carpeta',
  'local.pickFolderTitle': 'La carpeta que ahpd sirve',
  'local.backends': 'Backends',
  'local.bot': 'Bots',
  'local.installAndStart': 'Instalar e iniciar',
  'local.saveAndStart': 'Guardar e iniciar',
  'local.installing': 'Instalando ahpd…',
  'local.starting': 'Iniciando ahpd…',
  'local.oneOnly': 'Este equipo ya está en la lista.',
  'local.toList': 'Volver a la lista',
  'local.installed': 'instalado',
  'local.adding': 'Añadiendo los backends…',
  'form.connected': 'Conectado: {server}',
  'form.cancel': 'Cancelar',
  'form.test': 'Probar',
  'form.testing': 'Probando…',
  'form.save': 'Guardar',
  'form.back': 'Volver',
  'form.saveAndOpen': 'Guardar y abrir',

  'firstRun.welcomeTitle': 'Te damos la bienvenida a ahp-desk',
  'firstRun.welcomeText': 'ahp-desk abre tus hosts del Agent Host Protocol, cada uno en su propia ventana. Empieza añadiendo uno.',
  'firstRun.start': 'Empezar',
  'firstRun.whereTitle': '¿Dónde está tu host?',
  'firstRun.remote': 'En internet o en mi red',
  'firstRun.remoteHint': 'Un ahpd, o cualquier servidor AHP, en una dirección que conoces.',
  'firstRun.local': 'Este equipo',
  'firstRun.localHint': 'Un host que se ejecuta aquí, sin dirección que escribir.',
  'firstRun.soon': 'Próximamente',
  'firstRun.opening': 'Abriendo {name}…',
  'firstRun.toList': 'Lista de hosts',
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

import { openUrl } from '@tauri-apps/plugin-opener';
import { Mark } from './Mark.js';

const SITE = 'https://ahpio.com';

/** ahpw's mark and the site, centred at the bottom of the host list; the site opens in the browser. */
export function Footer() {
  return (
    <footer className="footer">
      <Mark size={20} />
      <a
        href={SITE}
        onClick={(event) => {
          event.preventDefault();
          openUrl(SITE).catch((err: unknown) => console.error('opening the site failed', err));
        }}
      >
        © ahpio.com
      </a>
    </footer>
  );
}

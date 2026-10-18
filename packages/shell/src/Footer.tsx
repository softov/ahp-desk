import { openUrl } from '@tauri-apps/plugin-opener';
import { AhpioLogo } from './AhpioLogo.js';

const SITE = 'https://ahpio.com';

/** ahpio's logo and the site, centred at the bottom of the host list; the site opens in the browser. */
export function Footer() {
  return (
    <footer className="footer">
      <AhpioLogo size={14} />
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

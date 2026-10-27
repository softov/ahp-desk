import { openUrl } from '@tauri-apps/plugin-opener';

const SITE = 'https://ahpio.com';

/** The site, centred at the bottom of the host list, opening in the browser; ahpio's logo is the title bar's. */
export function Footer() {
  return (
    <footer className="footer">
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

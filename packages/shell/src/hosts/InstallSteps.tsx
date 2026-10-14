import { useEffect, useRef } from 'react';

/** What npm and ahpd write during an install, as it comes, kept at its last line. */
export function InstallSteps({ lines }: { lines: readonly string[] }) {
  const box = useRef<HTMLPreElement>(null);
  useEffect(() => {
    if (box.current !== null) box.current.scrollTop = box.current.scrollHeight;
  }, [lines]);
  return (
    <pre className="steps" ref={box}>
      {lines.join('\n')}
    </pre>
  );
}

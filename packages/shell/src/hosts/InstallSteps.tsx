import { useEffect, useRef } from 'react';
import { t } from '../i18n.js';
import type { InstallStep } from './ipc.js';

/** A named step of an install or an uninstall over SSH, as it last stood. */
export interface StepState<S extends string = InstallStep> {
  id: S;
  status: 'running' | 'done' | 'failed' | 'skipped';
}

const MARK: Record<StepState['status'], string> = { running: '…', done: '✓', failed: '✗', skipped: '–' };

/**
 * What npm and ahpd write during an install, as it comes, kept at its last
 * line; over SSH, the named steps above it, in the install's words unless
 * `wordOf` gives others.
 */
export function InstallSteps<S extends string = InstallStep>({
  lines,
  steps = [],
  wordOf = (id) => t(`step.${id as InstallStep}`),
}: {
  lines: readonly string[];
  steps?: readonly StepState<S>[];
  wordOf?: (id: S) => string;
}) {
  const box = useRef<HTMLPreElement>(null);
  useEffect(() => {
    if (box.current !== null) box.current.scrollTop = box.current.scrollHeight;
  }, [lines]);
  return (
    <>
      {steps.length > 0 && (
        <ol className="step-list">
          {steps.map((step) => (
            <li key={step.id} className={`step-${step.status}`}>
              <span aria-hidden="true">{MARK[step.status]}</span> {wordOf(step.id)}
            </li>
          ))}
        </ol>
      )}
      {lines.length > 0 && (
        <pre className="steps" ref={box}>
          {lines.join('\n')}
        </pre>
      )}
    </>
  );
}

/** `steps` with `event`'s step put in its place, or added at the end. */
export function withStep<S extends string>(steps: readonly StepState<S>[], event: StepState<S>): StepState<S>[] {
  return steps.some((step) => step.id === event.id)
    ? steps.map((step) => (step.id === event.id ? event : step))
    : [...steps, event];
}

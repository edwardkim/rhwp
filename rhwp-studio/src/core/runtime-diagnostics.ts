const RESIZE_OBSERVER_MESSAGES = new Set([
  'ResizeObserver loop completed with undelivered notifications.',
  'ResizeObserver loop limit exceeded',
]);

function silence(event: Pick<Event, 'preventDefault' | 'stopImmediatePropagation'>): void {
  event.stopImmediatePropagation();
  event.preventDefault();
}

export function installRuntimeDiagnosticGuards(target: Window): void {
  target.addEventListener('error', (event) => {
    if (RESIZE_OBSERVER_MESSAGES.has(event.message)) silence(event);
  }, true);
}

if (typeof window !== 'undefined') installRuntimeDiagnosticGuards(window);

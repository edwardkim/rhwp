export type LinkKind = 'web' | 'mail' | 'documentAnchor' | 'relativeFile' | 'localFile' | 'registeredApplication';
export interface LinkDestination { kind: LinkKind; uri: string }
export interface LinkHostContext { fileName: string; sourceFormat: string }
export type LinkHostHandler = (destination: Readonly<LinkDestination>, context: Readonly<LinkHostContext>) => boolean | Promise<boolean>;

let host: LinkHostHandler | null = null;
export function setLinkHostHandler(handler: LinkHostHandler | null): void {
  if (handler !== null && typeof handler !== 'function') throw new TypeError('Expected a link handler or null');
  host = handler;
}

/** Called exclusively from a verified click; classification belongs to Rust. */
export function navigateLink(destination: LinkDestination, context: LinkHostContext, actions: {
  anchor(name: string): boolean;
  web(uri: string): boolean;
  mail(uri: string): boolean;
  browserFallback(destination: LinkDestination): void;
}): boolean | Promise<boolean> {
  if (destination.kind === 'documentAnchor') {
    try { return actions.anchor(decodeURIComponent(destination.uri.slice(1))); }
    catch { return false; }
  }
  if (destination.kind === 'web') return actions.web(destination.uri);
  if (host) {
    const fallback = (handled: boolean) => {
      if (handled === true) return true;
      if (destination.kind === 'mail') return actions.mail(destination.uri);
      actions.browserFallback(destination); return false;
    };
    return Promise.resolve(host(Object.freeze({ ...destination }), Object.freeze({ ...context }))).then(fallback);
  }
  if (destination.kind === 'mail') return actions.mail(destination.uri);
  actions.browserFallback(destination); return false;
}

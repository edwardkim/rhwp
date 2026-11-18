/** Tracks active opens separately from the last document's cancellation signal. */
export class DocumentOpenSession {
  private current: { controller: AbortController; pending: boolean } | undefined;

  private readonly superseded: Error;

  constructor(superseded: Error) {
    this.superseded = superseded;
  }

  begin(): AbortSignal {
    this.current?.controller.abort(this.superseded);
    const controller = new AbortController();
    this.current = { controller, pending: true };
    return controller.signal;
  }

  finish(signal: AbortSignal): void {
    if (this.current?.controller.signal === signal) this.current.pending = false;
  }

  get signal(): AbortSignal | undefined {
    return this.current?.controller.signal;
  }

  get pending(): boolean {
    return this.current?.pending === true;
  }
}

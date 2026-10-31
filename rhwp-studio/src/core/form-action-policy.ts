// A read-only host can revoke button execution without changing saved scripts.
// Every final application checks this again, including already running Workers.
let enabled = true;
export function setFormActionsEnabled(value: boolean): void {
  if (typeof value !== 'boolean') throw new TypeError('Expected a boolean');
  enabled = value;
}
export function assertFormActionsEnabled(): void {
  if (!enabled) throw new Error('FORM_ACTION_READ_ONLY: 호스트가 명령 단추 실행을 허용하지 않습니다.');
}

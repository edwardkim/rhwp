const LAYOUT_BOUNDARY_OPERATION_TYPES = new Set([
  'pageBreak',
  'columnBreak',
]);

function baseOperationType(operationType: string): string {
  return operationType.startsWith('snapshot:')
    ? operationType.slice('snapshot:'.length)
    : operationType;
}

/**
 * 쪽/단 경계 삽입은 mutation 직후의 WASM 좌표보다 CanvasView의 가상 쪽 배치가 늦게 갱신된다.
 * 해당 명령만 다음 mutation layout 완료 뒤 캐럿을 한 번 더 드러내도록 예약한다.
 */
export class CaretLayoutReveal {
  private pending = false;

  requestFor(operationType: string): void {
    if (LAYOUT_BOUNDARY_OPERATION_TYPES.has(baseOperationType(operationType))) {
      this.pending = true;
    }
  }

  clear(): void {
    this.pending = false;
  }

  consume(): boolean {
    const pending = this.pending;
    this.pending = false;
    return pending;
  }
}

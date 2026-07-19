import type { WasmBridge } from '@/core/wasm-bridge';
import type { DocumentPosition } from '@/core/types';
import { NO_TEXT_MUTATION_EFFECTS } from './command';
import type { EditCommand, TextMutationEffects } from './command';

/** 스택 내 모든 명령의 discard()를 호출하여 리소스 해제 */
function discardAll(stack: EditCommand[], wasm: WasmBridge): void {
  for (const cmd of stack) {
    cmd.discard?.(wasm);
  }
}

/**
 * [Task #2328] WASM 스냅샷 저장소 상한 미러 —
 * src/document_core/commands/document.rs 의 save_snapshot_native 내부
 * `const MAX_SNAPSHOTS`(함수-로컬). **양방향 결합**: Rust 값을 이 아래로
 * 낮추면 아래 예산(MAX-2)이 store 를 넘겨 WASM 무통보 축출이 재발한다.
 * 값 변경 시 반드시 양쪽을 함께 갱신한다(Rust 쪽에도 역참조 주석이 있다).
 */
const WASM_MAX_SNAPSHOTS = 100;

/**
 * [Task #2328] JS 측 살아있는 스냅샷 id 예산. 새 SnapshotCommand 의 최초 execute 는
 * before/after 2개를 **연속으로** 저장하는데(command.ts), 이 저장은 히스토리의
 * redo 정리·예산 강제(enforceSnapshotBudget)보다 **먼저** 일어난다. 예산을
 * MAX 와 같게 두면 그 순간적 +2 가 WASM store 를 MAX 초과로 밀어 WASM 자체의
 * 무통보 축출이 발동하고, 이때 축출되는 것은 JS 가 아직 참조하는 오래된 undo
 * 엔트리의 스냅샷이라 이후 그 엔트리 undo 가 restore 실패로 예외가 된다
 * (인터리브: 예산 채움→undo→새 편집→오래된 undo 시 재현). 따라서 예산에 그
 * 순간 +2 만큼 여유를 두어, 라이브 총합이 예산 이하면 새 저장 후에도 store 가
 * MAX 를 넘지 않게 한다 → WASM 축출은 결코 발동하지 않는다.
 */
const SNAPSHOT_ID_BUDGET = WASM_MAX_SNAPSHOTS - 2;

/** Undo/Redo 히스토리 관리 */
export class CommandHistory {
  private undoStack: EditCommand[] = [];
  private redoStack: EditCommand[] = [];
  private maxSize = 1000;
  private lastExecutionEffects: TextMutationEffects = NO_TEXT_MUTATION_EFFECTS;

  /** undo/redo 양 스택의 살아있는 스냅샷 id 총합. */
  private liveSnapshotIds(): number {
    let n = 0;
    for (const cmd of this.undoStack) n += cmd.snapshotResourceCount?.() ?? 0;
    for (const cmd of this.redoStack) n += cmd.snapshotResourceCount?.() ?? 0;
    return n;
  }

  /**
   * [Task #2328] 스냅샷 id 총합이 예산을 넘으면 undo 스택 front(최오래)부터
   * 연속 축출한다. front 축출은 contiguous 하므로 bounded-history 시멘틱을
   * 지키며(오래된 것부터 사라짐), 스냅샷 커맨드를 discard 해 WASM id 를 즉시
   * 반환한다. 텍스트 커맨드가 front 에 있으면 함께 밀려나지만(0 id) 오래된
   * 순서라 정합적이다. redo 스택은 새 명령 실행 시 항상 비워지므로 여기서만
   * front 를 다룬다.
   */
  private enforceSnapshotBudget(wasm: WasmBridge): void {
    while (this.liveSnapshotIds() > SNAPSHOT_ID_BUDGET && this.undoStack.length > 1) {
      const evicted = this.undoStack.shift();
      evicted?.discard?.(wasm);
    }
  }

  private captureExecutionEffects(command: EditCommand): void {
    this.lastExecutionEffects =
      command.consumeTextMutationEffects?.() ?? NO_TEXT_MUTATION_EFFECTS;
  }

  /** 직전 execute/redo의 effect를 한 번만 소비한다. */
  consumeLastExecutionEffects(): TextMutationEffects {
    const effects = this.lastExecutionEffects;
    this.lastExecutionEffects = NO_TEXT_MUTATION_EFFECTS;
    return effects;
  }

  /** 명령 실행 + 히스토리 기록. 실행 후 커서 위치 반환 */
  execute(command: EditCommand, wasm: WasmBridge): DocumentPosition {
    this.lastExecutionEffects = NO_TEXT_MUTATION_EFFECTS;
    const cursorAfter = command.execute(wasm);
    this.captureExecutionEffects(command);

    // 직전 명령과 병합 시도
    if (this.undoStack.length > 0) {
      const last = this.undoStack[this.undoStack.length - 1];
      const merged = last.mergeWith(command);
      if (merged) {
        this.undoStack[this.undoStack.length - 1] = merged;
        // redo 스택 정리 (리소스 해제 후 비움)
        discardAll(this.redoStack, wasm);
        this.redoStack = [];
        return cursorAfter;
      }
    }

    this.undoStack.push(command);
    // redo 스택 정리 (새 명령 실행 시 redo 불가)
    discardAll(this.redoStack, wasm);
    this.redoStack = [];

    // 크기 제한 — eviction 시 리소스 해제
    if (this.undoStack.length > this.maxSize) {
      const evicted = this.undoStack.shift();
      evicted?.discard?.(wasm);
    }
    // [Task #2328] 스냅샷 예산 정합 — WASM 상한 초과 전에 front 축출. push 이후에
    // 강제해야 방금 명령의 +2 가 계산에 포함된다. 스냅샷을 점유하는 명령만
    // 예산을 늘리므로 그 경우에만 강제한다(텍스트 편집은 예산 무영향 →
    // liveSnapshotIds O(n) 스캔 생략, 편집 hot-path 비용 제거).
    if ((command.snapshotResourceCount?.() ?? 0) > 0) {
      this.enforceSnapshotBudget(wasm);
    }

    return cursorAfter;
  }

  /** Undo — 성공 시 커서 위치 반환, 스택 비었으면 null */
  undo(wasm: WasmBridge): DocumentPosition | null {
    this.lastExecutionEffects = NO_TEXT_MUTATION_EFFECTS;
    const command = this.undoStack[this.undoStack.length - 1];
    if (!command) return null;

    // [Task #2328] op-우선 + 실패시-드롭 하이브리드.
    // - 성공: 스택을 이동한다(op 전에 pop 하지 않으므로 성공 엔트리 무손실).
    // - 실패(예: 복구 불가한 스냅샷 restore 오류): 오래동안 스택 top 에 남겨
    //   두면 Ctrl+Z 마다 같은 엔트리가 재예외 → 세션 undo 락업이 된다. 재시도로
    //   복구되지 않는 오류이므로, 오염 엔트리를 제거·discard 하고 전파한다
    //   (스냅샷 복원은 문서 전체 치환이라 다음(더 오래된) 엔트리 undo 가 안전).
    let cursorAfter: DocumentPosition;
    try {
      cursorAfter = command.undo(wasm);
    } catch (e) {
      this.undoStack.pop();
      command.discard?.(wasm);
      throw e;
    }
    this.undoStack.pop();
    this.redoStack.push(command);
    return cursorAfter;
  }

  /** Redo — 성공 시 커서 위치 반환, 스택 비었으면 null */
  redo(wasm: WasmBridge): DocumentPosition | null {
    this.lastExecutionEffects = NO_TEXT_MUTATION_EFFECTS;
    const command = this.redoStack[this.redoStack.length - 1];
    if (!command) return null;

    // [Task #2328] undo 와 동일 하이브리드 — 성공 시 이동, 실패 시 오염 엔트리 드롭.
    let cursorAfter: DocumentPosition;
    try {
      cursorAfter = command.execute(wasm);
    } catch (e) {
      this.redoStack.pop();
      command.discard?.(wasm);
      throw e;
    }
    this.captureExecutionEffects(command);
    this.redoStack.pop();
    this.undoStack.push(command);
    return cursorAfter;
  }

  /** execute() 없이 히스토리에만 기록 (IME compositionend용 — 텍스트가 이미 문서에 있는 경우) */
  recordWithoutExecute(command: EditCommand, wasm?: WasmBridge): void {
    this.lastExecutionEffects = NO_TEXT_MUTATION_EFFECTS;
    // 직전 명령과 병합 시도
    if (this.undoStack.length > 0) {
      const last = this.undoStack[this.undoStack.length - 1];
      const merged = last.mergeWith(command);
      if (merged) {
        this.undoStack[this.undoStack.length - 1] = merged;
        if (wasm) {
          discardAll(this.redoStack, wasm);
        }
        this.redoStack = [];
        return;
      }
    }

    this.undoStack.push(command);
    if (wasm) {
      discardAll(this.redoStack, wasm);
    }
    this.redoStack = [];

    if (this.undoStack.length > this.maxSize) {
      const evicted = this.undoStack.shift();
      if (wasm) evicted?.discard?.(wasm);
    }
  }

  canUndo(): boolean { return this.undoStack.length > 0; }
  canRedo(): boolean { return this.redoStack.length > 0; }

  /**
   * [Task #2337] 직전 undo/redo 로 방금 이동한 커맨드를 조회한다.
   * undo() 후 방금 되돌린 커맨드는 redoStack top(peekRedoTop), redo() 후 방금 다시
   * 실행한 커맨드는 undoStack top(peekUndoTop). InputHandler 가 HF/FN 편집 커맨드의
   * editContext() 를 읽어 커서 모드를 복원하는 데 쓴다(본문 커맨드면 null-컨텍스트).
   */
  peekUndoTop(): EditCommand | null { return this.undoStack[this.undoStack.length - 1] ?? null; }
  peekRedoTop(): EditCommand | null { return this.redoStack[this.redoStack.length - 1] ?? null; }

  /** 히스토리 초기화 (문서 로드 시). wasm이 있으면 스냅샷 리소스도 해제. */
  clear(wasm?: WasmBridge): void {
    if (wasm) {
      discardAll(this.undoStack, wasm);
      discardAll(this.redoStack, wasm);
    }
    this.undoStack = [];
    this.redoStack = [];
    this.lastExecutionEffects = NO_TEXT_MUTATION_EFFECTS;
  }
}

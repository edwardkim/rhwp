import type { WasmBridge } from '@/core/wasm-bridge';
import { ResizeObjectCommand, type ObjectResizeTarget } from './command';

/** 원본 행렬을 스칼라 크기 봉지로 재구성하지 않는다. 코어의 작은 그림별 저널만 소유한다. */
class PictureTransformResizeCommand extends ResizeObjectCommand {
  constructor(private allTargets: ObjectResizeTarget[], private ids: number[]) {
    super(allTargets.filter(t => t.type !== 'image'));
  }

  private apply(wasm: WasmBridge, redo: boolean) {
    const swapped: number[] = [];
    try {
      for (const id of this.ids) {
        wasm.swapPictureTransform(id);
        swapped.push(id);
      }
      if (redo) super.execute(wasm);
      else super.undo(wasm);
    } catch (error) {
      for (const id of swapped.reverse()) wasm.swapPictureTransform(id);
      throw error;
    }
    const first = this.allTargets[0];
    return { sectionIndex: first?.sec ?? 0, paragraphIndex: first?.ppi ?? 0, charOffset: 0 };
  }

  execute(wasm: WasmBridge) { return this.apply(wasm, true); }
  undo(wasm: WasmBridge) { return this.apply(wasm, false); }
  discard(wasm: WasmBridge): void {
    for (const id of this.ids) wasm.discardPictureTransform(id);
    this.ids = [];
  }
}

export class PictureResizeJournal {
  private constructor(private ids: number[]) {}

  static capture(wasm: WasmBridge, refs: { type: string; [key: string]: unknown }[]): PictureResizeJournal {
    const ids: number[] = [];
    try {
      for (const ref of refs) {
        if (ref.type === 'image') ids.push(wasm.capturePictureTransform(ref));
      }
      return new PictureResizeJournal(ids);
    } catch (error) {
      for (const id of ids) wasm.discardPictureTransform(id);
      throw error;
    }
  }

  command(targets: ObjectResizeTarget[]): ResizeObjectCommand {
    return this.ids.length === 0
      ? new ResizeObjectCommand(targets)
      : new PictureTransformResizeCommand(targets, this.ids);
  }

  /** 취소·원점 복귀·실패는 원본 상태를 복원하고 핸들을 반환한다. */
  cancel(wasm: WasmBridge): void {
    const errors: unknown[] = [];
    for (const id of this.ids) {
      try { wasm.swapPictureTransform(id); }
      catch (error) { errors.push(error); }
      finally { wasm.discardPictureTransform(id); }
    }
    this.ids = [];
    if (errors.length) throw new AggregateError(errors, '그림 리사이즈 취소 복원 실패');
  }
}

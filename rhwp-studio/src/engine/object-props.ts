/**
 * 선택 개체의 속성 조회·변경 라우팅 (Task #3230).
 *
 * 개체는 어디에 있느냐에 따라 setter/getter 가 넷으로 갈린다 — 본문 도형, 본문 그림, 셀 안
 * (`cellPath`), 머리말/꼬리말(`headerFooter`, [Task #831]). 이 분기는 지금까지
 * `command/commands/insert.ts` 안에만 있었는데, **역연산 커맨드도 undo 시점에 같은 분기가
 * 필요하다.** 커맨드는 `services` 가 아니라 `WasmBridge` 만 받으므로 여기서 `wasm` 기준으로
 * 두고 양쪽이 함께 쓴다.
 *
 * 분기가 갈라지면 적용과 되돌리기가 서로 다른 개체를 만지게 된다 — HF 그림이 그 예다
 * (본문 lookup 으로 떨어지면 조용히 아무것도 안 바뀐다).
 */

import type { CellPathLike } from '@/core/types';
import type { WasmBridge } from '@/core/wasm-bridge';

/** 선택 개체 ref — `cursor.selectedPictureRef` 와 정합 (headerFooter optional, [Task #831]). */
export type ObjectPropsRef = {
  sec: number;
  ppi: number;
  ci: number;
  type: string;
  cellPath?: CellPathLike;
  headerFooter?: { kind: 'header' | 'footer'; outerParaIdx: number; outerControlIdx: number };
};

/** 선택 개체의 현재 속성. */
export function getObjectProps(wasm: WasmBridge, ref: ObjectPropsRef): Record<string, unknown> {
  if (ref.type === 'shape') {
    if (ref.cellPath && ref.cellPath.length > 0) {
      return wasm.getCellShapePropertiesByPath(ref.sec, ref.ppi, ref.cellPath, ref.ci) as unknown as Record<string, unknown>;
    }
    return wasm.getShapeProperties(ref.sec, ref.ppi, ref.ci) as unknown as Record<string, unknown>;
  }
  // [Task #831] 머리말/꼬리말 picture 는 별도 API. 미적용 시 본문 lookup 실패 → props 빈/stale
  // → 회전/대칭 무동작.
  if (ref.headerFooter) {
    return wasm.getHeaderFooterPictureProperties(
      ref.sec,
      ref.headerFooter.outerParaIdx,
      ref.headerFooter.outerControlIdx,
      ref.ppi,
      ref.ci,
    ) as unknown as Record<string, unknown>;
  }
  if (ref.cellPath && ref.cellPath.length > 0) {
    return wasm.getCellPicturePropertiesByPath(ref.sec, ref.ppi, ref.cellPath, ref.ci) as unknown as Record<string, unknown>;
  }
  return wasm.getPictureProperties(ref.sec, ref.ppi, ref.ci) as unknown as Record<string, unknown>;
}

/** 선택 개체에 속성을 적용한다. `getObjectProps` 와 같은 분기를 쓴다. */
export function setObjectProps(
  wasm: WasmBridge,
  ref: ObjectPropsRef,
  props: Record<string, unknown>,
): unknown {
  if (ref.type === 'shape') {
    if (ref.cellPath && ref.cellPath.length > 0) {
      return wasm.setCellShapePropertiesByPath(ref.sec, ref.ppi, ref.cellPath, ref.ci, props);
    }
    return wasm.setShapeProperties(ref.sec, ref.ppi, ref.ci, props);
  }
  if (ref.headerFooter) {
    return wasm.setHeaderFooterPictureProperties(
      ref.sec,
      ref.headerFooter.outerParaIdx,
      ref.headerFooter.outerControlIdx,
      ref.ppi,
      ref.ci,
      props,
    );
  }
  if (ref.cellPath && ref.cellPath.length > 0) {
    return wasm.setCellPicturePropertiesByPath(ref.sec, ref.ppi, ref.cellPath, ref.ci, props);
  }
  return wasm.setPictureProperties(ref.sec, ref.ppi, ref.ci, props);
}

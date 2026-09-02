/**
 * 선택 개체의 속성 조회·변경 라우팅 (Task #3230).
 *
 * 개체는 어디에 있느냐에 따라 setter/getter 가 갈린다 — 본문 도형, 본문 그림, 셀 안
 * (`cellPath`), 머리말/꼬리말(`headerFooter`, [Task #831]). 이 분기는 지금까지
 * `command/commands/insert.ts` 안에만 있었는데, **역연산 커맨드도 undo 시점에 같은 분기가
 * 필요하다.** 커맨드는 `services` 가 아니라 `WasmBridge` 만 받으므로 여기서 `wasm` 기준으로
 * 두고 양쪽이 함께 쓴다.
 *
 * 원본의 분기 구조를 조건·인자 순서까지 그대로 옮겼다. 그래서 원본이 갖고 있던 비대칭
 * (도형 분기가 `headerFooter` 를 보지 않는 것)도 그대로다 — `getObjectProps` 주석 참고.
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

/**
 * 선택 개체의 현재 속성.
 *
 * **분기가 대칭이 아니다** — `type === 'shape'` 는 `cellPath` 만 보고 `headerFooter` 는 보지
 * 않는다(그림 분기는 본다). 머리말/꼬리말 안의 도형은 본문 lookup 으로 떨어질 수 있다.
 * 이 비대칭은 `command/commands/insert.ts` 에 있던 원본 그대로이며(옮기면서 조건·인자 순서를
 * 바꾸지 않았다) 이 모듈이 만든 것이 아니다. 고치려면 HF 안에 도형이 실제로 놓이는지와
 * `getSelectedPictureRef` 가 도형에 `headerFooter` 를 채우는 경로가 있는지부터 확인해야 해서
 * 별건으로 둔다. `setObjectProps` 도 같은 비대칭을 그대로 갖는다 — 조회와 적용이 갈리면
 * 되돌리기가 다른 개체를 만지므로 **둘은 반드시 같은 모양이어야 한다.**
 */
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

/**
 * 선택 개체에 속성을 적용한다.
 *
 * `getObjectProps` 와 **같은 분기**를 쓴다(HF 비대칭까지 포함해서). 조회 경로와 적용 경로가
 * 갈리면 before 를 읽은 개체와 되돌리는 개체가 달라진다.
 */
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

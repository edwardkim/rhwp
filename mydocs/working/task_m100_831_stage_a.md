# Task #831 Stage A 본질 정밀 진단

**작성일**: 2026-05-13
**브랜치**: `local/task831`

## A.1 selectedPictureRef 구조 확인

[`rhwp-studio/src/engine/cursor.ts:1208`](rhwp-studio/src/engine/cursor.ts#L1208):

```typescript
private selectedPictureRef: {
  sec: number;
  ppi: number;
  ci: number;
  type: 'image' | 'shape' | 'equation' | 'group' | 'line';
  cellIdx?: number;
  cellParaIdx?: number;
  headerFooter?: { kind: 'header' | 'footer'; outerParaIdx: number; outerControlIdx: number };
} | null = null;
```

`headerFooter` 필드 정확 정의됨 — Task #825 / PR #832 으로 추가됨.

`getSelectedPictureRef()` 가 `headerFooter` 포함 ref 반환 (line 1258-1260).

## A.2 wasm-bridge API signature 확인

[`rhwp-studio/src/core/wasm-bridge.ts:727-755`](rhwp-studio/src/core/wasm-bridge.ts#L727):

```typescript
getHeaderFooterPictureProperties(
  sec: number,
  outerPara: number,
  outerCtrl: number,
  innerPara: number,
  innerCtrl: number,
): PictureProperties

setHeaderFooterPictureProperties(
  sec: number,
  outerPara: number,
  outerCtrl: number,
  innerPara: number,
  innerCtrl: number,
  props: Record<string, unknown>,
): { ok: boolean }
```

5-tuple lookup: `(sec, outerPara, outerCtrl, innerPara, innerCtrl)` — 머리말/꼬리말 외부 컨트롤 위치 + 내부 picture 위치 모두 명시.

## A.3 picture-props-dialog 호출 패턴 확인 (참조 정합)

### get (line 240-246)

```typescript
if (headerFooter) {
  this.props = this.wasm.getHeaderFooterPictureProperties(
    sec, headerFooter.outerParaIdx, headerFooter.outerControlIdx, para, ci,
  );
} else {
  this.props = this.wasm.getPictureProperties(sec, para, ci);
}
```

### set (line 2141-2148)

```typescript
} else if (this.headerFooter) {
  this.wasm.setHeaderFooterPictureProperties(
    this.sec, this.headerFooter.outerParaIdx, this.headerFooter.outerControlIdx,
    this.para, this.ci, updated,
  );
} else {
  this.wasm.setPictureProperties(this.sec, this.para, this.ci, updated);
}
```

매개변수 순서: `(sec, outerPara, outerCtrl, innerPara, innerCtrl)` — `ref.sec` / `ref.headerFooter.outerParaIdx` / `ref.headerFooter.outerControlIdx` / `ref.ppi` / `ref.ci`.

## A.4 본질 위치 (insert.ts:397-410)

[`rhwp-studio/src/command/commands/insert.ts:397-410`](rhwp-studio/src/command/commands/insert.ts#L397):

```typescript
function getProps(services, ref): Record<string, unknown> {
  if (ref.type === 'shape') {
    return services.wasm.getShapeProperties(ref.sec, ref.ppi, ref.ci);
  }
  return services.wasm.getPictureProperties(ref.sec, ref.ppi, ref.ci);
  // ↑ ref.headerFooter 무시 — 본문 lookup 만 → 머리말 picture 시 빈/stale
}

function setProps(services, ref, props): void {
  if (ref.type === 'shape') { ... }
  else { services.wasm.setPictureProperties(...); }
  // ↑ 동일 — 머리말/꼬리말 IR 미갱신
}
```

**호출 chain**:
1. `applyRotationDelta(services, delta)` (line 413+) → `getProps(services, ref)` → `setProps(services, ref, { rotationAngle: next })`
2. `toggleFlip(services, key)` (line 429+) → `getProps` → `setProps(services, ref, { [key]: !cur })`

`getProps` 가 빈 객체 반환 → `cur = props.rotationAngle ?? 0 = 0` → `next = 90` 등 계산은 진행되나 `setProps` 도 본문 only → IR 미갱신 → **무동작**.

## A.5 정정 위치 확정

**단일 위치 변경**: `getProps` / `setProps` (총 2 함수, 각 1 분기 추가)

`applyRotationDelta` / `toggleFlip` 본체 변경 불필요 — get/setProps 위임으로 자동 정합.

타입 정합:
- `services.wasm: WasmBridge` (`rhwp-studio/src/command/types.ts:58`) — 이미 `get/setHeaderFooterPictureProperties` 노출
- `ref.headerFooter` 타입은 `selectedPictureRef.headerFooter` 와 동일 정의

## A.6 회귀 영향 분석

| 케이스 | 분기 | 변경 영향 |
|---|---|---|
| 본문 picture (ref.headerFooter undefined) | else → 기존 setPictureProperties | **회귀 0** |
| Shape (ref.type === 'shape') | 첫 분기 → 기존 setShapeProperties | **회귀 0** |
| Equation/Group/Line | applyRotationDelta/toggleFlip 내부 가드 (line 417, 433) → 진입 차단 | **회귀 0** |
| 머리말/꼬리말 picture (ref.headerFooter 보유) | 신규 분기 → setHeaderFooterPictureProperties | **시각 정합 (목표)** |

## Stage A 결론

본질 확정: `getProps`/`setProps` 의 `ref.headerFooter` 분기 누락. 정정 범위 단일 파일 (`insert.ts`) 의 2 함수 (각 1 분기 추가). picture-props-dialog 의 동일 패턴 정합 — wasm-bridge API 재사용으로 신규 코드 최소.

📋 **Stage A 완료. Stage B 정정 구현 진행 승인 요청드립니다.**

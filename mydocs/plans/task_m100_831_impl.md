# Task #831 구현계획서

**선행**: `task_m100_831.md` (수행계획서, 승인 완료)
**브랜치**: `local/task831`
**작성일**: 2026-05-13

## 단계 분해 (3 단계 + 승인 게이트)

### Stage A: 본질 정밀 진단

**A.1 `selectedPictureRef` 의 headerFooter 필드 구조 확인**:

[`rhwp-studio/src/command/commands/insert.ts:183-184`](rhwp-studio/src/command/commands/insert.ts#L183) 에서 picturePropsDialog 가 `ref.headerFooter` 를 사용하고 있음. 동일 객체를 회전/대칭에서도 활용 가능.

확인 항목:
- `ref.headerFooter` 의 타입: `{ kind: 'header' | 'footer'; outerParaIdx: number; outerControlIdx: number } | undefined`
- `ref.sec`, `ref.ppi`, `ref.ci` 의 의미 (본문: 본문 indices / 머리말: 머리말 내부 indices)

**A.2 wasm-bridge `getHeaderFooterPictureProperties` / `setHeaderFooterPictureProperties` signature**:

[`rhwp-studio/src/core/wasm-bridge.ts:727-754`](rhwp-studio/src/core/wasm-bridge.ts#L727):
- get: `(sec, outerPara, outerCtrl, innerPara, innerCtrl) → props`
- set: `(sec, outerPara, outerCtrl, innerPara, innerCtrl, props) → void`

picture-props-dialog 의 호출 패턴 확인 (line 240-246, 2141-2146).

**A.3 정정 위치 확정**:

`getProps` / `setProps` 에 `ref.headerFooter` 분기 추가 — 단일 위치 변경으로 `applyRotationDelta` + `toggleFlip` 모두 정합.

**산출**: `mydocs/working/task_m100_831_stage_a.md`

### Stage B: 정정 구현 + 검증

**B.1 정정 구현**:

[`rhwp-studio/src/command/commands/insert.ts:397-410`](rhwp-studio/src/command/commands/insert.ts#L397):

```typescript
type PictureRef = {
  sec: number;
  ppi: number;
  ci: number;
  type: string;
  headerFooter?: { kind: 'header' | 'footer'; outerParaIdx: number; outerControlIdx: number };
};

function getProps(services: import('../types').CommandServices, ref: PictureRef): Record<string, unknown> {
  if (ref.type === 'shape') {
    return services.wasm.getShapeProperties(ref.sec, ref.ppi, ref.ci) as unknown as Record<string, unknown>;
  }
  // [Task #831] 머리말/꼬리말 picture 의 경우 별도 API 호출 (PR #832 의 wasm-bridge).
  // 미적용 시 본문 lookup 실패 → props 빈 → 회전/대칭 무동작.
  if (ref.headerFooter) {
    return services.wasm.getHeaderFooterPictureProperties(
      ref.sec,
      ref.headerFooter.outerParaIdx,
      ref.headerFooter.outerControlIdx,
      ref.ppi,
      ref.ci,
    ) as unknown as Record<string, unknown>;
  }
  return services.wasm.getPictureProperties(ref.sec, ref.ppi, ref.ci) as unknown as Record<string, unknown>;
}

function setProps(services: import('../types').CommandServices, ref: PictureRef, props: Record<string, unknown>): void {
  if (ref.type === 'shape') {
    services.wasm.setShapeProperties(ref.sec, ref.ppi, ref.ci, props);
  } else if (ref.headerFooter) {
    // [Task #831] 머리말/꼬리말 picture setter
    services.wasm.setHeaderFooterPictureProperties(
      ref.sec,
      ref.headerFooter.outerParaIdx,
      ref.headerFooter.outerControlIdx,
      ref.ppi,
      ref.ci,
      props,
    );
  } else {
    services.wasm.setPictureProperties(ref.sec, ref.ppi, ref.ci, props);
  }
}
```

`applyRotationDelta` / `toggleFlip` 본체는 변경 없음 (분기 위임).

**B.2 검증**:

1. tsc --noEmit (TypeScript 타입 검증)
2. cargo test --release --lib (Rust 변경 없음 — 회귀 0 자동 보장)
3. cargo clippy --release --lib (경고 0)
4. **수동/E2E 시각 검증** (작업지시자):
   - hwp3-sample11.hwp 머리말 picture 선택 → 4 버튼 (왼쪽회전/오른쪽회전/좌우대칭/상하대칭) 모두 동작
   - 본문 picture 회귀 0
   - HWP5/HWPX 동등
5. 회귀 sweep:
   - sample14 의 본문 picture (Task #864 정합 유지)
   - sample10-hwp5/hwpx (Task #873 image 표시 유지)

**B.3 회귀 우려 영역**:

- 본문 picture (`ref.headerFooter == undefined`) → `else` 분기 → 기존 `setPictureProperties` 동일 동작 → 회귀 0
- Shape (`ref.type === 'shape'`) → 기존 분기 그대로 → 회귀 0
- 머리말/꼬리말 picture → **시각 정합 (목표)**

**산출**: `mydocs/working/task_m100_831_stage_b.md`

### Stage C: 종합 보고서 + 커밋

**C.1 종합 보고서**: 본질 / 정정 / 검증 / 영향도

**산출**: `mydocs/report/task_m100_831_report.md`

**커밋 메시지 (안)**:
```
Task #831: 머리말/꼬리말 picture 회전/대칭 버튼 정정 (closes #831)

PR #835 (본 issue 본문 정정) 가 본문 picture 만 정정. 머리말/꼬리말 picture 의
경우 ref.headerFooter 분기가 누락되어 getProps/setProps 가 본문 lookup 만 호출
→ rotation/flip 무동작.

정정: insert.ts 의 getProps/setProps 에 ref.headerFooter 분기 추가 → PR #832
(Task #825) 의 get/setHeaderFooterPictureProperties wasm API 호출.

applyRotationDelta / toggleFlip 본체는 변경 없음 — getProps/setProps 위임으로
머리말/꼬리말 picture 도 자동 동작.

검증: tsc --noEmit 에러 0, cargo test 회귀 0, hwp3-sample11.hwp 머리말 picture
회전/대칭 4 버튼 모두 정합. 본문 picture / shape 회귀 0.

closes #831
```

## 작업 순서 + 승인 게이트

```
A 본질 정밀 진단 → 산출 → 단계 완료 → 승인
                                        ↓
B 정정 구현 + 검증 → 산출 → 단계 완료 → 승인
                                        ↓
C 종합 보고서 + 커밋 → 단계 완료 → 승인
```

## 위험 + 회피

| 위험 | 회피 |
|---|---|
| `ref.headerFooter` 가 falsy 인 본문 case 회귀 | `if (ref.headerFooter)` 분기 — undefined → else → 기존 본문 호출 |
| WASM API signature mismatch | picture-props-dialog 의 동일 호출 패턴 정합 (Stage A 검증) |
| TypeScript 타입 정합 | PictureRef 타입 alias 추가 (headerFooter optional) |
| WASM 빌드 필요 (라이브러리 변경 없음) | 기존 빌드 재사용 — Rust 변경 없음 |

## 본 단계 범위 외

- Picture 외 객체 (Shape/Line) 의 머리말/꼬리말 회전/대칭 (별도 본질, 본 task 범위 외 — Shape 는 본문 only 사용 추정)
- Picture 의 다른 dialog 속성 — 이미 PR #832 정합

## 승인 요청

본 구현계획서 승인 후 → Stage A 부터 진행.

📋 **Task #831 구현계획서 승인 요청드립니다.**

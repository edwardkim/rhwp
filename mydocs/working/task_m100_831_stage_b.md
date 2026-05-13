# Task #831 Stage B 정정 구현 + 검증

**작성일**: 2026-05-13
**브랜치**: `local/task831`

## B.1 정정 구현

### 변경 파일

**`rhwp-studio/src/command/commands/insert.ts`** (단일 파일)

- 신규 타입 alias `PictureRef` 추가 (cursor.selectedPictureRef 와 정합, headerFooter optional)
- `getProps`: `if (ref.headerFooter)` 분기 추가 → `getHeaderFooterPictureProperties` 호출
- `setProps`: `else if (ref.headerFooter)` 분기 추가 → `setHeaderFooterPictureProperties` 호출
- `applyRotationDelta` / `toggleFlip` 본체 변경 없음 (위임)

### 코드 diff 요약

```diff
+type PictureRef = {
+  sec: number; ppi: number; ci: number; type: string;
+  headerFooter?: { kind: 'header' | 'footer'; outerParaIdx: number; outerControlIdx: number };
+};
+
-function getProps(services, ref: { sec: number; ppi: number; ci: number; type: string }): ... {
+function getProps(services, ref: PictureRef): ... {
   if (ref.type === 'shape') { ... }
+  if (ref.headerFooter) {
+    return services.wasm.getHeaderFooterPictureProperties(
+      ref.sec, ref.headerFooter.outerParaIdx, ref.headerFooter.outerControlIdx,
+      ref.ppi, ref.ci,
+    );
+  }
   return services.wasm.getPictureProperties(ref.sec, ref.ppi, ref.ci);
 }
```

setProps 동일 패턴 (`else if` 분기).

## B.2 검증 결과

### 빌드 + 정적 검증

| 검증 항목 | 결과 |
|---|---|
| `cd rhwp-studio && npx tsc --noEmit` | ✓ 에러 0 |
| `cargo test --release --lib` | ✓ 1247 passed (회귀 0 — Rust 변경 없음) |
| `cargo clippy --release --lib` | ✓ 경고 0 |

### 시각 검증 (작업지시자 환경)

본 정정은 rhwp-studio (TypeScript) 변경으로 WASM 빌드 불필요. dev server 또는 빌드 후 브라우저에서:

1. hwp3-sample11.hwp 머리말 편집 모드 진입
2. 머리말 그림 (DCT 로고) 선택
3. 도구 상자 → 왼쪽회전/오른쪽회전/좌우대칭/상하대칭 4 버튼 모두 동작 확인
4. 본문 picture 회귀 0 확인
5. HWP5 / HWPX 동등 확인

작업지시자 시각 검증 완료 후 Stage C 진행.

## B.3 회귀 영향 확인

| 케이스 | 분기 | 결과 |
|---|---|---|
| 본문 picture (`ref.headerFooter == undefined`) | else → 기존 setPictureProperties | 회귀 0 (코드 동일) |
| Shape (`ref.type === 'shape'`) | 첫 분기 → 기존 setShapeProperties | 회귀 0 |
| Equation/Group/Line | applyRotationDelta/toggleFlip 진입 가드 차단 | 회귀 0 |
| 머리말/꼬리말 picture (`ref.headerFooter` 보유) | 신규 분기 → setHeaderFooterPictureProperties | **시각 정합 (목표)** |

## Stage B 결론

단일 파일 정정 (`insert.ts`) 으로 머리말/꼬리말 picture 회전/대칭 4 버튼 정합 가능 확인. 정적 검증 (tsc/clippy/test) 통과. 시각 검증은 작업지시자 환경에서 진행.

📋 **Stage B 완료. Stage C 종합 보고서 + 커밋 진행 승인 요청드립니다.**

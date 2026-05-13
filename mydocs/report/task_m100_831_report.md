# Task #831 최종 결과 보고서

**이슈**: https://github.com/edwardkim/issues/831 (#831 후속, PR #835 미정정 부분)
**브랜치**: `local/task831`
**작성일**: 2026-05-13
**제목**: 머리말/꼬리말 picture 회전/대칭 버튼 정정

## 1. 본질

PR #835 (#831 본문 정정, merged) 가 본문 picture 회전/대칭 가드만 제거. 머리말/꼬리말 picture 의 경우:

- `cursor.selectedPictureRef.headerFooter` 정보는 정확 보유 (PR #832 / Task #825)
- `wasm-bridge` 의 `get/setHeaderFooterPictureProperties` API 도 존재 (PR #832)
- 그러나 **`insert.ts:397-410` 의 `getProps`/`setProps` 가 `ref.headerFooter` 무시** → 본문 lookup 만 호출 → 머리말 picture IR 미갱신 → 회전/대칭 무동작

HWP3 / HWP5 / HWPX 포맷 모두 공통 발현 (rhwp-studio 의 single dispatch 결함).

## 2. 정정 내용

**단일 파일** (`rhwp-studio/src/command/commands/insert.ts`):

1. 신규 타입 alias `PictureRef` 추가 (cursor 의 selectedPictureRef 와 정합):
   ```typescript
   type PictureRef = {
     sec: number; ppi: number; ci: number; type: string;
     headerFooter?: { kind: 'header' | 'footer'; outerParaIdx: number; outerControlIdx: number };
   };
   ```

2. `getProps` 에 `if (ref.headerFooter)` 분기 추가:
   ```typescript
   if (ref.headerFooter) {
     return services.wasm.getHeaderFooterPictureProperties(
       ref.sec, ref.headerFooter.outerParaIdx, ref.headerFooter.outerControlIdx,
       ref.ppi, ref.ci,
     );
   }
   ```

3. `setProps` 에 `else if (ref.headerFooter)` 분기 추가 (동일 패턴 setter)

4. `applyRotationDelta` / `toggleFlip` 본체 변경 없음 (위임)

PR #832 의 wasm-bridge API + picture-props-dialog 의 검증된 호출 패턴 그대로 재사용.

## 3. 검증 결과

| 검증 항목 | 결과 |
|---|---|
| `cd rhwp-studio && npx tsc --noEmit` | ✓ 에러 0 |
| `cargo test --release --lib` | ✓ 1247 passed (회귀 0 — Rust 변경 없음) |
| `cargo clippy --release --lib` | ✓ 경고 0 |
| **시각 검증** (작업지시자) — hwp3-sample11.hwp 머리말 그림 회전/대칭 4 버튼 | ✓ 정합 |
| **save-as 저장 후 정상 저장 검증** | ✓ 정상 |

## 4. 영향도

| 케이스 | 분기 | 결과 |
|---|---|---|
| 본문 picture (`ref.headerFooter == undefined`) | else → 기존 setPictureProperties | 회귀 0 |
| Shape (`ref.type === 'shape'`) | 첫 분기 → 기존 setShapeProperties | 회귀 0 |
| Equation/Group/Line | applyRotation/toggleFlip 진입 가드 차단 | 회귀 0 |
| 머리말/꼬리말 picture | **신규 분기 — 시각 정합 (목표)** | ✓ |

## 5. CLAUDE.md 규칙 준수

- rhwp-studio 영역만 변경 (`rhwp-studio/src/command/commands/insert.ts`)
- parser / renderer / Document IR / 공통 모듈 무수정
- HWP3 전용 분기 추가 없음 — 포맷 무관 (TypeScript dispatch 1곳)
- 기존 wasm-bridge API 재사용

## 6. 단계별 보고서

- 수행계획서: `mydocs/plans/task_m100_831.md`
- 구현계획서: `mydocs/plans/task_m100_831_impl.md`
- Stage A 본질 정밀 진단: `mydocs/working/task_m100_831_stage_a.md`
- Stage B 정정 + 검증: `mydocs/working/task_m100_831_stage_b.md`

## 7. 의존성

- 선행: PR #835 (Task #831 본문, merged), PR #832 (Task #825 머리말 인프라, merged), PR #875 (Task #873) 머지 가정
- 후행: 없음

## 8. 커밋 메시지 (안)

```
Task #831: 머리말/꼬리말 picture 회전/대칭 버튼 정정

PR #835 (본 issue 본문 정정, merged) 가 본문 picture 만 정정. 머리말/꼬리말
picture 의 경우 ref.headerFooter 분기가 누락되어 getProps/setProps 가 본문
lookup 만 호출 → 머리말/꼬리말 picture IR 미갱신 → rotation/flip 4 버튼 무동작.

정정: rhwp-studio/src/command/commands/insert.ts 의 getProps/setProps 에
ref.headerFooter 분기 추가 → PR #832 (Task #825) 의 get/setHeaderFooterPicture
Properties wasm API 호출. picture-props-dialog 의 검증된 패턴 정합.
applyRotationDelta / toggleFlip 본체는 변경 없음 (위임).

검증: tsc --noEmit 에러 0, cargo test 1247 passed (회귀 0), clippy 경고 0.
hwp3-sample11.hwp 머리말 picture 4 버튼 (왼쪽회전/오른쪽회전/좌우대칭/상하대칭)
정합. save-as 저장 후 정상 저장 검증 통과.

closes #831
```

## 9. 결론

PR #835 의 머리말/꼬리말 picture 미정정 부분 완료. 단일 파일 변경 (rhwp-studio TypeScript) 으로 모든 포맷 (HWP3/HWP5/HWPX) 의 머리말/꼬리말 picture 회전/대칭 4 버튼 정합. 1247 테스트 회귀 0, clippy 경고 0, 시각 검증 통과 (작업지시자 환경 + save-as 저장 정상).

📋 **Task #831 최종 결과 보고서 — 커밋 + push + PR 진행.**

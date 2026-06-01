# Task #1151 v3 Stage 4 완료 보고서 — WASM 재빌드 + 브라우저 시각 재시연 + 한컴 편집기 1:1 정합

수행계획서: [task_m100_1151_v3.md](../plans/task_m100_1151_v3.md) · Stage 3 보고서: [task_m100_1151_v3_stage3.md](task_m100_1151_v3_stage3.md)

## 1. WASM 재빌드

```bash
docker compose --env-file .env.docker run --rm wasm
```

결과:
- `Finished release profile in 51.60s` → `Done in 3m 06s`
- 산출: `pkg/rhwp_bg.wasm` (5,310,798 bytes, May 29 21:05)
- vite dev server 자동 page reload (`21:05:55`).

## 2. 자동 SVG 좌표 검증 (Stage 3 의 외적 검증 재확인)

`./target/debug/rhwp export-svg samples/tac-verify/scenario-*-after.hwp` 의 image y 좌표가 한컴 산출물 기준 표 아래로 정확 이동:

| 시나리오 | image y (v3 후) | 표 y + height + outer_margin | 정합 |
|---------|------------------|-------------------------------|------|
| A — 1×1 작은 picture | 306.45 | 132.27 + 166.64 + 3.77 = 302.68 | y ≥ 표 끝 ✓ |
| B — 1×1 큰 picture | 328.79 | 표 + margin | ✓ |
| C — 3×3 중앙 셀 | 511.83 | 표 + margin | ✓ |
| D — 본문 floating (표 없음) | 132.27 (불변) | 0 | 회귀 0 ✓ |

## 3. 사용자 직접 시각 재시연

http://localhost:7700/ (vite dev server, v3 fix 반영 WASM) 에서 4 시나리오 시연:

| 시나리오 | 사용자 확인 결과 |
|---------|-------------------|
| A — 1×1 + 작은 picture | 표가 paragraph 위쪽 정상 위치 + picture 가 표 아래에 inline (한컴 편집기 출력과 정합) ✓ |
| B — 1×1 + 큰 picture | 동일 패턴, 표가 picture 만큼 밀려나지 않음 (한컴 정합) ✓ |
| C — 3×3 중앙 셀 picture | 표가 정상 위치, picture 가 표 아래 inline ✓ |
| D — 본문 floating | 기존 동작 그대로, 회귀 없음 ✓ |

**작업지시자 검증**: "4시나리오 정합" — v1 의 이전 캡처 (오버랩) 와 비교 시 picture 가 표 아래로 정확히 이동, 한컴 편집기 (Windows) 출력과 시각적으로 동일.

## 4. v1+v2 회귀 시나리오 확인

- 본문 inline picture 신규 삽입 (v1 의 본문 inline path): 기존 동작 그대로.
- 셀 안 picture 삽입 (v1 의 cell_path floating): 기존 동작 그대로 (sibling 표 없거나 paragraph 의 다른 control 구조이므로 reserved=0).
- floating → inline 토글 (v2 의 model migration): 4 시나리오 단위/통합 테스트 PASS 그대로.

## 5. Stage 5 진입 조건 → Scope 확장 (v4 추가)

Stage 4 시각 정합 확정 → Stage 5 (통합 PR) 직전 사용자가 별도 결함 발견:

> "tac-img-02.hwp 샘플 중에 이 파일 표 내의 이미지가 클릭이 안됨" (v3 이전부터 존재한 기존 결함)

해당 파일의 표 셀 안 paragraph 의 inline picture (tac=true, dump line 222: `bin_id=2, tac=true, wrap=TopAndBottom, vert=Para(off=0), horz=Para(off=0)`) 가 rhwp-studio 에서 클릭되지 않음.

**Scope 추가 결정 (작업지시자 승인)**: 본 PR 의 한컴 정합 범위에 셀 안 inline picture 의 click hit-test 정합 (v4) 도 함께 포함. 별도 후속 task 로 분리하지 않고 같은 release 묶음.

→ 다음: v4 수행/구현계획서 작성 → 작업지시자 승인 → Stage 6 (Fix #1/#2/#3) → Stage 7 (시각·클릭 검증) → 통합 PR.

## 6. 검증 결과 종합 (v1+v2+v3 시점)

| 항목 | 결과 |
|------|------|
| `cargo test --lib` 전수 | 1442 passed, 0 failed, 6 ignored (회귀 0) |
| `cargo clippy --lib -- -D warnings` | clean |
| `cargo fmt --all -- --check` | clean |
| v2 통합 테스트 (model 정합) | 4/4 PASS |
| v3 helper 단위 테스트 | 6/0 |
| SVG 좌표 한컴 정합 (4 시나리오) | A/B/C 정확 이동, D 회귀 0 |
| WASM 빌드 | 5.3 MB, 3m 06s |
| 사용자 시각 시연 (4 시나리오) | 한컴 편집기와 정합 |

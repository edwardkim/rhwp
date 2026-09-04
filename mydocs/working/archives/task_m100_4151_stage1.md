---
kind: working
status: completed
issue: 4151
last_verified: 2026-08-08
---

# Task #4151 Stage 1 - 셀 블록 서식 토글 방향·툴바 동기화

## 구현

- `getCharPropertiesAtCellBlockAnchor`(블록 첫 셀 첫 글자) 단일 기준 신설 — 토글 방향과
  툴바 상태 방출이 공유한다.
- `applyToggleFormat`: 셀 블록 모드에서 `getCharPropertiesAtCursor()`(블록 밖을 읽음 — 재적용
  원인) 대신 블록 앵커 기준. 빈 블록(전 셀 제외)은 앵커가 없어 커서 폴백 — `applyCharFormat`이
  빈 블록에서 조기 종료하므로 방향은 무해(upstream #4119 빈 블록 의미 보존).
- `applyCharFormatToCellBlock`: 적용 직후 앵커 셀 기준 `cursor-format-changed` 방출(try/catch,
  실패 시 다음 캐럿 이동에서 자연 동기화).

## 검증 결과

- 신규 #4151 소스 계약 테스트 3종: 토글 방향 블록 분기 존재/앵커 함수 사용, 적용 후 방출이
  executeOperation 뒤에 앵커 기준으로 존재, 앵커 기준이 첫 셀 첫 글자 — red 증명: 수정 전
  파일(HEAD)에서 3종 전부 FAIL 확인 후 green.
- `tests/cell-block-format.test.ts` + `tests/mutation-routing-guard.test.ts`: 27 passed / 0.
- 실브라우저(:7701, hwp_table_test_saved 2×2 표, 셀 0-1 블록): pre bold=false·버튼 비활성 →
  1클릭 두 셀 bold=true·버튼 즉시 활성·블록 유지 → 2클릭 두 셀 bold=false·버튼 비활성 —
  wasm getCellCharPropertiesAt 로 문서 상태 직접 검증.
- 참고: upstream #4119 의 빈 블록 의미 변화와의 병합은 rebase 시 수동 해소 — 빈 블록 길이
  가드를 토글 방향 폴백에 추가.

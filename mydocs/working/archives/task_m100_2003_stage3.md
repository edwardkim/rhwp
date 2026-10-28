# 단계 완료 보고 — Task M100 #2003 3단계: 추출 2 (① B 블록 run 방출 루프)

- 작성일: 2026-07-06 / 브랜치: `local/task2003`

## 수행 내용 (계획대로 2커밋 분리)

- **커밋 1** (`0ed5c329`): `RunEmitState`/`RunEmitVars` struct 도입 (dead_code 허용,
  동작 무변경 — 컴파일 확인).
- **커밋 2** (본 커밋): run 방출 루프(1,056줄) 전체를 `emit_line_runs`로 추출.
  루프 통이동이라 내부 break/continue는 자체 루프 대상(라벨 break 0, return은 클로저
  내부 1곳뿐 — 전수 확인), 제어 흐름 수술 불요.

### 캐리오버 최종 확정 (컴파일러 검증으로 계획 대비 3건 정정)
- **baseline/raw_lh 는 오탐 제거** (eprintln 포맷 문자열의 `baseline=` 패턴이 mut 스캔에
  걸린 것 — 실제 읽기 전용) → `RunEmitVars`로 이동.
- **char_offset 추가** (문서 좌표 누적 카운터, `+=` 1곳) → `RunEmitState` 8번째 필드.
- **fn_marker_inserted/shape_marker_inserted 추가** (인덱스 대입 — 정규식 사각지대에서
  발견) → `&mut [bool]` 파라미터.
- 최종: `RunEmitState` **8필드**(x/y/char_offset/run_char_pos/inline_tab_cursor_render/
  pending 2종/tac 예약 높이) 값 왕복 + `RunEmitVars` 21필드 + 참조 18개.
- 기타 보정: `col_area`/`char_offset`/`header 클로저`(is_last_run_of_line 함수 내 재정의,
  caller 정의 제거)/`tab_stops.to_vec()` — 전건 컴파일러 검출, 기계적.

## 게이트 결과 (전수 통과)

| 게이트 | 결과 |
|---|---|
| cargo fmt --check / clippy | 통과 / **경고 0** |
| cargo test --profile release-test --tests | **2,912 통과 / 실패 0** |
| OVR baseline 5샘플 | **추가 변동 0** (기지 #1936발 3건 시그니처 동일) |

## 계측 (라운드 4 누적)

| 함수 | 라운드 시작 | 현재 |
|---|---|---|
| `layout_composed_paragraph` | 3,088줄 · 분기 493 | **2,093줄 · 365** |
| `parse_object_control_char` | 1,039줄 · 분기 215 | **289줄 · 57** |
| 신규: `emit_line_runs` / `parse_hwp3_object_dispatch` | — | 1,107·128 / 812·159 |

## 다음 단계

4단계 — 재평가(`--snapshot r4 --no-coverage`) + 공식 CC 대비 + 최종 보고. 승인 후 착수.

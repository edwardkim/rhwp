# Task #990 Stage 3 완료보고서 — 검증

**이슈**: [#990](https://github.com/edwardkim/rhwp/issues/990)
**브랜치**: `local/task990`

---

## 1. 전체 테스트

`cargo test` — **전 바이너리 합계 1477 passed, 0 failed** (`issue_990` 포함).
`cargo clippy --release` — 경고 0.
`cargo fmt --check` (수정 파일) — clean.

## 2. 광범위 SVG sweep (14 샘플 / 비-RFP 124 SVG)

`devel`(before) vs `local/task990`(after) `export-svg` 차분:

- **123 byte-identical** — exam_kor/math/science, synam-001, KTX, shortcut,
  table-in-tbox 등 회귀 0.
- 변경 2건:
  - `hy-001_002.svg` — 초기 fix(Stage 2)가 선행 표+Shape 문단(pi=27)에서
    글상자를 표 위로 올려 표를 가림. **Stage 2 v2**(`has_full_para_item`
    분기)로 복원 — devel 과 동일.
  - `group-box.svg` — `fill="none"`(투명) rect 1개 y 이동, PNG 렌더
    **바이트 동일**. 시각 변화 0(투명 도형 경계의 내부 좌표 정정).

## 3. 회귀 1건 발견 → 정정: `issue_table_vpos_01_page5_cell_hit_test`

초기 전체 테스트에서 이 파일 4건 실패(c2 본문 셀 hit-test).

- **원인**: `table-vpos-01` 5쪽 `pi=33`(빈 문단 + treat-as-char 다각형) 은
  RFP 와 동일 구조 — 본 수정으로 `pi=34`(외곽 표 + 내부 11x3) 가 위로 이동.
  테스트 기댓값은 커밋 `c2d2157d "test: update expectations for current
  layout"` 에서 #974 머지 직후 갱신 — **#974 이중 가산 버그가 박제된 좌표**.
- **이동량**: `--debug-overlay` 측정 — `devel` `pi=34` y=260.69 →
  수정 후 229.85 = **30.84px**(`.hwp` 기준).
- **정정**: `pi=34` inner 11x3 의 c0/c2 hit-test 좌표 8건 + cell-clip 주석을
  −30.84px 갱신. `pi=30/pi=32`(pi=33 위) 좌표는 불변. title 셀 검증은
  loose(임의 pi=34 nested hit) 라 기존 좌표 유지.
- **결과**: 13/13 통과.

한컴 PDF(`pdf/table-vpos-01-2022.pdf`) 대조 — 수정 후 빨강 배너↓~큰 표↑
간격이 devel 대비 PDF 에 부합(devel 71.84px → 수정 38.86px, `.hwpx` 기준).

## 4. 산출물

| 파일 | 변경 |
|------|------|
| `src/renderer/layout.rs` | `layout_shape_item` 빈 문단 분기 (Stage 2 v2, fmt 정리) |
| `tests/issue_990.rs` | RED→GREEN 회귀 테스트 (Stage 1) |
| `samples/issue-990-tac-box.hwpx` | 재현 픽스처 (Stage 1) |
| `tests/issue_table_vpos_01_page5_cell_hit_test.rs` | #974 박제 좌표 8건 −30.84px 정정 |

## 5. 다음 단계 (Stage 4)

Docker WASM 빌드, 최종 보고서, 오늘할일 갱신.

# Stage 2 완료 보고서 — Task #866 v2: 헤더 띠/소제목 zone 전환 간격 정밀화

GitHub Issue: edwardkim/rhwp#866 · 브랜치: `pr-task853`

## 변경

### `src/renderer/typeset.rs::process_multicolumn_break`
zone 전환 시 `solo_zone_pad`(=1200 HU ≈ 16px ≈ 한 본문 줄)를 추가:
- **진입 게이트**: 새 zone 첫 paragraph 가 1단 ColumnDef 이고 그 `간격`=0 인 경우 (헤더 띠 `파일`/`보기`/`입력`/`서식`/`표`/`도구`/`기타` 또는 `<...>` 소제목 zone 진입).
- **이탈 게이트**: 직전 zone 이 1단/간격=0 이었던 경우 (`st.col_count <= 1 && st.current_zone_design_spacing_px < 0.5`).
- 둘 중 하나라도 참이면 `+16px`. 둘 다 참이라도 `+16px`(한 번만 가산 — 진입과 이탈이 동일 transition).

`candidate_offset = current_zone_y_offset + vpos_zone_height + tac_band_extra + prev_design/2 + new_design/2 + solo_zone_pad`.

### `src/renderer/layout.rs::build_columns` (미러)
동일한 `solo_zone_pad` 를 `current_zone_start_y` 계산에 추가. `prev_zone_was_solo` 상태 변수로 직전 zone 의 1단 여부 추적.

## 결과 (shortcut.hwp ↔ `pdf/basic/shortcut-2022.pdf`)

| 항목 | Stage 1 후 | Stage 2 후 | 한컴 PDF |
|---|---|---|---|
| 페이지 수 | 7 | **7** | 7 ✓ |
| `LAYOUT_OVERFLOW` | 6 | **0** | — ✓ |
| 3쪽 마지막 그룹 | `<글상자에서>` 일부 초과 | **`<그림 넣기에서>` 그룹까지** | `<그림 넣기에서>` 그룹까지 ✓ |
| `cargo test --release` | 1232 통과 | **1232 통과** | — ✓ |
| svg_snapshot | 8/8 (golden 무변경) | **8/8 (golden 무변경)** | — ✓ |

### 페이지 3 zone 배치 (dump-pages 메타데이터, Stage 2)
| zone | items | zone_y_offset | 내용 |
|---|---|---|---|
| 단0 | pi=81 | 0 | "보기" 헤더 띠 |
| 단1/2 | pi=82-93 | 107.0 | 본문 (2단) |
| 단3/4 | pi=94/95 | 219.0 | `<편집 화면 분할에서>` + `화면 이동` (2단) |
| 단5 | pi=96 | 253.7 | "입력" 헤더 띠 |
| 단6/7 | pi=97-115 | 360.6 | 본문 (2단) |
| 단8 | pi=117 | 563.3 | `<그림 넣기에서>` |
| 단9/10 | pi=118-126 | 598.0 | 본문 (2단) — 페이지 하단 ~651px (h=701.7) |

→ 한컴 PDF 3쪽과 구조 정합.

## 회귀 확인
- `cargo test --release` 34 suites / 1232 tests / 0 failed. svg_snapshot 8/8 golden 무변경.
- `exam_math.hwp` / `21_언어_기출_편집가능본.hwp` (Stage 1 회귀 후보) — 통과.
- `LAYOUT_OVERFLOW` = 0 (다른 fixture sweep 영향 없음).

## 잔존
- band/zone 의 미세 위치 ~5~10px 잔존 오차 (96dpi PNG 측정 한계). 정밀 ±1px 정합은 Hancom 편집기 (Windows) cross-check 필요.
- 점선 단 구분선(`ColumnDef.separator_type=3`) 렌더 — Stage 3 (별도 단계).

## 커밋 예정
- `src/renderer/typeset.rs` (solo_zone_pad 가산)
- `src/renderer/layout.rs` (build_columns 미러)
- `mydocs/working/task_m100_866_v2_stage2.md` (본 보고서)

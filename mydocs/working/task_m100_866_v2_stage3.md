# Stage 3 완료 보고서 — Task #866 v2: zone 별 점선 단 구분선 + column_break_pad

GitHub Issue: edwardkim/rhwp#866 · 브랜치: `pr-task853`

## 변경

### `src/renderer/layout.rs::build_columns` + `emit_zone_column_separators` (신규)
- 기존 `build_column_separators` 는 page 전역 `layout` 만 보아서 페이지 내 다단 zone (예: shortcut.hwp 본문 2단/배분, 구분선 type=7) 의 ColumnDef 를 못 봤다.
- 새 `emit_zone_column_separators` 가 zone 별 `zone_layout.column_areas` 와 `separator_type/width/color` 를 사용해 그 zone 의 y 범위 안에서만 구분선 그린다. zone 진입 시점에 직전 zone 분 emit + 루프 종료 후 마지막 zone 분 emit.
- `StrokeDash` 매핑: 한컴 `구분선 type=6/7` 은 `Dot` 으로 확장 매핑(점선 변형).

### `src/renderer/typeset.rs::process_multicolumn_break` + `layout.rs::build_columns`
- `column_break_new_band` 게이트 추가: 새 zone 의 첫 paragraph `column_type == ColumnBreakType::Column` 이면 solo_zone_pad(+16px) 발동. Stage 1 에서 도입한 배분 다단의 마지막 컬럼 `[단나누기]` 라우팅과 정합 — 같은 ColumnDef 로 시작하는 다음 zone band 와 이전 band 사이에 ~한 본문 줄 여백.
- 적용 사례: shortcut.hwp 3쪽 `화면 확대 100%`↔`<편집 화면 분할에서>` (둘 다 2단/배분, 종전 pad 0).

## 결과 (shortcut.hwp ↔ `pdf/basic/shortcut-2022.pdf`)

| 항목 | Stage 2 후 | Stage 3+4 후 | 한컴 PDF |
|---|---|---|---|
| 페이지 수 | 7 | **7** | 7 ✓ |
| `LAYOUT_OVERFLOW` | 0 | **1** | — (작은 잔존) |
| 점선 단 구분선 | 미렌더 | **렌더 ✓** (1·2·3·4·5·6·7쪽 본문 2단) | dotted gray ✓ |
| pi=94 `<편집 화면 분할에서>` zone 진입 | offset 219 | **offset 235** (+16 column_break) | 한컴 ~+35 |

### SVG 검증 (shortcut_001.svg ~ shortcut_007.svg)
- 페이지별 본문 2단 zone 의 단 사이 vertical line: x=561.25, `stroke="#aeaeae"`, `stroke-dasharray="2 2"` (Dot), width=1.9 — 한컴 PDF 회색 점선 정합.
- 1쪽: 2개 (단 0 본문 zone, 단 1 본문 zone)
- 2쪽: 3개 (각 본문 zone)
- 3·4·5·6·7쪽: 다수 (페이지마다 본문 zone 개수)

## 잔존
- **`<스타일에서>` zone gap (4쪽, pi=148)**: 16px solo_pad 적용 후도 한컴 PDF 대비 ~10~20px 부족하다고 작업지시자가 보고. 20px 로 상향 시도 → 페이지 수 7→8, LAYOUT_OVERFLOW 1→18 회귀. 16px 가 회귀 없이 적용 가능한 최대치. 추가 정밀화는 Hancom 편집기 cross-check 또는 더 정교한 모델(transition 유형별 차등) 필요.
- **`도구` 띠 (6쪽, pi=210)**: ColumnDef 1단/간격=**1mm** (=3.78px) 로 solo/tac_band 게이트(`< 0.5`) 미발동 → 좁음. 게이트 임계값 `5.0` 으로 상향 시도 → `[다단나누기]` 새 zone 진입에서 pad 가 너무 일찍 발동하여 pi=209 가 단독 페이지로 분리 (페이지 수 7→9). 16px 정책으로 회복. 별도 게이트 또는 1단 미사용 spacing 무시 규칙 필요.
- `cargo test --release`: 광역 sweep 결과 별도 commit 시점에 첨부.

## 커밋 예정
- `src/renderer/typeset.rs` (`column_break_new_band` 게이트)
- `src/renderer/layout.rs` (`emit_zone_column_separators` 신규 + `column_break_new_band` 미러)
- `mydocs/working/task_m100_866_v2_stage3.md` (본 보고서)

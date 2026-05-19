# PR #1005 검토 — fix: HWP5/HWP3 한컴 정합 종합 fix — 격차 A/B/C/D

- 작성일: 2026-05-19
- 컨트리뷰터: [@jangster77](https://github.com/jangster77) (Taesup Jang)
- PR: https://github.com/edwardkim/rhwp/pull/1005
- base/head: `devel` ← `jangster77:local/task1001-pr` (cross-repo fork)
- 연결 이슈: closes #1001 (HWP5 HWP3 변환본 전반 한컴 정합)
- 규모: +1750 / -14, 22 files (소스 10, 문서 12)
- mergeable: UNKNOWN

## 1. 컨트리뷰터 사이클 (`feedback_contributor_cycle_check`)

@jangster77 24+ 사이클. #997(closes #994, 머지) → #999(closes #998, 머지) → **#1005(closes #1001, 본 PR)** → #1009 → #1011 연속. #997/#999 가 sample16 겹침/페이지수였다면 #1005 는 HWP3 변환본 전반 한컴 정합 종합.

## 2. 본질 커밋 (2개, 모두 작성자 Taesup Jang)

| 커밋 | 내용 | 비고 |
|------|------|------|
| `7c482851` | Task #1001 종합 fix (격차 A/B/C/D), 14파일 | 1차 |
| `5148102d` | variant 식별 FP 차단 + **격차 D revert**, 2파일 | **후속 자정** |

## 3. 격차별 분석

### 격차 A — 페이지 번호 외곽선 안/밖 (layout.rs build_page_borders)

HWP5 spec 표 136 bit 1(머리말 포함)/bit 2(꼬리말 포함) 미처리 → 변환본 `pgbf.attr=0x01` paper-based 외곽선이 꼬리말(페이지번호) 영역 침범. Fix: `header_inside=(attr&0x02)`, `footer_inside=(attr&0x04)` 판정 → `!header_inside` 시 border top 을 `body_area.y` 로 clip, `!footer_inside` 시 bottom clip.

- **평가**: HWP5 spec 정합. 주석에 자정 이력 명시 ("20px buffer 시도 → 페이지3 타이틀 outline 밖 회귀 → revert", "회귀 발견 시 paper_based 조건으로 좁힐 수 있음"). body 기준 + spacing>0 케이스도 동일 처리 — HWP3 sample16 baseline 변동 가능 (시각 검증 필요).

### 격차 B — PUA 글머리표 (composer.rs pua_plain_text_display)

`0xF03C5 → "□"` (U+25A1) 매핑 추가. sample16-hwp5 PUA codepoint 글자 분석 근거.

- **평가**: 단순 매핑 1줄. 저위험. `feedback_font_alias_sync` 무관 (display_text 매핑, metric 영향 없음).

### 격차 C — 변환본 식별 + ParaShape /4 보정 (parser/style_resolver)

HWP3→HWP5 변환본 ParaShape spacing/margin 이 HwpUnitChar(HWPUNIT 2배) → rhwp 2배 격차. Fix:
1. `cfb_reader.rs detect_hwp3_variant()` — HwpSummary 1990-2003년 검출
2. `model/document.rs is_hwp3_variant: bool` 필드
3. `parser/mod.rs` **다중 결합 가드**: `summary_hwp3_era AND total_paras>50 AND ps_r<0.20 AND cs_r<0.20`
4. `style_resolver.rs resolve_styles_with_variant` — variant 시 ParaShape `/4` (기본 `/2` + 추가 `/2`)

- **평가**: `feedback_hancom_compat_specific_over_general` **정합 (권위 사례 강화)** — 측정 의존 단일 ratio 가 아닌 **구조 가드 결합**(HwpSummary 시대 년 + para 수 + PS/CS 비율). 후속 `5148102d` 가 "ratio 단독 분기 제거"로 추가 강화 (hwpspec.hwp spec 문서 FP 차단). PR 본문 8 sample 검증 8/8 정확 (exam_eng FP→PS/CS 차단, 복학원서 FP→para<50 차단). **단 ParaShape /4 는 변환본 전체 paragraph 영향 — 광범위. variant 오판 시 전 문서 spacing 붕괴 위험** (가드 견고성에 의존).

### 격차 D — Shape paragraph y_offset double count (layout.rs)

1차 `7c482851`: `result_y = y_offset + line_spacing×3` 휴리스틱.
**후속 `5148102d` 에서 revert**: non-variant 문서(table-vpos-01)의 treat_as_char Shape 가 본 분기로 result_y 작아져 후속 Table 위치 어긋남 → cell hit-test 회귀 발견 → **이전 동작(`shape_y + line_advance.max(shape_h)`) 복원**. 시각 breathing room 은 "PR #1005 scope 외" 후속 분리 명시.

- **평가**: `feedback_hancom_compat_specific_over_general` + `feedback_fix_scope_check_two_paths` **권위 사례** — 일반화 휴리스틱(line_spacing×3)이 다른 케이스(table-vpos-01) 회귀 → 컨트리뷰터 검증 단계 자정 revert. **최종 2 커밋 적용 상태에서 격차 D 는 사실상 no-op(원복). 실효 fix 는 A/B/C 만**.

## 4. 검토 의견

### 강점

1. **컨트리뷰터 자정 이력 우수** — 격차 D revert + variant FP 차단 강화. 검증 중 회귀 발견 후 scope 좁힘 (`feedback_hancom_compat_specific_over_general` 권위 행동).
2. **격차 C 구조 가드 설계** — 측정 단일 ratio 아닌 4중 AND 결합. FP 차단 8/8 검증.
3. **HWP5 spec 근거** — 격차 A 표 136 bit 1/2 명시.
4. **HWP3 전용 분기 아님** — variant 는 런타임 식별 플래그, parser/style_resolver 공통 경로 조건. CLAUDE.md HWP3 규칙(공통 모듈 HWP3 전용 분기 금지) 위반 아님.
5. cargo test 1301/1306 + clippy 0 + WASM 4.83MB + 잔존 4건 투명 명시.

### ⚠️ 핵심 쟁점

#### (A) 격차 C ParaShape /4 광범위 영향 — variant 오판 시 전 문서 붕괴

variant=true 시 **전 paragraph ParaShape /4 보정**. 가드가 견고(8/8)하나 검증 sample 외 일반 HWP5(HwpSummary HWP3-era 텍스트 + para>50 + PS/CS 우연 <0.20) FP 시 전 문서 spacing 1/4 붕괴. **광범위 sweep 으로 일반 HWP5 다수 fixture variant=false 유지 확인 필수** (회귀 표면 큼).

#### (B) 격차 A body_area clip — HWP3 원본 baseline 변동 가능

주석 자인: "body 기준 + spacing>0 케이스도 동일 처리 — HWP3 sample16 baseline 변동 가능". HWP3 원본(sample16.hwp) 회귀 sweep 확인 필요.

#### (C) 격차 D 실효 없음 (revert) — PR 제목과 실제 효과 괴리

PR 제목 "격차 A/B/C/D" 이나 D 는 최종 no-op. 검토/보고서에 **실효 = A/B/C, D 는 후속 분리** 명확화 필요 (오해 방지).

### 확인 필요 (검증 단계)

1. cherry-pick `7c482851` + `5148102d` 순차 (충돌 여부)
2. cargo test --release --lib + clippy -D + fmt 0
3. **광범위 sweep** — 쟁점 A: 일반 HWP5 다수(exam_kor/math/eng, aift, biz_plan, 통합재정통계, 복학원서) **variant=false 유지 + diff 최소**, 쟁점 B: HWP3 원본 sample16.hwp 회귀, 타깃 sample16-hwp5 격차 A/B/C 개선
4. WASM 빌드 + 작업지시자 시각 판정 — sample16-hwp5 page 1/3 격차 개선 + 일반 HWP5 회귀 부재

## 5. 처리 옵션

- **옵션 A (수용)**: sweep 일반 HWP5 variant=false 유지 + 회귀 부재 + 작업지시자 시각 판정 통과 시. 쟁점 A(variant 오판) sweep 검증 + D revert 사실 보고서 명확화 필수.
- **옵션 B (수정 요청)**: 쟁점 A 가 일반 HWP5 FP/회귀로 확인되면 — variant 가드 추가 강화 또는 ParaShape /4 영역 축소 요청.
- **옵션 C (close)**: 본질 결함 시. 실효 A/B/C 개선 + 자정 이력으로 해당 낮음.

## 6. 메모리 룰 정합

- `feedback_contributor_cycle_check` — @jangster77 #997~#1011 연속, #1005 순차
- `feedback_hancom_compat_specific_over_general` — **권위 사례 강화**: 격차 C 구조 가드 결합 + 격차 D 일반화 휴리스틱 회귀→revert 자정
- `feedback_fix_scope_check_two_paths` — 격차 D non-variant(table-vpos-01) 경로 회귀 발견 후 revert (검증 단계 자정)
- `feedback_self_verification_not_hancom` — 격차 C "HWP3 정합"이 한컴 권위인지 작업지시자 시각 판정 게이트
- `feedback_v076_regression_origin` — variant 오판 시 작업지시자 환경 회귀 위험 — 일반 HWP5 sweep + 시각 판정 필수
- `feedback_visual_judgment_authority` — sample16-hwp5 격차 개선 + 일반 HWP5 무회귀 작업지시자 시각 판정 최종 게이트
- `feedback_image_renderer_paths_separate` — PR 잔존에 "rhwp-studio WASM 렌더링 path 진단" 명시 — Canvas2D/SVG 차이 후속

## 7. 권고

**옵션 A 조건부** — 검증 단계에서 (1) test/clippy/fmt GREEN, (2) **쟁점 A: 광범위 sweep 으로 일반 HWP5 다수 fixture variant=false 유지 + 회귀 부재 입증**, (3) 쟁점 B: HWP3 sample16.hwp 회귀 확인, (4) WASM + 작업지시자 시각 판정(sample16-hwp5 page 1/3 격차 개선 + 일반 HWP5 무회귀) 통과 시 2 커밋 cherry-pick no-ff merge. **보고서에 격차 D revert(실효 A/B/C) 명확화**. 쟁점 A 회귀 시 옵션 B. 잔존 4건(머릿말 inline image / 페이지 강제나눔 / gradient simplify / WASM path)은 PR 본문대로 후속 issue 분리.

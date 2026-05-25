# PR #1095 검토 — Task #977: WASM 미등록 폰트 폴백 native EmbeddedTextMeasurer 동기화

## 1. 개요

| 항목 | 내용 |
|------|------|
| PR | [#1095](https://github.com/edwardkim/rhwp/pull/1095) |
| 작성자 | planet6897 (Jaeuk Ryu) — 핵심 컨트리뷰터 (16+ 사이클) |
| base / head | `devel` / `planet6897:local/task977-v3-pr` |
| 이슈 | closes #977 — Skia replay 경로 선두 공백 CharShape 다른 목차 문단의 개요번호 x 우측 밀림 |
| label | enhancement |
| mergeable / merge state | MERGEABLE / **BLOCKED** (CI 진행 중) |
| 변경 | +394 / -9, 6 files (코드 1 + 문서 5) |
| CI | 진행 중 (2026-05-25, pending → 통과 확인 후 머지 가능) |

## 2. PR 의 v3 사이클 이력

| 차수 | PR | 처리 |
|------|----|------|
| v1 | #980 | 자진 close |
| v2 | #1045 | 메인테이너 close — "PR #1026 본질 흡수" 판단 |
| **v3** | **#1095** | 본 PR — PR #1026 가 정정한 `measure_char_width_hwp` 외에 **`measure_hangul_width_hwp`** 도 통일 필요 영역 발견 |

→ v3 의 본질이 v2 의 close 사유와 명확히 분리됨 (다른 함수 영역).

## 3. 본질 분석

### 3.1 결함 (이슈 #977)

`rhwp-studio` (WASM/Skia replay) 에서만:
- 선두 공백 CharShape 가 "다른 폰트 + 장평 95%" 인 문단의 개요번호 x 좌표가 ~9-10px 우측 밀림
- `export-svg` (네이티브) 는 정상 — **WASM 전용 결함**

### 3.2 근본 원인 (PR 설명 + 코드 정독 검증)

`src/renderer/layout/text_measurement.rs` 의 WASM `WasmTextMeasurer` 가 미등록 폰트
폭 폴백을 **두 함수** 에 분기:

| 함수 | 영역 |
|------|------|
| `measure_char_width_hwp` | 일반 문자 (라틴, 구두점 등) |
| `measure_hangul_width_hwp` | 한글 '가' 대리 측정 |

두 함수 모두 미등록 폰트 → `cached_js_measure` (JS Canvas `measureText`) 호출 → 브라우저
fallback 폰트로 측정 → 폰트별 폭 변동.

**한컴 LEFT tab + tab_extended[0] contract**: 한컴이 행별로 "tab_pos − 한컴 선행
텍스트 폭" 을 사전 계산하여 저장. WASM 한글 폭이 한컴 metric 과 다르면, 같은 ext[0]
누적이 행별 한글 개수 차이 × 폭차 만큼 어긋남 → 디지트 x 좌표 시프트.

### 3.3 PR #1026 (5/21 머지, HaimLee-4869) 와의 관계

PR #1026 가 같은 파일의 `measure_char_width_hwp` 만 native heuristic 동기화 +
narrow_punct 분기 추가. 본 v3 PR 가:
- `measure_char_width_hwp` 의 narrow_punct 분기 **보존** (PR #1026 정정 유지)
- `measure_char_width_hwp` 의 마지막 JS 폴백 분기를 native heuristic 으로 통일
- **`measure_hangul_width_hwp` 도** 동일 패턴 통일 ← v3 의 추가 본질

## 4. 코드 변경 정독 결과 (`src/renderer/layout/text_measurement.rs`)

### 4.1 `measure_char_width_hwp` 의 JS 폴백 분기 정정

```rust
// 변경 전 (3차 폴백: JS Canvas)
let raw_px = cached_js_measure(measure_font, c);
let actual_px = raw_px * font_size / 1000.0;
let hwp = (actual_px * 75.0).round() as i32;
hwp as f64 / 75.0

// 변경 후 (native heuristic 동기화)
if super::is_cjk_char(c) || super::is_fullwidth_symbol(c) {
    return font_size;  // CJK/fullwidth: 1.0 em
}
font_size * 0.5  // 일반 문자: 0.5 em
```

PR #1026 의 narrow_punct 분기 (0.3 em) 는 본 분기 **위에서 이미 처리** → 보존됨 확인.

### 4.2 `measure_hangul_width_hwp` 의 JS 폴백 정정

```rust
// 변경 전 (JS Canvas '가' 측정)
let raw_px = cached_js_measure(measure_font, '\u{AC00}');
let actual_px = raw_px * font_size / 1000.0;
(actual_px * 75.0).round() as i32

// 변경 후 (native CJK 휴리스틱)
(font_size * 75.0).round() as i32  // 1.0 em
```

`measure_font` 파라미터 unused 처리 (`_measure_font`) — 시그니처 보존 (caller 영향 없음).

### 4.3 영향 영역 평가

| 케이스 | 영향 |
|--------|------|
| 등록 폰트 (맑은 고딕, HCR Batang 등) | `measure_char_width_embedded` / `measure_hangul_width_embedded` 가 `Some` 반환 → 본 분기 미진입, **무회귀** ✓ |
| 미등록 폰트 (나눔바른고딕 등) | native heuristic 과 동일 폭 → SVG/WASM 일관 정렬 ✓ |
| CJK / fullwidth 일반 문자 | 1.0 em (PR #1026 정합) ✓ |
| narrow_punct (괄호, 따옴표 등) | 0.3 em (PR #1026 정합, 보존) ✓ |

→ **회귀 위험 매우 낮음**. 미등록 폰트에서만 동작 변경 + native 와 동일 결과.

## 5. PR 작성자 검증 (PR 본문 명시)

- `issue_874_ktx_toc_page_number_right_align` (PR #1026 회귀 가드): **1/1 PASS**
- `svg_snapshot` (golden SVG 8 종): **8/8 PASS**
- `cargo check --lib --target wasm32-unknown-unknown`: OK
- Docker WASM 빌드 성공
- 작업지시자 시각 재현 확인 (TOC 페이지 디지트 정렬)

## 6. 트러블슈팅 / 관련 자료

- `mydocs/troubleshootings/toc_leader_right_tab_alignment.md` — 목차 right-tab 정렬 (#279/#874 영역)
- 본 PR 의 본질이 위 트러블슈팅의 후속 영역 (WASM 한글 폭 폴백 영역 정합)

## 7. 위험 분석 + 권장

| 위험 | 평가 |
|------|------|
| 미등록 폰트의 한글 폭이 native heuristic 과 다른 정답이 있을 가능성 | 매우 낮음 — native 가 한컴 PDF 정합 기준 (#874 영역 검증) |
| `measure_font` 파라미터 미사용으로 인한 API 변화 | 시그니처 보존 (`_measure_font` 접두), caller 영향 없음 |
| CI 진행 중 (BLOCKED) | CI 통과 후 머지 영역 — 본 PR 의 변경 영역 (text_measurement.rs) 회귀 가드 + svg_snapshot 가드 통과 시 안전 |
| PR #1026 정정 영역 회귀 | narrow_punct 분기 보존 확인 (코드 정독) |

## 8. 검증 계획 (메인테이너 영역)

| 항목 | 명령 |
|------|------|
| 회귀 가드 1 (PR #1026 정합) | `cargo test issue_874_ktx_toc_page_number_right_align --release --lib` |
| svg snapshot 회귀 가드 | `cargo test --release --tests svg_snapshot` |
| 전체 lib | `cargo test --release --lib` |
| WASM target check | `cargo check --target wasm32-unknown-unknown --lib` |
| WASM Docker 빌드 | `docker compose --env-file .env.docker run --rm wasm` |
| CI (PR 자동) | Build & Test / CodeQL / Canvas visual diff |

## 9. 처리 권장

- **merge 권장** (CI pass 확인 후) — 본 변경의 본질 정확, 코드 정독 회귀 없음 확인,
  PR #1026 정정 영역 보존, 영향 영역 한정 (미등록 폰트만)
- merge 방식: squash (외부 컨트리뷰터 일반 패턴) 또는 rebase
- close 후 archives: `mydocs/pr/archives/pr_1095_*.md`

## 10. 메모리 룰 정합

- `feedback_contributor_cycle_check` — planet6897 16+ 사이클, v1/v2 이력 확인
- `feedback_pr_supersede_chain` — v1 (#980 close+통합) + v2 (#1045 close 후 본 v3) 패턴 확인
- `feedback_pr_comment_tone` — 반복 컨트리뷰터, 차분한 사실 중심 close/merge 메시지
- `feedback_release_sync_check` — devel merge 전 origin/devel 동기화 확인
- `feedback_push_full_test_required` — lib + tests + clippy + fmt 모두 통과 (PR 작성자 검증 + 메인테이너 재검증)

## 11. 작업지시자 승인 요청

1. 본 검토 (merge 권장) 승인 여부
2. 검증 영역 (회귀 가드 1 + svg snapshot + lib + WASM) 권장 수용 여부
3. merge 방식 (squash 또는 rebase) 결정

# Task #528 Stage 3 — Composer 인프라 + Renderer 시점 변환 적용

**작성일**: 2026-05-02
**이슈**: [#528](https://github.com/edwardkim/rhwp/issues/528)
**브랜치**: `local/task528`

## 1. 결론

> Stage 2 매핑 표를 적용하여 PUA 옛한글 → KS X 1026-1:2007 자모 시퀀스 변환을 통합. **렌더러 시점 변환 (Option A)** 채택으로 char_offsets / char_start / line_chars 등 인덱싱 불변성 유지. exam_kor p17 측정 결과 PUA 112 → 68 (**44 PUA 옛한글 변환**), 신규 Hangul Jamo (U+1100-11FF) 102 출현. 잔존 68 PUA = 책괄호 (U+F0854/F0855/F00DA — 별도 issue). lib 1116 + svg_snapshot 6 + issue_418/501 회귀 0.

## 2. 설계 결정 — Option A vs B

### 옵션 비교

| | Option A (렌더러 시점 변환) | Option B (Composer IR 치환) |
|--|------|------|
| 변환 위치 | `svg.rs::draw_text` / `web_canvas.rs::draw_text` | `composer.rs::convert_pua_old_hangul` (run.text 치환) |
| char_offsets / char_start | 불변 (PUA = 1 char 인덱스 보존) | **인덱스 인플레이션 — 광범위 회귀 위험** |
| 다운스트림 영향 | 없음 (renderer 시점만) | paragraph_layout / table_layout / picture_footnote 등 다수 영향 |
| 회귀 위험 | 낮음 | 높음 |
| 폭 계산 | 기존 PUA 폭 유지 (font fallback 의존) | 변환 후 자모 폭 기반 (Task #122 cluster 활용) |

### 채택: Option A

이유:
- `r.text.chars().count()` 사용 위치가 12+ 모듈에 분산 (paragraph_layout / table_layout / picture_footnote / table_partial / cursor_nav 등) — Option B 채택 시 모든 위치 검증 + 회귀 위험 광범위
- PUA 폭이 fallback 으로 이미 ~1 char 폭으로 측정되고 있어 (현재 SVG 출력에서 PUA 가 자리를 차지함) 시각적 정합성 영향 최소
- Composer 단계 `convert_pua_old_hangul` 함수 + `display_text` 필드는 인프라로 보존 — Stage 4 의 폭 계산 정합 보강 시 활용 가능

## 3. 구현

### 3-1. `src/renderer/composer.rs` (인프라)

```rust
pub struct ComposedTextRun {
    // ... 기존 필드 ...
    /// PUA 옛한글 변환 후 표시 텍스트 (Some 이면 렌더러는 본 필드 사용).
    /// `text` 는 IR 와 동일하게 PUA char 1글자로 보존하여 char_offsets /
    /// char_start / line_chars 등 인덱싱 불변성을 유지한다 (Task #528).
    pub display_text: Option<String>,
}
```

`compose_paragraph` 마지막 단계에 `convert_pua_old_hangul(&mut composed)` 호출 추가:
- 각 run 의 text 에서 PUA 옛한글 char 검출 시 `display_text` 에 자모 시퀀스 저장
- run.text 자체는 보존

### 3-2. `src/renderer/svg.rs` (렌더러 적용)

`draw_text` 의 PUA bullet 변환 직후에 옛한글 확장 추가:

```rust
fn draw_text(&mut self, text: &str, x: f64, y: f64, style: &TextStyle) {
    // [Task #509] PUA bullet → 표시 문자열
    let text = &text.chars().map(map_pua_bullet_char).collect::<String>();
    // [Task #528] Hanyang-PUA 옛한글 → KS X 1026-1:2007 자모 시퀀스
    let text = &expand_pua_old_hangul(text);
    // ... 기존 SVG emit ...
}
```

`expand_pua_old_hangul` 헬퍼 함수: PUA 옛한글 char 를 자모 시퀀스로 확장. PUA 미발견 시 원본 그대로 반환 (early return).

### 3-3. `src/renderer/web_canvas.rs` (Canvas 렌더러 적용)

svg.rs 와 동일 패턴 — `draw_text` 에 `expand_pua_old_hangul_canvas` 적용.

### 3-4. ComposedTextRun 초기화 보강

`display_text: None,` 추가 — 16 위치 (composer.rs) + 6 위치 (composer/tests.rs) — perl 일괄 치환.

## 4. 검증

### 4-1. exam_kor p17 시각 검증

```bash
target/release/rhwp export-svg samples/exam_kor.hwp -p 16 -o /tmp/exam17_after
```

**SVG 출력 비교**:

| 항목 | Stage 2 이전 | Stage 3 후 |
|------|------------|----------|
| PUA chars | 112 | **68** (44 변환) |
| Hangul Jamo (U+1100-11FF) | 0 | **102** |
| 샘플 텍스트 | `<U+E1A7>` 등 (tofu) | `ᅌᅴ` (Old Hangul ng-yu) |

잔존 PUA 68 = U+F0854 (33 회) + U+F0855 (33 회) + U+F00DA (2 회) — 책괄호 / 괄호류 (옛한글 아님, 별도 issue 영역).

### 4-2. 광범위 회귀 검증

| 게이트 | 결과 |
|--------|------|
| `cargo test --lib` | **1116 passed**, 0 failed |
| `cargo test --test svg_snapshot` | 6/6 통과 |
| `cargo test --test issue_418` | 1/1 통과 |
| `cargo test --test issue_501` | 1/1 통과 |
| `cargo clippy --lib` | 2 errors (pre-existing in `table_ops.rs` / `object_ops.rs`, 본 task 미관여 영역) |

### 4-3. 영역 충돌 검증

- Task #509 (PUA bullet) 영역과 영역 분리 확인 — `map_pua_bullet_char` 가 `draw_text` 시작점에 먼저 적용되어 bullet PUA 우선. KTUG 매핑은 옛한글 영역 (U+E0BC-F8F7 일부) 만 다룸. 단위 테스트 `test_no_collision_with_pua_bullet_range` 통과
- `convert_pua_enclosed_numbers` (테두리 숫자) 영역과 겹침 없음 — 테두리 숫자는 별도 PUA 매핑

## 5. 산출물

| 파일 | 변경 영역 |
|------|----------|
| `src/renderer/composer.rs` | `ComposedTextRun.display_text` 필드 + `convert_pua_old_hangul` 함수 호출 (compose_paragraph 마지막) |
| `src/renderer/composer/tests.rs` | 6 위치 `display_text: None,` 추가 (필드 추가에 따른 초기화 정합) |
| `src/renderer/svg.rs` | `expand_pua_old_hangul` 헬퍼 + `draw_text` 적용 |
| `src/renderer/web_canvas.rs` | `expand_pua_old_hangul_canvas` 헬퍼 + `draw_text` 적용 |

## 6. 잔존 영역 (Stage 4 / 별도 issue)

### 6-1. 폭 계산 정합 (Stage 4)

현재 PUA 옛한글 char 의 폭은 폰트 fallback 기반 (대부분 ~1 char 폭). 변환 후 자모 클러스터 폭은 자체 계산. 두 폭이 정확히 일치하지 않으면 시각적으로 텍스트 간격 미세 차이 가능.

→ Stage 4 검증으로 영향 측정. 필요 시 폰트 메트릭 측 보강.

### 6-2. Supp PUA-A 영역 (별도 issue 권장)

- U+F0854 (33 회) — 책괄호 시작 추정 (`《` 또는 `〔`)
- U+F0855 (33 회) — 책괄호 끝
- U+F00DA (2 회) — 괄호류

→ 옛한글이 아닌 한컴 자체 기호 인코딩. Task #509 패턴으로 별도 매핑 필요. 본 task 미커버.

### 6-3. 폰트 fallback 합자 검증 (Stage 4)

KS X 1026-1 자모 시퀀스를 합자 (`ccmp/ljmo/vjmo/tjmo`) 렌더링하려면 OpenType 합자 피처 지원 폰트 필요:
- Noto Serif KR — 부분 지원 (검증 필요)
- Source Han Serif K — 완전 지원 (필요 시 도입)

→ Stage 4 에서 검증 후 결정.

## 7. 다음 단계

작업지시자 승인 후 Stage 4 (폰트 fallback 보강 + 합자 검증):

1. 변환 후 자모 시퀀스의 합자 렌더링 시각 확인 (브라우저 + SVG)
2. Noto Serif KR fallback 미흡 시 Source Han Serif K subset 도입 검토
3. 폭 계산 정합 영향 측정 (필요 시 보강)
4. `mydocs/tech/font_fallback_strategy.md` 갱신

## 8. 승인 게이트

- [x] Composer 인프라 (`display_text` 필드 + `convert_pua_old_hangul`)
- [x] Renderer 시점 변환 (svg + web_canvas)
- [x] exam_kor p17 PUA → Jamo 100% 변환 검증 (Old Hangul 영역)
- [x] lib 1116 / svg_snapshot 6 / issue_418/501 회귀 0
- [x] Task #509 / `convert_pua_enclosed_numbers` 영역 충돌 없음

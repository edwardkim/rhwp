# Task #528 구현계획서 v2 — 옛한글 PUA 변환 (HncPUAConverter 정합)

**작성일**: 2026-05-02 (v1: 폰트 fallback / **v2: PUA 변환 본질**)
**이슈**: [#528](https://github.com/edwardkim/rhwp/issues/528) (← #512 흡수)
**수행계획서**: [task_m100_528.md](task_m100_528.md) v2

## Stage 1 — 본질 발견 + 매핑 표 자료원 조사 (완료)

[Stage 1 보고서](../working/task_m100_528_stage1.md) — exam_kor p17 PUA 영역 본질 확정.

산출물:
- `mydocs/working/task_m100_528_stage1.md`
- 본 구현계획서 v2 + 수행계획서 v2

---

## Stage 2 — 매핑 표 확보 + 변환 함수 구현

**목표**: PUA → KS X 1026-1:2007 자모 시퀀스 매핑 표 코드화 + 변환 함수.

### 2-1. 오픈소스 자료원 조사

```bash
# (1) nuhwp / hwp.js / pyhwp / libhwp 등의 PUA 매핑 표 검색
mkdir -p /tmp/oss_pua_eval
cd /tmp/oss_pua_eval

# pyhwp
git clone --depth 1 https://github.com/mete0r/pyhwp.git 2>&1 | tail -1
grep -rn "PUA\|U+E\|0xE\|0xF8\|HncPUA\|pua_to" pyhwp/ 2>/dev/null | head -20

# hwp.js
git clone --depth 1 https://github.com/hahnlee/hwp.js.git 2>&1 | tail -1
grep -rn "PUA\|U+E\|0xE\|HncPUA" hwp.js/ 2>/dev/null | head -20

# 결과 → mydocs/tech/pua_oldhangul_mapping_sources.md 작성
```

**산출**: 발견 자료원 + 라이선스 정합 + 매핑 표 추출 가능성

### 2-2. KS X 1026-1:2007 표준 자료 조사

- 한국산업표준 ks 사이트 — KS X 1026-1:2007 부속서 확인 (유료)
- ISO/IEC 10646 (Unicode) Hangul Jamo 영역 표준 정합
- 한컴 매뉴얼 [hncpuaconverter.htm](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/hwp/Help/extracted/hwpbase/hncpuaconverter.htm) 본문 정독

### 2-3. `gen-pua` 도구 보강

기존 `src/main.rs::gen_pua_subcommand` (Task #509) 의 18 종 PUA 코드포인트에 본 task 의 25 종 추가:

```rust
// src/main.rs::gen_pua_subcommand
let pua_old_hangul_set: Vec<u32> = vec![
    // exam_kor p17 측정 영역 (Stage 1 보고서 §2-3)
    0xE17A, 0xE1A7, 0xE1C2, 0xE288, 0xE38A, 0xE40A,
    0xE474, 0xE560, 0xE566, 0xE79C, 0xE8A7, 0xE8B2,
    0xE95B, 0xEB66, 0xEB68, 0xEBD4, 0xECF0, 0xECFB,
    0xED41, 0xED98, 0xED9A, 0xF152, 0xF154, 0xF1C4, 0xF537,
    // Supplementary PUA-A
    0xF00DA, 0xF0854, 0xF0855,
];
```

신규 서브명령 `gen-pua-oldhangul` 또는 기존 `gen-pua` 의 옵션:

```bash
rhwp gen-pua-oldhangul -o samples/pua-oldhangul-test.hwp
# → 한컴 편집기에서 열어 PDF 캡처 → 정답지 영역
```

### 2-4. 매핑 표 코드화

위치 결정: 기존 `composer.rs` 와 분리하여 신규 모듈 권장:

```
src/renderer/pua_oldhangul.rs  (신규)
  - PUA_OLDHANGUL_MAP: phf::Map<char, &'static [char]>
  - map_pua_old_hangul(ch: char) -> Option<&'static [char]>
  - is_pua_old_hangul(ch: char) -> bool
```

매핑 표 형식:

```rust
// PUA → KS X 1026-1:2007 자모 시퀀스
pub static PUA_OLDHANGUL_MAP: phf::Map<u32, &'static [char]> = phf::phf_map! {
    0xE38Au32 => &['\u{1112}', '\u{1163}', '\u{11AB}'],  // 가설 예시 — 실제 매핑은 자료원 기반
    // ...
};
```

### 2-5. 단위 테스트

```rust
// tests/pua_oldhangul.rs
#[test]
fn test_exam_kor_p17_pua_codepoints_have_mappings() {
    let exam_kor_pua = [0xE17A, 0xE38A, 0xF537, 0xF00DA, 0xF0854, 0xF0855, /* ... 25종 */];
    for cp in exam_kor_pua {
        let ch = char::from_u32(cp).unwrap();
        assert!(map_pua_old_hangul(ch).is_some(), "PUA U+{:04X} 매핑 없음", cp);
    }
}

#[test]
fn test_no_collision_with_pua_bullet() {
    // Task #509 의 map_pua_bullet_char 영역과 겹치지 않는지 검증
    // ...
}
```

### 2-6. Stage 2 보고서

```
mydocs/working/task_m100_528_stage2.md
```

내용:
- 자료원 비교 (오픈소스 / KS 표준 / 폰트 cmap / gen-pua 검증)
- 채택 매핑 표 + 코드 변경 요약
- 단위 테스트 결과
- 한컴 PDF 비교 (작업지시자 시각 검증)

**승인 게이트**: Stage 2 보고서 → 승인 → Stage 3

---

## Stage 3 — Composer 단계 변환 적용

**목표**: 렌더 직전 Composer 단계에서 PUA 변환. 1:N 길이 변환의 LINE_SEG 영향 정합.

### 3-1. Composer 통합 위치

`src/renderer/composer.rs::compose_paragraph` 내부에서 텍스트 런 단계 처리:

```rust
// 기존 (예시)
fn compose_paragraph(...) -> ComposedParagraph {
    // ...
    convert_pua_enclosed_numbers(&mut composed);  // 기존 (Task #509 후속)
    // ...
}

// 추가
fn compose_paragraph(...) -> ComposedParagraph {
    // ...
    convert_pua_old_hangul(&mut composed);        // 신규 (본 task)
    convert_pua_enclosed_numbers(&mut composed);  // 기존
    // ...
}
```

`convert_pua_old_hangul`: 각 텍스트 런에서 PUA 옛한글 char 를 자모 시퀀스로 치환. 글리프 폭 / char_offsets 갱신.

### 3-2. 1:N 변환의 인덱스 영향

PUA 1 char → N 자모 char 변환 시:
- `char_offsets`: 자모 1개당 폭 vs 합자 후 폭 (Task #122 인프라 활용)
- `line_break_char_idx` (Task #518): PUA char 위치 → 변환 후 자모 클러스터 시작 위치
- `text_len`: 변경됨 → IR 보존 vs 변환 후 텍스트 분리

**설계 결정**:
- 옵션 A: 원본 PUA char 보존, 렌더 시점에만 자모로 합성 (display 분리)
- 옵션 B: Composer 단계에서 IR 자체를 자모 시퀀스로 치환 (text 분리)

옵션 A 추천 — char_offsets / line_break_char_idx 안정성. Task #122 의 자모 클러스터 처리는 `display_chars` 영역으로 정합.

### 3-3. 자모 클러스터 폭 계산

Task #122 의 `is_hangul()` / `is_cjk_char()` 가 이미 옛한글 자모 영역 (U+1100-11FF, U+A960-A97F, U+D7B0-D7FF) 처리. 폭 계산은 자모 클러스터 단위 (초성+중성+종성 = 1 음절 폭).

PUA char 의 폭은 변환 후 자모 클러스터의 합자 폭과 정합해야 함:
- 폰트가 합자 미지원 → 자모 개별 글리프 폭 합계 (visual artifact)
- 폰트가 합자 지원 → 단일 음절 글리프 폭

→ Stage 4 의 폰트 fallback 영향. 우선 자모 영역 폭으로 처리하고 Stage 4 에서 합자 폰트 보강.

### 3-4. 단위 테스트 + integration_tests

```rust
// src/renderer/composer.rs (단위)
#[test]
fn test_convert_pua_old_hangul_text_run() {
    let mut composed = /* ... PUA 옛한글 포함 paragraph ... */;
    convert_pua_old_hangul(&mut composed);
    // 자모 시퀀스로 치환됐는지 검증
}

// tests/issue_528.rs (integration)
#[test]
fn test_exam_kor_p17_old_hangul_renders() {
    let svg = render_exam_kor_p17();
    // 옛한글 영역 자모 시퀀스 출현 검증
}
```

### 3-5. Stage 3 보고서

```
mydocs/working/task_m100_528_stage3.md
```

내용:
- 옵션 A vs B 결정 + 사유
- LINE_SEG 영향 분석
- 단위 + integration 테스트 결과
- 7 샘플 회귀 byte 비교

**승인 게이트**: Stage 3 보고서 → 승인 → Stage 4

---

## Stage 4 — 폰트 fallback 보강

**목표**: 변환 후 자모 영역 (U+1100-11FF, U+A960-A97F, U+D7B0-D7FF) 의 합자 렌더링용 폰트 fallback 보강.

### 4-1. 합자 지원 폰트 검증

Stage 3 변환 후 결과를 시각 확인:

```bash
target/release/rhwp export-svg samples/exam_kor.hwp -p 16 -o /tmp/svg_check
# 브라우저에서 /tmp/svg_check/*.svg 열어 옛한글 영역 자모 합자 확인
```

- **Noto Serif KR** (기존 fallback 체인) — 합자 지원 여부 확인
  - 미지원 시: 본명조 (Source Han Serif K) Old Hangul subset 도입 (v1 영역 일부 재활용)
- **시스템 세리프** (AppleMyungjo 등) — 합자 미지원 → fallback 체인 우선순위 조정

### 4-2. (조건부) 본명조 subset 추출

Stage 1 의 v1 작업 정합:

```bash
pyftsubset SourceHanSerifKR-Regular.otf \
  --unicodes=U+1100-11FF,U+A960-A97F,U+D7B0-D7FF \
  --layout-features+=ccmp,ljmo,vjmo,tjmo \
  --output-file=SourceHanSerifK-OldHangul-subset.woff2 \
  --flavor=woff2

# rhwp-studio/public/fonts/ + 브라우저 확장에 동봉
# SourceHanSerifK-OFL.txt LICENSE 동봉
```

### 4-3. fallback 체인 보강

`src/renderer/style_resolver.rs::resolve_font_substitution` — Noto Serif KR 미지원 시 본명조 추가.

CSS:
```css
@font-face {
  font-family: "Source Han Serif K Old Hangul";
  src: url("/fonts/SourceHanSerifK-OldHangul-subset.woff2") format("woff2");
  font-display: swap;
  unicode-range: U+1100-11FF, U+A960-A97F, U+D7B0-D7FF;
}
```

### 4-4. Stage 4 보고서

```
mydocs/working/task_m100_528_stage4.md
```

**승인 게이트**: Stage 4 보고서 → 승인 → Stage 5

---

## Stage 5 — 광범위 회귀 + 시각 판정 + 최종 보고서

### 5-1. 광범위 회귀

```bash
# 단위 테스트
cargo test --lib 2>&1 | tail -5

# 골든 SVG
cargo test --test svg_snapshot 2>&1 | tail -5

# clippy
cargo clippy --all-targets 2>&1 | tail -3

# 7 샘플 byte 비교
./scripts/svg_regression_diff.sh build before
# (Stage 3 변경 후)
./scripts/svg_regression_diff.sh build after
./scripts/svg_regression_diff.sh diff before after

# PUA 광범위 사용 샘플 점검
# - synam-001 (PUA bullet — Task #509 영역)
# - mel-001, kps-ai (PUA Supplementary)
# - exam_kor (옛한글)
```

### 5-2. PUA 영역 충돌 검증

`map_pua_old_hangul` vs `map_pua_bullet_char` (Task #509) vs `convert_pua_enclosed_numbers` (테두리 숫자) — 영역 겹침 0 검증:

```rust
#[test]
fn test_pua_areas_disjoint() {
    for cp in 0xE000..=0xF8FF {
        let ch = char::from_u32(cp).unwrap();
        let is_old_hangul = is_pua_old_hangul(ch);
        let is_bullet = map_pua_bullet_char(ch).is_some();
        let is_enclosed_num = pua_enclosed_border_type(ch).is_some();
        let count = (is_old_hangul as u8) + (is_bullet as u8) + (is_enclosed_num as u8);
        assert!(count <= 1, "PUA U+{:04X} 영역 충돌", cp);
    }
}
```

### 5-3. 시각 판정

```
samples/2010-exam_kor.pdf  (한컴 2010 — 작업지시자 환경)
samples/2020-exam_kor.pdf  (한컴 2020)
samples/hancomdocs-exam_kor.pdf  (한컴독스 — 보조 ref)
```

p17 옛한글 표기 정합 시각 비교 — 작업지시자 ★ 판정.

### 5-4. WASM 빌드 검증

```bash
docker compose --env-file .env.docker run --rm wasm
ls -lh pkg/rhwp_bg.wasm  # 페이로드 크기 측정
ls -lh rhwp-studio/public/fonts/  # 폰트 자산 (Stage 4 도입 시)
```

### 5-5. 최종 보고서

```
mydocs/report/task_m100_528_report.md
```

내용:
- 본질 발견 (Stage 1) + 해결 영역 (Stage 2-4) 통합 정리
- 매핑 표 출처 + 라이선스
- Composer 통합 영역 + LINE_SEG 영향
- 폰트 fallback 영역 (조건부)
- 검증 게이트 통과 데이터
- 시각 판정 ★
- 향후 운영 — 새 PUA 코드 발견 시 매핑 표 확장 절차

### 5-6. 이슈 close

```bash
gh issue close 528 --repo edwardkim/rhwp --comment "Task #528 완료: PUA → KS X 1026-1:2007 자모 변환 + 폰트 fallback. 보고서: mydocs/report/task_m100_528_report.md"
gh issue close 512 --repo edwardkim/rhwp --comment "Task #528 흡수 완료 — PUA 옛한글 변환 구현. #528 정합."
```

---

## 전체 진행 순서

```
Stage 1 (완료) ─→ 작업지시자 승인 (현재)
                    ↓
Stage 2 (매핑 표 + 변환 함수)
  └─ gen-pua + 한컴 PDF 검증 → 보고서 → 승인
                    ↓
Stage 3 (Composer 통합)
  └─ LINE_SEG 영향 검증 → 보고서 → 승인
                    ↓
Stage 4 (폰트 fallback)
  └─ 합자 검증 → 보고서 → 승인
                    ↓
Stage 5 (회귀 + 시각 판정)
  └─ 최종 보고서 → 승인 → close
```

## 회귀 / 리스크 요약

| Stage | 회귀 위험 | 완화책 |
|-------|----------|--------|
| 1 | 0 (조사) | — |
| 2 | 0 (자료 조사 + 함수 추가) | 단위 테스트 + 한컴 PDF 비교 |
| **3** | **고** — Composer 변환 → 모든 렌더러 영향 | 7 샘플 byte 비교 + PUA 영역 충돌 검증 |
| 4 | 중 — fallback 체인 변경 | unicode-range 격리 + 회귀 검증 |
| 5 | 0 (검증) | — |

핵심 회귀: **Stage 3** — `convert_pua_old_hangul` 영역 격리 + Task #509 / `convert_pua_enclosed_numbers` 영역 분리가 핵심.

## 산출물 인덱스

| 파일 | Stage |
|------|-------|
| `mydocs/working/task_m100_528_stage{1,2,3,4}.md` | 각 단계별 |
| `mydocs/report/task_m100_528_report.md` | 5 |
| `mydocs/tech/pua_oldhangul_mapping_sources.md` (신규) | 2 |
| `src/renderer/pua_oldhangul.rs` (신규) | 2 |
| `src/renderer/composer.rs` (수정) | 3 |
| `src/main.rs` (gen-pua 보강) | 2 |
| `tests/pua_oldhangul.rs` (신규) | 2 |
| `tests/issue_528.rs` (신규) | 3 |
| `rhwp-studio/public/fonts/SourceHanSerifK-OldHangul-subset.woff2` (조건부) | 4 |
| `src/renderer/style_resolver.rs` (조건부 수정) | 4 |

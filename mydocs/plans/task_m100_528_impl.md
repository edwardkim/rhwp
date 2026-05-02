# Task #528 구현계획서 — 옛한글 글리프 폴백 지원

**작성일**: 2026-05-02
**이슈**: [#528](https://github.com/edwardkim/rhwp/issues/528)
**수행계획서**: `mydocs/plans/task_m100_528.md`

## Stage 1 — 폰트 후보 검증 + 결정

**목표**: exam_kor p17 옛한글 시퀀스를 본명조 (Source Han Serif K) 가 정상 합성하는지 사전 검증. 미커버 시 보조 폰트 (나눔명조 옛한글) 결합 검토.

### 1-1. exam_kor p17 옛한글 코드포인트 추출

```bash
cargo run --release --bin rhwp -- dump samples/exam_kor.hwp -s 1 -p 16 \
  > /tmp/exam_kor_p17_dump.txt

# 옛한글 영역 코드포인트 추출 (Python 스크립트)
python3 -c "
import re
with open('/tmp/exam_kor_p17_dump.txt') as f:
    text = f.read()
codepoints = set()
for ch in text:
    cp = ord(ch)
    if (0x1100 <= cp <= 0x11FF) or (0xA960 <= cp <= 0xA97F) or (0xD7B0 <= cp <= 0xD7FF):
        codepoints.add(cp)
for cp in sorted(codepoints):
    print(f'U+{cp:04X} {chr(cp)!r}')
" > /tmp/oldhangul_codepoints.txt
```

**산출물**: `/tmp/oldhangul_codepoints.txt` — exam_kor p17 에 등장하는 옛한글 코드포인트 + 빈도

### 1-2. 본명조 다운로드 + 합자 검증

```bash
mkdir -p /tmp/font_eval && cd /tmp/font_eval

# Source Han Serif K v2.003 (Adobe-Fonts/source-han-serif GitHub)
curl -L -o SourceHanSerif-K.zip \
  https://github.com/adobe-fonts/source-han-serif/releases/download/2.003R/01_SourceHanSerifKR.zip
unzip SourceHanSerifKR.zip

# 합자 피처 검증 (FontTools)
python3 -m pip install fonttools --user --quiet
python3 -c "
from fontTools.ttLib import TTFont
font = TTFont('SourceHanSerifKR-Regular.otf')
gsub = font['GSUB'].table
features = [f.FeatureTag for f in gsub.FeatureList.FeatureRecord]
print('GSUB features:', sorted(set(features)))
print('CCMP present:', 'ccmp' in features)
print('LJMO/VJMO/TJMO present:', any(t in features for t in ['ljmo','vjmo','tjmo']))
"
```

**검증 항목**:
- GSUB 테이블에 `ccmp` (Glyph Composition/Decomposition) 피처 존재
- 옛한글 합자 피처 `ljmo` (leading jamo), `vjmo` (vowel jamo), `tjmo` (trailing jamo) 또는 `nlc` (Non-finalized Letter) 존재
- exam_kor p17 의 모든 옛한글 코드포인트가 폰트 cmap 에 매핑

### 1-3. 비교 표 작성

| 폰트 | 라이선스 | GSUB 합자 | exam_kor 커버리지 | 파일 크기 (subset) |
|------|---------|-----------|-------------------|-------------------|
| 본명조 (Source Han Serif K) | SIL OFL 1.1 | ? | ? / N | ? KB |
| 나눔명조 옛한글 (대안) | SIL OFL 1.1 | ? | ? / N | ? KB |

**의사결정**:
- 본명조가 100% 커버 → 본명조 단독 채택
- 본명조 미커버 시 → 나눔명조 옛한글 보조 + unicode-range 분리

### 1-4. Stage 1 보고서

```
mydocs/working/task_m100_528_stage1.md
```

내용:
- 수집한 옛한글 코드포인트 목록 (빈도 포함)
- 본명조 검증 결과 (GSUB 피처 + cmap 커버리지)
- 시각 검증 (FontForge 또는 HarfBuzz 로 합성 결과 캡처)
- 폰트 결정 + 사유

**승인 게이트**: Stage 1 보고서 작성 → 작업지시자 승인 → Stage 2 진행

---

## Stage 2 — Subset 추출 + WASM 빌드 통합

**목표**: 결정된 폰트의 옛한글 영역만 추출하여 WASM 웹 빌드에 번들. `@font-face unicode-range` 로 lazy load.

### 2-1. Subset 추출

```bash
mkdir -p /tmp/font_subset && cd /tmp/font_subset

# pyftsubset 으로 옛한글 영역 + 합자 피처 보존
pyftsubset /tmp/font_eval/SourceHanSerifKR-Regular.otf \
  --unicodes=U+1100-11FF,U+A960-A97F,U+D7B0-D7FF \
  --layout-features+=ccmp,ljmo,vjmo,tjmo \
  --output-file=SourceHanSerifK-OldHangul-subset.woff2 \
  --flavor=woff2 \
  --no-hinting

ls -lh SourceHanSerifK-OldHangul-subset.woff2
```

**산출 검증**:
- 파일 크기 < 1MB
- `python3 -c "from fontTools.ttLib import TTFont; ..."` 로 GSUB 합자 보존 확인
- 브라우저에서 실제 합자 렌더링 시각 확인

### 2-2. WASM 빌드 통합

번들 위치 (작업지시자 결정 사항 정합):

```
rhwp-studio/public/fonts/
├── SourceHanSerifK-OldHangul-subset.woff2     # 본명조 subset
└── SourceHanSerifK-OFL.txt                     # SIL OFL 1.1 라이선스
```

브라우저 확장 (rhwp-chrome / rhwp-firefox) 빌드 시 동일 폰트 + LICENSE 동봉:

```
rhwp-chrome/dist/fonts/
rhwp-firefox/dist/fonts/
```

각 빌드 스크립트 (`rhwp-studio/vite.config.ts`, `rhwp-chrome/build.mjs`, `rhwp-firefox/build.mjs`) 의 정적 파일 복사 영역에 추가.

### 2-3. CSS @font-face 등록

`rhwp-studio/index.html` 또는 신규 `rhwp-studio/public/fonts/oldhangul.css`:

```css
@font-face {
  font-family: "Source Han Serif K Old Hangul";
  src: url("/fonts/SourceHanSerifK-OldHangul-subset.woff2") format("woff2");
  font-display: swap;
  unicode-range: U+1100-11FF, U+A960-A97F, U+D7B0-D7FF;
}
```

브라우저 확장의 content-script CSS 에도 동일 등록.

### 2-4. Stage 2 보고서

```
mydocs/working/task_m100_528_stage2.md
```

내용:
- subset 파일 크기 측정값
- WASM 빌드 페이로드 영향
- 브라우저에서 합자 렌더링 시각 캡처 (작업지시자 검증용)

**승인 게이트**: Stage 2 보고서 → 승인 → Stage 3

---

## Stage 3 — fallback 체인 보강

**목표**: `style_resolver.rs::resolve_font_substitution` 의 한컴바탕/함초롬바탕 fallback 체인 말단에 본명조 추가. CSS unicode-range 가 옛한글 영역에서만 발동하도록 단일 진실 원천 구성.

### 3-1. 코드 변경 — `src/renderer/style_resolver.rs`

**변경 영역**: `resolve_font_substitution` 함수 (현재 라인 565-600 근방)

**변경 내용**:
- `lookup_font_name` 의 반환 폰트 이름 체인에 "Source Han Serif K Old Hangul" 추가
- CSS 에서 `font-family: "한컴바탕", ..., "Noto Serif KR", "Source Han Serif K Old Hangul", serif` 형태로 사용되도록 출력 갱신

**구체 변경 (예시 — Stage 1 결과 후 확정)**:

```rust
// resolve_font_substitution 내부
"한컴바탕" | "함초롬바탕" => Some("함초롬바탕"),  // 기존
// 후처리: 출력 SVG/Canvas 에서 폰트 체인을 다단계로 emit
```

실제 변경 위치는 SVG/Canvas 출력 측에서 `font-family` 체인을 확장하는 영역. Stage 1 의 코드 조사로 확정.

### 3-2. 단위 테스트

`src/renderer/style_resolver.rs` 의 기존 테스트 패턴 정합:

```rust
#[test]
fn test_oldhangul_fallback_chain() {
    // 옛한글 영역 코드포인트가 본명조 fallback 으로 이어지는지 검증
    let result = resolve_font_substitution("한컴바탕", 0, 0);
    assert!(result.is_some());
    // ...
}
```

### 3-3. 영향 범위 검증

```bash
# 변경 전 7 샘플 byte 비교
./scripts/svg_regression_diff.sh build before

# 변경 후
./scripts/svg_regression_diff.sh build after
./scripts/svg_regression_diff.sh diff before after
```

**기대 결과**: 변경 byte 0 (옛한글 영역 외 영향 없음). 변경 발생 시 Stage 4 회귀 분석.

### 3-4. Stage 3 보고서

```
mydocs/working/task_m100_528_stage3.md
```

**승인 게이트**: 코드 변경 + 회귀 0 확인 → 승인 → Stage 4

---

## Stage 4 — 문서화 + 회귀 검증

**목표**: 본 작업의 디자인 결정과 운영 절차를 영구 문서화. 광범위 회귀 검증.

### 4-1. `mydocs/tech/font_fallback_strategy.md` 갱신

신규 섹션 "8. 옛한글 fallback 전략":

- 본명조 채택 사유 (라이선스 + 합자 지원 + 검증 이력)
- 번들 위치 + subset 절차 (재생성 방법)
- `@font-face unicode-range` 작동 원리
- Canvas 2D 의 unicode-range 한계 (FontFace API 보강 필요 시)

### 4-2. `ttfs/FONTS.md` 갱신

신규 섹션 "OFL 폰트 (웹 빌드 번들)":

- 본 폰트는 `ttfs/` 정책 (Git 미포함) 의 예외
- 웹 빌드 한정 번들 위치 (`rhwp-studio/public/fonts/`, `rhwp-{chrome,firefox}/public/fonts/`)
- 라이선스 동봉 의무 (OFL 4조)

### 4-3. 광범위 회귀 검증

```bash
# 단위 테스트
cargo test --lib 2>&1 | tail -5

# 골든 SVG
cargo test --test svg_snapshot 2>&1 | tail -5

# clippy
cargo clippy --all-targets 2>&1 | tail -3

# 7 샘플 byte 비교
./scripts/svg_regression_diff.sh diff before after

# WASM 빌드 페이로드 측정
docker compose --env-file .env.docker run --rm wasm
ls -lh pkg/rhwp_bg.wasm
ls -lh rhwp-studio/public/fonts/
```

### 4-4. Stage 4 보고서

```
mydocs/working/task_m100_528_stage4.md
```

**승인 게이트**: 모든 검증 통과 → 승인 → Stage 5

---

## Stage 5 — 시각 판정 + 최종 보고서

**목표**: 작업지시자 시각 판정 + 최종 보고서.

### 5-1. 로컬 시각 판정

```bash
# 네이티브 SVG (--font-path 없이도 본명조 폰트 체인 작동 확인)
cargo run --release --bin rhwp -- export-svg samples/exam_kor.hwp -p 16 -o /tmp/exam_kor_p17

# 브라우저 (rhwp-studio)
cd rhwp-studio
npx vite --host 0.0.0.0 --port 7700 &
# 브라우저에서 samples/exam_kor.hwp p17 로딩
```

### 5-2. PDF 비교

```
samples/2010-exam_kor.pdf  (한컴 2010 — 작업지시자 환경)
samples/2020-exam_kor.pdf  (한컴 2020 — 작업지시자 환경)
samples/hancomdocs-exam_kor.pdf  (한컴독스 — 보조 ref)
```

p17 의 옛한글 표기 (`'다'(되다)`, `'(혼자)'`, `'​​'` 등) 가 PDF 와 시각적으로 정합하는지 작업지시자가 직접 비교.

**메모리 정합** (`feedback_pdf_not_authoritative`):
- 한컴 2010 / 2020 / 한컴독스 비교
- 작업지시자 직접 시각 판정 게이트

### 5-3. 최종 보고서

```
mydocs/report/task_m100_528_report.md
```

내용:
- 본질 정리
- 정정 요약 (코드 + 폰트 + CSS 변경)
- 검증 게이트 통과 데이터
- 시각 판정 결과 ★
- 잔존 결함 (있을 경우)
- 향후 운영 (subset 재생성 절차 등)

**승인 게이트**: 작업지시자 최종 승인 → 이슈 #528 close + orders 갱신 + local/devel merge

---

## 전체 진행 순서 (요약)

```
Stage 1 (폰트 검증)        ─┐
  └─ 보고서 → 승인 ────────│
                            ↓
Stage 2 (subset + 번들)    ─┐
  └─ 보고서 → 승인 ────────│
                            ↓
Stage 3 (fallback 체인)    ─┐
  └─ 보고서 → 승인 ────────│
                            ↓
Stage 4 (문서 + 회귀)      ─┐
  └─ 보고서 → 승인 ────────│
                            ↓
Stage 5 (시각 판정)        ─┐
  └─ 최종 보고서 → 승인 ──│
                            ↓
                      이슈 close + merge
```

## 회귀 / 리스크 요약 (수행계획서 갱신)

| 단계 | 회귀 위험 | 완화책 |
|------|----------|--------|
| Stage 1 | 0 (조사만) | — |
| Stage 2 | 0 (정적 자산 추가만) | WASM 페이로드 측정 |
| Stage 3 | **중간** — fallback 체인 변경 | 7 샘플 byte 비교, 단위 테스트, unicode-range 격리 |
| Stage 4 | 0 (문서) | — |
| Stage 5 | 0 (검증) | — |

핵심 회귀 영역: Stage 3. unicode-range 분리로 일반 한글 영역 영향 차단이 핵심.

## 산출물 인덱스

| 파일 | Stage |
|------|-------|
| `mydocs/working/task_m100_528_stage{1,2,3,4}.md` | 각 단계별 |
| `mydocs/report/task_m100_528_report.md` | Stage 5 |
| `rhwp-studio/public/fonts/SourceHanSerifK-OldHangul-subset.woff2` | Stage 2 |
| `rhwp-studio/public/fonts/SourceHanSerifK-OFL.txt` | Stage 2 |
| `rhwp-{chrome,firefox}/public/fonts/...` | Stage 2 |
| `src/renderer/style_resolver.rs` (수정) | Stage 3 |
| `mydocs/tech/font_fallback_strategy.md` (수정) | Stage 4 |
| `ttfs/FONTS.md` (수정) | Stage 4 |

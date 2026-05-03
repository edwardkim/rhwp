# Task #528 Stage 4 — 폰트 fallback 보강 + 합자 검증

**작성일**: 2026-05-02
**이슈**: [#528](https://github.com/edwardkim/rhwp/issues/528)
**브랜치**: `local/task528`

## 1. 결론

> Source Han Serif K Old Hangul subset (Adobe + Google, **SIL OFL 1.1**, **234 KB woff2**) 를 `rhwp-studio/public/fonts/` 에 도입. CSS `unicode-range` 격리로 일반 한글 영향 0. font-family 체인 말단에 등록. 357/368 옛한글 자모 + 4 합자 피처 (ccmp/ljmo/vjmo/tjmo) 모두 보존.

## 2. 시스템 폰트 합자 지원 검증

### 2-1. 측정 (작업지시자 macOS 환경)

```python
# fontTools 로 GSUB 피처 + cmap 검증
fonts_to_check = [
    NotoSerifKR-Regular.woff2 (rhwp-studio 기존 fallback),
    NanumMyeongjo (시스템 기본 한글 명조),
    AppleMyungjo (시스템 기본),
    Pretendard (rhwp-studio 기존),
]
```

### 2-2. 결과

| 폰트 | ccmp | ljmo | vjmo | tjmo | 옛한글 cmap |
|------|------|------|------|------|------------|
| NotoSerifKR-Regular | ✅ | ❌ | ✅ | ❌ | **0 / 6 test cps** |
| NanumMyeongjo | ❌ | ❌ | ❌ | ❌ | **0 / 6 test cps** |
| Pretendard | ❌ | ❌ | ❌ | ❌ | 0 / 6 |
| AppleMyungjo | ❌ | ❌ | ❌ | ❌ | 0 / 6 |

→ **현재 fallback 체인의 어떤 폰트도 옛한글 자모를 합자 렌더링 불가**. PUA 변환 후 자모 시퀀스를 그대로 두면 브라우저는 tofu (.notdef) 표시.

→ Source Han Serif K (또는 동등 옛한글 지원 폰트) **번들 필수** 확정.

## 3. Source Han Serif K 채택

### 3-1. 검증

```python
font = TTFont('SourceHanSerifK-Regular.otf')
# GSUB features: ccmp + ljmo + vjmo + tjmo 모두 ✅
# Old Hangul cmap 357/368 (Hangul Jamo + Ext-A/B 영역)
# KTUG 매핑 target jamo 357/357 (100%)
```

### 3-2. 라이선스

- **Adobe + Google 공동 저작**, **SIL OFL 1.1**
- 출처: https://github.com/adobe-fonts/source-han-serif
- 라이선스 텍스트 동봉: `rhwp-studio/public/fonts/SourceHanSerifK-OFL.txt`

### 3-3. Subset 추출

```bash
pyftsubset SourceHanSerifK-Regular.otf \
    --unicodes='U+1100-11FF,U+A960-A97F,U+D7B0-D7FF' \
    --layout-features='*' \
    --output-file=SourceHanSerifK-OldHangul-subset.woff2 \
    --flavor=woff2 --no-hinting
```

| 항목 | 값 |
|------|-----|
| 원본 | 23 MB (`SourceHanSerifK-Regular.otf`) |
| **Subset** | **234 KB woff2** |
| 자모 cmap | 357/368 (옛한글 영역) |
| GSUB 합자 피처 | ccmp + ljmo + vjmo + tjmo (4/4 보존) |

`--layout-features='*'` 가 **결정적** — 처음 시도한 `+ccmp,+ljmo,...` 는 합자 피처가 모두 stripped 됨 (subset 41 KB 였으나 합자 작동 불가).

## 4. 통합 적용

### 4-1. 폰트 자산 (`rhwp-studio/public/fonts/`)

```
SourceHanSerifK-OldHangul-subset.woff2  (234 KB)
SourceHanSerifK-OFL.txt                  (4.4 KB)
```

### 4-2. CSS @font-face 등록 (`rhwp-studio/src/core/font-loader.ts`)

```typescript
// FontEntry 타입 확장
interface FontEntry {
  name: string;
  file: string;
  format?: 'woff2' | 'woff';
  unicodeRange?: string;  // 신규
}

// FONT_LIST 신규 엔트리
{
  name: 'Source Han Serif K Old Hangul',
  file: 'fonts/SourceHanSerifK-OldHangul-subset.woff2',
  unicodeRange: 'U+1100-11FF, U+A960-A97F, U+D7B0-D7FF',
}

// CSS template 갱신
@font-face {
  font-family: "Source Han Serif K Old Hangul";
  src: url("fonts/SourceHanSerifK-OldHangul-subset.woff2") format("woff2");
  font-display: swap;
  unicode-range: U+1100-11FF, U+A960-A97F, U+D7B0-D7FF;
}
```

**핵심**: `unicode-range` 가 옛한글 영역에서만 매칭 → **일반 한글 (U+AC00-D7AF) 영향 0** + 옛한글 영역 사용 시에만 다운로드 발생.

### 4-3. font-family 체인 보강 (`src/renderer/mod.rs::generic_fallback`)

세 위치에 `'Source Han Serif K Old Hangul'` 추가:

| 카테고리 | 체인 |
|----------|------|
| 한글 serif | `... 'Noto Serif KR', 'Noto Serif CJK KR', 'Source Han Serif K Old Hangul', serif` |
| 영문 serif | (동일 한글 serif 체인 사용) |
| 한글 sans-serif (기본) | `... 'Pretendard', 'Source Han Serif K Old Hangul', sans-serif` |

## 5. 검증

### 5-1. 단위 테스트

```
running 1116 tests
test result: ok. 1116 passed; 0 failed; 1 ignored
```

`test_generic_fallback` 도 갱신 (예상 체인에 'Source Han Serif K Old Hangul' 포함).

### 5-2. SVG 회귀 (svg_snapshot)

| 게이트 | 결과 |
|--------|------|
| **변경 전** | 5/6 FAIL (font-family 체인 변경) |
| **변경 후** | **6/6 PASS** (golden 갱신, font-family 체인 추가만 차이) |

골든 SVG 갱신: 변경 영역은 **순수 font-family 체인 추가만** — 좌표 / 색 / 글자 변동 0. `git diff` 가 1840 lines insertions / 1840 deletions (추가 토큰 1개) 로 cosmetic 변경.

### 5-3. issue_418 / issue_501

| 게이트 | 결과 |
|--------|------|
| issue_418 | 1/1 PASS |
| issue_501 | 1/1 PASS |

### 5-4. exam_kor p17 SVG 출력

```html
<text font-family="HY신명조,'Batang','바탕','Nanum Myeongjo','AppleMyungjo','Noto Serif KR','Noto Serif CJK KR','Source Han Serif K Old Hangul',serif" ...>ᅌᅴ</text>
```

체인 말단에 `Source Han Serif K Old Hangul` 추가 확인. 브라우저는 `unicode-range` 매칭으로 옛한글 영역에서 본 폰트 사용.

## 6. 잔존 영역 / 한계

### 6-1. 단독 SVG (rhwp export-svg) 영역

체인에 폰트 이름은 추가되지만 **@font-face 정의는 SVG 내장 안 됨**. 단독 SVG 를 브라우저에서 열면:
- OS 에 'Source Han Serif K Old Hangul' 미설치 → fallback 'serif' 로 떨어짐 → tofu

→ **단독 SVG 사용자는 `--font-style` / `--embed-fonts` 옵션 사용 필요**. 현재 구현은 web 빌드 (rhwp-studio) 기준 정합.

→ 향후 보강: `--embed-fonts` 시 본 subset 도 자동 임베딩. Stage 5 또는 별도 issue.

### 6-2. Supp PUA-A 영역

U+F0854 / F0855 (책괄호 33회 each) + U+F00DA — 옛한글 아님. 본 task 미커버. 별도 issue 권장.

### 6-3. 폭 계산 정합

PUA char 폭 (현재 fallback 으로 ~font_size) vs 자모 cluster 합자 폭 (Source Han Serif K 메트릭 기반) 의 미세 차이 가능. 시각 판정 시 측정 필요. 현재 회귀 테스트는 통과 (좌표 변동 0).

## 7. 산출물

| 파일 | 변경 |
|------|------|
| `rhwp-studio/public/fonts/SourceHanSerifK-OldHangul-subset.woff2` | 신규 (234 KB) |
| `rhwp-studio/public/fonts/SourceHanSerifK-OFL.txt` | 신규 (라이선스, 4.4 KB) |
| `rhwp-studio/src/core/font-loader.ts` | FontEntry.unicodeRange 추가 + Source Han Serif K Old Hangul 엔트리 + CSS template |
| `src/renderer/mod.rs` | 3 위치 font-family 체인에 'Source Han Serif K Old Hangul' 추가 |
| `tests/golden_svg/*` | 5 파일 골든 갱신 (font-family 체인 추가만, cosmetic) |
| `mydocs/tech/font_fallback_strategy.md` | "10. 옛한글 fallback (Task #528)" 섹션 추가 |
| `mydocs/working/task_m100_528_stage4.md` | (현재) Stage 4 보고서 |

## 8. 다음 단계

작업지시자 시각 판정 (Stage 5):

1. rhwp-studio 로컬 서버 실행 → 브라우저에서 `samples/exam_kor.hwp` p17 로딩
2. 옛한글 표기 (`'다'`, `'(혼자)'` 등) 정상 표시 확인
3. 한컴 PDF 와 시각 비교
4. 한컴 2010 / 2020 정답지 비교 (메모리 `feedback_pdf_not_authoritative`)

`mydocs/report/task_m100_528_report.md` 최종 보고서 작성 후 close.

## 9. 승인 게이트

- [x] 시스템 폰트 합자 지원 부재 측정 (Source Han Serif K 도입 필수 확인)
- [x] Source Han Serif K Old Hangul subset 추출 (234 KB, 합자 피처 보존)
- [x] 라이선스 (SIL OFL 1.1) 동봉
- [x] CSS unicode-range 격리 (일반 한글 미영향)
- [x] font-family 체인 보강
- [x] cargo test --lib 1116 / svg_snapshot 6 / issue_418/501 통과
- [x] 골든 SVG 갱신 (font-family 추가만, cosmetic)
- [ ] **작업지시자 시각 판정** (Stage 5)

---

## 부록 — Stage 4 hotfix (책괄호 + 예시 마커)

### 본질

작업지시자 시각 검증 결과 **Supplementary PUA-A 영역의 한컴 자체 기호** (옛한글 영역 외) 도 정정 필요 확인:

| 코드포인트 | 빈도 (exam_kor p17) | 본질 |
|----------|------|------|
| **U+F0854** | 33회 | 책괄호 시작 (`《`) — 용비어천가 등 책 제목 둘러싸기 |
| **U+F0855** | 33회 | 책괄호 끝 (`》`) |
| **U+F00DA** | 2회 | 예시 마커 — `(F00DA 단풍 철 : 철 성분)` 패턴 |

원래 Stage 3 보고서에 "별도 issue 영역" 으로 명시했지만 사용자 시각에서는 같은 PUA 미렌더 결함으로 보임 → Task #528 범위에 흡수.

### 정정 위치

`src/renderer/layout/paragraph_layout.rs::map_pua_bullet_char` 의 Supplementary PUA-A 핸들러 확장:

```rust
// Supplementary PUA-A — 한컴 책괄호 / 예시 마커 (Task #528 exam_kor p17)
if (0xF00D0..=0xF09FF).contains(&code) {
    return match code {
        0xF0854 => '\u{300A}', // 《 LEFT DOUBLE ANGLE BRACKET
        0xF0855 => '\u{300B}', // 》 RIGHT DOUBLE ANGLE BRACKET
        0xF00DA => '\u{25B8}', // ▸ BLACK SMALL TRIANGLE (잠정, 시각 판정 필요)
        _ => ch,
    };
}
```

### 검증

| 항목 | 결과 |
|------|------|
| PUA 잔존 (exam_kor p17) | **0** (이전 68 → 0) ✅ |
| `《 》` 출현 | 66 (33+33) |
| `▸` 출현 | 2 |
| `cargo test --lib` | 1116 passed |
| `svg_snapshot` | 6/6 |

### 잔존 시각 검증 영역

`U+F00DA → ▸` 는 **잠정 매핑**. 한컴 PDF 와 시각 비교 후 정정 필요 (Stage 5).

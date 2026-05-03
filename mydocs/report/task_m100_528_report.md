# Task #528 최종 보고서 — 옛한글 PUA → KS X 1026-1 자모 변환 + 폰트 fallback

**완료일**: 2026-05-02
**이슈**: [#528](https://github.com/edwardkim/rhwp/issues/528) (← #512 흡수)
**브랜치**: `local/task528`
**Stage**: 5/5 완료 (작업지시자 시각 판정 ★)

## 1. 결론

> `samples/exam_kor.hwp` p17 의 중세국어 표기 미렌더 결함을 해결. 본질은 v1 가설 (Hangul Jamo Extended-A/B 폰트 fallback) 이 아닌 **HanCom Hanyang-PUA 인코딩** 이었음 (Stage 1 측정으로 확정). KTUG HanyangPuaTableProject (Public Domain) 매핑 표 + Source Han Serif K Old Hangul subset (SIL OFL 1.1) 폰트 + 한컴 책괄호 / 예시 마커 매핑 통합으로 PUA 112 → 0 (100%) 정상 렌더링 달성. **작업지시자 시각 판정 ★**.

## 2. 본질 발견 (Stage 1)

### 2-1. 가설 정정

| | v1 (초기 가설) | v2 (실측) |
|--|------|------|
| 본질 | Hangul Jamo Extended-A/B 폰트 fallback | **HanCom Hanyang-PUA 인코딩** |
| 영역 | U+1100-11FF, U+A960-A97F, U+D7B0-D7FF | **U+E0BC-F8F7 (BMP PUA) + U+F0000+ (Supp PUA-A)** |
| 측정 데이터 | (없음) | exam_kor p17 textcontent 분석 |

### 2-2. exam_kor p17 측정

`rhwp export-text -p 16` 결과:

| 영역 | 발견 |
|------|------|
| Hangul Jamo (U+1100-11FF) | 0 (가설 영역, 부재) |
| Hangul Jamo Extended-A/B | 0 (가설 영역, 부재) |
| **Basic PUA (U+E000-F8FF)** | **22 unique, 50 회** ★ |
| **Supplementary PUA-A** | **3 unique (U+F00DA, F0854, F0855), 68 회** ★ |

→ 한컴 자체 PUA 인코딩이 본질. Issue #512 가 정확히 이 영역 — 본 task 가 #512 흡수로 피벗.

## 3. 정정 영역

### 3-1. 매핑 표 (Stage 2)

**자료원**: KTUG HanyangPuaTableProject (Public Domain — 다년 커뮤니티 검증, 함초롬바탕 정합)

- 5,660 매핑 (U+E0BC ~ U+F8F7 BMP PUA)
- 출력: Hangul Jamo (U+1100-11FF) + Extended-A (U+A960-A97F) + Extended-B (U+D7B0-D7FF)
- 자동 생성 스크립트: `scripts/gen_pua_oldhangul_rs.py`
- exam_kor p17 BMP PUA: **25/25 (100%) 커버**

### 3-2. 변환 함수 (Stage 2-3)

**위치**: `src/renderer/pua_oldhangul.rs` (자동 생성, 5,773 라인)

```rust
pub fn map_pua_old_hangul(ch: char) -> Option<&'static [char]>
pub fn is_pua_old_hangul(ch: char) -> bool
```

**적용 위치 (Stage 3 — Option A 렌더러 시점 변환)**:
- `src/renderer/svg.rs::draw_text` 내부 `expand_pua_old_hangul`
- `src/renderer/web_canvas.rs::draw_text` 내부 `expand_pua_old_hangul_canvas`
- `src/renderer/composer.rs` 의 `display_text` 필드 + `convert_pua_old_hangul` 함수 (Stage 4 영역 인프라 보존)

**옵션 A 채택 사유**: char_offsets / char_start / line_chars 인덱싱 불변성 유지. `r.text.chars().count()` 사용 위치 12+ 모듈 영향 차단.

### 3-3. 폰트 fallback (Stage 4)

**시스템 폰트 측정 결과** (작업지시자 macOS 환경):

| 폰트 | ccmp | ljmo | tjmo | 옛한글 cmap |
|------|------|------|------|------------|
| NotoSerifKR-Regular | ✅ | ❌ | ❌ | 0/6 |
| NanumMyeongjo | ❌ | ❌ | ❌ | 0/6 |
| Pretendard | ❌ | ❌ | ❌ | 0/6 |
| AppleMyungjo | ❌ | ❌ | ❌ | 0/6 |

→ 어떤 시스템 폰트도 옛한글 합자 렌더링 불가. **Source Han Serif K 번들 필수**.

**채택**: Source Han Serif K (Adobe + Google, **SIL OFL 1.1**) Old Hangul subset

| 항목 | 값 |
|------|-----|
| 원본 | 23 MB (`SourceHanSerifK-Regular.otf`) |
| **Subset (woff2)** | **234 KB** |
| 자모 cmap | 357/368 (KTUG target 357/357 100%) |
| GSUB 합자 | ccmp + ljmo + vjmo + tjmo (4/4) |

**Subset 추출 인사이트**: `pyftsubset --layout-features='*'` 가 결정적 — 특정 피처 지정 시 (`+ccmp,+ljmo,...`) 합자 피처 모두 stripped (subset 41 KB 였으나 합자 작동 불가).

**unicode-range 격리**:
```css
@font-face {
  font-family: "Source Han Serif K Old Hangul";
  unicode-range: U+1100-11FF, U+A960-A97F, U+D7B0-D7FF;
}
```

→ 일반 한글 (U+AC00-D7AF) 미영향. 옛한글 영역 사용 시에만 다운로드.

### 3-4. 한컴 책괄호 + 예시 마커 (Stage 4 hotfix)

작업지시자 시각 검증으로 잔존 PUA 추가 정정:

| 코드포인트 | 빈도 | 매핑 | 본질 |
|----------|------|------|------|
| **U+F0854** | 33회 | `《` (U+300A) | 책괄호 시작 |
| **U+F0855** | 33회 | `》` (U+300B) | 책괄호 끝 |
| **U+F00DA** | 2회 | `▸` (U+25B8) | 예시 마커 (잠정) |

**위치**: `src/renderer/layout/paragraph_layout.rs::map_pua_bullet_char` (Task #509 패턴 정합)

원래 Stage 3 보고서에 "별도 issue 영역" 으로 명시했지만 사용자 시각에서는 같은 PUA 미렌더 결함 → 본 task 범위에 흡수.

## 4. 검증 게이트 통과

| 게이트 | 결과 |
|--------|------|
| `cargo test --lib` | **1116 passed** |
| `cargo test --test svg_snapshot` | 6/6 PASS (golden 갱신 — font-family chain 추가만, cosmetic) |
| `cargo test --test issue_418` | 1/1 PASS |
| `cargo test --test issue_501` | 1/1 PASS |
| **exam_kor p17 PUA 잔존** | **0** (이전 112 → 0, 100%) ★ |
| **작업지시자 시각 판정** | **★ 정상 표시 확인** |

### 4-1. 변환 결과 (exam_kor p17)

| 영역 | Before | After |
|------|--------|-------|
| PUA chars (미렌더) | 112 | **0** |
| Hangul Jamo (U+1100-11FF) | 0 | 102 |
| 책괄호 `《 》` | 0 | 66 |
| 예시 마커 `▸` | 0 | 2 |

### 4-2. 시각 변환 예시

```
원본 IR (PUA)              변환 후 (자모 + 합자)
────────────────────────────────────────────────
'다'(되다)             →   '다'(되다)        (정상 표시)
'(혼자)'              →   '(혼자)'         (정상 표시)
'스스​'                →   '스스'           (정상 표시)
󰡔용비어천가󰡕         →   《용비어천가》    (책괄호 정합)
(󰃚 단풍 철 :)        →   (▸ 단풍 철 :)    (예시 마커, 잠정)
```

## 5. 산출물 인덱스

### 5-1. 코드

| 파일 | 변경 영역 |
|------|----------|
| `src/renderer/pua_oldhangul.rs` (신규, 자동 생성) | 5,660 KTUG 매핑 + API |
| `src/renderer/composer.rs` | `ComposedTextRun.display_text` 필드 + `convert_pua_old_hangul` 함수 |
| `src/renderer/composer/tests.rs` | 6 위치 `display_text: None` 정합 |
| `src/renderer/svg.rs` | `expand_pua_old_hangul` + `draw_text` 적용 |
| `src/renderer/web_canvas.rs` | `expand_pua_old_hangul_canvas` + `draw_text` 적용 |
| `src/renderer/mod.rs` | `generic_fallback` 3 위치 chain 보강 |
| `src/renderer/layout/paragraph_layout.rs` | `map_pua_bullet_char` 책괄호 + 예시 마커 |
| `src/renderer/mod.rs` (test) | `test_generic_fallback` 갱신 |

### 5-2. 자료

| 파일 | 용도 |
|------|------|
| `web/fonts/SourceHanSerifK-OldHangul-subset.woff2` | 폰트 subset (234 KB) |
| `web/fonts/SourceHanSerifK-OFL.txt` | 라이선스 동봉 (4.4 KB) |
| `rhwp-studio/src/core/font-loader.ts` | FontEntry + FONT_LIST 등록 |
| `tests/golden_svg/*` | 5 파일 갱신 (cosmetic — font-family chain) |

### 5-3. 도구 / 스크립트

| 파일 | 용도 |
|------|------|
| `scripts/gen_pua_oldhangul_rs.py` | KTUG 데이터 → Rust 모듈 자동 생성 |

### 5-4. 문서

| 파일 | 용도 |
|------|------|
| `mydocs/plans/task_m100_528.md` | 수행계획서 v2 |
| `mydocs/plans/task_m100_528_impl.md` | 구현계획서 v2 |
| `mydocs/working/task_m100_528_stage{1,2,3,4}.md` | 단계별 보고서 |
| `mydocs/report/task_m100_528_report.md` | (현재) 최종 보고서 |
| `mydocs/tech/pua_oldhangul_mapping_sources.md` | 자료원 정리 |
| `mydocs/tech/font_fallback_strategy.md` | "10. 옛한글 fallback" 섹션 추가 |

## 6. 커밋 이력

```
0687cfc Task #528 Stage 4 hotfix: 한컴 책괄호 + 예시 마커 PUA 매핑 추가
e37acdc Task #528 Stage 4: Source Han Serif K Old Hangul subset 도입 (234 KB woff2, OFL)
a15847c Task #528 Stage 3: PUA 옛한글 → KS X 1026-1 자모 변환 적용 (Option A — 렌더러 시점)
c3f6a95 Task #528 Stage 2: KTUG Hanyang-PUA 매핑 표 + 변환 함수 (5660 entries, exam_kor 100%)
532c9b3 Task #528 Stage 1: 본질 발견 + 수행/구현계획서 v2 (PUA 변환으로 피벗)
33351e1 Task #528 구현계획서: Stage 1-5 세부 절차
ef33a7a Task #528 수행계획서: 옛한글 글리프 폴백 지원 (Source Han Serif K subset 번들)
```

## 7. 메모리 정합 / 가이드

| 메모리 | 적용 |
|--------|------|
| `feedback_pdf_not_authoritative` | 한컴 2010/2020/한컴독스 PDF 비교 + 작업지시자 시각 판정 게이트 ★ |
| `feedback_essential_fix_regression_risk` | Composer 영역 변경 → 광범위 회귀 검증 (1116 + 6 + 2 통과) |
| `feedback_rule_not_heuristic` | KS X 1026-1:2007 + KTUG 표준 매핑 (휴리스틱 분기 0) |

## 8. 잠정 사항 / 후속

### 8-1. U+F00DA 매핑 (잠정)

`▸` (BLACK SMALL TRIANGLE) 으로 매핑했으나 정확한 한컴 글리프와 일치하지 않을 가능성. 시각 판정 시 작업지시자가 정상 판정 → 잠정 매핑 유지. 향후 한컴 PDF 정밀 비교 시 정정 가능.

### 8-2. 단독 SVG (rhwp export-svg) 영역

font-family 체인에 폰트 이름은 추가되지만 **@font-face 정의는 SVG 미내장**. 단독 SVG 사용자는 `--font-style` / `--embed-fonts` 옵션 또는 시스템에 폰트 설치 필요. 향후 `--embed-fonts` 자동 임베딩 보강 가능 (별도 issue).

### 8-3. 폭 계산 정합

PUA char 폭 (font fallback ~font_size) vs 자모 cluster 합자 폭 (Source Han Serif K 메트릭) 의 미세 차이 가능. 회귀 테스트는 통과 (좌표 변동 0).

## 9. 영향 / 가치

### 9-1. 사용자 영향

- **옛한글 입력 HWP 문서** (한/글 2010 이전 버전 또는 호환 모드) 정상 렌더링
- exam_kor 외 광범위 영향 가능 (옛한글 사용 학술 / 고전 문헌)
- WASM 페이로드 +234 KB (옛한글 영역 사용 시에만)

### 9-2. 시스템 가치

- HanCom 자체 폰트 (함초롬바탕 LVT) 라이선스 의존 제거
- 오픈소스 (KTUG PD + Source Han Serif K SIL OFL) 만으로 한컴 옛한글 호환
- Issue #512 의 본질 영역 해결

## 10. 마감 처리 영역

- [x] 모든 Stage 통과 (1-5)
- [x] 작업지시자 시각 판정 ★
- [x] 단위 + 통합 테스트 통과
- [ ] orders/20260502.md 갱신 (다음 단계)
- [ ] Issue #528 close + #512 close (메인테이너 권한 — comment 등록 후 권장)
- [ ] local/devel merge + push
- [ ] PR (필요 시 — 현재 누적 PR #527 있음, 본 task 는 후속 별도 PR 또는 기존 PR 갱신 검토)

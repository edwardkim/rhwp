# Task #574 Stage 4 — 광범위 회귀 sweep + 전체 테스트 + clippy

**브랜치**: `local/task574`
**이슈**: https://github.com/edwardkim/rhwp/issues/574

---

## 1. 7개 샘플 SVG sweep (fix 전후 비교)

### 1.1 변경 파일 수

| 샘플 | 페이지 수 | 변경 페이지 | 비고 |
|------|----------|----------|------|
| `exam_science.hwp` | 4 | **4** | 본 이슈 샘플 |
| `exam_kor.hwp` | 20 | 20 | HY견명조 사용 |
| `exam_eng.hwp` | 8 | 8 | HY견명조 사용 |
| `exam_math.hwp` | 20 | 20 | HY견명조 사용 |
| `synam-001.hwp` | 35 | **0** | HY견명조 미사용 |
| `복학원서.hwp` | 1 | 1 | HY견명조 사용 |
| `text-align.hwp` | 1 | **0** | **Task #146 v4 base — HY헤드라인M 사용, 회귀 없음** ✓ |

→ Task #146 v4 핵심 케이스 (text-align.hwp HY헤드라인M heavy bold) **회귀 없음** 확인.

### 1.2 변경 라인 분석 (HY견명조 한정)

```
=== exam_science === 변경 라인: before=82 (HY견명조外=0), after=82 (HY견명조外=0)
=== exam_kor === 변경 라인: before=759 (HY견명조外=0), after=759 (HY견명조外=0)
=== exam_eng === 변경 라인: before=156 (HY견명조外=0), after=156 (HY견명조外=0)
=== exam_math === 변경 라인: before=244 (HY견명조外=0), after=244 (HY견명조外=0)
=== 복학원서 === 변경 라인: before=168 (HY견명조外=0), after=168 (HY견명조外=0)
```

→ **모든 변경 라인의 100% 가 HY견명조 사용 텍스트** — 다른 폰트 회귀 0건.

### 1.3 변경 본질 (font-weight="bold" 제거만)

```bash
diff <(sed 's/ font-weight="bold"//g' before/*.svg) \
     <(sed 's/ font-weight="bold"//g' after/*.svg)
# → 5개 샘플 모두 출력 0 (no diff)
```

→ **변경 영역은 순전히 `font-weight="bold"` 추가/제거만**. 좌표/크기/색상/scale/font-family 등 다른 속성 변경 0건.

### 1.4 의도된 정정 검증 (예: exam_science p1 line 167 — 본 이슈 쪽번호 "1")

**Before** (HY견명조 강제 bold):
```xml
<text transform="translate(924.36,114.87) scale(0.9000,1)"
      font-family="HY견명조,..." font-size="44"
      font-weight="bold" fill="#000000">1</text>
```

**After** (CharShape.bold=false 권위 회복):
```xml
<text transform="translate(924.36,114.87) scale(0.9000,1)"
      font-family="HY견명조,..." font-size="44"
      fill="#000000">1</text>
```

CharShape cs_id=0 의 IR 값 (bold=false) 와 정합.

## 2. 전체 lib 테스트

```
$ cargo test --release --lib

test result: ok. 1120 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out
```

→ 회귀 0건 (Stage 2 추가 통합 테스트 1건 포함).

## 3. clippy

```
$ cargo clippy --release --lib --tests 2>&1 | grep -E "(style_resolver|integration_tests)" | grep -E "warning:|error:"
(빈 출력)
```

→ Task #574 변경 파일 한정 신규 clippy 경고 0건.
   기존 54 warnings (Task #574 무관, 다른 코드의 누적 — 본 이슈 범위 외).

## 4. 결론

| 검증 항목 | 결과 |
|----------|------|
| 7개 샘플 sweep | 5개 변경 (전부 HY견명조 한정), 2개 무변경 (synam-001, text-align) |
| Task #146 v4 base 회귀 | **없음** (text-align.hwp HY헤드라인M 보존) |
| 변경 본질 | `font-weight="bold"` 제거만 (좌표/크기/색상 등 변경 0건) |
| HY견명조外 폰트 회귀 | 0건 |
| `cargo test --release --lib` | 1120 passed |
| 신규 clippy 경고 | 0건 |

→ **회귀 없음. 의도된 정정만**.

## 5. 산출물

| 파일 | 변경 |
|------|------|
| `mydocs/working/task_m100_574_stage4.md` | 본 보고서 |

(코드 변경 없음 — 검증만)

## 6. 다음 단계

Stage 5 — 한컴 PDF 시각 검증 (작업지시자 판정 게이트) + 최종 보고서.

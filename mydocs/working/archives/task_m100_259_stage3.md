---
타스크: #259 한글 폰트명 → 메트릭 DB 영문명 매핑 누락
단계: Stage 3 — text-align.hwp 회귀 검증 + svg_snapshot
브랜치: local/task259
작성일: 2026-04-23
---

# Stage 3 완료 보고서

## 1. 목적

Stage 2 매핑 추가가 의도된 효과 (HY중고딕 글자 폭 정상화) 를 내고, 그 외 영역에 회귀가 없는지 검증.

## 2. 검증 결과

### 2.1 svg_snapshot 회귀 테스트

```
cargo test --test svg_snapshot

running 3 tests
test table_text_page_0 ... ok
test form_002_page_0 ... ok
test render_is_deterministic_within_process ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured
```

3건 그린. **Golden 재생성 불필요** (form-002 / table-text 샘플에 HY 계열 폰트 미사용).

### 2.2 text-align.hwp 렌더 비교

#### 2.2.1 SVG 출력

```
output/svg/text-align-after/      — PR #256 머지 후 (= #259 fix 전 base)
output/svg/text-align-fix-after/  — #259 fix 후
output/svg/text-align-259-compare.html — 3-way 비교 페이지
```

#### 2.2.2 정량 분석 (4번 문단, HY중고딕 사용)

`samples/text-align.hwp` 0.4 문단:

```
텍스트: "    ** 10지점: 고양시, 춘천시, 청주시, ..." (cc=67)
[CS] pos=0 id=34 spacing=-4%
[CS] pos=2 id=8 spacing=-14%
[CS] pos=4 id=38 spacing=-17% char="*"
[CS] pos=7 id=35 spacing=-29% char="1"
```

문단 4번째 줄 (지자체 열거) 의 라틴/숫자 글자 좌표 비교:

| 글자 | Before x | After x | 변화 |
|---|---|---|---|
| `1` | 301.41 | 300.41 | -1.0 |
| `,` | 309.08 | 309.45 | +0.37 |
| `0` | 316.74 | 315.03 | -1.71 |
| `0` | 324.41 | 324.07 | -0.34 |
| `0` | 332.08 | 333.11 | +1.03 |
| `항` (한글) | 339.41 | 342.41 | +3.0 |

핵심: **숫자 `0` 의 advance 폭 7.67px → 9.04px** (기본 fallback → HYGothic-Medium DB 실측 폭).
한글 글자는 모두 16px (1em) 이라 절대 좌표만 누적 시프트 됨.

#### 2.2.3 영향 범위 한정

```
diff text-align-after/text-align.svg text-align-fix-after/text-align.svg
  | grep font-family | sort -u
→ font-family="HY중고딕,..."   (단일 결과)
```

변경은 **HY중고딕 폰트가 적용된 `<text>` 요소에만 국한**. 다른 폰트 (맑은고딕, 함초롬바탕, Pretendard 등) 사용 영역은 바이트 수준 동일. 회귀 0.

총 diff 라인 수: 322줄 (모두 HY중고딕 텍스트 좌표 변경).

### 2.3 시각 확인

`output/svg/text-align-259-compare.html` 페이지를 열어 3-way 비교 (Before / PR #256 / #259 fix) 가능. WASM 재빌드 후 작업지시자 웹 에디터 확인은 Stage 5 단계 또는 별도 진행.

## 3. 결론

- HY중고딕 글자 폭이 기본 fallback 에서 DB 실측 폭으로 전환됨 — 의도된 동작
- 한글 글자 폭은 변경 없음 (DB 의 한글 폭이 1em 기본값과 동일하기 때문)
- 라틴/숫자 폭만 변경 → 영문/숫자 글자 겹침이 해소될 것으로 예상
- 다른 폰트 영향 0
- svg_snapshot 회귀 0

## 4. 다음 단계

Stage 4 — 스모크 스위프 (다른 한글 폰트 사용 샘플 회귀 확인):

- `samples/hwp/exam_kor.hwp` (25 페이지)
- `samples/hwp/biz_plan.hwp` (6 페이지)
- `samples/hwpx/hwpx-02.hwpx`
- 본한글/본명조 사용 샘플 탐색

## 5. 산출물

- `output/svg/text-align-fix-after/text-align.svg` — #259 fix 후 렌더
- `output/svg/text-align-259-compare.html` — 3-way 비교 페이지 (Before / PR #256 / #259 fix)
- 본 문서 (`mydocs/working/task_m100_259_stage3.md`)

Stage 3 완료.

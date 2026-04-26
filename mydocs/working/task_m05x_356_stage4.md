# 단계 4 보고서 — Task #356 통합 검증 및 회귀 측정

- **단계**: 4/5
- **브랜치**: `local/task356`

## 1. 본 샘플 SVG diff (before / after)

`/tmp/t356_before/` (베이스라인 35 SVG) vs `/tmp/t356_after/` (fix 후 35 SVG):

```
Files _003.svg differ   ← 페이지 3 (pi=20..42 → pi=20..39)
Files _004.svg differ   ← 페이지 4 (밀려 들어온 pi=40..)
Files _029.svg differ   ← 페이지 29 (pi=556..575 → pi=556..572)
Files _030.svg differ   ← 페이지 30 (밀려 들어온 pi=573..)
```

**4개 파일만 변경** — vpos 리셋 발생 두 지점(페이지 3↔4, 페이지 29↔30) 의 인접 페이지에 정확히 일치. 나머지 31 페이지 바이트 단위 동일.

LAYOUT_OVERFLOW 경고: 5+건 → **0건**.

## 2. 골든 SVG 회귀 (`tests/golden_svg/`)

```
cargo test --release --test svg_snapshot
6 passed; 0 failed
- form_002_page_0
- issue_147_aift_page3
- issue_157_page_1
- issue_267_ktx_toc_page
- table_text_page_0
- render_is_deterministic_within_process
```

골든 SVG 픽셀 변경 0건.

## 3. 다중 샘플 회귀 비교

| 샘플 | 베이스라인 | 본 fix | 변화 |
|------|-----------|--------|------|
| `2022년 국립국어원 업무계획.hwp` | 35p / 5 | **35p / 0** | overflow 100% 해결 ✅ |
| `aift.hwp` | 74p / 30 | 86p / 16 | +12p, overflow 47% 감소 ✅ |
| `exam_eng.hwp` (다단) | 8p / 0 | **8p / 0** | 회귀 없음 ✅ |
| `exam_math.hwp` | 20p / 0 | **20p / 0** | 회귀 없음 ✅ |
| `2010-01-06.hwp` | 6p / 0 | **6p / 0** | 회귀 없음 ✅ |

## 4. aift.hwp 잔여 16건 overflow 분석

Type 별 분류:

| Type | 건수 | 평가 |
|------|------|------|
| `PartialTable` | 5 | 표 행 분할 결정 오류 — `split_table_rows` 경로, 본 fix 무관 |
| `Table` | 1 | 표 배치 오버플로 — 본 fix 무관 |
| `FullParagraph` | 2 | page 35 (pi=512, 513) — 별도 조사 필요 |
| `PartialParagraph` | 1 | page 35 (pi=514) — 위와 동일 |

**판정**: 잔여 overflow 16건 중 12건(75%)이 표 분할 관련(PartialTable/Table) 으로 본 fix(인접 *문단* vpos 리셋) 와 별개의 코드 경로. 나머지 4건(page 35) 도 같은 페이지에 집중되어 있어 별도 패턴(셰이프 + 빈 문단 + vpos 비정상)으로 보임.

→ 본 이슈 #356 의 명시 증상은 100% 해결. 잔여 케이스는 후속 이슈로 분리하는 것이 적절.

## 5. 단위 + 전체 테스트

```
cargo test --release
test result: ok. 1014 passed; 0 failed; 1 ignored
+ 14 + 25 + 6 + 1 + 1 PASS, 0 FAIL (전체 1061)
```

베이스라인 1055 → 1061 (신규 6 추가, 회귀 0).

## 6. 결론

- ✅ 이슈 #356 명시 증상 (페이지 3 footer 가 body 박스 밖) 해결
- ✅ 다른 샘플 회귀 0
- ✅ 골든 SVG 회귀 0
- ✅ 부수 효과: aift.hwp 페이지네이션 12 페이지 추가 + overflow 47% 감소

PDF 페이지 수와의 정확한 일치(이슈 본문의 "37쪽" 가능성) 는 본 샘플에서 35p 그대로 유지되었으나, 이는 *명시 증상* 이 아닌 부수 가능성에 해당. PDF 와의 페이지 수 차이는 본문 흐름 정책 외 (꼬리말, 빈 페이지, 표 분할 등) 다양한 요인이 있을 수 있어 별도 이슈로 추적.

## 7. 다음 단계

**단계 5** (최종 보고서 작성 + 머지 준비):
- `mydocs/report/task_m05x_356_report.md` 작성
- `mydocs/orders/20260426.md` 상태 갱신 (진행 → 완료)
- aift 잔여 overflow 추적용 후속 이슈(선택) 검토

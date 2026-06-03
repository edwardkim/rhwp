# Task #713 Stage 4-5 (회귀 + 광범위) 완료 보고서

**Issue**: [#713](https://github.com/edwardkim/rhwp/issues/713)
**Stage**: 4 + 5
**작성일**: 2026-05-08

---

## Stage 4 — 회귀 검증

```
$ cargo test --release
passed=1250  failed=0  ignored=3
```

→ 회귀 0.

## Stage 5 — 광범위 검증 (181 샘플)

페이지 수 비교 — 패치 적용 전(stream/devel) vs 적용 후(local/task713):

```
$ diff /tmp/task713_pagecount_before.txt /tmp/task713_h4_after.txt
(0 lines)
```

→ **181 샘플 페이지 수 회귀 0**.

### 가설 H1 (폐기) 와의 비교

H1 (RowBreak 인트라-로우 분할 차단) 시도 시 회귀 3 샘플:

| 샘플 | H1 (회귀) | H4 (정합) | PDF 권위 |
|------|-----------|-----------|----------|
| inner-table-01.hwp | 2 → 3 | 2 → 2 ✓ | 2 |
| k-water-rfp.hwp | 27 → 29 | 27 → 27 ✓ | 27 |
| synam-001.hwp | 35 → 39 | 35 → 35 ✓ | 35 |

H4 의 임계값 25 px 가드는 본 결함 (17.6 px) 만 차단하고 위 3 샘플 (각각 ≥ 27 px 분할) 은 변경 없음.

## 결과 요약

- 회귀 테스트: 1250/1250 passed
- 광범위 페이지 수 회귀: 0/181
- 결함 정정: row 8 (한국어교육 내실화) 가 페이지 37 상단으로 이동 (PDF 권위 정합)
- 분할 연결 페이지: 무영향 (gate 추가 분기는 Stage 3 가드만)

## 승인 요청

Stage 4-5 완료. Stage 6 (최종 보고서 + close #713) 진행 승인 요청.

# Task #716 Stage 5 (광범위) 완료 보고서

**Issue**: [#716](https://github.com/edwardkim/issues/716)
**Stage**: 5 — 169 샘플 광범위 회귀 검증
**작성일**: 2026-05-08

---

## 검증 방법

전체 샘플 (169개 = HWP + HWPX):
1. **Before** (= Stage 2 commit, fix 미적용): `git checkout 86be59f0 -- src/renderer/layout.rs` 후 빌드 → 169 샘플 export
2. **After** (= Stage 3 GREEN): 현재 브랜치 → 169 샘플 export
3. 각 샘플별 (페이지 수, `LAYOUT_OVERFLOW_DRAW` 카운트, `LAYOUT_OVERFLOW` 카운트, panic 카운트) 측정
4. Before / After 차이 분석

스캔 스크립트: `/tmp/task716-broad/scan3.sh` (dump-pages stdout 에서 페이지 수, export-svg stderr 에서 OVERFLOW 카운트)

## 총괄 결과

| 메트릭 | Before (baseline) | After (with fix) | Δ |
|--------|-------------------|------------------|---|
| `LAYOUT_OVERFLOW_DRAW` 총 건수 | 187 | **185** | **−2** |
| `LAYOUT_OVERFLOW` 총 건수 | 279 | **277** | **−2** |
| panic 총 건수 | 0 | 0 | 0 |
| 페이지 수 변동 샘플 | — | — | **0** |

→ **모든 차감은 의도된 정정**. 신규 발생 0건. 페이지 수 변동 0건.

## 샘플별 차이 (per-sample diff)

```
diff <(sort before.tsv) <(sort after.tsv)

145a146,147
> 4	0	1	20250130-hongbo.hwp
> 4	0	1	20250130-hongbo-no.hwp
147,150c149,150
< 4	1	1	20250130-hongbo.hwp
< 4	1	1	20250130-hongbo-no.hwp
< 5	0	1	table-vpos-01.hwp
< 5	0	1	table-vpos-01.hwpx
---
> 5	0	0	table-vpos-01.hwp
> 5	0	0	table-vpos-01.hwpx
```

해석 표:

| 샘플 | 페이지 | Before | After | 변화 |
|------|--------|--------|-------|------|
| 20250130-hongbo.hwp | 4 | DRAW=1, FLOW=1 | DRAW=**0**, FLOW=1 | **본 결함 정정** ✓ |
| 20250130-hongbo-no.hwp | 4 | DRAW=1, FLOW=1 | DRAW=**0**, FLOW=1 | 동일 패턴 정정 ✓ |
| table-vpos-01.hwp | 5 | DRAW=0, FLOW=1 | DRAW=0, FLOW=**0** | 부수 개선 (FLOW 차감) ✓ |
| table-vpos-01.hwpx | 5 | DRAW=0, FLOW=1 | DRAW=0, FLOW=**0** | 부수 개선 ✓ |
| 그 외 165 샘플 | (모두 동일) | — | — | 회귀 0 ✓ |

### 부수 개선 (table-vpos-01)

`table-vpos-01.hwp/hwpx` 의 LAYOUT_OVERFLOW 1건이 fix 로 0건이 됨. 이 샘플도 TAC 표 + 음수 ls 빈 paragraph 패턴이 있어 동일 메커니즘 적용.

→ Task #9 fix_overlay 의 빈 paragraph push 가 본 샘플에서도 drift 를 발생시키던 것이 함께 해소.

### 페이지 수 변동

```
diff <(awk '{print $4"\t"$1}' before3.tsv | sort) <(awk '{print $4"\t"$1}' after3.tsv | sort)
(empty diff)
```

→ **169 샘플 모두 페이지 수 변동 0**. 본 정정은 layout y_offset 만 정정 (drift 차단), pagination height_measurer 는 별도 경로라 영향 없음.

## 골든 SVG 회귀 (Stage 4 재확인)

`tests/svg_snapshot.rs` 7개 테스트 PASS:
- `table_text_page_0`
- `issue_157_page_1`
- `issue_267_ktx_toc_page`
- `form_002_page_0`
- `render_is_deterministic_within_process`
- `issue_147_aift_page3`
- `issue_617_exam_kor_page5`

→ 골든 SVG 6 샘플 시각 출력 변화 없음.

## 결론

Stage 5 광범위 검증 완료:

- **본 결함 (hongbo)**: 정정 ✓
- **동일 패턴 (hongbo-no)**: 정정 ✓
- **부수 개선 (table-vpos-01)**: FLOW 카운트 2건 차감 ✓
- **신규 발생**: 0건 ✓
- **페이지 수 변동**: 0건 ✓
- **panic**: 0건 ✓
- **골든 SVG 회귀**: 0건 ✓

`is_empty_para` 가드는 좁고 안전한 정정으로 입증. Task #9 의 본래 의도(텍스트 paragraph push) 보존하면서 빈 paragraph 의 의미 없는 drift 만 차단.

## 다음 단계 (Stage 6 — 최종 보고)

1. 최종 결과 보고서 (`mydocs/report/task_m100_716_report.md`) 작성
2. closes #716 커밋
3. plans/archives/ 로 계획서 이동
4. 작업지시자 승인 후 `pr-task716` 브랜치 (stream/devel 베이스) 생성, origin push, PR 생성

## 승인 요청

Stage 5 광범위 검증 완료. 모든 메트릭 0 회귀. Stage 6 (최종 보고서 + close) 진입 승인 요청.

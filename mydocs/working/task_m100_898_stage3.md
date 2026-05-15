# Stage 3 보고서: Task #898 — 시각 회귀 검증

## 1. exam_math.hwp 전 20쪽

전 페이지 바탕쪽 표 셀 y 좌표 일관성 확인:

| 페이지 | 표 셀 y |
|--------|---------|
| 1 | 1378.28 |
| 2 | 1378.28 |
| 5 | 1378.28 |
| 10 | 1378.28 |
| 19 | 1378.28 |
| 20 | 1378.28 |

모든 페이지 동일 — `v_offset (101954) + outer_margin_top (1417)` = 103371 HU = 1378.28 px.

시각 확인:
- 페이지 1 (`/tmp/em_fixed_p1.png`): 가운데 세로선과 `1/20` 박스 사이 ≈20px 여백 ✓
- 페이지 5 (`/tmp/em_p5.png`): 동일 ✓

## 2. 바탕쪽 사용 다른 샘플 회귀

`grep "바탕쪽: [1-9]"` 로 식별된 18개 샘플 중 대표 6건 렌더:

| 샘플 | 페이지 수 | 오류 |
|------|---------|------|
| samples/exam_kor.hwp | 20 | 없음 |
| samples/exam_eng.hwp | 8 | 없음 |
| samples/exam_social.hwp | 4 | 없음 |
| samples/exam_science.hwp | 4 | 없음 |
| samples/basic/shortcut.hwp | 7 | 없음 |
| samples/basic/KTX.hwp | 1 | 없음 |

시각 확인:
- exam_kor 1쪽 (`/tmp/exam_kor_p1.png`): 정상 (페이지 번호 `1` 우상단)
- shortcut 1쪽 (`/tmp/shortcut_p1.png`): 정상 (페이지 번호 `1` 우하단)

## 3. 골든 SVG 회귀

`cargo test --release --test svg_snapshot`:

```
test issue_157_page_1 ... ok
test issue_677_bokhakwonseo_page1 ... ok
test issue_267_ktx_toc_page ... ok
test form_002_page_0 ... ok
test render_is_deterministic_within_process ... ok
test issue_147_aift_page3 ... ok
test issue_617_exam_kor_page5 ... ok
test result: ok. 8 passed; 0 failed
```

특히 `issue_617_exam_kor_page5` 는 바탕쪽 사용 문서 골든 → **변화 없음**.

## 4. 전체 테스트

`cargo test --release`:
- **TOTAL passed=1412, failed=0**
- 40개 테스트 그룹 전부 통과
- 신규 `tests/issue_898.rs::master_page_table_includes_outer_margin_top` 포함

## 5. 결론

- exam_math.hwp 사용자 보고 결함 해결
- 다른 바탕쪽 사용 문서 회귀 없음
- 골든 SVG 변화 없음 (Paper-relative + outer_margin_top > 0 케이스는 exam 시리즈 / shortcut 등 일부 문서에만 해당, 골든은 영향 없음)
- 전체 단위 테스트 통과

## 6. 다음 단계

Stage 4 — 최종 마무리:
- clippy 검증 (lib 단독)
- 골든 SVG 갱신 필요성 재확인 (불필요 예상)
- 최종 결과 보고서: `mydocs/report/task_m100_898_report.md`
- 커밋

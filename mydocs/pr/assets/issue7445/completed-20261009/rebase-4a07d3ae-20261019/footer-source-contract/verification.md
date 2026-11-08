# 보정 단계의 검증 기록

Issue: #7445. 이 문서는 해당 단계의 역사적 검증 신원·수치·판독 기록이다. 현재 브랜치 전체의 통과나 PR 준비 완료를 뜻하지 않는다. 기존 PNG·전쪽 TSV·구현·정규 입력은 유지했다.

생성 JSON은 실제 source 소비자가 없는 임시 실행 결과이므로 ignored output으로 SHA를 유지해 옮긴다. 아래에는 관련 검증 사실을 보존한다. 글꼴별 임시 파일 원장과 자산 복사 목록의 전체 원문은 로컬 원장에 남긴다.

로컬 원장: `output/pr-review/7445-completed-20261009/rebase-4a07d3ae-20261019/repair978-generated-evidence-relocation/raw/` 아래의 이전 저장소 상대 경로.

## independent-footer-origins

이전 생성 파일: `independent-footer-origins.json` · 675 bytes · SHA-256 `6374749dd517f1a9a784759c6e44e1211f86b056ad239209a4caf57361a1e506`.

| 항목 | 당시 기록 |
|---|---|
| `[0].pdf` | pdf/issue7336/nested_table_fragment_cut-2020.pdf |
| `[0].page` | 6 |
| `[0].glyphs[0].text` | 6 |
| `[0].glyphs[0].x_css` | 393.1199951171875 |
| `[0].glyphs[0].baseline_css` | 1062.719970703125 |
| `[1].pdf` | pdf/issue7382-regression-review/issue_1133-2020.pdf |
| `[1].page` | 3 |
| `[1].glyphs[0].text` | 3 |
| `[1].glyphs[0].x_css` | 392.9070638020833 |
| `[1].glyphs[0].baseline_css` | 1080.4192708333333 |
| `[2].pdf` | pdf/issue1133/page-number-bottom15-2020.pdf |
| `[2].page` | 3 |
| `[2].glyphs[0].text` | 3 |
| `[2].glyphs[0].x_css` | 392.9070638020833 |
| `[2].glyphs[0].baseline_css` | 1061.5598958333333 |

## validation

이전 생성 파일: `validation.json` · 18448 bytes · SHA-256 `f03049cad8f7bf3c45c016886d6cb3fac9f7d649aed9ff38893b2faf4dd2f8ff`.

| 항목 | 당시 기록 |
|---|---|
| `head_before_review` | 66859c77039400713917b45cd1f8823ec9de2844 |
| `scope` | 기존 쪽번호 검사 기대값/소유 관계 재검토. 제품 구현 변경 없음. 원본/여백4건12쪽+style7336전6쪽+1937전50쪽, backend별68쪽. 전체PR통과 아님. |
| `producer_path` | /Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-4a07d3ae-20261019/repair523-footer-current-whole/producer.json |
| `producer_sha256` | a1d23d51308933caffbfd5fe7f0331335b9ec20a1ae35669a2270aac542b7243 |
| `visual_results[0].backend` | native |
| `visual_results[0].key` | footer15 |
| `visual_results[0].counts.rhwp` | 3 |
| `visual_results[0].counts.pdf` | 3 |
| `visual_results[0].minimum` | 95.17397 |
| `visual_results[0].worstPage` | 1 |
| `visual_results[0].tsv` | /Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-4a07d3ae-20261019/repair523-footer-current-whole/native/footer15/silhouette.tsv |
| `visual_results[0].directReading` | pending |
| `visual_results[1].backend` | native |
| `visual_results[1].key` | bottom15 |
| `visual_results[1].counts.rhwp` | 3 |
| `visual_results[1].counts.pdf` | 3 |
| `visual_results[1].minimum` | 95.17397 |
| `visual_results[1].worstPage` | 1 |
| `visual_results[1].tsv` | /Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-4a07d3ae-20261019/repair523-footer-current-whole/native/bottom15/silhouette.tsv |
| `visual_results[1].directReading` | pending |
| `visual_results[2].backend` | native |
| `visual_results[2].key` | original1133-hwp |
| `visual_results[2].counts.rhwp` | 3 |
| `visual_results[2].counts.pdf` | 3 |
| `visual_results[2].minimum` | 95.17397 |
| `visual_results[2].worstPage` | 1 |
| `visual_results[2].tsv` | /Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-4a07d3ae-20261019/repair523-footer-current-whole/native/original1133-hwp/silhouette.tsv |
| `visual_results[2].directReading` | pending |
| `visual_results[3].backend` | native |
| `visual_results[3].key` | original1133-hwpx |
| `visual_results[3].counts.rhwp` | 3 |
| `visual_results[3].counts.pdf` | 3 |
| `visual_results[3].minimum` | 95.17148 |
| `visual_results[3].worstPage` | 1 |
| `visual_results[3].tsv` | /Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-4a07d3ae-20261019/repair523-footer-current-whole/native/original1133-hwpx/silhouette.tsv |
| `visual_results[3].directReading` | pending |
| `visual_results[4].backend` | native |
| `visual_results[4].key` | style7336 |
| `visual_results[4].counts.rhwp` | 6 |
| `visual_results[4].counts.pdf` | 6 |
| `visual_results[4].minimum` | 95.13463 |
| `visual_results[4].worstPage` | 5 |
| `visual_results[4].tsv` | /Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-4a07d3ae-20261019/repair523-footer-current-whole/native/style7336/silhouette.tsv |
| `visual_results[4].directReading` | pending |
| `visual_results[5].backend` | native |
| `visual_results[5].key` | footnote-number25 |
| `visual_results[5].counts.rhwp` | 50 |
| `visual_results[5].counts.pdf` | 50 |
| `visual_results[5].minimum` | 94.90661 |
| `visual_results[5].worstPage` | 22 |
| `visual_results[5].tsv` | /Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-4a07d3ae-20261019/repair523-footer-current-whole/native/footnote-number25/silhouette.tsv |
| `visual_results[5].directReading` | pending |
| `visual_results[6].backend` | wasm |
| `visual_results[6].key` | footer15 |
| `visual_results[6].counts.rhwp` | 3 |
| `visual_results[6].counts.pdf` | 3 |
| `visual_results[6].minimum` | 95.18760 |
| `visual_results[6].worstPage` | 1 |
| `visual_results[6].tsv` | /Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-4a07d3ae-20261019/repair523-footer-current-whole/wasm/footer15/silhouette.tsv |
| `visual_results[6].directReading` | pending |
| `visual_results[7].backend` | wasm |
| `visual_results[7].key` | bottom15 |
| `visual_results[7].counts.rhwp` | 3 |
| `visual_results[7].counts.pdf` | 3 |
| `visual_results[7].minimum` | 95.18760 |
| `visual_results[7].worstPage` | 1 |
| `visual_results[7].tsv` | /Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-4a07d3ae-20261019/repair523-footer-current-whole/wasm/bottom15/silhouette.tsv |
| `visual_results[7].directReading` | pending |
| `visual_results[8].backend` | wasm |
| `visual_results[8].key` | original1133-hwp |
| `visual_results[8].counts.rhwp` | 3 |
| `visual_results[8].counts.pdf` | 3 |
| `visual_results[8].minimum` | 95.18760 |
| `visual_results[8].worstPage` | 1 |
| `visual_results[8].tsv` | /Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-4a07d3ae-20261019/repair523-footer-current-whole/wasm/original1133-hwp/silhouette.tsv |
| `visual_results[8].directReading` | pending |
| `visual_results[9].backend` | wasm |
| `visual_results[9].key` | original1133-hwpx |
| `visual_results[9].counts.rhwp` | 3 |
| `visual_results[9].counts.pdf` | 3 |
| `visual_results[9].minimum` | 95.18502 |
| `visual_results[9].worstPage` | 1 |
| `visual_results[9].tsv` | /Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-4a07d3ae-20261019/repair523-footer-current-whole/wasm/original1133-hwpx/silhouette.tsv |
| `visual_results[9].directReading` | pending |
| `visual_results[10].backend` | wasm |
| `visual_results[10].key` | style7336 |
| `visual_results[10].counts.rhwp` | 6 |
| `visual_results[10].counts.pdf` | 6 |
| `visual_results[10].minimum` | 94.96148 |
| `visual_results[10].worstPage` | 5 |
| `visual_results[10].tsv` | /Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-4a07d3ae-20261019/repair523-footer-current-whole/wasm/style7336/silhouette.tsv |
| `visual_results[10].directReading` | pending |
| `visual_results[11].backend` | wasm |
| `visual_results[11].key` | footnote-number25 |
| `visual_results[11].counts.rhwp` | 50 |
| `visual_results[11].counts.pdf` | 50 |
| `visual_results[11].minimum` | 94.96082 |
| `visual_results[11].worstPage` | 22 |
| `visual_results[11].tsv` | /Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-4a07d3ae-20261019/repair523-footer-current-whole/wasm/footnote-number25/silhouette.tsv |
| `visual_results[11].directReading` | pending |
| `important_pages.original1133-hwp.page` | 3 |
| `important_pages.original1133-hwp.css_tolerance` | 2 |
| `important_pages.original1133-hwp.native_raw` | 100.00000 |
| `important_pages.original1133-hwp.wasm_raw` | 100.00000 |
| `important_pages.original1133-hwpx.page` | 3 |
| `important_pages.original1133-hwpx.css_tolerance` | 2 |
| `important_pages.original1133-hwpx.native_raw` | 100.00000 |
| `important_pages.original1133-hwpx.wasm_raw` | 100.00000 |
| `important_pages.bottom15.page` | 3 |
| `important_pages.bottom15.css_tolerance` | 2 |
| `important_pages.bottom15.native_raw` | 100.00000 |
| `important_pages.bottom15.wasm_raw` | 100.00000 |
| `important_pages.style7336.page` | 6 |
| `important_pages.style7336.css_tolerance` | 2 |
| `important_pages.style7336.native_raw` | 99.88197 |
| `important_pages.style7336.wasm_raw` | 99.88244 |
| `important_pages.footnote-number25.page` | 25 |
| `important_pages.footnote-number25.css_tolerance` | 2 |
| `important_pages.footnote-number25.native_raw` | 99.97779 |
| `important_pages.footnote-number25.wasm_raw` | 99.97744 |
| `direct_reading` | Native/WASM 원본1133 3쪽·bottom15 3쪽·style7336 6쪽·1937 25쪽을 직접 읽고 번호·본문·표·각주 소유 확인. 원본1133 1쪽 제목의 자형 잔여를 완전일치로 주장하지 않음. |
| `native_gate.exitCode` | 0 |
| `native_gate.seconds` | 35.3735370420618 |
| `native_gate.summary[0]` |      Summary [   0.512s] 22 tests run: 22 passed, 997 skipped |
| `native_gate.sourceUnchanged` | True |
| `native_gate.scope` | Native current footer source/ownership plus original1133 and original1937 controls; not full nextest |
| `residual` | 별도 issue265 HWP5 저장본의 그림 뒤 본문 이동과 Science p3 base 대비0.32810%p 하락,최종 전체 회귀·소비경로 검증 미완료. |


## 보존한 전체 TSV

- [native-bottom15-whole.tsv](native-bottom15-whole.tsv)
- [native-footer15-whole.tsv](native-footer15-whole.tsv)
- [native-footnote-number25-whole.tsv](native-footnote-number25-whole.tsv)
- [native-original1133-hwp-whole.tsv](native-original1133-hwp-whole.tsv)
- [native-original1133-hwpx-whole.tsv](native-original1133-hwpx-whole.tsv)
- [native-style7336-whole.tsv](native-style7336-whole.tsv)
- [wasm-bottom15-whole.tsv](wasm-bottom15-whole.tsv)
- [wasm-footer15-whole.tsv](wasm-footer15-whole.tsv)
- [wasm-footnote-number25-whole.tsv](wasm-footnote-number25-whole.tsv)
- [wasm-original1133-hwp-whole.tsv](wasm-original1133-hwp-whole.tsv)
- [wasm-original1133-hwpx-whole.tsv](wasm-original1133-hwpx-whole.tsv)
- [wasm-style7336-whole.tsv](wasm-style7336-whole.tsv)

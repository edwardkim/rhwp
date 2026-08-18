# --text-only 경로 카탈로그

Chrome·pypdfium2 는 `--text-only` 에서 요구하지 않는다. pypdf 만.

| id | mode | parse | artifacts | 제목 |
| --- | --- | --- | --- | --- |
| path-reg-bunjang-text-only | registered | ok | provenance.tsv,report.tsv,text-report.tsv… | 등록 키 bunjang / text-only |
| path-reg-bunjang-text-only-export-all-svg | registered | ok | provenance.tsv,report.tsv,text-report.tsv… | 등록 키 bunjang / text-only+export-all-svg |
| path-reg-bunjang-text-only-layout-ledger | registered | ok | provenance.tsv,report.tsv,text-report.tsv… | 등록 키 bunjang / text-only+layout-ledger |
| path-reg-bunjang-text-only-export-all-svg-layout-ledger | registered | ok | provenance.tsv,report.tsv,text-report.tsv… | 등록 키 bunjang / text-only+export-all-svg+layout-ledger |
| path-reg-eng-text-only | registered | ok | provenance.tsv,report.tsv,text-report.tsv… | 등록 키 eng / text-only |
| path-reg-eng-text-only-export-all-svg | registered | ok | provenance.tsv,report.tsv,text-report.tsv… | 등록 키 eng / text-only+export-all-svg |
| path-reg-eng-text-only-layout-ledger | registered | ok | provenance.tsv,report.tsv,text-report.tsv… | 등록 키 eng / text-only+layout-ledger |
| path-reg-eng-text-only-export-all-svg-layout-ledger | registered | ok | provenance.tsv,report.tsv,text-report.tsv… | 등록 키 eng / text-only+export-all-svg+layout-ledger |
| path-reg-korexam-text-only | registered | ok | provenance.tsv,report.tsv,text-report.tsv… | 등록 키 korexam / text-only |
| path-reg-korexam-text-only-export-all-svg | registered | ok | provenance.tsv,report.tsv,text-report.tsv… | 등록 키 korexam / text-only+export-all-svg |
| path-reg-korexam-text-only-layout-ledger | registered | ok | provenance.tsv,report.tsv,text-report.tsv… | 등록 키 korexam / text-only+layout-ledger |
| path-reg-korexam-text-only-export-all-svg-layout-ledger | registered | ok | provenance.tsv,report.tsv,text-report.tsv… | 등록 키 korexam / text-only+export-all-svg+layout-ledger |
| path-reg-manual-text-only | registered | ok | provenance.tsv,report.tsv,text-report.tsv… | 등록 키 manual / text-only |
| path-reg-manual-text-only-export-all-svg | registered | ok | provenance.tsv,report.tsv,text-report.tsv… | 등록 키 manual / text-only+export-all-svg |
| path-reg-manual-text-only-layout-ledger | registered | ok | provenance.tsv,report.tsv,text-report.tsv… | 등록 키 manual / text-only+layout-ledger |
| path-reg-manual-text-only-export-all-svg-layout-ledger | registered | ok | provenance.tsv,report.tsv,text-report.tsv… | 등록 키 manual / text-only+export-all-svg+layout-ledger |
| path-reg-math-text-only | registered | ok | provenance.tsv,report.tsv,text-report.tsv… | 등록 키 math / text-only |
| path-reg-math-text-only-export-all-svg | registered | ok | provenance.tsv,report.tsv,text-report.tsv… | 등록 키 math / text-only+export-all-svg |
| path-reg-math-text-only-layout-ledger | registered | ok | provenance.tsv,report.tsv,text-report.tsv… | 등록 키 math / text-only+layout-ledger |
| path-reg-math-text-only-export-all-svg-layout-ledger | registered | ok | provenance.tsv,report.tsv,text-report.tsv… | 등록 키 math / text-only+export-all-svg+layout-ledger |
| path-reg-plan-text-only | registered | ok | provenance.tsv,report.tsv,text-report.tsv… | 등록 키 plan / text-only |
| path-reg-plan-text-only-export-all-svg | registered | ok | provenance.tsv,report.tsv,text-report.tsv… | 등록 키 plan / text-only+export-all-svg |
| path-reg-plan-text-only-layout-ledger | registered | ok | provenance.tsv,report.tsv,text-report.tsv… | 등록 키 plan / text-only+layout-ledger |
| path-reg-plan-text-only-export-all-svg-layout-ledger | registered | ok | provenance.tsv,report.tsv,text-report.tsv… | 등록 키 plan / text-only+export-all-svg+layout-ledger |
| path-direct-text-only | direct | ok | provenance.tsv,report.tsv,text-report.tsv… | direct pair 215쪽 전수 텍스트 |
| path-direct-text-only-min | direct | ok | provenance.tsv,report.tsv,text-report.tsv… | direct pair 최소 --text-only |
| path-error-unknown-key | error | error | provenance.tsv,report.tsv,text-report.tsv… | 미등록 키 |
| path-error-direct-incomplete | error | error | provenance.tsv,report.tsv,text-report.tsv… | direct pair 불완전 |
| path-error-grade-on-registered | error | error | provenance.tsv,report.tsv,text-report.tsv… | 등록 키에 grade |
| path-error-end-before-start | error | error | provenance.tsv,report.tsv,text-report.tsv… | 쪽 범위 역전 |
| path-error-direct-three-positionals | error | error | provenance.tsv,report.tsv,text-report.tsv… | direct pair 에 키까지 |
| path-error-non-integer-page | error | error | provenance.tsv,report.tsv,text-report.tsv… | 쪽 번호 비정수 |

## 산출 계약

| 파일 | 조건 |
| --- | --- |
| provenance.tsv | 항상 |
| report.tsv | 항상 |
| text-report.tsv | 항상 |
| svg-glyph-risk-report.tsv | 항상 |
| text-owner-shift-candidates.tsv | 항상 |
| text-owner-sequence-candidates.tsv | 항상 |
| page-boundary-fidelity-candidates.tsv | 항상 |
| visible-text-excess-candidates.tsv | 항상 |
| page-count-ledger.tsv | 항상 |
| run-state.tsv | 항상 |
| 파일 | 조건 |
| --- | --- |
| layout-candidates.tsv | --layout-ledger |
| table-fragment-candidates.tsv | --layout-ledger |
| table-cell-text-overlap-candidates.tsv | --layout-ledger |
| table-cell-text-boundary-candidates.tsv | --layout-ledger |
| svg-text-band-clip-candidates.tsv | --layout-ledger |
| svg-table-border-clip-candidates.tsv | --layout-ledger |
| svg-table-horizontal-border-clip-candidates.tsv | --layout-ledger |
| float-owner-shift-candidates.tsv | --layout-ledger |

| svg/export-svg-manifest.json | --export-all-svg |

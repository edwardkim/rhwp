---
kind: investigation
status: completed
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-10
---

# Task #3820 Stage 116 — 정책연구 p199 visible-text-excess 감사

## 목적

Stage 98에서 p199에 남은 visible SVG-only 99자가 조기·중복 paint인지, 기준 PDF
text extractor의 누락인지 최신 raster와 독립 추출기로 판정한다.

- 시작 commit: `dba5cd586`
- 검증 바이너리: `target/pr-review/release-test/rhwp`
- 공유 작업공간의 별도 p120 renderer 수정은 포함하지 않았다.

## 자동 원장

- PDF / SVG / render tree: `215/215/215`쪽
- p199 요청 / 완료 / 누락: `1/1/0`
- pypdf 기준 text counter: PDF-only 0자, visible SVG-only 99자
- owner-shift / owner-sequence / page-boundary: 0건
- body-footnote / table-footer / cell overlap / text-band clip: 0건
- pixel diff: `11.84%`

visible SVG-only 99자의 순서 보존 문자열은 다음 p199 각주와 footer다.

```text
257) 제11조(장기등의 적출·이식의 금지 등) ④ ... 4촌 이내의 친족에게
이식하는 경우가 아니면 적출할 수 없다. - 199 -
```

## 독립 추출과 PDF 직접 판정

`fidelity_compare.py`가 쓰는 `pypdf`의 p199 text layer에는 위 각주 257과 footer가
누락됐다. 반면 Poppler `pdftotext -f 199 -l 199 -layout`은 각주 257과 `- 199 -`를
정상 추출한다. 즉 99자는 rhwp가 추가로 paint한 본문이 아니라 pypdf가 세지 못한
기준 PDF의 실제 가시 text다.

p199 원본 크기 compare에서도 PDF와 rhwp 양쪽에 다음이 모두 보인다.

- 동일한 본문 단락과 `(라) 생존 기증자 동의 취득 관련 개선 방안`
- 각주 marker 257과 페이지 하단 각주 257
- 같은 footer `- 199 -`

본문·각주·footer의 페이지 소유권, 수직 순서와 겹침도 같다. 따라서 이 후보는 기준
PDF text extraction false positive이며 renderer를 변경하지 않는다.

## 결론

p199의 visible-text-excess 99자 신호를 폐기한다. Stage 98에서 순위화한 미판정
후보 다섯 범위(p23–24, p160–165, p72, p199)는 모두 최신 PDF 직접 판정으로 닫혔다.
다음 단계에서는 별도 p120 renderer 수정이 커밋된 뒤 새 바이너리로 영향 범위와 전체
후보 원장을 다시 생성한다.

## 증적

- [p199 비교](../pr/assets/task_m100_3820_stage116_policy_p199_extraction_audit/compare_p199.png)
- [pypdf 문자 원장](../pr/assets/task_m100_3820_stage116_policy_p199_extraction_audit/text-report.tsv)
- [visible excess 원장](../pr/assets/task_m100_3820_stage116_policy_p199_extraction_audit/visible-text-excess-candidates.tsv)
- [layout 원장](../pr/assets/task_m100_3820_stage116_policy_p199_extraction_audit/layout-candidates.tsv)

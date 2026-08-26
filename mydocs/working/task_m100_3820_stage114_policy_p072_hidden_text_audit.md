---
kind: investigation
status: completed
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-10
---

# Task #3820 Stage 114 — 정책연구 p72 raw SVG-only 문자 감사

## 목적

Stage 98의 p72 `text-report.tsv`에 남은 SVG-only 29자가 실제 화면의 중복·유출인지,
clip 밖의 stale source text를 raw SVG extractor가 센 것인지 최신 renderer와 한컴 PDF로
구분한다.

- 시작 commit: `083e1ef4a`
- 검증 바이너리: `target/pr-review/release-test/rhwp`
- 공유 작업공간의 별도 p120 renderer 수정은 이 바이너리와 stage에 포함하지 않았다.

## 결과

- PDF / SVG / render tree: `215/215/215`쪽
- p72 요청 / 완료 / 누락: `1/1/0`
- raw text: PDF-only 0자, SVG-only 29자
- visible text excess: 0건
- owner-shift / owner-sequence / page-boundary: 0건
- text-band clip / body-footnote / table-footer / cell overlap: 0건
- pixel diff: `6.59%`

SVG-only 29자는 `간 기증...related...` 조각이지만 실제 body/cell clip 교집합을 통과한
visible SVG 원장에는 존재하지 않는다. p72 원본 크기 raster를 PDF와 직접 비교하면
붉은 본문, URL, 각주 94–96이 같은 페이지와 순서를 소유하며 해당 문자열이 중복되어
그려지지 않는다. PDF에만 있는 가시 문자도 없다.

따라서 Stage 98 신호는 raw SVG tree 안의 완전 clip된 stale source text를 센
candidate-only 오탐이다. 실제 paint나 페이지 owner를 바꾸는 renderer 수정은 하지
않는다.

## 결론

p72는 현재 정상이며 raw SVG-only 29자 후보를 폐기한다. 다음 잔여 후보는 p199의
visible-text-excess 99자 신호를 최신 visible 원장과 PDF로 다시 확인하는 것이다.

## 증적

- [p72 비교](../pr/assets/task_m100_3820_stage114_policy_p072_hidden_text_audit/compare_p072.png)
- [raw 문자 원장](../pr/assets/task_m100_3820_stage114_policy_p072_hidden_text_audit/text-report.tsv)
- [visible excess 원장](../pr/assets/task_m100_3820_stage114_policy_p072_hidden_text_audit/visible-text-excess-candidates.tsv)
- [layout 원장](../pr/assets/task_m100_3820_stage114_policy_p072_hidden_text_audit/layout-candidates.tsv)

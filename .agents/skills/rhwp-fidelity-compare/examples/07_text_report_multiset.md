# 예제 — text-report 소실/과잉/치환

이슈 #5329. 실 에이전트 경로. gym 아님.

## 카탈로그

```
page	reference_only	svg_only	reference_only_chars	svg_only_chars	note
1	0	0			-
12	6	6	①②③	□□□	substitution-candidate
18	24	0	각주본문누락		loss-candidate
19	0	24		각주본문과잉	excess-candidate
```

p12 는 원문자→두부 치환 후보 (#3385 유형).
p18/p19 는 owner 이동 후보. `text-owner-shift-candidates.tsv` 를 연다.

0/0 을 시각 동일로 읽지 않는다.

관련: `references/06_text_report.md`.
픽스처: `fixtures/tsv/text_report_mixed.tsv`.
정지 F02.

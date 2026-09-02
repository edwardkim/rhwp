# 예제 — 최악 쪽부터 읽기

이슈 #5329. 실 에이전트 경로. gym 아님.

## 명령

```bash
venv/bin/python tools/fidelity_compare/fidelity_compare.py plan 0 34 \
  --out-dir /tmp/rhwp-fidelity-plan
column -t -s $'\t' /tmp/rhwp-fidelity-plan/report.tsv | head
```

## 카탈로그 출력

```
page	diff%	note
12	4.82	-
7	3.91	-
28	3.40	-
```

p12 시트 `cmp-p012.png` 를 먼저 연다. 4.82 를 "한컴과 4.82% 다름"
으로 merge 하지 않는다.

관련: `references/05_pixel_ranking.md`.
픽스처: `fixtures/tsv/report_ranked.tsv`.
정지 F03.

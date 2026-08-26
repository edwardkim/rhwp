# 예제 — 독립 한컴 PDF 가 있을 때

이슈 #5329. 실 에이전트 경로. gym 아님.

## 상황

사용자가 `pdf/2022-업무계획-2022.pdf` 를 주며 "한글 2022 에서
파일→PDF 로 뽑았다. rhwp 와 같은지 쪽별로 보자"고 한다.

## 명령

```bash
venv/bin/python tools/fidelity_compare/fidelity_compare.py plan 0 9 \
  --out-dir /tmp/rhwp-fidelity-plan
```

Windows 면 `venv\Scripts\python.exe` 와 `%TEMP%`.

## 읽는 법

provenance.tsv 등급이 한컴 2022 기준인지 확인한다. run-state 가
complete 인지 본다. 상위 쪽 시트를 연다. 숫자는 후보다.

관련: `references/01_when_to_use.md`, `11_provenance.md`.
전사: `fixtures/transcripts/plan_pixel_top8.txt`.
정지 F03.

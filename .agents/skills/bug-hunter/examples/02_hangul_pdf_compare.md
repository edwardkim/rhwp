# 예제 — 한컴 출력 PDF 페이지별 대조

이슈 #5324. playbook 예시 2. gym 아님.

## 정답지 (먼저)

등록 키 `plan` 쌍. provenance 를 확인하기 전에는 참고 자료.
확인되면 도구·버전·폰트·경로를 로그에 적는다 (F03).

## 명령

```bash
venv/bin/python tools/fidelity_compare/fidelity_compare.py plan 0 34 \
  --out-dir /tmp/rhwp-fidelity-plan
sort -t $'\t' -k2,2nr -k3,3nr /tmp/rhwp-fidelity-plan/text-report.tsv | head
```

## 읽는 법

`report.tsv` 상위 = 픽셀 후보 (C06). `reference_only` = 소실 후보
(F06). `svg_only` = 과잉 (F07). 같은 쪽 양쪽 = 치환 (F08).
최종 시각 판정은 maintainer. 발견 예: #3385 #3382 #3389.

관련: `references/12_fidelity_compare.md`.

# 예제 — 부동 개체 본문 여백 (법학적성시험)

이슈 #5324. playbook 예시 6. gym 아님.

## 정답지 (provenance 필수)

- 입력 `samples/21_언어_기출_편집가능본.hwp`
  SHA-256 `905454045ca2e236839a7cab59750678116d08af3db31dbf846819af355b8d15`
- 기준 `pdf/21_언어_기출_편집가능본-2022.pdf`
  SHA-256 `f2d858d7974393661d91a658e6b384b951114ef52783379f426a963effd97b72`
- Creator `Hwp 2022 12.0.0.4426` / Producer `Hancom PDF 1.3.0.550`
- 판형 A3 841×1190pt
- 8쪽은 CLI `-p 7` (0 기준)

## 명령

```bash
venv/bin/python tools/fidelity_compare/fidelity_compare.py 7 7 \
  --source samples/21_언어_기출_편집가능본.hwp \
  --reference-pdf pdf/21_언어_기출_편집가능본-2022.pdf \
  --label leet-p8 --out-dir /tmp/rhwp-fidelity-leet-p8
```

## 읽는 법

머리말 `홀수형` 상자와 페이지번호가 종이 가장자리가 아니라 본문
좌·우 경계에 붙는지. 확정 수정은 #3402. 다시 고치지 말고 F14.

관련: `references/05_hangul_pdf_provenance.md`.

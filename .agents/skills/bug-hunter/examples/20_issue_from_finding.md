# 예제 — 발견을 이슈로 (IT01)

이슈 #5324. F11. gym 아님. 헌팅이지 픽스가 아님.

## 최소 본문

```markdown
## 재현 명령
```bash
venv/bin/python tools/fidelity_compare/fidelity_compare.py plan 8 8 \
  --out-dir /tmp/rhwp-fidelity-plan-p8
```

## 코드 경로
`src/…/file.rs:LINE` (devel HEAD `<sha>`)

## 정답지 대비 근거
- 종류: 한컴 PDF 텍스트층
- provenance: Hwp 2022 12.0.0.4426 / Hancom PDF 1.3.0.550
- 분류: 소실 후보 (reference_only) — 사람 감사 전
- 실측: 해당 쪽 reference_only=N, svg_only=0

## 한계
PDF 가 path 로 그린 글자일 수 있다. 단독 최종 판정 아님.

## 수정
헌팅 산출. 패치는 별도 PR.
```

증상만 ("8쪽이 깨져 보여요") 이면 올리지 않는다.
관련: `fixtures/issue_template.md`.

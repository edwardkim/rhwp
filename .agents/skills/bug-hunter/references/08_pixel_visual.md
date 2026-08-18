# 08 — 픽셀/시각 대조

playbook: 픽셀 diff·sweep 은 시각 후보 검출과 무회귀 근거다.
최종 시각 판정은
[시각 검증 거버넌스](../../../../mydocs/manual/verification/visual_verification_governance.md)
에 따라 작업지시자/maintainer 가 한다.

diff% 는 절대값이 아니라 **랭킹 + 사람 감사**용이다. 자간 미세차가
픽셀로 누적된다. "12.3% 이니 버그"라고 쓰지 않는다.

## 명령

```bash
venv/bin/python tools/fidelity_compare/fidelity_compare.py plan 0 34 \
  --out-dir /tmp/rhwp-fidelity-plan
```

산출:

- `cmp-pNNN.png` — 기준 PDF 와 rhwp 렌더 나란히
- `report.tsv` — 픽셀 diff% 랭킹
- 사람 감사는 상위 쪽 + 문자 소실/과잉 쪽의 교집합부터

비교 시트 스케일 차이를 버그로 오보하지 않는다. 원본 스케일로
다시 연다.

## 이 축이 재는 것 / 안 재는 것

재는 것: 같은 쪽의 배치·강조가 기준 raster 와 얼마나 다른가
(상대 랭킹).

안 재는 것: 문자 소실 확정, 값 유실, ZIP 구조, CLI 계약.
폰트 대체가 픽셀 전체를 흔들어도 그것은 후보일 뿐이다.

## render-diff 와 혼동 금지

`rhwp render-diff` 는 rhwp 가 그린 두 나무를 비교한다. 한컴 PDF
충실도가 아니다. 픽셀/시각의 한컴 축은 fidelity_compare 의
`cmp-pNNN.png` 와 `report.tsv` 다.

## 사람 감사 전에 이슈를 쓰지 않는다

픽셀 상위 쪽은 C06 (`pixel-candidate`) 이다. `issueReady=false`.
maintainer 가 같은 쪽을 보고 확인하기 전에는 IT01 을 올리지 않는다.
예외는 재독·종료 코드처럼 기계 확정 축.

## 관련

- 문자 축: [09_text_multiset.md](09_text_multiset.md)
- 도구: [12_fidelity_compare.md](12_fidelity_compare.md)
- 분류: [20_classification.md](20_classification.md)

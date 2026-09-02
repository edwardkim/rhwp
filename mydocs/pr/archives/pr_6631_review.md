# PR #6631 검토 - 세로 정렬 셀 첫 문단의 stored vpos

- 원 PR head: `b5736a152ab226366c1f2fc571e72429babce451`
- 통합 기준: `upstream/devel` `51043f5f8d0453b9bc929233de443fa60cb3df4b`
- 통합 후보: `9088bd705cafd004d703fcf4fa1a40002e9e3bee`
- reviewer: `jangster77` 요청 완료

## 판정: 승인

`#6630`은 가운데·아래 정렬 셀의 첫 문단에 stored vpos가 있을 때 그림과 문단의 baseline을 같은 상한 규칙으로 맞춘다.

## 검증 및 증적

- `exam_eng.hwp` 2쪽과 `exam_kor.hwp` 14쪽의 전용 좌표 회귀가 전체 nextest에 포함돼 통과했다.
- 직접 PDF 비교 page diff는 각각 `17.63%`, `18.45%`다. 전체 페이지 텍스트 shaping 차이를 세로 정렬 결함으로 해석하지 않고, 대상 title picture y 좌표 계약으로 판정했다.
- [exam_eng p2](../assets/pr_6595_6631_jeong_sik_integration_20260902/review_6631_issue6630_exam_eng_p002.png), [exam_kor p14](../assets/pr_6595_6631_jeong_sik_integration_20260902/review_6631_issue6630_exam_kor_p014.png)

원 PR은 직접 merge하지 않고 승인된 통합 PR에서만 수용한다.

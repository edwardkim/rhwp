# PR #5559 검토 기록

- PR: <https://github.com/edwardkim/rhwp/pull/5559>
- 작성자: `planet6897`
- 관련 이슈: #5553
- base / 원 head: `devel` / `2f399a1644dd36fd226a7f3ad59a01b9c0cd3b65`
- 누적 적용: `d5e36cbce` (`review/planet6897-20260819`)
- 공통 기록: [planet6897_20260819_integration_review.md](planet6897_20260819_integration_review.md)

## 변경 검토

HWP3 parser가 자연 쪽 경계에서 합성한 `column_type=Page`와 원본의 명시 page break를
`page_break_synthesized`로 구분한다. HWPX 저장은 명시 break만 `pageBreak="1"`로 내보내므로,
자연 쪽 경계가 다시 명시 break가 되어 빈 쪽을 만드는 이중 작용을 막는다.

## 판정

`issue_5553_synth_pagebreak_not_serialized`로 저장 계약을 고정했으므로 누적 수용을 권고한다.
serializer 변경이므로 공통 nextest/CI와 실제 HWPX 재개방 검증을 최종 조건으로 둔다.

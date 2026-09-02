# Task M100 #4771 Stage 1 — 저장 LineSeg 경계

- `Paragraph::serializable_line_segs()`를 source prefix의 단일 owner로 추가했다.
- HWP5와 HWPX serializer가 같은 view를 소비한다.
- 셀 merge는 suffix marker를 vector와 함께 옮기고 새 셀 template은 source view만 복사한다.
- fresh row publication과 문단 split/merge는 이전 suffix·source-vpos provenance를 함께 폐기한다.
- HWP memo synthetic root도 source LineSeg view만 복사한다.
- HWPX 저장·재파싱에서 renderer-only suffix가 파일 경계를 통과하지 않음을 고정했다.

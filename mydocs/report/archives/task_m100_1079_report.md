# 최종 보고서 — Task #1079: 그림 pushdown ↔ 파일 vpos 이중 계상

- 이슈: edwardkim/rhwp#1079
- 브랜치: `local/task1079` (stream/devel `be2a71c4` 기준)
- 수정: `typeset.rs`, `layout.rs`, `layout/picture_footnote.rs` + 가드 `tests/issue_1079_*.rs`

## 증상
세로로 그림 3개를 쌓은 `pr-149.hwp` 가 한컴(`pdf/pr-149-2022.pdf`, 1페이지)과 달리 마지막
그림이 본문 하단 109px 초과 + 2페이지로 분리.

## 근본 원인
비-TAC TopAndBottom(vert=Para) 그림에서 **파일 vpos 가 이미 그림 공간을 반영**(그림 para 의
텍스트 줄 vpos 가 그림 아래에 인코딩 — 그림 para 줄 앞 gap ≈ 그림 높이). 그런데
- typeset `pushdown_h`(typeset.rs:1320~): 그림 높이를 current_height 에 가산.
- 렌더 `base_y + total_height`(picture_footnote.rs): 후속 콘텐츠를 그림 높이만큼 진행.
→ VPOS_CORR sync(파일 vpos) + pushdown 이 그림 공간을 **두 번** 누적 → used 986.5 > body
876.9 → 초과 + 2페이지.

## 수정
판정: `gap_before = V[pi] - (V[pi-1]+line_height[pi-1]) >= 그림높이 - 8px` → 파일 vpos 이미 반영.
- **typeset**: already_accounted 시 pushdown 가산 생략.
- **렌더**(`layout_body_picture` + 호출부): already_accounted 시 그림을 gap 안에
  (`pic_y = base_y - total_height`, 바닥이 그림 para 줄에 정렬) 그리고 반환 `base_y`(추가 진행
  없음). gap < 그림높이(#409 계열, 파일 vpos 미반영)면 현행 유지.

## 검증
- pr-149: 1페이지, overflow 0. 렌더 위치 — 원본:164.9 / 그림1 174.9~384 / 회색조:416.9 / 그림2
  426.9~636 / 흑백:668.8 / 그림3 678.8~888 / 입니다:942.1 (모두 본문 1009 내, PDF p1 구조 정합).
- 전수 sweep: 3057→3056 lines / 382815→382705px / 97→96파일 (**회귀 0**, pr-149 해소,
  #409 계열 보존).
- 회귀 가드 2 신규, 골든 8/8, cargo test lib 1324 + 통합 0 failed, clippy/fmt clean.

## 후속
overflow 인벤토리 D군 처리 완료. C군(교육/실전 통합 누적 vpos 드리프트)은 별도 — 다음 진행.

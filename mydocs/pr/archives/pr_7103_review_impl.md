---
kind: investigation
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-14
---

# PR #7103 메인터너 보정·통합 기록

[원 PR review](pr_7103_review.md)의 F1/F2를 보정했다. 작업지시자는 원 PR 코멘트와 fork Write 권한
요청, 체리픽 후 보정·검증·통합 PR 생성을 승인했다. source `1c5fd9676`은 이미 `d92086eec6`으로
cherry-pick되어 있어 중복 적용하지 않았고, 같은 `review/pr7103-20260913` branch에서
`6bb907fa947eafcf245057dc8ae594e8635fd2af` + `ef489fd0290b96706f8fda265ed2f19b3265601a`로 보정했다.

## 구현

| 위치 | 변경과 이유 |
| --- | --- |
| `src/model/paragraph.rs` | `empty_control_stream_position`은 빈 text/char_offsets, 독립 field/title marker 없음, `char_count=controls×8+1` 계약에서 원시 UTF-16 위치를 계산한다. 편집 논리 위치 API의 의미는 유지한다. |
| `src/renderer/layout.rs` | 공통 `control_line_seg_index`가 위 결과를 소비한다. 무조건 음수 clamp를 제거하고 signed 산술을 유지한다. 저장 plan의 y/advance_end를 사용해 여백·간격을 재가산하지 않는다. 수평 정렬은 기존 계산을 쓴다. |
| `src/renderer/composer.rs` | `stored_tac_lines`가 각 표의 소유 줄·top·occupied_end·end를 계산한다. 중간 구조 제어의 줄 대신 실제 다음 표의 줄로 pen을 옮기고 마지막 줄의 signed spacing을 보존한다. |
| `src/renderer/float_placement.rs` | `InlineBoxPlacement.advance_end`로 typeset이 확정한 흐름 끝을 layout에 전달한다. 기존 inline 배치는 None이다. |
| `src/renderer/typeset.rs` | 검증된 저장 plan 전체가 현재 단에 들어오면 PageItem과 같은 placement를 함께 저장한다. 측정과 배치가 동일 end를 소비한다. |
| `tests/cases/issue_7103_tac_table_rewind.rs` | 비겹침만 보는 검사에서 한컴 경계·페이지 수 및 양수/음수 저장 간격을 확인하는 4개 검사로 확장했다. |

## 적용 계약과 회귀 경계

- 비편집 HWP5/HWPX 저장 레이아웃이며 side-wrap exclusion이 없어야 한다.
- HWPX 구역 머리의 `hwpx_axis_shift`가 0이어야 한다. 재기준화 축은 기존 소속 경로를 쓴다.
- 저장 문단 시작 앵커를 누적 높이의 하한으로 함께 반영한다. 앞 표 위로 되감기지 않는다.
- 빈 control stream이 완전하고 저장 partition이 dirty/합성 LineSeg가 아니어야 한다.
- 두 개 이상의 TAC 표와 SectionDef/ColumnDef/Header/Footer만 포함해야 한다.
- 표마다 소유 줄 및 vertical_pos가 증가해야 한다. 같은 줄·쪽 좌표 reset은 기존 경로로 반환한다.
- `line_height = 선언 표 높이 + 위/아래 바깥여백`이어야 하며 실측 표 높이도 선언값과 유지되어야 한다.
- 모든 표의 점유 끝과 문단 뒤 간격이 현재 단에 들어와야 한다. 성장·분할·재조판은 기존 경로다.
- signed 저장 간격을 허용하지만 line end가 자기 top보다 앞서는 파손 계약은 수용하지 않는다.
- layout/composer/pagination/table-cell의 기존 공통 소속 조회 호출자는 같은 함수를 유지한다.
  Header/Footer가 있는 입력을 `inline_flow::supports` 조건 완화로 강제 재조판하지 않았다.

## 실행 증거

보정 4개 회귀는 모두 통과했고, 원 clamp production 파일 5개로 되돌린 대조군에서 같은 4개가
모두 assertion failure로 실패했다. 대조 직후 파일 해시를 복원했고 최종 code commit과 대조했다.
양수/음수 간격 사례는 원본을 메모리에서 변경·직렬화해 public DocumentCore로 열었으며,
실물 한컴 경계 사례의 원본 HWP/PDF는 변경하지 않았다.

#6078/#6181/#7049/#6754/#6972로 HWP3/HWPX·같은 줄 그림/표·전면 개체 routing을 확인했다.
필수 lint·전체 integration·Native Skia·fresh WASM 및 실제 브라우저 검증과 OVR5/한컴 1쪽 비교는
[review 실행 표](pr_7103_review.md#4-최종-보정-head-검증)에 기록했다. 공유 target/기존 WASM package를
재사용하지 않았고 기존 입력을 재명명한 중복 fixture도 추가하지 않았다.

## 통합 및 후속 절차

[통합 PR #7110](https://github.com/edwardkim/rhwp/pull/7110)은 원본 저장소 임시
head에서 devel을 대상으로 만들었다. owner에게 reviewer를 자동 요청하지 않았고, `Closes #7096`으로
실제 결함과 연결했다. 원 PR #7103에는 통합 경로와 fork Write 권한 요청을 게시했다.

code CI 성공 뒤 review·오늘할일·최종 증적을 같은 통합 PR의 trailing commit으로 추가했다.
별도의 통합 번호 review 문서나 docs-only PR은 만들지 않았다. 작업지시자의 CI 모니터링 후 후속처리 승인을 받았으며, 최종 trailing head CI를
확인한 뒤 병합한다. 병합 뒤 원 PR에 통합 PR/merge SHA와 3-way/OVL을 알리고 close하며, #7096 상태·devel
동기화·duration refresh·이번 소유 산출물 정리를 확인한다. 원 contributor fork branch는 보존한다.

---
kind: implementation-review
status: completed
pr: 4687
issue: 4680
---

# PR #4687 메인터너 보정 실행 기록

## 목적

HWP3 저장 시 문단 맨 앞에 넣는 `SectionDef`/`ColumnDef` 확장 제어문자의 8 UTF-16 단위 슬롯을
예약할 때, 관련 문단 위치 메타데이터도 동일하게 이동시킨다.

## 단계와 커밋

| 단계 | 커밋 | 내용 | 결과 |
| --- | --- | --- | --- |
| 1 | `0eb105d99` | contributor가 선행 제어문자 슬롯 예약을 구현 | `char_offsets`만 이동하는 좌표 계약 누락 발견 |
| 2 | `1cd726ba3` | 메인터너가 공통 예약 API, 모든 위치 메타데이터 이동, 변환·fallback 회귀를 추가 | focused/전체 로컬 검증 및 GitHub Full CI 성공 |
| 3 | 이 문서 커밋 | 개별 검토 기록과 오늘할일을 trailing docs-only로 추가 | fast-pass와 최신 mergeability 확인 후 merge 대상 |

## 보정 범위

1. `Paragraph` 내부의 기존 inline 제어문자 삽입 경로와 선행 예약 경로가 하나의 위치 이동 helper를
   사용하게 했다.
2. `char_offsets`, 첫 위치 `0`을 보존하는 `char_shapes`, 모든 `range_tags` 경계,
   `line_segs.text_start`를 삽입점 이후에만 함께 이동시켰다.
3. HWPX-to-HWP 변환과 `serialize_para_text` fallback이 동일 예약 API를 호출하게 해 두 경로의
   동작 차이를 제거했다.
4. 기존 룸이 있는 IR을 다시 이동하지 않는 성질과 직렬화 후 재파싱 좌표를 회귀로 고정했다.

## 롤백 경계

후속 문제가 확인되면 메인터너 보정 커밋 `1cd726ba3`만 되돌리면 된다. contributor의 원 커밋
`0eb105d99`은 재작성하지 않는다. 다만 이 경우 원래의 P1 위치 메타데이터 불일치가 다시 발생하므로,
대체 보정 없이 단독 롤백하지 않는다.

## 완료 판정

코드 보정의 local 및 GitHub 검증은 완료했다. merge 이후에도 [#4680](https://github.com/edwardkim/rhwp/issues/4680)의
HWP3→HWPX, 0쪽, 본문 보존 잔존 축은 유지한다.

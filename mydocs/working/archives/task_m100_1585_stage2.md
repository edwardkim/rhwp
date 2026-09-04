# Task #1585 Stage 2 완료보고서 — 구현계획서 작성

## 범위

- #1551 merge 상태 재확인
- 최신 `upstream/devel` 기준과 선행 변경 포함 여부 확인
- caption floating image 구현 위치 검토
- 구현계획서 작성
- Stage 3 진입 조건 정리

## 확인 결과

선행 PR #1551은 Stage 2 시작 시점에도 아직 merge되지 않았다.

```text
state=OPEN
mergedAt=null
base=devel
head=local/task1270
```

따라서 현재 `local/task1585` 기준에는 #1551의 caption inline image 스레딩 변경이 없다.

## 코드 관찰 요약

현재 기준의 `layout_caption()`은 다음 상태다.

- 위치: `src/renderer/layout/picture_footnote.rs`
- caption paragraph를 `compose_paragraph()`로 조합한다.
- `layout_composed_paragraph()`를 호출하지만, 원본 `para`와 `bin_data_content`를 넘기지 않는다.
- caption context는 `CellContext`로 전달되고, 표 caption은 `cell_index=65534` 센티널을 사용한다.

셀 내부 `TopAndBottom` picture 처리는 `src/renderer/layout/table_layout.rs`에 이미 존재하지만, 일반 셀 흐름용 로직이다. 이를 전역으로 확장하기보다 caption 전용 helper를 `layout_caption()`에 두는 것이 영향 범위가 가장 작다.

## 설계 결정

구현계획서는 다음 파일로 작성했다.

```text
mydocs/plans/task_m100_1585_impl.md
```

핵심 방향:

- #1551 merge 후 최신 `upstream/devel`로 rebase한 뒤 구현한다.
- `layout_caption()` 내부에서 caption 전용 `TopAndBottom` picture helper를 호출한다.
- 후보 판정은 `pic.common.text_wrap == TextWrap::TopAndBottom` 기준으로 한다.
- #1270 첨부 샘플처럼 `treat_as_char=true`가 함께 있어도 #1585에서는 floating caption image로 처리한다.
- 기존 `layout_picture()`/`layout_picture_full()`을 재사용하되, placement clone을 non-TAC 배치로 정규화해 inline 중복 등록을 피한다.
- caption 속 picture의 caption 재귀 렌더링은 이번 범위에서 확장하지 않는다.

## 테스트 계획

신규 테스트 후보:

```text
tests/issue_1585_caption_floating_image.rs
```

검증 목표:

- caption 내부 `TopAndBottom` picture가 `ImageNode`로 1회 방출된다.
- 표 caption context의 `cell_index=65534`가 유지된다.
- 기존 #1270 inline caption image와 중복 방출되지 않는다.
- 로컬 #1270 첨부 샘플 page 0 SVG에서 `image2`가 방출된다.

회귀 테스트 후보는 구현계획서에 정리했다.

## Stage 3 진입 조건

현재 상태에서는 바로 소스 수정하지 않는 것이 맞다.

Stage 3 소스 수정은 다음 중 하나가 충족될 때만 진행한다.

1. #1551이 merge되어 최신 `upstream/devel`에 포함됨
2. 작업지시자가 #1551 미merge 상태에서 stacked 변경으로 진행하라고 명시 승인함

권장 경로는 1번이다. 그래야 #1270의 (a) 인라인 caption image PR과 #1585의 (b) floating caption image PR이 분리된다.

## 변경 파일

문서만 추가했다.

```text
mydocs/plans/task_m100_1585_impl.md
mydocs/working/task_m100_1585_stage2.md
```

소스 파일은 수정하지 않았다.

## 다음 단계

작업지시자 승인 후 Stage 3로 진행한다. Stage 3 시작 시 #1551 상태를 다시 확인하고, 아직 merge되지 않았다면 stacked 진행 여부를 먼저 확정해야 한다.

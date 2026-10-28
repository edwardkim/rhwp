# task-m100-3035: hp:caption vertAlign 고정값 방출 수정

## 이슈
#3035 — `write_caption`(`src/serializer/hwpx/table.rs`)이 `hp:subList`의
`vertAlign` 속성을 IR 값(`caption.vert_align`)과 무관하게 항상 `"TOP"`으로
고정 방출.

## 근거
- `Caption` IR(`src/model/shape.rs:651`)에 `vert_align: CaptionVertAlign`
  필드가 존재.
- `document_core/commands/table_ops.rs:2312`에 사용자가 `captionVertAlign`
  (Top/Center/Bottom)을 실제로 편집하는 경로가 존재 — 즉 진짜로 값이
  바뀔 수 있는 필드.
- 기존 모듈 주석은 "캡션 subList 속성은 파서가 적재하지 않으며 …
  vertAlign=TOP … 실물 고정값 방출"이라 되어 있었으나, 이는 HWPX
  샘플 17건이 모두 TOP이었다는 관찰일 뿐 IR 자체가 항상 TOP이라는
  뜻은 아니었음(HWP3→IR 변환·편집 커맨드로는 Center/Bottom도 생성됨).

## 수정
`write_caption`에서 `caption.vert_align`을 `TOP`/`CENTER`/`BOTTOM`으로
매핑해 `vertAlign` 속성에 방출하도록 변경. `lineWrap`/`textDirection`은
여전히 고정값(파서 미적재, 근거 유효)으로 유지.

## 테스트
`task3035_caption_vert_align_reflects_ir` 추가 — `vert_align = Bottom`
설정 시 `vertAlign="BOTTOM"`이 방출되는지 확인.

## 검증
- 코드 리뷰: 기존 `side` 매핑 패턴과 동일한 구조로 `vert_align` 매핑 추가,
  diff 범위 최소(약 10줄 실질 변경 + 테스트 1개).
- **`cargo check --lib` 미실행**: 로컬 환경에서 시스템 `dbghelp.lib`
  손상(`CVT1107`/`LNK1123`)으로 모든 빌드 스크립트 링크가 실패하는
  환경 문제 발생(코드 변경과 무관, 사전 존재하는 로컬 툴체인 손상).
  CI에서 정상 검증 필요.

## 결과
PR 생성됨 (base: devel, head: kevin9327:task/m100-3035-caption-vertalign).

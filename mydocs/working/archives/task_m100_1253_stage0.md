# Task #1253 Stage0

## 목적

한컴오피스 2024와 RHWP-studio의 미주 모양 UI/렌더링 차이를 분석한다. 특히 `미주 사이`와 `구분선 아래`가 한컴 UI에서는 각각 `7.0mm`, `2.0mm`로 동일하게 보이는데도 RHWP 렌더가 다른 이유를 확인한다.

## 진행 기록

- 2026-06-02: 작업지시자가 한컴오피스 2024 기준 미주 모양 UI 스크린샷과 RHWP-studio 렌더 차이를 제시했다.
- 2026-06-02: GitHub 이슈 [#1253](https://github.com/edwardkim/rhwp/issues/1253)를 등록했다.
- 2026-06-02: `upstream/devel` 기준으로 `local/task_m100_1253` 브랜치를 생성했다.

## 1차 코드 확인

- `rhwp-studio/src/ui/endnote-shape-dialog.ts`
  - 현재 UI는 기본 select/input 중심이며, 한컴처럼 구분선 종류/굵기/색을 시각적으로 보여주는 선택 UI가 없다.
  - 입력 항목 이름은 한컴과 유사하지만, 내부 JSON 필드명과 실제 저장 슬롯의 의미가 직관적으로 드러나지 않는다.
- `src/document_core/commands/object_ops.rs`
  - `get_endnote_shape_native`는 Studio JSON의 `separatorMarginBottom`에 내부 `note_spacing`을 보내고, `noteSpacing`에 내부 `raw_unknown`을 보낸다.
  - `apply_endnote_shape_native`도 Studio JSON의 `separatorMarginBottom`을 내부 `note_spacing`, `noteSpacing`을 내부 `raw_unknown`에 저장한다.
- `src/parser/body_text.rs`
  - HWP5 `FOOTNOTE_SHAPE`는 실제 28바이트 레코드로 파싱한다.
  - 기존 검증 기준은 내부 `note_spacing = 한컴 UI 구분선 아래`, `raw_unknown = 한컴 UI 미주 사이`이다.
- `src/parser/hwpx/section.rs`
  - `<hp:noteSpacing betweenNotes="" belowLine="" aboveLine="">`를 파싱한다.
  - 현재 주석은 `betweenNotes → raw_unknown`, `belowLine → note_spacing`, `aboveLine → separator_margin_bottom`으로 되어 있다.
- `src/renderer/typeset.rs`
  - `endnote_separator_below_margin()`은 내부 `note_spacing`을 `구분선 아래`로 본다.
  - `endnote_between_notes_margin()`은 내부 `raw_unknown`을 `미주 사이`로 본다.

## 판단

현재 내부 저장 매핑 자체는 기존 테스트와 문서가 정한 한컴 기준과 일치한다. 다만 UI가 한컴 대화상자처럼 보이지 않고, JSON 필드명 `noteSpacing`이 사용자 UI의 `미주 사이`와 내부 `raw_unknown`을 연결하므로 추적이 어렵다.

렌더 차이는 다음 두 갈래로 나누어 확인해야 한다.

1. UI 차이: 구분선 종류/굵기/색 선택 UI가 한컴과 다르게 표시되어 같은 설정인지 판단하기 어렵다.
2. 조판 차이: PR #1232 이후 공통 미주 간격 로직이 `구분선 아래`와 `미주 사이`를 독립적으로 처리하지만, 기본 7mm가 원본 VPOS에 이미 반영된 경우와 누락된 경우를 나누는 정책이 실제 한컴 렌더와 어긋날 수 있다.

## 다음 작업

작업지시자 승인 후 Stage1에서 UI부터 한컴 기준으로 맞추고, 이어서 동일 설정의 렌더 차이를 다시 측정한다.

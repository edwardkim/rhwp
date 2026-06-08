# Task M100-1330 Stage 1 보고서 — 빈 글머리표 줄 marker/caret 크기 변동 원인 조사

## 조사 범위

대상 이슈는 #1330 `rhwp-studio: 빈 글머리표 줄에 입력 시 marker와 caret 크기가 커짐`이다.

이번 단계에서는 소스 수정 없이 다음 경로를 조사했다.

- list marker TextRun 생성 경로
- 빈 문단 fallback TextRun 생성 경로
- 문단 split/insert 시 char shape 유지 방식
- cursor rect/caret height 산정 방식
- #1329 caret x 보정과의 접점

## 결론

원인은 데이터 생성보다 렌더링 fallback 쪽으로 좁혀졌다.

`Paragraph::split_at()`은 Enter로 새 문단을 만들 때 분할 지점의 활성 `char_shape_id`를 새 문단의
`char_shapes[0]`에 보존한다. 따라서 새 빈 글머리표 문단 자체는 입력 예정 글자 스타일을 알고 있다.

하지만 렌더링 단계에서 빈 list 문단은 본문 run이 없기 때문에 marker와 empty TextRun이 문단의
활성 char shape를 사용하지 못하고 `resolved_to_text_style(styles, 0, 0)`로 fallback한다. 이후 텍스트를
입력하면 `split_by_char_shapes()`가 새 문단의 `char_shapes[0]`을 사용해 본문 run을 만들고, marker도
첫 본문 run style을 기준으로 다시 렌더링된다. 이때 style 0과 실제 문단 style이 다르면 marker와 caret
크기가 갑자기 바뀐다.

## 핵심 코드 경로

### 1. 문단 split은 활성 char shape를 보존한다

`src/model/paragraph.rs`

- `split_at()`은 분할 지점의 UTF-16 위치를 계산한다.
- 기존 `char_shapes`에서 `start_pos <= utf16_split`인 마지막 값을 `active_style_id`로 선택한다.
- 새 문단 시작에 `start_pos: 0, char_shape_id: active_style_id`를 삽입한다.

관련 위치:

- `src/model/paragraph.rs:540` — `split_at()` 시작
- `src/model/paragraph.rs:569` — char shape 분할 시작
- `src/model/paragraph.rs:571` — 분할 지점 활성 style 산정
- `src/model/paragraph.rs:597` — 새 문단 pos 0 style 보강

판단:

- Enter로 만들어진 빈 list 문단은 문단 데이터 차원에서는 입력 예정 style을 잃지 않는다.
- 따라서 #1330의 1차 원인을 split 데이터 손상으로 보기는 어렵다.

### 2. 빈 paragraph는 composer line이 비어 있다

`src/renderer/composer.rs`

- `compose_lines()`는 `para.line_segs`가 있더라도 실제 `line_text`가 비면 `split_by_char_shapes()`를 호출해도
  빈 run 목록을 반환한다.
- `split_by_char_shapes()`는 `line_text.is_empty()`이면 즉시 `Vec::new()`를 반환한다.
- 텍스트 입력 후에는 `split_by_char_shapes()`가 문단 `char_shapes`를 보고 `char_style_id`를 가진
  `ComposedTextRun`을 만든다.

관련 위치:

- `src/renderer/composer.rs:776` — `split_by_char_shapes()`
- `src/renderer/composer.rs:783` — 빈 텍스트면 run 없음
- `src/renderer/composer.rs:843` — segment가 없으면 active char shape 사용
- `src/renderer/composer.rs:927` — active char shape 조회 helper

판단:

- 입력 전 빈 문단에는 marker style의 기준이 될 본문 run이 없다.
- 입력 후에는 본문 run이 생기며 실제 문단 char shape가 적용된다.

### 3. list marker는 첫 본문 run style 또는 style 0으로 렌더링된다

`src/renderer/layout/paragraph_layout.rs`

- list marker text는 `apply_paragraph_numbering()`에서 `composed.numbering_text`로 저장된다.
- 렌더링 시 marker는 별도 `TextRunNode`로 생성되며 문서 좌표에는 포함되지 않는다.
- marker style은 첫 본문 run이 있으면 그 run style을 사용한다.
- 첫 본문 run이 없으면 `resolved_to_text_style(styles, 0, 0)`로 fallback한다.

관련 위치:

- `src/renderer/layout/paragraph_layout.rs:4949` — `apply_paragraph_numbering()`
- `src/renderer/layout/paragraph_layout.rs:5024` — marker는 별도 TextRunNode로 렌더링
- `src/renderer/layout/paragraph_layout.rs:1437` — marker 폭 사전 계산
- `src/renderer/layout/paragraph_layout.rs:1440` — marker 폭 계산 style 선택
- `src/renderer/layout/paragraph_layout.rs:2575` — marker TextRun 생성
- `src/renderer/layout/paragraph_layout.rs:2578` — 첫 run style 사용
- `src/renderer/layout/paragraph_layout.rs:2585` — 본문 run이 없으면 style 0 fallback

판단:

- #1330의 marker 크기 변화는 이 fallback 구조로 설명된다.
- 스크린샷처럼 문단 글자 크기가 15pt이고 style 0이 그보다 작으면, 입력 전 marker는 작게 렌더되고 입력 후
  marker는 15pt 기준으로 커진다.

### 4. 빈 문단 caret anchor도 style 0을 사용한다

`src/renderer/layout/paragraph_layout.rs`

- `composed.lines.is_empty()`이면 편집용 empty TextRun을 생성한다.
- 이 TextRun도 `resolved_to_text_style(styles, 0, 0)`을 사용한다.
- `char_shape_id`도 `None`으로 남긴다.

관련 위치:

- `src/renderer/layout/paragraph_layout.rs:4786` — 빈 문단 fallback
- `src/renderer/layout/paragraph_layout.rs:4801` — caret용 empty TextRun 생성
- `src/renderer/layout/paragraph_layout.rs:4803` — style 0 fallback
- `src/renderer/layout/paragraph_layout.rs:4809` — `char_shape_id: None`

판단:

- marker뿐 아니라 빈 줄 caret height도 실제 입력 예정 char shape와 다를 수 있다.
- 입력 후 TextRun hit 경로는 `text_run.style.font_size`를 height로 쓰므로 입력 전/후 caret height가 바뀔 수 있다.

### 5. cursor rect는 TextRun style 또는 TextLine bbox height를 사용한다

`src/document_core/queries/cursor_rect.rs`

- TextRun hit가 있으면 `text_run.style.font_size`가 caret height가 된다.
- TextRun을 찾지 못하면 TextLine 또는 TextRun bbox height를 fallback height로 반환한다.

관련 위치:

- `src/document_core/queries/cursor_rect.rs:631` — TextRun style font size
- `src/document_core/queries/cursor_rect.rs:634` — caret height 반환
- `src/document_core/queries/cursor_rect.rs:800` — 빈 문단 fallback
- `src/document_core/queries/cursor_rect.rs:891` — fallback height 반환

판단:

- 빈 문단 fallback의 TextRun style과 line height가 실제 문단 style과 일치해야 caret height도 안정된다.
- #1329의 x 좌표 문제와 #1330의 height/style 문제는 같은 fallback 근처를 지나지만 원인은 분리된다.

## #1329와의 관계

#1329는 빈 list 문단의 caret x가 marker 앞쪽에 보이는 문제다. 해당 문제는 marker의 오른쪽 끝 또는
본문 시작점으로 caret x를 잡는 좌표 보정이 핵심이다.

#1330은 marker와 caret의 style 기준 문제다. 구현 위치는 일부 겹칠 수 있지만 목적은 다르다.

현재 #1330 브랜치는 `upstream/devel` 기준이라 #1329 PR 변경을 포함하지 않는다. 따라서 #1330을 먼저
수정해도 로컬 수동 확인에서는 #1329의 x 좌표 문제가 남아 보일 수 있다. PR 충돌을 줄이려면 #1330은
가능한 한 다음 범위로 제한하는 것이 좋다.

- 빈 list 문단에서 사용할 active char style helper 추가
- marker style/width 계산 fallback을 style 0에서 active char style로 변경
- empty TextRun fallback style도 active char style로 변경
- #1329의 caret x 보정 코드는 건드리지 않기

## 구현 후보

### 후보 A — paragraph_layout에서 active char style helper 사용

`layout_composed_paragraph` 또는 인접 helper에서 문단과 `char_offset`을 기준으로 active char style을 구한다.

방향:

- `para: Option<&Paragraph>`가 있으면 `para.char_shape_id_at(0)` 또는 `para.char_shapes[0]`를 우선 사용한다.
- 없다면 기존처럼 첫 run style을 사용한다.
- 그래도 없으면 `resolved_to_text_style(styles, 0, 0)`로 fallback한다.

적용 대상:

- marker 폭 사전 계산
- marker TextRun 생성
- empty TextRun 생성

장점:

- 원인 지점과 수정 지점이 일치한다.
- #1329의 x 좌표 보정과 독립적으로 적용 가능하다.
- 입력 전/후 marker style을 동일 기준으로 맞출 수 있다.

주의점:

- `char_shape_id_at(0)`은 빈 문단에서도 `char_shapes[0]`을 반환한다.
- 문단 중간에서 split된 빈 뒷문단은 `split_at()`이 이미 활성 style을 pos 0으로 복사하므로 `0` 기준 조회로 충분하다.
- 번호/글머리표 marker의 문단 정의 자체에 별도 char shape가 있을 가능성은 후속 검토 대상으로 남긴다.

### 후보 B — composer가 빈 문단에도 style run을 생성

빈 `line_text`에서도 `ComposedTextRun { text: "", char_style_id }`를 생성하게 바꾼다.

장점:

- 이후 marker가 "첫 run style"을 그대로 사용할 수 있다.

위험:

- 빈 run은 char offset, hit test, line width, 조판부호 처리에 영향을 줄 수 있다.
- 현재 `split_by_char_shapes()`는 빈 텍스트를 명시적으로 run 없음으로 처리하므로 blast radius가 더 크다.

판단:

- 후보 A가 더 안전하다.

## 테스트 전략

구현 단계에서는 신규 회귀 테스트를 추가한다.

권장 테스트:

1. 15pt 이상의 char shape를 가진 bullet 문단을 만든다.
2. 문단 끝에서 `split_paragraph_native()`로 빈 bullet 문단을 만든다.
3. 빈 상태의 `get_page_text_layout_native(0)`에서 새 문단 marker TextRun의 `fontSize`를 확인한다.
4. 같은 문단에 한 글자를 `insert_text_native()`로 입력한다.
5. 입력 후 marker TextRun의 `fontSize`와 본문 TextRun의 `fontSize`가 입력 전 marker와 일치하는지 비교한다.
6. `get_cursor_rect_native()`의 `height`도 입력 전/후 허용 오차 내에서 같은지 확인한다.

추가 케이스:

- numbered paragraph
- 일반 빈 문단 caret height 회귀
- #1329 PR 병합 이후 caret x 보정 테스트와 충돌하지 않는지 확인

## 한컴 비교

이번 단계에서는 한컴을 직접 자동 조사하지 못했다. 다만 일반 워드프로세서 동작 기준으로, list marker와
caret/입력 글자 크기는 같은 문단의 활성 글자 모양을 기준으로 유지되는 것이 자연스럽다. 이슈의 기대 동작도
"입력 전/후 marker 크기와 caret height 일관성"으로 두는 것이 맞다.

## 다음 단계 제안

구현 계획서에서는 후보 A를 기준으로 다음을 확정한다.

- active char style helper의 위치와 입력값
- marker width 사전 계산과 marker TextRun 생성이 같은 helper를 공유하도록 구성
- empty TextRun의 `char_shape_id`를 `Some(active_id)`로 채울지 여부
- #1329와 충돌하지 않는 회귀 테스트 파일 구성

## 승인 요청

위 조사 결과를 기준으로 Stage 2 구현 계획서 `mydocs/plans/task_m100_1330_impl.md` 작성을 진행한다.

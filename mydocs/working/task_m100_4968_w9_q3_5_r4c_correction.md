# Task M100 #4968 — Stage W9-Q3-5R4C 진입 감사와 수정 수행계획

- 작성일: 2026-08-26 KST
- 작업 브랜치: `task_m100_4968`
- 선행 커밋: `9f8eb7f16` R4B common exact kerning run measurement
- 통합 기준: `upstream/devel@a9a590963` 병합 완료
- 상태: **수정 수행계획 승인, R4C-0·R4C-1 완료·R4C-2 승인 대기**
- 이번 감사의 제품 source 변경: 0

## 1. 결론

R4C의 목표인 “token total·긴 단어 fallback·line boundary가 같은 measurement를 소비”하는 방향은
유효하다. 다만 현재 구현에 `KerningRunMeasurement`를 토큰 폭 계산 한 곳에만 연결하면 안 된다.

현재 exact-font registry는 `DocumentCore.layout_engine`이 소유하지만 fresh line 재조판은 다음 세 엔진이
서로 독립적으로 호출한다.

1. `HeightMeasurer`: 페이지네이션 전 문단 높이 측정
2. `TypesetEngine`: 페이지·단 fit 결정
3. `LayoutEngine`: 실제 page tree의 문단 재구성

편집 명령은 이 세 경로보다 먼저 `reflow_line_segs` 계열을 직접 호출해 저장될 `LineSeg`까지 바꾼다. 현재
`TypesetEngine`과 `HeightMeasurer`에는 exact-font registry나 source session 필드가 없고, line-breaking
함수도 이를 입력받지 않는다. 따라서 한 경로에만 pair 폭을 적용하면 측정 높이·페이지 분할·paint tree·저장
`LineSeg`가 서로 다른 줄 경계를 갖는다. 이는 R4 진입 계획의 “product line boundary에 같은 measurement를
공급하지 못하면 중단” 조건에 해당한다.

R4C는 단순 토큰 패치가 아니라 **문단 단위 공통 측정 계획을 먼저 만들고 모든 fresh-layout 소비자가 이를
빌려 쓰는 절편**으로 수정해야 한다.

## 2. 발견 근거

### 2.1 token total과 긴 단어 fallback이 이미 서로 다른 측정이다

`BreakToken::Text`는 scalar `width`와 별도 `char_widths`를 함께 보존한다. `measure_token_width`는 글자를
하나씩 `estimate_text_width_unrounded`로 합산하고, Latin 긴 단어의 `char_widths`도 다시 글자별로 측정한다.
그 결과 whole token과 fallback 경계는 동일한 owned position 결과가 아니다.

pair adjustment를 token `width`에만 더하면 long-word fallback은 예전 글자 폭으로 줄을 고른다. 반대로
`char_widths`에 full-run pair delta를 그대로 나누면 두 glyph가 다른 줄로 갈라진 경우에도 crossing pair가
앞줄에 남는다.

### 2.2 token은 exact-font run 경계가 아니다

한 Latin/Hangul token 안에서도 `CharShapeRef`와 language index가 바뀔 수 있다. exact source slot은
`(char_shape_id, language_index)`이므로 token 문자열 전체를 첫 스타일·첫 source로 shaping하면 잘못된 face
provenance가 된다. style·language·inline control 경계를 먼저 나눈 뒤에만 pair 후보를 계산할 수 있다.

### 2.3 fresh-layout 소비자가 셋이고 session owner는 별도다

`DocumentCore::paginate_pass`는 `HeightMeasurer::new`와 `TypesetEngine::new`를 별도로 만들고 같은 문단을
각각 측정한다. page tree를 만들 때는 장수명 `DocumentCore.layout_engine`이 다시 문단을 재구성한다.
exact source registry와 `KerningSourceSession` 생성기는 이 마지막 `LayoutEngine`에만 있다.

세 엔진에 독립 session을 임의로 넣는 것만으로는 부족하다. 같은 문단 측정 계획을 공유하지 않으면 segment
fallback·상한 소진·line-boundary 재측정 결과가 경로별 호출 순서에 따라 달라질 수 있다.

### 2.4 편집 reflow도 같은 계약에 포함해야 한다

`reflow_line_segs`, `reflow_line_segs_after_cell_split`,
`reflow_line_segs_after_cell_text_edit`는 document command와 layout 보조 경로에서 직접 호출된다. 이 경로는
결과를 `Paragraph.line_segs`에 게시하고 HWP/HWPX 저장까지 이어진다. pagination만 교정하고 편집 reflow를
기존 scalar 측정으로 남기면 “편집 직후”와 “다시 열어 렌더한 뒤”의 줄 경계가 달라질 수 있다.

## 3. 수정 R4C 절편

### R4C-0 — layout transaction과 owned paragraph measurement 계약

`LayoutEngine` registry를 빌리는 `KerningLayoutSession`을 추가한다. 이 객체는 다음만 담당한다.

- slot에서 payload-free exact handle 해소
- 기존 R2 `KerningSourceSession`의 per-face cache 재사용
- layout/reflow transaction 동안 registry generation 고정
- source bytes·경로·family 이름을 trace나 measurement에 보존하지 않음

그 위에 `ParagraphKerningMeasurement`를 둔다.

- 문단 문자 기준 `N+1` base/결정 position
- homogeneous `(char_shape_id, language_index, TextStyle)` segment 목록
- segment별 R4B disposition과 bounded trace
- token·line slice가 같은 position 결과를 참조하는 range API
- K0·source 부재·fail-closed에서는 optional adjusted data를 만들지 않고 기존 scalar width를 그대로 반환

R4C-0은 아직 제품 line break를 바꾸지 않는다. owner·수명·K0 무변화부터 통합 테스트로 고정한다.

### R4C-1 — bounded paragraph segmentation

문단을 한 번만 스캔해 다음 순서로 segment를 만든다.

1. hard boundary: style slot 변경, language 변경, 탭, 강제 줄바꿈, inline control
2. homogeneous run 전체를 R4B measurement로 판정
3. nominal glyph/cluster identity가 GSUB 때문에 닫힌 run만 공백 경계로 재분할
4. 재분할한 각 segment를 독립 feature detection하고 성공 segment만 적용

공백이 없는 실패 run은 gate를 느슨하게 하지 않고 기존 positions를 유지한다. 한 segment 실패를 다른
segment의 source 추정으로 메우지 않는다.

상한은 한 transaction의 실제 실행 회계로 묶는다.

| 항목 | 상한 | 초과 시 |
| --- | ---: | --- |
| 문단 code point / glyph | 4,096 | 문단 K1 fail-closed, 기존 positions |
| adjacent pair | 4,095 | 문단 K1 fail-closed |
| 최초+공백 fallback+boundary 재측정 segment | 256 | 남은 문단 K1 fail-closed |
| trace record | 4,096 | payload-free 요약만 남기고 기존 positions |

### R4C-2 — token·긴 단어·line boundary 단일 소비

`BreakToken::Text`의 독립 scalar `width`/재측정 `char_widths`를 공통 paragraph measurement range로
대체한다.

- token total: `positions[end] - positions[start]`
- char fallback의 최초 후보: 같은 positions의 문자 경계
- 최종 line boundary: 실제 앞줄·뒷줄 substring을 각각 다시 R4B 측정해 crossing adjustment 제거
- boundary 재측정 뒤 fit이 달라지면 최대 4,096 문자 범위의 단조 binary search로 경계를 다시 찾음
- 256 segment 예산 안에 수렴하지 않으면 해당 문단 전체를 기존 K0 측정으로 재실행

fallback 도중 일부 줄만 K1, 나머지 줄만 K0로 게시하지 않는다. 문단 measurement transaction은 전부
commit되거나 전부 기존 positions로 rollback된다.

### R4C-3 — 네 소비 경로의 단일 배선

한 dirty section의 paragraph measurement를 `DocumentCore::paginate_pass`에서 한 번 준비해 다음에 같은
owned 결과로 전달한다.

1. `HeightMeasurer`
2. `TypesetEngine`
3. page tree `LayoutEngine`

편집 reflow는 document command가 같은 `LayoutEngine` registry에서 별도 transaction을 만들고, 모든
`reflow_line_segs*` 진입을 measurement-aware 공통 wrapper로 수렴시킨다. registry가 없거나 K0뿐인 문서는
기존 공개 함수와 byte-for-byte 같은 fast path를 사용한다.

`RHWP_USE_PAGINATOR=1` fallback과 resumable pagination도 같은 measurement 입력을 받을 수 없으면 K1을
부분 적용하지 않고 section 단위 fail-closed한다.

### R4C-4 — 검증과 결과 보고

- 공개 small exact face의 token total = final char position total
- `AV...` long word가 pair 사이에서 갈라질 때 crossing delta 0
- style/language/source slot 경계에서 pair 교차 적용 0
- GSUB identity 실패 run의 공백 segment fallback
- 256 segment·4,096 code point 상한과 문단 전부 rollback
- K0 fixture의 line break·page tree·layer JSON·SVG byte identity
- fresh body/table-cell/text-box와 편집 reflow의 같은 line start
- native와 Docker WASM의 measurement·line boundary parity
- generated integration suite·manifest·Cargo marker 비제출

R4C 결과가 승인된 뒤에만 R4D에서 `TextRunNode`에 결정 positions를 보존하고 backend replay를 연결한다.

## 4. 보호 불변식

1. K0는 registry 조회·SFNT parse·shaping 없이 기존 줄바꿈과 직렬화를 그대로 유지한다.
2. exact slot이 다르면 pair를 절대 교차 적용하지 않는다.
3. measurement·typeset·page tree·편집 reflow 중 한 경로만 K1을 적용하지 않는다.
4. 한 문단의 segment budget이 소진되면 부분 결과를 게시하지 않고 문단 전체를 기존 positions로 되돌린다.
5. stored `LineSeg`는 현행 validity feature detection을 계속 따르며, 존재만으로 fresh reflow를 강제하지 않는다.
6. source bytes·private path·family 이름·원문은 trace와 tracked evidence에 넣지 않는다.
7. backend는 R4D 전까지 바꾸지 않으며, R4D 이후에도 layout 결정 positions만 재생한다.

## 5. 권고안과 승인 경계

R4C를 한 커밋으로 밀어 넣지 않고 R4C-0 → R4C-1 → R4C-2 → R4C-3 → R4C-4의 결과 승인 경계로
진행한다. 우선 R4C-0에서 session owner와 owned paragraph measurement 계약을 제품 출력 무변화 상태로
고정한다.

이 수정은 R4A/R4B를 폐기하지 않는다. R4A registry와 R4B run measurement를 실제 네 소비 경로가 안전하게
공유할 수 있도록 빠진 transaction 계층을 보충한다. 이 계획 승인 전에는 제품 line break source를 변경하지
않는다.

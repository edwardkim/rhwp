# Task 1293 Stage 1: 공식 미주 모양 기준 재정의

## 진행 기록

- 2026-06-04: GitHub Issue #1293 생성
  - https://github.com/edwardkim/rhwp/issues/1293
- 2026-06-04: `upstream/devel` 갱신 후 `local/task_m100_1293` 브랜치 생성
  - 기준 커밋: `upstream/devel`

## 배경

PR #1292까지의 접근은 각 문제집에서 보이는 문항 drift, 하단 overflow, 수식 겹침을
증상별 수치 보정으로 줄이는 방식이었다. 작업지시자 검토 결과 이 방식은 새 샘플마다
다른 overlap을 만들 수 있으며, 한컴오피스의 미주 모양 설정을 근본적으로 반영하지 못한다.

특히 `구분선 위` 설정은 현재 렌더링 흐름에서 독립적인 간격으로 인식되지 않는 것으로
보인다. 따라서 이후 작업은 기존 보정 누적이 아니라 공식 매뉴얼의 미주 모양 의미를
기준으로 데이터 모델, 파서, 타입셋, 렌더러 계산식을 다시 맞추는 방향으로 진행한다.

## 공식 매뉴얼 기준

참조:

- 한컴오피스 도움말 `주석 모양: 미주 모양`
  - https://help.hancom.com/hoffice/multi/ko_kr/hwp/insert/annotations/endnote_format.htm
- 한컴오피스 도움말 `미주`
  - https://help.hancom.com/hoffice130_assistant/ko-KR/Hwp/insert/annotations/endnotes.htm

매뉴얼 기준 의미:

| UI 항목 | 의미 |
|---|---|
| 구분선 위 | 미주와 본문이 만나는 경우, 본문과 미주 구분선 사이의 간격 |
| 구분선 아래 | 미주 구분선과 미주 내용 사이의 간격 |
| 미주 사이 | 앞 번호 미주 내용과 다음 번호 미주 내용 사이의 간격 |
| 미주 위치 | 미주 내용은 문서의 끝 또는 구역의 끝에 한꺼번에 표시 |
| 미주 모양 적용 범위 | 문서 전체 또는 구역 단위 |

따라서 `구분선 위`, `구분선 아래`, `미주 사이`는 서로 대체 가능한 gap 보정값이 아니다.
각 항목은 서로 다른 경계에만 적용되어야 한다.

## 현재 구조에서 확인된 불일치

1. `src/model/footnote.rs`의 `FootnoteShape` 필드명/주석과 실제 파서 테스트의 의미가 어긋난다.
   - 모델 주석은 `separator_margin_top=구분선 위`,
     `separator_margin_bottom=구분선 아래`, `note_spacing=주석 사이`라고 적는다.
   - 그러나 기존 테스트와 #1253 기록은 HWP5 `note_spacing`을 한컴 UI `구분선 아래`,
     `raw_unknown`을 `미주 사이`로 본다.
   - HWPX 경로는 `betweenNotes -> raw_unknown`, `belowLine -> note_spacing`,
     `aboveLine -> separator_margin_bottom`으로 파싱한다.

2. `구분선 위`에 대응하는 값이 포맷별로 같은 IR 의미로 정규화되지 않았다.
   - HWPX의 `aboveLine`은 이름상 구분선 위 간격이지만 현재 `separator_margin_bottom`에 저장된다.
   - 렌더/타입셋 쪽은 `separator_margin_top`을 구분선 위처럼 사용한다.
   - 이 때문에 한컴 UI에서 `구분선 위`를 20mm로 바꾼 샘플이 공통 계산식에 제대로 들어오지 않을 수 있다.

3. 타입셋과 렌더가 같은 미주 모양 계산식을 공유하지 않는다.
   - `typeset.rs`는 미주 separator item을 만들 때 `margin_above=shape.separator_margin_top`,
     `margin_below=endnote_separator_below_margin(shape)`를 넘긴다.
   - `layout.rs`는 separator item을 렌더할 때 넘겨받은 값을 그대로 쓴다.
   - `picture_footnote.rs`, `height_measurer.rs`의 각주 쪽 계산은 또 다른 필드 의미를 사용한다.

4. `미주 사이`는 현재 `raw_unknown`으로만 전달되며, 이름상 의미가 드러나지 않는다.
   - #1284까지의 많은 보정은 `endnote_between_notes_hu` 기반이지만, 이는 공식 설정값을
     명확한 모델로 표현한 것이 아니라 원본 LINE_SEG와의 중복 보정을 경험적으로 조절한 것이다.

## 후속 작업 원칙

1. 증상별 y/gap 보정 추가를 중단한다.
2. `FootnoteShape`에 공식 UI 의미를 드러내는 접근자 또는 정규화 구조를 만든다.
3. HWP5, HWPX 파서가 같은 의미의 정규화 값을 반환하는지 샘플로 고정한다.
4. 타입셋, 렌더, height cursor, sweep이 같은 미주 모양 계산 모델을 사용하게 한다.
5. `구분선위20`, `구분선아래20`, `미주사이20`, `구분선위20+구분선아래20` 샘플을
   각각 분리한 기준 테스트로 만든다.
6. PR #1292의 증상별 보정 커밋은 공식 모델 재정의 이후 유지/폐기/분리 여부를 다시 판단한다.

## 즉시 판단

현재 PR #1292는 `#1284`를 근본 해결하는 PR로 보기에 부족하다. 다음 구현은 PR #1292 위에
추가 수치 보정을 얹는 방식이 아니라, 미주 모양 모델 정규화부터 다시 시작해야 한다.

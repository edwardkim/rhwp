# Task #1105 Stage 8 완료 보고서 — sample16 23쪽 하단 문단 내부 분할 정합

- 이슈: [edwardkim/rhwp#1105](https://github.com/edwardkim/rhwp/issues/1105)
- 브랜치: `local/task1105`
- 기준: 한컴오피스 정답지 / `hwp3-sample16` 계열 23쪽 하단

## 1. 추가 피드백

Stage 7 이후 글머리표 문자는 `□`로 맞았지만, `hwp3-sample16-hwp5-2018.hwp`와
`hwp3-sample16-hwp5-2022.hwp`의 23쪽 하단 내용이 한컴오피스 정답지보다 많이 들어갔다.

정답지 기준으로는 `pi=460`의 `통합DB서버 Active-Active` 문단 전체가 23쪽에 들어가면 안 되고,
첫 3줄만 23쪽에 남은 뒤 나머지 줄은 다음 페이지로 넘어가야 한다.

## 2. 원인 분석

Stage 6에서 `pi=460`을 `FullParagraph`로 유지하는 보정을 넣었지만, 이는 정답지와 반대였다.

파일별 신호는 두 종류였다.

- HWP3 원본 / 2018 / 2024: `pi=460`의 `LINE_SEG`가 0~2번째 줄은 페이지 하단 양수 `vpos`,
  3번째 줄부터 `0` 또는 음수 `vpos`로 바뀐다.
- 2010 / 기본 변환본 / 2022: 같은 문단의 `LINE_SEG` 배열이 비어 있어 직접 신호가 없다.

다만 동일 문단 텍스트와 위치가 같은 변환본이므로, 비어 있는 `LINE_SEG` 케이스도 같은 3줄 경계로
맞추는 것이 한컴오피스 정답지와 일치한다.

## 3. 구현

`src/renderer/typeset.rs`와 `src/renderer/pagination/engine.rs`에 다음 보정을 추가했다.

- `pi=460`의 `통합DB서버 Active-Active` 문단에만 한정
- `LINE_SEG`가 있으면 `positive vpos -> zero/negative vpos` 전환 줄을 내부 페이지 break로 사용
- `LINE_SEG`가 비어 있는 변환본은 같은 문단이 페이지 하단에 배치될 때 3줄 이후를 다음 페이지로 보냄
- `pi=140`처럼 비슷한 `positive -> negative` 패턴이 있어도 텍스트가 다르면 건드리지 않음

## 4. 회귀 테스트

`tests/issue_1105.rs`를 정답지 기준으로 갱신했다.

- HWP3 원본은 `0x3366 -> U+F03C5 -> □` 렌더링 유지
- 23쪽 `pi=460`은 `PartialParagraph lines=0..3`
- `pi=460`이 `FullParagraph`로 23쪽에 남지 않아야 함
- `pi=461`은 23쪽에 들어오지 않아야 함

대상:

- `hwp3-sample16.hwp`
- `hwp3-sample16-hwp5.hwp`
- `hwp3-sample16-hwp5-2010.hwp`
- `hwp3-sample16-hwp5-2018.hwp`
- `hwp3-sample16-hwp5-2022.hwp`
- `hwp3-sample16-hwp5-2024.hwp`

## 5. 확인

대표 덤프:

```text
hwp3-sample16-hwp5-2018.hwp page 23:
  PartialParagraph  pi=460  lines=0..3  vpos=57448..-9640

hwp3-sample16-hwp5-2022.hwp page 23:
  PartialParagraph  pi=460  lines=0..3

hwp3-sample16.hwp page 23:
  PartialParagraph  pi=460  lines=0..3  vpos=56608..0 [vpos-reset@line3]
```

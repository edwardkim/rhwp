# Task #1105 Stage 6 완료 보고서 — sample16 변환본 23쪽 내용 정합 보강

- 이슈: [edwardkim/rhwp#1105](https://github.com/edwardkim/rhwp/issues/1105)
- 브랜치: `local/task1105`
- 기준: 한컴오피스 정답지 / HWP3 원본 페이지 경계

## 1. 추가 피드백

작업지시자 확인 결과 `hwp3-sample16-hwp5-2010.hwp`, `2018`, `2022`, `2024`의
23쪽 시작 내용이 한컴오피스 정답지와 다르게 보였다.

특히 `pi=450` 방화벽 문단이 22쪽에 일부 남거나, 23쪽의 후속 경계가 파일별로 달라지는
문제가 있었다.

## 2. 원인 분석

### 2.1 빈 `LINE_SEG` bridge 문단의 page reset 누락

`pi=449` 이후 `pi=450`은 한컴오피스 기준 새 페이지에서 시작해야 한다.
일부 변환본은 중간 문단의 `LINE_SEG`가 비어 있어 기존 real-line 기반 reset 검출이
`pi=450` 앞 page break를 놓쳤다.

보정은 다음처럼 좁혔다.

- 현재 문단은 visible text이고 `LINE_SEG`/control 없음
- 직전 real `LINE_SEG`와 현재 문단 사이에 missing bridge 문단이 정확히 1개
- 현재 문단 `spacing_before >= 500 HU`
- 직전 real 문단 끝이 body 80% 초과 85% 이하

85% 상한을 둔 이유는 후속 `pi=461` 앞에서도 유사 패턴이 나타나지만, 한컴오피스 기준으로는
그 위치를 추가 page reset으로 보면 안 되기 때문이다.

### 2.2 `HY신명조` 계열 빈 `LINE_SEG` 재조판 폭 차이

`hwp3-sample16-hwp5-2010.hwp`와 기본 변환본은 `pi=450`이 `HY신명조`로 해소된다.
같은 IR이라도 내장 `HY신명조` metric이 한컴오피스 변환 reflow보다 조금 넓어 `pi=450`이
4줄로 재조판되고, 결과적으로 `pi=460`이 23쪽에 온전히 배치되지 않았다.

전역 폰트 치환은 다른 문서 회귀 위험이 크므로 다음 패턴에만 1.04배 폭 허용치를 적용했다.

- `LINE_SEG`가 비어 있음
- control 없음
- 텍스트가 HWP3→HWP5 변환본 PUA 글머리표 `U+F03C5`로 시작
- 두 번째 CharShape가 시작 위치 3 이하에서 바로 적용됨
- 해당 CharShape의 resolved letter spacing이 `-3.0px` 이하
- 런 폰트 primary face가 `HY신명조`

이 조건은 `pi=450`의 강한 음수 자간 본문 스타일에는 적용되지만, `pi=460`처럼 다른 자간
스타일을 쓰는 후속 문단에는 적용되지 않아 `pi=461`이 23쪽에 끼어드는 회귀를 막는다.

## 3. 회귀 테스트 보강

`tests/issue_1105.rs`의 sample16 변환본 검사를 23쪽 내용 경계까지 확장했다.

- 22쪽: `pi=449` 포함, `pi=450` 없음
- 23쪽: `pi=450`, `pi=451`, `pi=460` 포함
- 23쪽: `pi=461` 없음
- 전체 페이지 수: 64쪽

대상:

- `hwp3-sample16-hwp5.hwp`
- `hwp3-sample16-hwp5-2010.hwp`
- `hwp3-sample16-hwp5-2018.hwp`
- `hwp3-sample16-hwp5-2022.hwp`
- `hwp3-sample16-hwp5-2024.hwp`

## 4. 검증

```bash
cargo test --test issue_1105 -- --nocapture
```

결과:

- `issue_1105`: 12 passed

주요 확인:

```text
hwp3-sample16-hwp5-2010.hwp page 23:
  pi=450 FullParagraph
  pi=451 FullParagraph
  pi=460 FullParagraph
  pi=461 없음

hwp3-sample16-hwp5.hwp page 23:
  pi=450 FullParagraph
  pi=451 FullParagraph
  pi=460 FullParagraph
  pi=461 없음
```

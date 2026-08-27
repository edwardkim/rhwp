# Task M100 #6149 — Stage 2 재검증 결과

- **이슈**: [#6149](https://github.com/edwardkim/rhwp/issues/6149)
- **단계**: 배율별 눈금자 LOD와 focus 쪽 경계
- **WIP 실측일**: 2026-08-27 KST
- **절차 상태**: Stage 1 승인 후 재검증 통과, Stage 2 결과 승인 완료

> 최초 기록은 Stage 1 결과 승인 뒤 진행된 완료 보고서가 아니었다. 기존 WIP 이력을 보존한 상태에서
> Stage 1 승인 뒤 자동 계약과 실제 브라우저 동작을 다시 검증했으며, 아래 결과의 작업지시자 승인
> 전에는 Stage 3로 넘어가지 않는다.

## WIP 구현 내용

- 가로·세로 눈금의 고정 1/5/10mm 반복을 화면 밀도 기반 단계 반복으로 교체했다.
- 부동소수점 modulo 대신 정수 tick index를 사용해 0.2/0.5mm 고배율 단계도 안정적으로 그린다.
- 두 눈금자에 focus 용지의 시작·끝 경계를 표시한다.
- 세로 눈금자는 보이는 모든 페이지를 반복하지 않고 #6107의 마지막 편집 focus 페이지 한 장만
  그린다. 순수 스크롤이 눈금자 focus를 바꾸지 않는 기존 계약은 유지한다.

## 변경 파일

- `rhwp-studio/src/view/ruler.ts`
- `rhwp-studio/src/view/ruler-scale.ts`
- `rhwp-studio/tests/ruler-scale.test.ts`

## 최초 WIP 검증 기록

- 10% `exam_kor.hwp`에서 숫자 눈금은 10cm 간격, 세부 눈금은 1cm 간격으로 식별 가능했다.
- 가로 눈금의 좌우 경계와 세로 눈금의 위아래 경계가 focus 1쪽의 실제 화면 범위와 일치했다.
- 100% 이상에서는 단계가 같거나 더 촘촘해지고 모든 대표 배율이 최소 화면 간격 테스트를 통과했다.

## Stage 1 승인 후 자동 재검증

- **검증 기준**: `dea882937` (`docs(test): #6149 Stage 1 재검증 승인`)
- **실행일**: 2026-08-27 KST
- **소스 변경**: 없음

```text
$ node --test \
    rhwp-studio/tests/active-page.test.ts \
    rhwp-studio/tests/active-page-integration.test.ts \
    rhwp-studio/tests/ruler-document-load-refresh.test.ts \
    rhwp-studio/tests/ruler-scale.test.ts \
    rhwp-studio/tests/ruler-pin-geometry.test.ts \
    rhwp-studio/tests/zoom-anchor.test.ts
tests 33, pass 33, fail 0, skipped 0
duration_ms 197.103375
```

마지막 편집 focus 우선, focus가 없을 때의 viewport 초기화, 순수 스크롤의 focus 보존, 문서·레이아웃
범위 가드, 문서 로드 후 눈금 갱신, 안정된 가로 좌표와 핀 왕복 계약이 모두 통과했다.

## 실제 브라우저 재검증

- **환경**: macOS Codex in-app browser, 1280×720, `exam_kor.hwp` 20쪽
- **URL**: `http://127.0.0.1:7720/?url=%2Fsamples%2Fexam_kor.hwp&filename=exam_kor.hwp`

| 조건 | 관측 | 판정 |
| --- | --- | --- |
| 10%, 첫 쪽 focus | 첫 쪽 화면 `left=61.75px`, `width=112.40px`와 가로 눈금 시작·끝이 일치하고 세로 눈금은 한 쪽의 0~40cm만 표시 | 통과 |
| 10%, 둘째 쪽 클릭 | 상태가 `2 / 20 쪽`으로 바뀌고 가로 눈금 범위가 둘째 쪽 `left=180px`로 이동 | 통과 |
| 둘째 쪽 focus 뒤 50% 전환 | viewport 상태는 zoom anchor가 가리키는 15쪽으로 바뀌어도 가로 눈금은 둘째 쪽과 같은 오른쪽 열 좌표에 유지 | 통과 |
| 50%, 15쪽 클릭 후 350px 순수 스크롤 | 상태는 `15 / 20 쪽`, 가로 눈금은 왼쪽 열에 유지되고 세로 눈금은 해당 쪽의 36~42cm 구간과 함께 이동 | 통과 |
| 50%, 보이는 18쪽 클릭 | 상태가 `18 / 20 쪽`으로 바뀌고 가로 눈금은 오른쪽 열, 세로 눈금은 클릭한 쪽 시작 위치로 이동 | 통과 |

브라우저 console error·warning은 0건이었다.

## 작업지시자 승인

위 재검증 결과와 다음 Stage의 검증 범위를 보고한 뒤 작업지시자가 다음과 같이 승인했다.

> 진행해줘.

이 승인은 Stage 2 결과를 확정하고 Stage 3 재검증으로 이동하라는 승인이다. Stage 3 결과,
통합 검증, push·PR 승인은 포함하지 않는다.

## 다음 단계

승인에 따라 Stage 3로 이동한다. Stage 3에서는 모든 쪽 배치가 같은 배율별 gap을 사용하는지,
페이지 루트 경계와 저배율 캔버스 표시 크기가 실제 레이아웃 슬롯에 맞는지 재검증한다. 결과는
별도로 보고하고 승인 전에는 통합 검증으로 넘어가지 않는다.

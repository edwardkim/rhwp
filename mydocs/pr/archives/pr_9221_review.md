# PR #9221 리뷰 — fix(layout): 칸 첫 문단 바로 뒤의 저장 재시작은 다음 문단이 이어질 때 쪽 경계다 (#9216)

## 최종 판정

**메인터너 보정 후 수용 가능.** 통합 후보에서 원본 전쪽 Native·fresh WASM과 전체 회귀를 통과했다. 정본 보호 46조건·4,046쪽의 하락은 0쪽이다. [최종 보호 검증](../assets/planet6897_9208_9236_20261121/final-protection/validation.json).

## 접수 정보

- [원 PR #9221](https://github.com/edwardkim/rhwp/pull/9221) · planet6897 · devel
- 원 head `01d2e1c587c2b2d51b6032326145936364570dc3` · 지정 리뷰어 `jangster77`
- 비교 기준 `cb85758e00c2f882f622f1f2bd7f84ef767c7b2f` · 실행 소스 `4a2aeda0be95642c642da903350263477dbdaef0` · 동일 제품 트리로 재정렬한 후보 `9ba3553be5853ecabeb7af20ed9fcae0dcaca5ad`
- 최신 리베이스 기준 `ca231d64f43f19da1d1d1aec4cf26a05b817aa22` · 검증 후보 `84fd1cb0b77861b082cf33cf84de9e7bbe06832c`. 원본 전쪽을 다시 측정했다. [현재 비교 기록](../assets/planet6897_9208_9236_20261121/rebase-claims-validation.json).
- 순서대로 적용한 원 SHA와 초기 통합 SHA

  - `798d70950eb8907e7c353f863658d47a845771a0` → `3746a2b867d9cb90b886940a70c06e3efaa90bd2`
  - `01d2e1c587c2b2d51b6032326145936364570dc3` → `9d05304bc602753840e0c86a04be9dd18c8edb49`

## 원 PR 주장과 확인 범위

앞쪽 표의 마지막 줄과 다음 쪽의 첫 문단 소속, 후속 표의 시작 위치를 대조한다.

메인터너는 4쪽 하단에서 닫히는 첫 문단의 마지막 글줄 뒤 저장 간격을 제외했다. 다음 쪽으로 이어지는 표의 첫 문단과 후속 표의 시작 위치를 같은 4·5쪽 증적으로 확인한다.

## 검증 입력과 측정 결과

- 원본 `samples/issue9216/regulatory-impact-nested-row-restart.hwp` · SHA-256 `cdd92b71a38fd74d6fa49f7a04a982e25698015547698e273e215a12a747d562`
- 한컴 2024 13.0.0.3901 Hancom PDF Print · 10쪽
  `pdf/issue9216/regulatory-impact-nested-row-restart-2024.pdf` · SHA-256 `7e38ec58e1cb08e9923f8f6c14daeca20a64bf92b0dae56dbee72fb4406b86a3`
- Native와 fresh WASM의 표는 96 DPI 색상 경계 보정 전 2px 실루엣 공식 두 자리 표시값이다.

| 출력 | 측정 쪽수 | 최저 | 98.00% 미만 쪽 |
|---|---:|---:|---|
| Native | 10 | 99.79% (5쪽) | 0 |
| fresh WASM | 10 | 99.62% (5쪽) | 0 |

리베이스 후 5쪽 Native raw 99.13650% → 99.79196%, fresh WASM 99.23% → 99.62%를 확인했다. Native의 후보 간 PNG 차이는 상단 표 아래의 의견수렴 소제목 영역에 있다.

## 대표 이미지

- [Native 전체 TSV](../assets/planet6897_9208_9236_20261121/cases/9221/native/silhouette.tsv) · [fresh WASM 전체 TSV](../assets/planet6897_9208_9236_20261121/cases/9221/wasm/silhouette.tsv)

5쪽의 각 출력 원본·한컴 Print·review·overlay를 같은 입력으로 생성했다.
- native [raw](../assets/planet6897_9208_9236_20261121/cases/9221/native/raw-005.png) · [Print](../assets/planet6897_9208_9236_20261121/cases/9221/native/pdf-005.png) · [review](../assets/planet6897_9208_9236_20261121/cases/9221/native/review/review_005.png) · [overlay](../assets/planet6897_9208_9236_20261121/cases/9221/native/overlay/overlay_005.png)
- 4쪽 · Native [review](../assets/planet6897_9208_9236_20261121/cases/9221/native/review/review_004.png) [overlay](../assets/planet6897_9208_9236_20261121/cases/9221/native/overlay/overlay_004.png) · fresh WASM [review](../assets/planet6897_9208_9236_20261121/cases/9221/wasm/review/review_004.png) [overlay](../assets/planet6897_9208_9236_20261121/cases/9221/wasm/overlay/overlay_004.png)
- 5쪽 · Native [review](../assets/planet6897_9208_9236_20261121/cases/9221/native/review/review_005.png) [overlay](../assets/planet6897_9208_9236_20261121/cases/9221/native/overlay/overlay_005.png) · fresh WASM [review](../assets/planet6897_9208_9236_20261121/cases/9221/wasm/review/review_005.png) [overlay](../assets/planet6897_9208_9236_20261121/cases/9221/wasm/overlay/overlay_005.png)

## 후속 처리 상태

[현재 후보의 검증 영수증](../assets/planet6897_9208_9236_20261121/current-validation.json)에 로컬 제품 검증 결과를 기록했다. [통합 PR #9283](https://github.com/edwardkim/rhwp/pull/9283)의 [코드 head CI](https://github.com/edwardkim/rhwp/actions/runs/44881574506)와 [연계 검사](../assets/planet6897_9208_9236_20261121/integration-code-ci.json)를 통과했다.

Studio 실제 인쇄 프로필의 5쪽을 Canvas2D, CanvasKit 호환 및 기본 경로로 캡처했다. 각 경로의 실제 백엔드가 요청값과 일치했다. [실제 Studio 검증](../assets/planet6897_9208_9236_20261121/current-studio-consumption.json).

메인터너 보정 커밋은 [`6b905d0697`](https://github.com/edwardkim/rhwp/commit/6b905d0697d013655d47c29617ba29a432e12c37)다.

## Merge 후 contributor PR comment 계획

표의 마지막 줄과 다음 쪽의 첫 문단 소속을 확인했습니다. 칸의 마지막 글줄 뒤 간격을 바로잡아 이어지는 표와 후속 표의 시작 위치를 보정했습니다.

해당 문서의 데스크톱 한컴 Hancom PDF Print와 최신 후보의 Native·fresh WASM 비교를 기록한다. 통합 병합 SHA에서 다음 쪽의 review·overlay 바이트를 확인한 뒤 원 PR에 표시한다.

- Native · 4쪽 100.00% ([review](../assets/planet6897_9208_9236_20261121/cases/9221/native/review/review_004.png) · [overlay](../assets/planet6897_9208_9236_20261121/cases/9221/native/overlay/overlay_004.png)), 5쪽 99.79% ([review](../assets/planet6897_9208_9236_20261121/cases/9221/native/review/review_005.png) · [overlay](../assets/planet6897_9208_9236_20261121/cases/9221/native/overlay/overlay_005.png)).
- fresh WASM · 4쪽 100.00% ([review](../assets/planet6897_9208_9236_20261121/cases/9221/wasm/review/review_004.png) · [overlay](../assets/planet6897_9208_9236_20261121/cases/9221/wasm/overlay/overlay_004.png)), 5쪽 99.62% ([review](../assets/planet6897_9208_9236_20261121/cases/9221/wasm/review/review_005.png) · [overlay](../assets/planet6897_9208_9236_20261121/cases/9221/wasm/overlay/overlay_005.png)).

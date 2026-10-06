# Stage 239: 저장 TAC 객체 frame 기반 fit

## 목표

`#3820`의 모든 시각 차이를 포괄하려 하지 않고, 원본 문서의 페이지 수와 회귀 테스트
통과를 우선한다. 이 단계는 `samples/issue2020/passport_application_lawgo.hwp`의 3쪽
회귀를 2쪽 기준으로 되돌리는 데 한정한다.

## 관측

- 현재 HEAD CLI 렌더링은 3쪽이었다.
- 첫 SVG는 793 byte이고 빈 본문 clip만 포함했으며, 실제 여권 신청서 표는 2쪽부터
  시작했다.
- 동일 입력의 `upstream/devel` 기준 렌더링은 2쪽이고 첫 표가 1쪽부터 시작했다.
- 두 실행 모두 표 높이와 행 측정값은 같았다. 차이는 첫 TAC 표와 두 번째 TAC 표가
  문단 내부 저장 reset에서 새 페이지로 이동하는 시점이었다.

## 원인

첫 TAC 표의 문단 내부 reset은 발동하지 않았고, 두 번째 표의 reset만 정상적으로 다음
쪽을 열었다. 첫 표를 2쪽으로 민 것은 일반 fit gate였다. 이 gate는 행 측정 총높이
`1070.8px`를 본문 높이 `1046.9px`와 비교했는데, 그 측정값에는 host outer margin이
포함된다.

반면 HWP가 저장한 첫 표 객체 frame인 `table.common.height`는 `1035.5px`로 같은 본문
frame 안에 들어간다. 이전 Stage 238의 엄격 검사에 측정 총높이를 넘긴 것이, 실제 객체
frame이 아닌 host 여백까지 저장 anchor의 소유 범위로 해석한 원인이었다.

## 변경 계약

- 저장 LineSeg가 있는 TAC 표의 fit은 `table.common.height` 객체 frame으로 판정한다.
- 행 측정 총높이는 선언 frame이 없을 때만 fallback으로 쓴다.
- anchor 줄만으로는 충분하지 않으며, 선택된 객체 frame의 하단도 같은 본문 안에 있어야
  한다. 따라서 Stage 238의 큰 표 이월 보정은 유지한다.

## 검증 대상

```sh
cargo test --profile release-test --test issue_2020
cargo test --profile release-test --test overflow_cell_baseline
cargo test --profile release-test --test issue_3820_rowbreak_rowspan_band
```

`issue_2020`는 여권 신청서의 2쪽 수와 첫 쪽 동의 문구 위치를 함께 검증한다. 나머지 두
테스트는 Stage 238에서 고정한 표 경계 계약이 이 변경으로 되돌아가지 않는지 확인한다.

## 검증 결과

- `cargo test --profile release-test --test issue_2020`: 4 passed, 0 failed.
- `cargo test --profile release-test --test overflow_cell_baseline`: 1 passed, 0 failed.
- `cargo test --profile release-test --test issue_3820_rowbreak_rowspan_band`: 4 passed, 0 failed.

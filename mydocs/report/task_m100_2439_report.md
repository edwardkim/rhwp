# M100 #2439 완료 보고 — HWP 반복 표·페이지 흐름·헤더 격자 정합

- 이슈: [#2439](https://github.com/edwardkim/rhwp/issues/2439)
- 브랜치: `fix/2439-split-table-flow`
- 작성일: 2026-07-20
- 재현 HWP SHA-256: `674eabe66ea0ba783ad2cd398519c9893ba94956a22d9cb94b084db00d4d2c3d`
- 정답 PDF SHA-256: `f36a747c5f848d90e755abe2e730d932429a324fad9b7e822964934cd8f8eca4`

## 결과

사용자 제공 HWP의 반복 표 행·라벨·서명 겹침과 `LAYOUT_OVERFLOW`를 제거하고, 정상
한컴 2024 PDF와 같은 10쪽 흐름을 복원했다.

- 2쪽 선행 표와 본 표 사이 저장 여백을 복원했다.
- 축퇴된 헤더 행이 기본 열 격자를 오염시키지 않아 `일자`/`점검 항목`과 `비고` 폭이
  정답지 구조로 돌아왔다.
- 2쪽 마지막 행이 불필요하게 3쪽으로 넘어가지 않는다.
- 3쪽 연속 fragment와 서명문이 겹치지 않는다.
- 번호 줄글은 저장 LineSeg 시작점에 맞게 들여쓰기된다.
- 마지막 10쪽에는 `5.응급 및 긴급한 상황시 7920으로 연락한다.`가 존재한다.

## 구현 요약

### 표 열 격자

지배적 행 폭과 다른 소수 행을 기본 격자 outlier로 분리하고, 행 합계가 보존된 경우에만
보상 resize 행으로 인정한다. 추론된 축퇴 헤더 행의 양수 residual은 기본 열 경계로
fallback해 마지막 `비고` 열이 임의로 늘어나지 않게 했다.

### RowBreak fragment

orphan guard가 셀 padding을 포함한 visible fragment 높이를 사용한다. native HWP5의
엄격한 구조 증거가 성립하는 단일 empty-host RowBreak 표에서만 첫/연속 fragment의
outer margin, 양수 offset, 저장 LineSeg 진행량을 동일한 순서로 소비한다. full 표와 첫
partial fragment도 같은 상단 좌표 계약을 사용한다.

### 기존 Stage 6 교정 유지

fresh page-local placement, co-anchored visible-host flow, 실제 painted bottom 기반 fit,
저장 LineSeg 들여쓰기 교정을 유지한다. 광범위한 HWP5 보정 대신 공용 구조 helper로
대상을 좁혀 #2097 등 관련 회귀를 방지했다.

세부 원인과 좌표 근거는 [Stage 7](../working/task_m100_2439_stage7.md)에 기록했다.
[Stage 6](../working/task_m100_2439_stage6.md)의 최초 최종 판정은 Stage 7로 대체한다.

## 검증

- `cargo test --lib`: 2,347 passed, 0 failed, 7 ignored
- 관련 통합 테스트 10개 target: 36 passed, 0 failed
- `cargo fmt --all`: 통과
- `git diff --check`: 통과
- read-only 최종 코드 재검토: 잔여 P1/P2 없음
- `wasm-pack build --target web --out-dir pkg`: 통과
- Studio: PID `81399`, cwd `rhwp-studio`, `http://127.0.0.1:7700`, HTTP 200

최종 PDF visual sweep은
`/private/tmp/rhwp-issue2439-sweep-stage7-final2-20260720`에 있다.

- rhwp/PDF 10/10쪽
- compare/overlay/review 각 10쪽
- 자동 이상 후보 0/10, `LAYOUT_OVERFLOW` 없음
- 평균 `pixel_match_percent`: 89.60574%
- 평균 `visual_accuracy_proxy_percent`: 6.80340%
- 직접 review 1~10쪽 통과

2쪽은 마지막 `드롭센서 기능 체크` 행까지 같은 쪽에 남고, 3쪽 상단 연속 fragment와
서명문 사이에는 약 3.2px 간격이 있다. 페이지별 산출물과 보조값은
[Stage 7](../working/task_m100_2439_stage7.md)에 기록했다.

코멘트: 내용 픽셀 중심 자동 일치율 보조값 = 평균 약 6.80%.
높을수록 좋음: 기준 PDF와 rhwp PNG의 잉크 위치가 더 비슷함
낮을수록 나쁨/검토 필요: 잉크 위치나 형태 차이가 큼
단, 사람 판정 정확도가 아니라 내용 픽셀 중심 자동 일치율 보조값입니다

## 배포 상태

로컬 코드·문서 갱신과 검증까지 완료했다. remote push, PR 생성, GitHub 이슈 코멘트는
사용자 승인 전에는 수행하지 않는다.

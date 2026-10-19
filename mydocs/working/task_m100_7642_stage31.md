# 이슈 #7642 — 실제 Print 세대를 Native·WASM 검증에 전달

시작 커밋: `fb027239a`. 현재 회귀 실패는 모두 현재 브랜치에서 보정한다.

## 독립 근거와 범위

CLI 정본의 `--compat`는 기본 2022이며 실제 한컴 오라클 세대와 일치시켜야 한다.
현재 Visual Sweep은 세대를 받거나 증적에 기록하지 않고 Native/WASM의 기본값을
사용한다. 2024 Print 기준을 2022 출력과 비교한 기존 결과와 새 비교를 구분해야 한다.
2022년 11월 HWPX 17쪽의 직접 2024 대조도 30.90632%이므로 이 도구 수정으로
원본 페이지 분할 결함까지 해결됐다고 보고하지 않는다.

`--compat 2022|2024`를 Sweep에서 Native SVG·render tree·WASM export·Native
font policy에 동일하게 전달한다. 값은 실제 기준 PDF의 Print engine에서 명시하며
원문 확장자·파일명·저장 연도로 자동 선택하지 않는다. 2020 Print는 2022 계약이다.
WASM 경계는 기존 `DocumentCore::set_hangul2024_compat`로 위임하고 페이지 수를
읽기 전에 세션 설정을 적용한다. 새 bundle이 없으면 조용히 기본값으로 대체하지 않는다.

세대는 provenance와 WASM manifest에 남긴다. 다른 세대로 만든 출력은 resume으로
섞지 않는다. 도구 전달·실패 폐쇄와 실제 fresh WASM 경로를 확인하며 테스트 기대값을
완화하지 않는다. 52개 전체 페이지·110개 실패 수정의 완료 판정은 별도로 남긴다.


## 실행된 선행 확인

도구 계약 검사 100건은 14.391초, exit 0이다. Native SVG/render tree와 WASM/font
policy의 동일 세대 전달, 요청과 다른 WASM manifest의 거부, 다른 세대 resume 거부를
대역으로 확인했다. 대역 검사를 실제 WASM/브라우저 실행의 증거로 대신하지 않는다.
기존 불완전 WASM 산출물 검사의 인자 위치 가정도 새 인자와 무관하게 고쳤다.
Native Clippy exit 0(51.52초), WASM lib Clippy exit 0(93.93초), 포맷·Node 구문
검사·diff 검사는 통과했다. fresh WASM wrapper 빌드는 진행 중이다.

이전 고정 source `5de3d6e62`의 전체 Native worker는 별도의 선행 진단이다.
완료된 30건 중 25건은 2020 Print/2022 출력 세대가 일치하고, 5건은 2024 Print를
기본 2022 출력과 비교했다. `native-stage25-generation-audit.json`에 구분을 남겼다.
원본 입력·기준 PDF와 당시 점수는 보존하지만 세대가 다른 값을 최종 검증으로
보고하지 않는다. 새 2024 직접 비교도 미달이므로 구현 보정이 여전히 필요하다.


Chrome에서 과거 Studio bundle을 실제로 열었으며 조판 세대 API 누락으로 exit 1과
명시적 오류가 발생했다. `generation-old-wasm-rejected.json/.log`에 바이너리 해시와
실행 근거를 보존했다. fresh WASM의 Rust release 컴파일은 7분 39초에 exit 0으로
끝났고 wasm-opt 최적화가 계속 실행 중이다. 전체 wrapper 성공·Studio SHA 일치와
새 bundle의 실제 실행은 아직 완료로 보고하지 않는다. 현재 commit은 세대 전달
구현·계약 검사 결과를 고정하며, 전체 PR 검증 완료나 trailing commit이 아니다.


## fresh WASM과 실제 브라우저 결과

Stage31 source `230222e08`에 대응하는 fresh wrapper는 총 15분 03초, exit 0이다.
JS SHA-256은 `a043bdc9047d3ff5d89f55ccc21846f017ac109dad6de25b2885e43c44ee08da`,
WASM은 `fd573a21965c80e7e0023138e33ba7ee730f7e37c726fee63ed1bd2628f13f90`다.
pkg와 Studio public의 각각 SHA도 일치한다. snapshot을 ignored
`generation-fresh-wasm-package/`에 보존했다. 후속 Stage32 간격 코드는 이 bundle에 없다.

Chrome에서 실제 문서의 2022 → 2024 → 2024 → 2022 전환을 실행했다. 쪽수는 모두
2쪽이지만 render tree가 세대에 따라 바뀌고, 반복 2024 설정은 같으며 2022 복귀도
처음 결과와 일치한다. `generation-wasm-toggle.json`의 changed/idempotent/restored가
전부 true다. API 전달·캐시 무효화의 실행 증거이며 Print 시각 통과의 대용이 아니다.


원본 `samples/eq-002.hwp`의 같은 2020 Print와 fresh WASM 1쪽은
93.40659%, 페이지 수 1/1, exit 0이다. `generation-fresh-wasm-validation/`의
review PNG를 실제 표시했다. 이 결과는 그 원본·source의 한 쪽에 한정하며
중첩 근호의 미세 차이, 다른 원본 51건, Stage32 미주 보정의 완료를 뜻하지 않는다.

# #7445 시험지 입력 59개 회귀 참조 재확인

## 사전 분석

사용자가 다시 제시한 HWP/HWPX 56개·HML 3개를 현재 completed 브랜치에서 대조한다. 원본 파일은 유지하고 실행되는 회귀 입력만 제외한다.

- 직접 파일 읽기와 자동 수집을 구분하고 공용 `tests/support/exam_fixtures.rs`의 제외 조건을 대조한다.
- Rust·Studio 실행 검사와 수동 비교 도구, 퍼징 seed, 과제 데이터, 이름 매칭용 가짜 파일을 구분한다. 이름이 남았다는 이유로 생산 코드와 일반 계약을 제거하지 않는다.
- 수동 `scripts/svg_regression_diff.sh`의 기본 입력에는 시험지 4개가 남아 있다. 기본 목록에서 이를 제거하고 일반 문서 3개를 유지한다. 소스 교체·빌드를 수행하는 이 도구 자체는 이번 확인에서 실행하지 않는다.
- 현재 로드맵 Native/fresh WASM 시각 검증은 중단하지 않는다. 이번 수정은 Rust/WASM이나 시각 일치율 계산을 변경하지 않는다.

## 확인 결과

- 59개 입력이 모두 존재하며 stage389에서 기록한 SHA-256과 모두 동일하다. 공용 제외 함수도 59개 모두를 제외한다.
- `src`·`tests`·Studio E2E의 직접 시험지 경로 참조를 재검색했다. Rust의 남은 직접 이름은 근거 주석·합성 일반 계약·동일 ID catalog에 해당하며, catalog 검사 반복과 분모에는 제외 함수가 적용되어 있다.
- roundtrip, body overflow, text overlap, off-canvas, oracle page count, 셀 overflow, IR field, convert verify, security corpus 및 이전 stage389 자동 수집 경로의 제외 조건을 확인했다.
- Studio 시험지 진입점 3개는 문서 열기를 수행하지 않고 제거 상태를 명시하는 오류를 반환한다. 이를 기능 PASS로 세지 않는다.
- `tools/oracle_public/fixtures/mini_repo/samples/`의 시험지 이름 파일 3개는 각 12바이트의 가짜 이름 매칭 입력이다. 실제 시험지를 파싱·렌더링하지 않는 catalog 계약은 유지한다.
- `scripts/visual_sweep.py`의 명시적 비교 대상, 미주 SSOT 수동 진단 도구, `fuzz/corpus`의 seed, `gym` 과제 데이터에는 시험지 참조가 남아 있다. 이 도구·데이터까지 모두 삭제했다고 보고하지 않는다. 일반 nextest/Studio 시험지 회귀 실행과 별도 범위다.

검증 명령·결과와 입력 해시 정본은 ignored `output/pr-review/7445-completed-20261009/stage389-exam-reference-cleanup/fixture-live-audit.json` 및 `latest-code-references.txt`, `latest-collectors.txt`에 보존한다. 새 회귀 검사는 추가하지 않는다.

## 수정 검증

- SVG 비교 기본 입력에서 시험지 4개를 제거했다. 명시적으로 전달하는 수동 입력은 도구 기능으로 유지한다.
- `bash -n scripts/svg_regression_diff.sh` 및 `git diff --check`가 종료 코드 0으로 통과했다.
- 이번 수정은 Rust source·test·PDF·WASM·시각 비교 알고리즘을 변경하지 않았다. 전체 nextest를 다시 통과했다고 보고하지 않는다.

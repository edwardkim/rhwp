# Stage 54: 개인정보 65쪽 완료 경계의 회귀 보강과 fresh WASM 증적

Issue: #7445. Runtime parent는 `a340511936499170626bef6408a16b419b4466ac`, base는 `f7abae4a8e70c062c378db06ad64d2d2eacb88cd`다. 새 문서 보정이나 Runtime 변경 없이 기존 #5846 검사에 원본 상하 안 여백의 첫 조각 소유 관계와 독립 Print 행 높이 ±2px 검사를 보완했다. Native/fresh WASM 전 66쪽 최저 96.57165/96.56850%, 65·66쪽 양쪽 raw 100%와 확대 판독을 먼저 확인했다.

정확한 수정 전 P47 Runtime에서 같은 새 assertion이 26.8933px/29.8667px 관계 위반으로 실제 FAIL했고, 수정 후 P48은 PASS다. 임시 재현 뒤 네 Runtime 파일을 저장한 hash와 대조하여 원복했고 기존 원본·PDF·검사 관계를 유지했다. frozen rlib SDK 연결 오류는 결함 FAIL로 세지 않고 정식 Cargo/manifest 경로의 실행으로 대체했다.

최종 source 주석·형식 정리 뒤에도 같은 검사 1 PASS / 0 FAIL, 테스트 0.576초를 확인했다. 기존 P48의 집중 123 PASS와 새 assertion 재검증을 구분한다. 전체 회귀를 대신하지 않는다. fresh WASM은 Mac locked wrapper no-opt 로컬 대체이며 pkg/Studio public/보존 JS·WASM hash 일치를 확인했다. 실제 초기화 응답과 전체 소비 경로는 아직 미완료다.

같은 환경으로 다시 빌드한 fixed base의 Native 전 66쪽과 비교한 결과 41쪽이 99.23712→97.93657%, 1.30055%p 하락하므로 PR 준비를 보류한다. 다른 65쪽에는 0.1%p 이상 하락이 없다. 원래 제출 범위의 최종 시각·전체 nextest·필수 gate는 계속 필요하며, 새 HWP 파일로 보정 범위를 확대하지 않는다.

[전쪽 TSV·Native/fresh WASM review/overlay·회귀와 잔여](../pr/assets/issue7445/completed-20261009/stage54-pii-regression-20261022/verification.md). 실행 로그는 ignored `rebase-8155-20261022/pii65-padding-regression/{before-cargo,after-cargo,final-after-cargo}.log` 및 `final-stage53/fresh-wasm.log`에 보존한다.

후속 정책 검사에서 `canvaskit_policy.rs`의 파일 최상위 테스트 전용 import가 신규 `cfg(test)` support 항목으로 검출됐다. 해당 import를 기존 테스트 모듈 안으로 옮겼고, 정책·허용 기준선이나 Runtime 동작은 바꾸지 않았다. 고정 base 대비 unit-tier 검사는 4101 tests / 296 modules / cfg support items 28로 통과했다. 기존 CanvasKit 정책 검사도 실제 50 PASS / 0 FAIL, 0.095초를 확인했다. 로그는 같은 ignored 디렉터리의 `unit-tiers-after.log`, `canvaskit-policy-tests.log`다. 이는 전체 회귀 통과를 의미하지 않는다.

# #8174 T08 완료 범위와 종료 조건

2026-10-28. 기준은 T07 완료 `676b2fa457495224f32f037ec592bedb4bee4b28`이다.
사용자의 「이제 T08 이 남았군요. 시작합니다」를 구현·최종 검증·보고·로컬 커밋까지의
착수 승인으로 기록한다. G01–G06 전체를 한 단위로 완료하며 중간 승인을 반복하지 않는다.

| ID | 처리 범위 | 완료 조건 |
| --- | --- | --- |
| G01 | source unit 증가·신규 지원 항목을 공개 API 회귀로 이관 | 고정 최신 devel 기준의 unit/suite 정책 통과. 기존 원본·SHA/함수·독립 계약과 기존/새 공개 검사의 대응을 보존하고 실제 자동 수집 경로로 실행. 기준선·기대값 완화와 새 테스트 전용 생산 API를 만들지 않음 |
| G02 | 최종 source의 Rust 전체 게이트 | fmt·Native/WASM/workspace all-target Clippy·workspace build·focused/전체 nextest·Native Skia 3종·workspace 밖 backend 단독 unit 통과. 기존 사용자 지정 page-border 제외를 추적하며 임의 제외를 추가하지 않음 |
| G03 | 표준 제품과 실제 host 동작 | 최적화 fresh WASM을 root pkg/Studio public에 동기화하고 실제 HTTP 응답 hash 확인. Native 제품·Worker·Studio의 실패/취소·상태 보존·복구·편집/양쪽 저장, Node/TypeScript/빌드와 실제 브라우저 통과 |
| G04 | 영향 전쪽·직접 시각 판독 | T07과 같은 38개 문서·Native/WASM 각 104쪽을 현재 제품으로 검증. 쪽수·90%·완료 페이지 98%·내용/소유·직접 확대 판독과 T07 출력 보존 확인. 새 원인/회귀가 있으면 보정·영향 재검증 |
| G05 | 최종 비용·다페이지 로딩/편집 확인 | 동일한 구성·환경의 보존된 T06/T07 제품과 최종 제품으로 Native/PDF·WASM 시간/메모리/크기 비교. 기존 S09 대조 제품을 사용한 다페이지 115쪽 HWP/HWPX·415쪽 HWP의 실제 Studio 로딩/편집 누적 영향도 확정. 빌드·다른 측정과 겹치지 않고 hash·반복·한계를 기록 |
| G06 | 테스트/CI 연결·포장 영향·문서·완료 커밋 | 기존 suite/Native/CI 실행 경로와 신규 이관 검사의 실제 실행을 대조. 실행 누락 때만 최소 연결을 보완. T07 archive와 생산 바이트의 변경을 분리하고 필요한 포장/소비 증거 갱신. source/제품/evidence 원장을 고정하고 필수 잔여 0건으로 보고 |

최신 fetch 기준은 `db5cd5bcc60a69e42f419f18a7f57a33b6fb7fdc`다. base 확인만을 이유로
rebase하지 않는다. 실제 대조에서 이슈의 source unit 33개와 upstream의 기존 HWP3/column-width 검사
변경 두 건을 구분했다. 기존 HWP3 계약은 공개 검사로 보존하고 column-width는 upstream의
물리 단위/관계 보정 검사로 유지한다. 원 해결/제외 PR·실제 통합·독립 증거를 확인하고 조정한다.
T09의 최신 devel 기능 보존·PR 제출과 실제 게시·token/owner·태그는 별도다.

이관은 기존 공개 검사로 이미 보존한 계약을 먼저 확인하고 누락만 보완한다. helper의 반환값을
다시 assertion하는 검사를 만들거나 private helper를 공개해 위치만 옮기지 않는다.
렌더링 검사는 기존 독립 PDF와 90%/98% 선행 증거를 유지하며 synthetic 예약 계약을
한컴 피델리티 근거로 설명하지 않는다. 생산 코드가 바뀌면 해당 실제 동작·시각 검증도 갱신한다.

모든 Cargo 작업은 공용 `target/pr-review`에서 순차 실행한다. 임시 로그·TSV·PNG·원본
보존·체크섬과 중간 진단은 ignored `output/pr-review/issue8174/t08-completion-20261028/`에
둔다. 비공개 입력/식별 자료는 공개하지 않는다. 원격 변경·토큰 값 열람은 수행하지 않는다.

## 종료 기록

2026-10-28 G01–G06 완료. [완료 보고](../working/task_m100_8174_t08_completion.md)에
현재 source/제품·정확한 명령과 결과·전쪽/직접 판독·비용·포장 소비의 증적을 연결했다.
T08 필수 잔여는 0건이며 T09·원격 제출·실제 registry 게시 절차와 구분한다.

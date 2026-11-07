# R43 리베이스와 충돌 해소 기록

## 실제 결과와 남은 검사

새 base `5077b216cbc51d2046da07350de562f6f521a7c1` 위에서 139개 커밋 replay를 종료 코드 0으로 완료했다. 리베이스 직후 head는 `46486e26ee3bbb0a887c4a7d78529cc8150a1341`다. 기본 작업 트리에 반영한 검증 source는 `0261687265f3a10a64e51809d006b2305a36d950`이며, 실제 전체 Rust 회귀는 12,910 passed / 0 failed / 45 skipped, 충돌 경계 집중 검사는 19 passed로 완료했다. 최종 Native/fresh WASM 출력 대조는 별도 원장에 연결하며, 회귀 통과만으로 시각 통과를 대신하지 않는다.

리베이스는 분리된 검토 worktree에서 먼저 완료했다. 작업지시자의 즉시 반영 요청에 따라 기본 브랜치에 `0261687265f3a10a64e51809d006b2305a36d950`을 반영했고, 기본 작업 트리의 실제 format 검사는 종료 코드 0으로 완료했다. 이전 R41 측정은 불변 Native binary/WASM package를 사용했으므로 진행 중 기본 작업 트리 HEAD가 바뀐 사실과 제품 producer SHA를 구분해 보존했다. 원 기여자 PR head를 merge·rebase·push하지 않았다.

## 기준과 적용 보존

- 이전 base `1e16a19aede5ccb2657d35974940eb596286cdf1` → 새 base `5077b216cbc51d2046da07350de562f6f521a7c1`.
- 원 PR 27건의 출처 footer 56개, author·이메일·순서를 모두 보존했다. 원 PR을 새로 체리픽하지 않았다.
- [커밋별 대응표](contributor-commit-map-r43.tsv)는 실제 Git replay 전후를 연결한다.
- [전체 139개 커밋 대응표](rebase-r43-all-commit-map.tsv)는 이전 구현 기록의 메인터너 보정 SHA까지 현재 적용 SHA로 추적한다. 기여자 56개 표와 범위를 구분한다.
- 작업지시자의 범위 재지정에 따라 다른 기여자 PR의 독립 시뮬레이션·전체 검증은 수행하지 않는다. 새 base의 이력은 현재 PR의 충돌·영향을 판단하는 근거로만 참조한다.

## 실제 충돌과 처리

| replay 위치·원 커밋 | 충돌 파일 | 실제 보정과 유지한 계약 |
| --- | --- | --- |
| 50/139 · `3ffeb3d9441ba2428de7cb6c40ccdbf513f56240` | `src/renderer/height_measurer.rs` | 저장 줄 없는 셀의 형식 중립 내용 높이 계약을 유지했다. 이는 새 base의 제한된 native 프레임 조건을 포함한다. 중복 지역 flag만 제거하고 중첩 표의 실제 프레임 높이·줄 소유 생산과 측정/배치 소비는 유지했다. |
| 67/139 · `8383a739ebf2d2a10184ca1ea2d0ea33c4411230` | `src/renderer/composer/composition.inc.rs` | 캡션 문단 소유권과 현재 프레임의 표 소유 목록을 `try_compose_paragraph_with_owners`에 함께 전달했다. 문단 소유·프레임 소유·기존 inline 수용을 보존하여 어느 경로도 이미 배정한 표를 선언 폭으로 다시 블록 줄에 넣지 않는다. |

새 base의 `stored_cell_reflow` 원시 슬롯·실측 표 높이 → 공통 조합 본체 → `composed_tac_line_assignment` → `tac_flow`·전체/부분 배치의 실제 소비를 영향 검사에서 대조한다. helper 이름·자동 적용·컴파일만으로 기능 유지라고 판정하지 않는다.

## 로그

아래는 로컬 ignored 로그의 실제 경로다. 전체 로그와 각 종료 코드 파일을 보존했다.

- `output/pr-review/planet6897-integration-20261106/logs/r43-fetch-upstream.log`: fetch exit 0.
- `output/pr-review/planet6897-integration-20261106/logs/r43-merge-simulation.log`: 사전 시뮬레이션 exit 1, 두 충돌을 탐지한 원 기록.
- `output/pr-review/planet6897-integration-20261106/logs/r43-rebase.log`: 실제 50/139 충돌 중단 exit 1.
- `output/pr-review/planet6897-integration-20261106/logs/r43-rebase-continue-01.log`: 첫 충돌 해소 뒤 67/139 충돌 중단 exit 1.
- `output/pr-review/planet6897-integration-20261106/logs/r43-rebase-continue-02.log`: 둘째 충돌 해소 뒤 139/139 완료 exit 0.

사전 시뮬레이션의 exit 1을 성공으로 바꿔 쓰지 않는다. 현재 원 PR 필수 검증과 새 source의 최종 시뮬레이션 결과는 이후 기록한다.

## 기본 검사에서 발견한 준비 환경 문제

분리된 sparse worktree의 `cargo fmt --all`은 nested vendor workspace 탐색에서 실패했다. 이를 제품 오류로 판정하거나 Cargo workspace 설정을 바꾸지 않는다. 기본 작업 트리에 반영한 뒤 정상 경로에서 format 검사를 수행해 종료 코드 0을 확인했다.

`git diff --check`는 기존 커밋 대응표·inventory 3개의 CRLF를 trailing whitespace로 검출했다. 이 세 metadata 파일만 LF로 정리했으며 실측 원본 TSV의 바이트·지표는 변경하지 않았다. 코드 assertion·기대값·fixture·baseline 변경도 없다.

## 실제 최종 검증 연결

- [현재 source의 Rust 검증](final-r43/rust/README.md): 최소 공개 글꼴 5개, 공용 `target/pr-review`, 컴파일 8 jobs·테스트 12 threads.
- 실제 Native 공식 편람 gate 383쪽은 통과했다. 전체 원 PR 입력·보호 문서와 같은 입력의 최신 base 대조는 최종 원장에서 구분한다.
- 새 base 기능을 보존하는 집중 검사와 기존 원 PR 주장의 전쪽·직접 판독을 모두 확인한 뒤 제출한다.

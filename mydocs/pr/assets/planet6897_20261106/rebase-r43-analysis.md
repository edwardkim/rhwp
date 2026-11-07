# R43 리베이스와 충돌 해소 기록

## 실제 결과와 남은 검사

새 base `5077b216cbc51d2046da07350de562f6f521a7c1` 위에서 139개 커밋 replay를 종료 코드 0으로 완료했다. 리베이스 직후 head는 `46486e26ee3bbb0a887c4a7d78529cc8150a1341`다. 현재 통합 PR의 새 source 영향 검증은 실행 전이며, 이 기록을 새 head의 회귀·시각 통과 판정으로 사용하지 않는다.

기본 작업 트리는 기존 WASM 측정 provenance를 유지하기 위해 `693f0186f688b4f29cb8d16b49a33ebec963ec68`에 고정했다. 분리된 검토 worktree에서 replay를 완료했으며, 기존 측정 종료 뒤 해당 결과를 기본 브랜치에 반영한다. 원 기여자 PR head를 merge·rebase·push하지 않았다.

## 기준과 적용 보존

- 이전 base `1e16a19aede5ccb2657d35974940eb596286cdf1` → 새 base `5077b216cbc51d2046da07350de562f6f521a7c1`.
- 원 PR 27건의 출처 footer 56개, author·이메일·순서를 모두 보존했다. 원 PR을 새로 체리픽하지 않았다.
- [커밋별 대응표](contributor-commit-map-r43.tsv)는 실제 Git replay 전후를 연결한다.
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

분리된 sparse worktree의 `cargo fmt --all`은 nested vendor workspace 탐색에서 실패했다. 이를 제품 오류로 판정하거나 Cargo workspace 설정을 바꾸지 않는다. 기본 작업 트리에 반영한 뒤 정상 경로에서 format 검사를 수행한다.

`git diff --check`는 기존 커밋 대응표·inventory 3개의 CRLF를 trailing whitespace로 검출했다. 이 세 metadata 파일만 LF로 정리했으며 실측 원본 TSV의 바이트·지표는 변경하지 않았다. 코드 assertion·기대값·fixture·baseline 변경도 없다.

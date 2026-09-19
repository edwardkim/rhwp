## 변경 요약

새 다운로드의 최초 상태 저장보다 filename/complete 이벤트가 먼저 처리되면 파일은 저장돼도
뷰어가 열리지 않는 경합을 수정합니다. Chrome adapter에서 download ID별로 이벤트의 최초
상태 조회·저장부터 후보 처리·terminal 기록까지 순서대로 실행합니다. 다른 ID는 독립적으로
진행하며, 기존 미추적·과거 다운로드와 XLSX·자체 Blob·autoOpen=false 제외 조건을 유지합니다.

## 관련 이슈

Closes #6988

## 테스트

- 변경 범위: Chrome 다운로드 adapter, Node/실제 Chrome E2E, 문서. Rust·Studio·조판 변경 없음.
- 검증한 구현: `cf76f3120a89745fd39fe1f31b48ddeba10e28e4`
- 비교 base: `a3de5826c3b404bba8d7f3383d55c947f5a35aef`
- `node --test rhwp-chrome/sw/*.test.mjs rhwp-shared/sw/*.test.js`: 170 passed.
- JS 문법 및 Chrome extension dist 계약: 통과.
- `npm --prefix rhwp-chrome run test:e2e:download`: 빌드 성공. 실제 Chrome에서 대조군 3회와
  최초 저장 지연군 3회 모두 다운로드 ID당 viewer 1개·원본 바이트 보존. XLSX 2건과 자체 Blob은 탭 0.
- 같은 회귀 테스트를 수정 전 adapter로 실행하면 Node와 실제 Chrome 모두 자동 열기 누락을 검출.
- macOS arm64 / Chrome 153.0.8010.48 / Node 24.15.0. 최신 기준 Rust의 native `--no-opt` WASM 사용.
- [x] 변경 범위의 로컬 검증 완료. 후속 문서 커밋은 검증한 구현을 변경하지 않음.
- [x] 문서 및 commit 범위 `git diff --check`, 실행 절차 확인.
- Rust·Studio source, 편집 command, package API, 조판 변경의 검증 항목은 비해당.

실제 worker suspend/resume과 자연 발생 빈도는 미검증입니다. 지연은 테스트에서 storage write에만
주입하며 다운로드·이벤트는 실제 Chrome API를 사용합니다.

상세 근거: [#6988 검증 보고서](https://github.com/postmelee/rhwp/blob/codex/issue-6988-download-event-race/mydocs/report/task_m100_6988_report.md)

# #3587 Stage 13 — C3 공통 준비·dry-run과 실행 경로

- 승인: 2026-09-13 메인테이너 「다음 절차 진행을 승인합니다」.
- 기준: `b98656ccd`, `task_m100_3587`. C2 실물 성공 기록을 커밋한 뒤 시작했다.
- 계획: [C 상세계획](../plans/task_m100_3587_impl_c.md) §5–6.
- 상태: C3 연결 구현·집중 188건·Rust lint 3종 및 workspace build 통과. C 종료 통합 검증 또는 #3587 전체 완료 선언이 아니다.

문단 복제·고정 양식 채우기의 준비/반영 경계를 행 복제와 같이 분리한다.
dry-run은 동일 준비 경로에서 실제 detached 채우기와 ID/참조 검증까지 수행하고,
원본 IR·이벤트·조판 상태·파일은 변경하지 않는다. 기존 간이 선검증을 실행 결과로
포장하지 않으며 전체 Document 복제나 적용 후 되돌리기를 사용하지 않는다.

이 경계를 WASM options JSON과 기존 CLI `run`/MCP `hwp_run_plan`에 연결한다.
새 action은 단독 step만 허용하며 기존 네 action의 다중 step 계약을 유지한다.
집중 계약·동일 입력 preview/실행 대조 후 C 종료 통합 검증의 실행 결과를 별도로 기록한다.
이번 승인에는 원격 push·PR·댓글, D 또는 Gym 구현은 포함하지 않는다.

## 구현 결과

제품·테스트 커밋은 `c66ff24b0`이다. 행 복제의 기존 준비/반영 경계를 문단 복제와
고정 양식 채우기에도 적용했다. 세 연산은 `TemplateOperation`의 typed 요청을 공통으로
사용하며 `preview_template_operation_native`와 `execute_template_operation_native`는
동일한 준비 결과를 버리거나 반영한다. 복제 후 setter 실패를 되돌리는 방식이 아니다.

- WASM: `HwpDocument.applyTemplateOperation(optionsJson)` 하나로 세 action을 전달한다.
  `dryRun`은 boolean만 허용하고 생략 시 false다. 이번 native 계약은 실제 Rust wrapper를
  호출했지만 브라우저 JS 실행·JS 예외 전달을 검증한 것으로 간주하지 않는다.
- CLI/MCP: 기존 `run` 및 `hwp_run_plan`을 사용한다. 새 복제 엔진·Gym 의존성을 추가하지 않았다.
  새 action은 단독 step만 허용하며 기존 네 action의 조건절·다중 step·입력 SHA 보호는 유지한다.
- 요청 JSON 8 MiB 상한은 문자열 decode 전에, CLI의 기존 Value는 제한된 writer로
  인코딩할 때부터 검사한다. typed 입력의 기존 실제 작업량 상한도 유지한다.
- 잘못된 독립 target은 최대 16개까지 함께 진단한다. 예산·key·구조의 선행 오류는
  먼저 중단하며 의존 대상의 오류를 추측하지 않는다.
- 반환은 생성 경로·대응표·입력 작업량, 출처 표지를 포함한다. `changedPages`는 null이다.
  `workload`의 record/target/문자열 바이트 수는 실행 시간이나 메모리 실측값이 아니다.
- 계획 스키마는 1.2 → 1.3이다. `planVersion`과 봉투 `schemaVersion`은 1.0을 유지한다.
  [소비자 매뉴얼](../manual/template_automation.md)에 경로·offset·상한·재실행 의미를 기록했다.

CLI 실제 실행은 선검증과 반영에서 준비 경로를 두 번 수행한다. 전체 문서 복제는 하지 않지만
이 중복 준비의 비용은 C 종료 1/10/100회 계측에서 별도로 확인해야 한다.

## 검증과 정정

검증 worktree는 `/home/edward/mygithub/rhwp-review-3587`, 공유 target은
`/home/edward/mygithub/rhwp/target/pr-review`다. 로그는 `output/3587/c3/`에 보관한다.
소스·테스트의 main/review byte 일치를 `source-test-final-sha256.log`와
`review-final-byte-match.log`로 확인한 뒤 위 제품 커밋으로 고정했다.
검증 종료 후 review index도 해당 커밋과 동일함을 검사하여 detached HEAD를
`c66ff24b0`로 맞췄다. 강제 checkout·변경 삭제 없이 review 추적 변경은 비워 두었다.

| 검사 | 결과 | 로그 |
| --- | --- | --- |
| #3587 기존 119건 + 신규 경로 11건 + 기존 run/schema 58건 | **188 PASS**, 비대상 2,699 skipped | `contracts-r2.log` |
| source-side test tier 정책 | 통과, 4,205 tests / 298 modules | `unit-tiers-final.log` |
| Rust fmt check | 통과 | `fmt-check.log` |
| native Clippy `-D warnings` | 통과 | `clippy-native.log` |
| WASM32 lib Clippy `-D warnings` | 통과 | `clippy-wasm.log` |
| workspace build | 통과 | `build-workspace.log` |
| workspace all-targets Clippy `-D warnings` | 통과 | `clippy-all-targets.log` |
| 파생 integration manifest | 통과, 1,279 sources / 48 targets | `manifest-check.log` |
| 변경 문서 내부 링크 | 7문서 통과 | `docs-links-final.log` |

신규 경로 계약은 고정 양식·문단 복제·행 복제의 native/dry-run 대조, native에서 WASM
Rust wrapper 호출, CLI HWP/HWPX 저장·재열기를 검사한다. MCP는 실제 stdio `tools/call`로
행 복제를 실행했다. HWPX 입력은 원본 연구노트에서 변환한 **파생 형식 probe**이며
독립적인 한컴 정답지로 주장하지 않는다. 파일명뿐 아니라 실제 파일 형식도 검사한다.

진행 중 실패도 다음과 같이 구분한다.

1. 첫 CLI 테스트는 존재하지 않는 `tempfile` 의존성 때문에 compile 실패했다. 별도 제품 의존성을
   추가하지 않고 테스트가 소유한 고유 임시 디렉터리를 표준 라이브러리로 관리하도록 고쳤다.
2. 테스트 source 수정 후 파생 suite 재생성을 빠뜨린 실행은 **0건/exit 4**였다. 통과가 아니다.
   이후 source 수정·fmt 후 prepare를 다시 수행하고 최신 manifest로 test target을 조회했다.
3. main의 오래된 generated suite로 fmt를 실행한 시도는 누락 source 참조로 실패했다.
   review worktree에서 prepare 후 fmt를 수행하고 해당 변경 파일만 main으로 동기화했다.
4. 중간 185 PASS 이후 확대 검사에서 **187 PASS / 1 FAIL**이 발생했다. 내가 만든 테스트가
   HWP 입력의 출력 이름만 `.hwpx`로 바꾸면 형식도 바뀐다고 가정한 것이 원인이었다.
   기존 `edit_output_format`은 입력 형식을 보존한다. 제품 저장 정책을 바꾸지 않고 실제
   HWPX 입력을 준비했으며, 테스트와 매뉴얼을 바로잡은 재실행이 위 188 PASS다.

nextest 설치판 0.9.137/권장판 0.9.140 및 JUnit unknown key는 기존 환경 경고다.
generated suite·manifest는 review 검증 산출물이며 제품 커밋에 포함하지 않았다.

## 남은 절차

이 집중 검증을 C 종료 전체 회귀 통과로 대체하지 않는다. 다음 통합 검증에서는 전체 nextest,
변경 범위 Native Skia, Docker WASM 빌드와 실제 JS 실행·오류 대조, C 채우기 1/10/100회
비용, 공개 실행 경로의 실물 산출물을 확인한다. 기존 C1/C2 메인테이너 시각 성공은 유지하되
이번 C3 산출물의 새로운 시각 판정으로 전용하지 않는다.

#7065 Studio 재편집 페이지네이션과 #7084 이모티콘 폭은 별도 이슈다.
D의 다른 문서 자원 가져오기와 선택적 Gym 시나리오는 아직 시작하지 않았다.

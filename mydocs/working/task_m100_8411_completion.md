# #8411 PDF 가져오기 대화창 개선 완료 기록

- 날짜: 2026-10-31 (KST)
- 이슈: https://github.com/edwardkim/rhwp/issues/8411
- 브랜치: `task/8411-pdf-import-dialog-ui`
- 기준 devel: `1cfa5726cdd8c500df612ec5708103b4a1c4d147`
- 계획: [수행 계획](../plans/task_m100_8411.md), 계획 커밋 `0f78e95c62`
- 구현·검증·사용자 안내 커밋: `c288c9e4db`
- 판정: 승인된 로컬 구현·검증 완료. PR 생성·push·merge·이슈 close는 미실행.

## 변경과 사용자 안내

기본 화면을 파일 안내·처리 설정·고급 설정으로 정리했다. 공통 테마 스타일을 적용하고
상세 내용만 스크롤되도록 하여 낮은 화면에서도 가져오기·취소 버튼을 유지했다.
JSON 입력과 전체 정책은 기본적으로 접힌 전문가 영역으로 분리했다.

사용자의 “맞춤 한도 JSON을 어떻게 입력해야 하는지 모르겠다”는 의견을 반영했다.
일반 사용자는 비워 두면 된다는 안내, 필드 의미·단위, 8쪽·8 MiB 예시와
「예시 넣기」·「기본 한도 사용」 버튼을 추가했다. 한도를 넘으면 중단하며
앞의 몇 쪽만 가져오는 기능은 아니라는 설명을 화면과
[사용자 안내](../manual/studio_document_features.md)에 함께 기록했다.

select·summary·읽기 전용 JSON의 기본 키 동작을 보존하고, Tab은 열린 영역의
활성 컨트롤 사이에서 순환한다. 입력 오류는 해당 입력 옆에 표시하며 확인 시 포커스한다.
처리 중 설정은 비활성화하고 취소는 유지한다. 시간 초과 후 오류와 다시 시도 버튼을
확인할 수 있다. 실제 정책 값·Worker 전달·변환 엔진·조판 코드는 변경하지 않았다.

## 실행 검증

| 검사 | 결과 |
| --- | --- |
| `npx tsc --noEmit` | 통과 |
| Studio `npm run build` | PR 준비 단계에서 production bundle·PWA 생성 통과 |
| Studio `npm test` | 2,702개 중 2,700개 통과, 실패 0, 기존 skip 2 |
| `pdf-import-dialog-issue8411.test.mjs` | 실제 Chromium·WASM·Worker에서 통과 |
| `pdf-import-busy-issue8398.test.mjs` | 선택·취소·드롭·오류·재시도·기존 문서 보존 통과 |
| `zoom-dialog-transaction.test.mjs` | 공통 모달 변경의 정상 대조군 통과 |
| `scripts/check_e2e_manifest.py` | tracked 181개 / 원장 181행, 통과 |
| `git diff --check` | 통과 |

새 E2E는 두 테마·긴 파일명·420×320 화면·Tab/방향키/Enter/Space/Escape,
예시의 실제 정책 검증과 기본값 복원, 잘못된 JSON·상한 초과·시간 입력을 확인했다.
실제 1초 제한으로 실패시킨 뒤 동일 대화창에서 30초로 재시도하여 공개 PDF 변환을
완료했다. Worker에 전달된 프로필과 맞춤 한도를 대조했다.

기준 devel의 `dialog.ts`만 별도 Vite에서 공급했을 때 새 검사는 방향키 선택에서
`standard != constrained`로 실패했고, 현재 코드에서는 통과했다. 원본 작업 파일을
되돌리지 않고 확인했다. 수정 전후 실행 로그는 `output/pdf-import-dialog-20261031/ui-before-keyboard.log`와
`ui-final.log`에 보관했다.

#8398 하네스는 #8397 / `99a176b7cb` 이후 문서 열기에서 표시되는 로컬 글꼴 선택창에
「대체 글꼴로 보기」를 선택하도록 보완했다. 로컬 글꼴 권한을 요청하지 않으며 기존
문서 보존 assertion을 유지했다. 새 보조 버튼과 혼동하지 않도록 취소 선택자를 footer로
한정했다. 실행 기록은 같은 output의
`busy-preservation.log`·`modal-control.log`에 보관했다.

원장 검사에서 #8397이 추가했던 기존 글꼴 E2E 4개의 누락이 발견되어 실제 소스·샘플·
호출 경로를 확인하고 원장 행만 보완했다. 해당 4개의 재실행 결과로 주장하지 않는다.

## WASM과 화면 근거

UI 전용 변경이므로 #8403에서 검증한 release + wasm-opt 산출물을 재사용했다.
신규 WASM 빌드로 표기하지 않는다. 제품 소스 `b13334f327193b7a6ed76c9f501f6fdee71c5e11`,
SHA-256 `b9a7ebc5875dee4cd2d068a2e3bf3283b8beede8fd8f26c24eca0cb0d1f78d78`의
브라우저 수신 바이트를 세 E2E에서 확인했다.
재사용 기록은 `output/pdf-import-dialog-20261031/wasm-reuse.json`이며,
[기존 최적화 산출물 근거](../tech/investigations/issue-8403/validation/rebase/README.md).
이번 작업은 문서 조판 출력 변경이 아니므로 UI 검증을 Print 피델리티 수치로 표현하지 않는다.

아래 최종 화면을 직접 판독했다. 기본 화면은 JSON 없이 간결하고, 전문가 영역에는
입력 방법·예시·초기화가 표시된다. 작은 화면의 하단 버튼과 시간 초과 오류도 보인다.

- [수정 전 고급 화면](assets/issue_8411/expanded.png)
- [수정 후 기본 화면 — 어두운 테마](assets/issue_8411/default-dark.png)
- [수정 후 기본 화면 — 밝은 테마](assets/issue_8411/default-light.png)
- [맞춤 한도 설명과 예시](assets/issue_8411/after-dark-expert.png)
- [작은 화면의 하단 버튼](assets/issue_8411/after-dark-small.png)
- [시간 초과 후 재시도](assets/issue_8411/after-timeout.png)

실행 중간 자료는 Git 제외 경로 `output/pdf-import-dialog-20261031/`에 두었고,
후속 검토에 필요한 최종 화면은 위 정식 경로에 보존했다. 실행 로그·중간 JSON은
PR 제출 규칙에 따라 output에만 보관하며 저장소 제출 대상에서 제외했다.

## PR 제출 전 확인

2026-10-31 작업지시자의 다음 절차 진행 지시에 따라 PR 준비·생성을 진행한다.
본인 계정의 열린 PR은 0건이며 최신 `upstream/devel`은 기준 SHA와 동일하다.
현재 head와 최신 devel의 `merge-tree --write-tree`가 충돌 없이 통과했다.
WASM producer와 현재 head의 `Cargo.toml`·`Cargo.lock`·`rust-toolchain.toml`·
`build.rs`·`src`·`crates`·`vendor`가 동일함을 확인했다. 검증한 제품·테스트 코드를
변경하지 않고 기존 로컬 실행 결과를 재사용한다. 승인된 계획에 따른 WASM 재사용이며,
production build도 성공했다. 머지·이슈 종료는 별도 승인 후 처리한다.

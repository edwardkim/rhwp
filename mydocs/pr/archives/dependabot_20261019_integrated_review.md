# Dependabot 7건 통합 검토 — 2026-10-19

## 최종 판정

**승인 — #8070~#8076의 누적 통합 code head에 대한 로컬 검증과 GitHub Actions CI를 완료했습니다.**
원 PR #8075·#8076은 CodeQL 단계 간 버전이 달라 단독 head를 그대로 병합할 수 없으며, 두 갱신을 함께 적용한 이 통합 후보를 수용합니다. 검사·기준값·권한을 완화하지 않았습니다.

통합 PR은 [#8078](https://github.com/edwardkim/rhwp/pull/8078)입니다. 이 문서와 오늘할일을 code CI 성공 뒤 동일 브랜치의 문서 전용 trailing commit으로 반영하며, 최신 trailing head의 required check·mergeability·head SHA를 재확인한 뒤 정상 병합합니다.

## 접수와 적용

- 사용자 지시: 열린 Dependabot PR을 누적 체리픽하여 통합 검토합니다. 봇이므로 개별 PR 리뷰 문서는 만들지 않고 이 기록 한 건으로 대체합니다.
- 기본 경로: collaborator 매개 외부 PR의 주 작업공간 체리픽 통합. 보조 경로: 접수·로컬 검증·다수 PR·Dependabot 예외·review-only trailing·merge 후속 처리.
- 원 PR 작성자는 모두 `dependabot[bot]`, base는 `devel`, Draft가 아닙니다. 상세 조사·fetch 전에 각 원 PR의 reviewer `jangster77` 요청을 API로 확인했습니다. 통합 PR은 작성자 self-review이며 reviewer를 별도로 지정하지 않습니다.
- 작업 브랜치: `review/dependabot-20261019`.
- 고정 base: `4a07d3ae4dd3255a57fb0166b2b2a2dc335a61fa`. 제출 직전 재조회에서도 base가 같아 추가 rebase가 필요하지 않았습니다.
- 검증 code head: `44cd5e45377463f26c24839c1188ddfce59f114c`. 원저자와 `(cherry picked from commit ...)`를 보존했습니다.

| 원 PR | 원 head | 적용 commit |
| --- | --- | --- |
| [#8070](https://github.com/edwardkim/rhwp/pull/8070) | `b67bbd44d88cc28fb932d523ccd05f9dbd8c34fe` | `93d82e8b9a1431d31b040c377724375bcd6d316a` |
| [#8071](https://github.com/edwardkim/rhwp/pull/8071) | `fc17bd614dc9f205c3e1b06d9fdfcdb904b67307` | `500811b1c00e7397e77a36636f23aa9b79674fca` |
| [#8072](https://github.com/edwardkim/rhwp/pull/8072) | `9b26af1dcd4a702d78f57408f186b57ea6b8a637` | `854db94d982cfddc5012d607d2b6ee3b4607c3f1` |
| [#8073](https://github.com/edwardkim/rhwp/pull/8073) | `7e701d573f0caf92adf15168dfbcf244a4f8759e` | `f23308a812adc10e0aabdc2e7c47f4688cf0ab0d` |
| [#8074](https://github.com/edwardkim/rhwp/pull/8074) | `ed8db2818126eda4cbae4cabe554fbdebe44870f` | `9d8a9f6e68e2c41a3a5820bc29d64784e6e26cca` |
| [#8075](https://github.com/edwardkim/rhwp/pull/8075) | `659f2485a5aaa8761e3a289a8c5a25c07b718594` | `514fc5ed45b9826f8b17aef3a256b01a79951c16` |
| [#8076](https://github.com/edwardkim/rhwp/pull/8076) | `8ed6f12de7dae17989032e8f7e7200becba531d7` | `44cd5e45377463f26c24839c1188ddfce59f114c` |

#8070·#8071은 Studio의 `puppeteer-core 25.14.0`·`@types/chrome 0.3.5`, #8072는 Chrome의 `puppeteer 25.14.0`, #8073은 VS Code의 `webpack 5.112.0`입니다. #8074는 `install-action 2.87.29`, #8075·#8076은 `codeql-action 4.39.0`입니다.
#8071의 인접 manifest 줄 충돌 한 건은 두 갱신 버전을 모두 유지해 해결했고 lockfile은 자동 병합했습니다. 최종 manifest·lockfile 여섯 파일은 `npm ci` 뒤에도 후보 commit과 바이트가 같습니다.

## 실패 원인과 구현 주장 대조

원 PR #8075·#8076의 실제 CI 로그에서 `Loaded a configuration file for version ... but running version ...`와 기존 `test_analysis_steps_follow_selection_while_check_names_stay_stable`의 `2 != 1: CodeQL 단계의 버전 불일치`를 확인했습니다. 같은 기존 검사를 원 PR 두 head의 workflow 문자열에 각각 실행하면 해당 원인으로 FAIL, 통합 후보에서는 PASS입니다. 테스트나 CodeQL 설정을 바꾸어 실패를 숨기지 않았습니다.

CodeQL의 독립 근거는 [공식 v4.39.0 release](https://github.com/github/codeql-action/releases/tag/v4.39.0)의 동일 버전 사용 계약과 최소 bundle 2.20.7입니다. 현재 workflow는 구형 `tools` bundle을 지정하지 않고 workspace root를 checkout합니다. init/analyze를 모두 공식 tag의 `6f2ac81f5cd81fe351a197ba0b00d91d03cd2aee`로 정렬했습니다. [install-action v2.87.29](https://github.com/taiki-e/install-action/releases/tag/v2.87.29)도 공식 tag의 `8cca0918f8471c33515202b73f56ab27f2bed283`와 일치했습니다.
trigger·언어 선택·build-mode·checkout 경로·permissions·fork 신뢰 경계는 변경하지 않았습니다. 실제 GitHub CodeQL Analyze와 archive builder/worker의 설치 Action 실행까지 확인했습니다.

lockfile의 추가 버전 변경은 개발 도구의 전이 의존성입니다. Studio의 browsers·modern-tar·ws, Chrome의 browsers·modern-tar·zod, VS Code의 acorn·enhanced-resolve·minimizer-webpack-plugin·watchpack·webpack-sources를 대조했습니다. 제품의 직접 runtime 의존성 버전은 바꾸지 않았습니다.

## 로컬 검증

실행 환경은 Ubuntu 24.04 x86_64, Node 24.15.0, npm 11.12.1, Puppeteer 25.14.0, Chrome for Testing 155.0.8059.79입니다. 아래 12개 단계가 모두 exit 0입니다.

| 검증 | 명령·결과 |
| --- | --- |
| workflow 계약 | `python3 -m unittest discover -s scripts/tests -p 'test_*workflow*.py'` — 251 PASS |
| CI 영향·정책 | `node --test scripts/tests/ci-impact-classifier.test.cjs scripts/tests/ci-impact-policy.test.cjs` — 95 PASS |
| workflow 구문 | 변경한 세 workflow의 `actionlint` PASS |
| 설치 3단계 | Studio·Chrome·VS Code 각각 `npm ci --no-audit --no-fund` PASS |
| Studio Node | `npm test` — 2,338 PASS / 0 FAIL / 2 SKIP |
| Studio 타입 | `tsc --noEmit` PASS |
| Chrome 빌드 | `npm run build` PASS, Studio production Vite 포함 |
| VS Code 빌드 | `npm run compile` PASS, Extension Host·Webview 타입 검사와 webpack production 모두 성공 |
| Studio 실제 브라우저 | `npm run e2e:document-title` PASS. 원본 열기·손상 입력 후 기존 문서 보존·보조 bridge·HWP/HWPX 저장·창 제목을 실제 WASM 경로로 확인 |
| 확장 전체 E2E | `node e2e/run.mjs --dist /home/tsjang/rhwp/rhwp-chrome/dist --output /home/tsjang/rhwp/output/pr-review/dependabot-20261019/chrome-e2e` — packaged smoke·download-interceptor·extension-lifecycle 모두 PASS, 실행 전후 candidate SHA 동일 |

원시 로그·명령·실행 시간·전후 CodeQL 실패 증거는 작업 중 ignored `output/pr-review/dependabot-20261019/`에 기록했습니다. 종료 시 저장소 밖 `/home/tsjang/rhwp-local-validation/dependabot-20261019/`에 해시 목록과 함께 보존하며 로그·중간 JSON을 Git에 커밋하지 않습니다.
확장 E2E는 unpacked Chrome for Testing 기준입니다. 스토어 설치·업그레이드, Edge/Firefox 또는 배포 승인을 주장하지 않습니다.

## 정확한 code head의 CI

다음은 위 code head에 대한 Full 실행 결과입니다. 원 PR의 과거 녹색 CI나 문서 전용 fast-pass로 대신하지 않았습니다.

| workflow | 결과 |
| --- | --- |
| [CI Impact Policy Controller](https://github.com/edwardkim/rhwp/actions/runs/39196838647) | SUCCESS |
| [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/39196838643) | SUCCESS |
| [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/39196838562) | SUCCESS |
| [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/39196838707) | SUCCESS |
| [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/39196838590) | SUCCESS |
| [CI](https://github.com/edwardkim/rhwp/actions/runs/39196838538) | SUCCESS |

code head의 필수 check와 `CI Impact Policy` status가 성공했고, 전체 check도 완료됐습니다(정책상 skipped·neutral 포함). 이 PR에는 Actions 변경이 있으므로 trailing 문서의 재사용은 trusted controller의 exact Full candidate·review-only tail 증명으로만 허용합니다. 증명이 부족해 Full 실행으로 돌아가면 그 최신 head의 CI 완료를 기다립니다.

## 조판 원칙·시각 검증 및 입력 확인

**조판 원칙은 비해당입니다.** diff는 개발 의존성 manifest/lockfile 여섯 파일과 Actions pin 세 파일뿐이며 Rust·제품 TypeScript·조판·paint·WASM·입력·golden·회귀 테스트를 변경하지 않았습니다. 좌표·줄 높이·문단 소유·분할·측정/배치의 생산·소비 계약도 바꾸지 않았습니다. 새 렌더링 회귀나 baseline 갱신이 없어 Native/fresh WASM Visual Sweep·90%/98% 자격 부여를 주장하지 않습니다.
공유 `target/pr-review`는 보존했고 로컬 Cargo 실행은 비해당입니다. 기존 WASM을 재사용한 브라우저 호환 검증을 fresh engine 빌드나 한컴 Print와의 시각 검증으로 표시하지 않았습니다. 제출 뒤 GitHub CI는 자체 영향 분류에 따라 Full Native/Rust/fresh WASM package gate를 실행했습니다.

실제 실행한 입력 네 개는 code head의 Git object와 바이트가 같습니다. 독립 PDF를 사용하는 fidelity 검증은 이번 범위에 없습니다.

| 입력 | SHA-256 |
| --- | --- |
| `samples/para-001.hwp` | `bab4561ceb02cdfa184a1689be9619c08e18d6021cdbc423486b848bc14d267e` |
| `samples/hwp3-pagedef-1915.hwp` | `b272fdd218b4e91355167e63438a1605ef6902d75970c4ff5b8bae67087122d0` |
| `samples/re-font-dotum-empty-hancom.hwp` | `1ee8871f37bec2e97d0928709dc411c0eacad656f35aa3d92c5cf89f61c5761b` |
| `samples/hwpx_sample2.hwpx` | `188bdfe21f89e117d8897f4102aa6f741962b23a3019ad2aaf7bdc222d90fdb2` |

## 병합·후속 처리 계획

동일 통합 PR에 이 기록과 [오늘할일](../../orders/20261019.md)을 trailing commit으로 반영합니다. push 전 최신 base/head를 고정한 merge-tree·문서 링크·기존 오늘 기록 보존 검사를 수행합니다. 최신 required check 성공 뒤 정상 merge하고, merge SHA의 devel 포함을 확인한 다음 원 봇 PR 7건에 통합 PR을 연결해 정리합니다. 관련 제품 issue의 종료를 주장하지 않습니다.
병합 뒤 검증 CI를 다시 실행하지 않고 duration 갱신/보류와 이슈 종료 운영 workflow를 확인합니다. 원시 증적을 외부에 보존한 뒤 이번 작업의 branch·fetch ref·종료된 임시 산출물과 루트 output을 정리하며 공유 target·기본 작업공간·관련 없는 worktree는 유지합니다.

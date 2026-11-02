# Dependabot PR 일괄 검토 — 2026-11-02

## 최종 판정

**8건 승인, #8456은 메인터너 보정 후 수용 가능 — 필수 로컬 검증 완료.** 원격 push·GitHub review·comment·merge는 아직 수행하지 않았으며 작업지시자의 승인을 기다린다. #8456 원 head의 실패를 성공으로 바꾸어 적지 않으며 보정 source의 새 GitHub CI 성공을 merge 조건으로 유지한다.

## 원 PR별 판정

| PR | 변경 | 고정한 원 head | 판정 |
| --- | --- | --- | --- |
| [#8449](pr_8449_review.md) | Chrome 타입 | `2fa686d569a078ce9e44e05ca493e65d704784c7` | 승인 |
| [#8450](pr_8450_review.md) | zip | `94fdf6fea2a7b56865e41ef3e8c8923033211c07` | 승인 |
| [#8451](pr_8451_review.md) | Anthropic SDK | `b4b31c7fc834c2ed502ef1983e9fc36b341ab41f` | 승인 |
| [#8452](pr_8452_review.md) | unicode-width | `5829d65eb6fd368a2557c016df4747d7b12362ca` | 승인 |
| [#8453](pr_8453_review.md) | webpack | `dc8b52ad418594a42b52560c7b706ca0fe4f1783` | 승인 |
| [#8454](pr_8454_review.md) | libc | `4dd0e7eb196a1bfede636c4314fbe8fcc7bec8eb` | 승인 |
| [#8455](pr_8455_review.md) | roxmltree | `db38e6af4d65e5a8bdc7ba2697d3658a9839e505` | 승인 |
| [#8456](pr_8456_review.md) | Skia | `54d0255c594241d1593adff0367a236a9b6f3ebc` | 메인터너 보정 후 수용 가능 |
| [#8457](pr_8457_review.md) | install-action | `cd0ccf51995d9e985f8137d211db6077d7003a8b` | 승인 |

## 검증한 source와 원격 제출안

- 기준 devel: `d3f7209bbb0fdb1711c5f40a70ad22f0ca0be66d`. 실행 중 원격도 같은 SHA임을 재확인했다.
- 결합 검증 head: `e58024b15e11438a0e1162a25d836d3c3c4ff7a9`.
- Skia 최소 보정: `8777e11c165636b19fe4f5db10eb0a7ee7ed06cf`. `prettyplease 0.2.37 → 0.3.0`과 checksum·syn 연결 3줄만 고쳐 bindgen의 syn 3 타입을 공유했다. Native·Canvas CI의 원 실패는 같은 bindgen E0277이며 Canvas 비교 자체는 시작되지 않았다.
- 제출 source 정렬: `fc77674ff1574e30ef514af2b8fde04fb266eff0`의 전체 tree는 결합 검증 head와 같다. 원 bot commit·8개 원 head를 normal merge로 보존했다. 정식 기록·증적만 이어서 commit한다.
- 제출 branch: `review/pr-8456-skia-upgrade`. 누적 검토 branch: `review/integration-20261102-dependabot`. 기본 작업공간의 devel과 사용자 배포 WASM은 바꾸지 않았다.
- 외부 contributor first-time 경로는 bot에 적용하지 않는다. 기본 `maintainer_general`, 보조 `intake_and_review`·`local_validation`·`multi_pr_update_branch`·`rework_and_exceptions` 및 renderer 증적 절차를 적용했다.

8개 원 PR을 먼저 개별 merge한 뒤 #8456을 실제 최신 devel과 정렬한다. 검증한 runtime·dependency tree가 같은지 확인하고 보정·증적을 원 PR source branch에 한 번 push한다. 새 exact-head CI와 self-review 성공 뒤 마지막으로 #8456을 merge한다. [원격 처리 계획·댓글 초안](assets/dependabot-20261102/remote-action-plan.md)을 승인 대상으로 준비했다.

## 완료한 검증

- npm 변경 3개: 실제 변경 entry의 NPMScan·registry·라이선스·install script·전이 delta 검토. 새 취약점·스크립트·출처 교체·identity mismatch 없음. 변경 3개에 한정된 감사이며 전체 npm 그래프 감사가 아니다.
- Cargo OSV: 변경·추가 버전을 포함한 정확 버전 15개에서 advisory ID 0건. 전체 C/C++ 라이브러리 감사와 구분한다.
- install-action: full SHA와 tag·manifest delta·실행 코드 동일성, workflow 계약 18건, 실제 GitHub archive의 새 nextest 0.9.150 설치·실행 성공.
- Studio TypeScript·단위 검사 2,716 PASS/2 skip·build, VS Code webpack compile 성공. 선택한 Claude/Chrome 정상 대조군 28건도 통과했다. 실제 Chromium AI 설정·패널에서 fresh WASM 초기화 응답의 해시까지 확인했다.
- Rust fmt·Native/WASM Clippy·workspace build·all-targets Clippy·suite manifest 통과. 집중 회귀 31건, Native 라이브러리 4,040 PASS/기존 12 ignored 및 누락 그림·직접 PDF 24건 통과.
- 전체 nextest 12,369 PASS/기존 45 skip, 실패·timeout 0건. 개인·상용 글꼴을 숨긴 최소 글꼴 환경에서 실행했다. 로컬 nextest 0.9.137의 CI duration JUnit 필드 경고는 명시하며, action의 새 버전 실행 증거는 실제 원 head CI로 구분한다.
- 조판 원본 4문서 전쪽 5쪽: Native/fresh WASM 각각 기준/후보 픽셀이 전쪽 동일. 독립 Print 대비 raw 최저 Native 94.40%·WASM 98.77%, 누락·쪽수 불일치·90% 미만·2%p 초과 하락 0건. 직접 2배 판독과 남은 기존 face·획 차이를 별도로 기록했다.

[실제 명령·종료 코드·입력·producer·글꼴·TSV·PNG·직접 판독](assets/dependabot-20261102/README.md), [검증 수량·skip/ignored 이유](assets/dependabot-20261102/validation-results-summary.json)에 보존했다. 원래 성공한 8개 head의 broad CI는 재사용했고 새 Skia 보정 때문에 결합 그래프의 필수 검증만 한 번 수행했다. 필수 실패·미실행을 GitHub CI에 넘기는 방식으로 완료하지 않았다.

## 다음 원격 조건

현재 원격 9건은 모두 고정한 원 Dependabot head이며 #8456만 원 빌드 실패가 남아 있다. 작업지시자 승인 뒤 각 원 PR의 최신 head·required checks와 실제 최신 devel의 clean merge-tree를 다시 대조한다. #8456은 보정 push 뒤 새 CI 성공이 필요하다. GitHub comment·approve·merge, workflow rerun·cancel, 보안 metadata 변경은 현재 수행하지 않았다.

# semanticist21 PR 10건 누적 검토 계획 — 2026-11-05

## 범위와 승인

작업지시자의 2026-11-05 지시에 따라 semanticist21의 Open·non-draft 중 요청 reviewer·team 및 타인의 미해제 리뷰가 없는 PR을 생성 시각 오름차순으로 10개 선정했다. 10개 원 PR 각각에 `edwardkim` reviewer를 지정하고 API로 확인했다. CI 실패의 메인터너 보정/피드백 선택과 필요한 CI 원인 댓글은 작업지시자가 위임했다. 새 원격 branch·Open 통합 PR·merge는 로컬 최종 후보와 원격 실행안을 갖춘 뒤 승인 범위를 확인한다. workflow 취소·재실행·검증 기준 변경은 아직 실행하지 않았다.

기본 경로: `maintainer_general`. 보조: `intake_and_review`, `local_validation`, `multi_pr_update_branch`, `visual_fixture_evidence`, `rework_and_exceptions`; 원격 단계에서는 `review_only_fast_pass`·`post_merge`를 적용한다. semanticist21은 기존 기여자다. 공통 검증은 누적하되 원 PR의 판정과 근거는 분리한다.

기준 base: `22947f6a90c998e95fa51238d5b907a0ebd37162`. 작업 branch: `review/integration-20261105-semanticist21-batch`. 공용 target은 기본 작업공간의 `target/pr-review`이고 Cargo 명령은 순차 실행한다. 기존 `pr-8296-seal-pua` worktree와 사용자의 배포용 WASM은 보존한다.

| 원 PR | 지정 시 고정 head | 최신 base simulation | 주장 |
| --- | --- | --- | --- |
| [#8326](https://github.com/edwardkim/rhwp/pull/8326) | `32c2a1e9febb303ac0662021279cf9171b346cb8` | 충돌 4개 파일 | fix(hwpx): 내보낸 HWPX의 칸 배경 그림과 HWP 그림 글머리표가 다른 그림을 가리키지 않게 한다 |
| [#8331](https://github.com/edwardkim/rhwp/pull/8331) | `5b36dbd078acd5add18592511682ce268dd66e82` | clean | perf(web_canvas): 회색 JPEG 판정이 디코드한 픽셀을 첫 캔버스 그리기에 넘긴다 |
| [#8332](https://github.com/edwardkim/rhwp/pull/8332) | `c773e5060c0f186b4bf8281133004f5afe120fa4` | clean | perf(equation): 같은 수식 값의 흐름 메트릭을 다시 계산하지 않는다 |
| [#8347](https://github.com/edwardkim/rhwp/pull/8347) | `19aed211d178996cf83aae89388ed91d6ec93bcf` | clean | fix(layout): 엇갈린 병합 칸의 행 높이를 칸 내용으로 푼다 |
| [#8349](https://github.com/edwardkim/rhwp/pull/8349) | `ffc0550e8b9c6db8627f9a82cc7f0f3ff39a5415` | clean | fix(renderer): 아래 일점쇄선을 글자 크기와 원본 run 위상에 맞춤 |
| [#8358](https://github.com/edwardkim/rhwp/pull/8358) | `9a709cdcd31300731cefbdbec4a978557f871183` | clean | fix(pagination): 구역 정의의 시작 쪽 번호로 쪽 번호를 매긴다 |
| [#8359](https://github.com/edwardkim/rhwp/pull/8359) | `2587dd58dac9a8bf312d7cd2f59b37e803d1a387` | 충돌 1개 파일 | fix(layout): 기본 탭 자리를 구역 정의의 기본 탭 간격으로 정한다 |
| [#8363](https://github.com/edwardkim/rhwp/pull/8363) | `9d6e420da55d4b9d9684c0e9c8f61cdf84ff29ff` | clean | fix(renderer): 칸 대각선 모양 값의 꼭짓점 비트만 대각선으로 그린다 |
| [#8366](https://github.com/edwardkim/rhwp/pull/8366) | `5993e4d32e08cce8785e79364dd6ac66eab80dfb` | clean | fix(renderer): 입체선 글자·문단 테두리는 긋지 않는다 |
| [#8367](https://github.com/edwardkim/rhwp/pull/8367) | `0f73bc67fcffb6126c745d86685df893ffb5267b` | clean | fix(layout): 머리말·꼬리말의 번호·개요 문단에 번호를 그리고 본문과 한 계수기로 센다 |

## CI 조사와 판단

- #8331 Render Diff job 122831095416: checkout 36분 51초 뒤 WASM 빌드 중 job 45분 상한, Canvas 비교 미실행.
- #8347 Lint 122978871306: native/WASM Clippy 성공 뒤 workspace 검사 중 20분 상한. Archive B worker 122992608288은 checkout 중 60분 상한.
- #8349 Lint 123059106152: workspace 검사 중 20분 상한. Archive B builder 123059178187은 build 중 30분 상한.
- #8366 Archive C builder 123331958306: checkout 중 30분 상한.

각 job의 실제 annotation·step·시각·로그를 확인했다. 해당 실패 로그에서 assertion·컴파일 오류는 발견하지 않았다. 실패를 PASS로 바꾸지 않으며 미실행 검사는 미실행이다. 최신 base는 이미 Lint 45분·archive builder 60분 정책을 가진다. 기여자 코드 재작업 댓글 대신 현재 정책과 통합 후보에서 필수 검증을 수행하는 메인터너 경로를 선택한다. 60분 checkout 정체는 단순 예산 연장으로 해결됐다고 주장하지 않는다. 제품·검사 명령·상한은 이번 조사만으로 바꾸지 않는다.

## 승인된 충돌 보정 방침

작업지시자는 2026-11-05에 “기존 계약 보존·메인터너 보정”을 승인했다. 아래 방침의 통합과 실제 보정은 완료했으며 원 기여 merge와 메인터너 수정 commit을 구분했다. 최종 검증·판정은 [검토 결과](semanticist21_20261105_review.md)에서 갱신한다.

1. #8326 네 파일: 최신 `Bullet::image_reference`, `hwpx_image_without_ref`, 그림 효과와 오류 처리, streaming writer·Storage 예산, HWPX 기본 문단 모양 scope를 유지한다. HWP 그림 글머리표는 #8438→#8517의 기존 해결 범위와 원 입력을 먼저 대조한다. 남은 header 그림 ID 정규화와 참조 바이트 보존을 통합하며 오래된 모델 가정을 복원하지 않는다. 기존에 해결된 범위와 새 보정을 구분한다.
2. #8359 한 파일: 최신 fallible style resolution·Storage·HWP3 native 출처·TTF/HFT 선택을 유지하며 구역 기본 탭 처리를 통합한다. 첫 구역 값을 모든 구역에 쓰는 원 PR의 제한은 실제 두 구역 경계와 사양으로 검토하고, 새 결함을 만들면 보정하거나 원 PR별로 보류/해제 조건을 기록한다.
3. 누적 적용 중 같은 계약의 인접 충돌도 원 기여 기능과 최신 base 계약을 모두 유지하는 방향으로 처리하고 원 PR·원 SHA·보정 SHA를 구분한다. 의미가 다른 해결책이나 범위 확대는 먼저 기록한다.

## 실제 검증 순서와 완료 조건

1. 각 PR의 관련 이슈·선행 해결/통합 commit·독립 증거와 현재 base/head를 대조한다. 이미 해결된 것과 미해결을 구분한다.
2. 원 기여 commit·순서를 보존한 누적 후보를 구성한다. 원 branch를 임의로 갱신하지 않으며 원 head와 보정 commit을 구분한다.
3. 같은 공개 입력·독립 한컴 Print PDF에서 고정 base의 전쪽 Native/fresh WASM TSV를 먼저 만들고 최종 후보와 비교한다. 전쪽 raw 90% 이상·2%p 초과 하락 없음·쪽수 일치와 변경/경계 영역 확대 판독을 모두 확인한다. 원 PR의 수치를 메인터너 직접 측정으로 바꾸어 쓰지 않는다.
4. #8331은 실제 WASM Canvas decode 인계의 픽셀·수명·다음 큰 그림·밝기 보정과 같은 환경 첫 그리기 비용을, #8332는 실제 편집/재조판 경로의 키 변경·pass 초기화·출력 보존과 비용을 대조한다. 성능 결과는 측정 환경과 한계를 적는다.
5. #8347의 행 높이 측정/배치·분할/후속 내용, #8349의 Native/SVG/Canvas2D/CanvasKit 기하·run 위상, 구역/번호/대각선/테두리의 정상 대조군을 실제 실행한다. 관련 없는 대형 재작성으로 범위를 늘리지 않는다.
6. 필수 최종 Rust lint 묶음·suite/unit 정책·최소 글꼴 focused/전체 release-test·Native 공식 3종·최적화 fresh WASM·해당 Studio 검사와 실제 브라우저·입력 commit/보안 확인을 완료한다. 조판 영향이 있으므로 최종 head에서 Native 편람 383쪽 `--handbook-gate`를 통과한다.
7. 원 PR별 최종 review·검증 증적·CI 처리 판단·필요한 댓글을 준비한다. 통합 후보가 준비되면 exact SHA·증거와 함께 원격 push/Open PR/CI/self-review/merge 실행안을 구체화한다. 필수 증거·회귀 실패는 완료로 대신하지 않는다.

로컬 통합의 원 head·merge commit·추가 보정 ancestry는 [원 기여 보존 기록](../assets/semanticist21-20261105/contribution-ancestry.json)에 있다. 원격 실행은 본 계획의 별도 승인 범위를 유지한다.

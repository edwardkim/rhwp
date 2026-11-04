# PR #8284 리뷰 — Dependabot 21건 통합과 SDK 1.x 회귀 보정

## 최종 판정

**승인.** 통합 후보의 로컬 검증과 정확한 code head의 CI·CI Impact Policy 성공을
확인하고 일반 merge했다. 사용자가 승인한 옵션 2로 이 통합 검토와 오늘할일을 별도 문서 PR에
반영한다. 봇 원 PR별 review 문서는 생성하지 않는다.

- 코드 PR: [#8284](https://github.com/edwardkim/rhwp/pull/8284), 작성자 `jangster77`, base `devel`.
- 검증·병합 head: `3f675bb96c7a9714d7bb8f091ca892e7ce5090de`.
- 병합 직전 base: `49bc935a149c20b942d3045c2f5c733aeccf57d0`.
- merge SHA: `9d18fa05be06f4960c80b492cfba4131a2557b6e` (2026-10-26 22:12 KST).
- 통합 브랜치: `review/integration-20261026-dependabot-updates-r2`.
- 기록 브랜치: `review/integration-20261026-dependabot-review-records`.
- 통합 PR은 collaborator self-review이며 self reviewer를 지정하지 않았다. 원 PR 21개는 상세
  조사·fetch 전에 reviewer `jangster77`을 지정하고 API로 확인했다. 원 PR head는 변경하지 않았다.

## 변경과 선행 분석

원 PR #8241~#8261의 고정 head 21개를 번호 순서대로 `git cherry-pick -x`했다.
원 저자·SHA·적용 순서와 리베이스 전후 적용 SHA는
[통합 계획](../../working/review_integration_20261026_dependabot_plan.md)과
[검증 원장](../assets/dependabot_20261026/validation.json)에 보존했다.
별도 대표 이슈가 없어 날짜를 식별자로 썼다. 관련 이슈를 임의로 만들거나 과거 작업을 재개하지 않는다.

| 원 PR | 통합 내용 | 소비 경로·검토 |
| --- | --- | --- |
| #8242·#8245·#8248·#8251·#8253·#8255·#8256·#8257 | Rust 8건 | read-fonts 0.46/skera 0.9, serde_json, CRC, ml-kem, ZIP, proptest, sha2 |
| #8241·#8243·#8244·#8246·#8247·#8249·#8250·#8252 | npm 8건 | Studio·Chrome·Firefox의 Vite/Puppeteer, SDK 1.1, Chrome types, VS Code webpack |
| #8254·#8258·#8259·#8260·#8261 | Actions 5건 | setup-python, CodeQL init/analyze, install-action, rust-cache의 고정 SHA |

누적 충돌에서는 먼저 반영한 read-fonts 0.46·Puppeteer 25.16·Vite 8.3.5를 보존했다.
Cargo의 단일 버전 rand_core/getrandom 참조를 정규화하고 참조가 없어진 read-fonts 0.45.1의
13줄 package 항목을 제거했다. `--locked` 검증으로 최종 lockfile을 확인했다.

SDK 검사의 원 해결은 #8228 merge `3e02fe2749866a8453af3eb3901b9c1a674cccaf`다.
공식 [0.137.0 client](https://github.com/anthropics/anthropic-sdk-typescript/blob/sdk-v0.137.0/src/client.ts)의
X-Api-Key와 [1.1.0 client](https://github.com/anthropics/anthropic-sdk-typescript/blob/sdk-v1.1.0/src/client.ts)의
Bearer 인증을 대조했다. 구 버전에서 유효했던 검사와 응답 대역을 새 계약에 맞췄다.
제품 인증 구현은 SDK를 소비하는 기존 경로를 유지한다.

- `rhwp-studio/e2e/ai-panel.test.mjs`: 정확한 Bearer 키·기존 x-api-key 미전송·API 2026-10-21을 검사한다.
- `rhwp-studio/e2e/ai-settings.test.mjs`: 잘못된 키와 올바른 키의 전송값을 각각 검사하고,
  응답 대역도 같은 Bearer 계약으로 401/성공을 분기한다.
- Action의 낡은 버전 주석은 실제 고정 commit에 맞췄다. CodeQL init/analyze는 4.39.1의 같은 SHA,
  rust-cache는 workspace root hash 보정 commit `c8037a86dde8214a8310e9f54a8fc61ab362aa31`이다.
  workflow permission·event·archive matrix·worker timeout은 이 통합에서 변경하지 않았다.

## 로컬 검증과 리베이스 수용

| 검증 | 실제 결과 |
| --- | --- |
| 전체 workspace nextest | **11,573 PASS·45 skip·exit 0**, 실행 4,740.241초, 사전 컴파일 시간 별도 |
| Rust lint·빌드 | fmt, Native/WASM32/workspace Clippy, workspace build, suite 정책 통과 |
| npm | 네 package npm ci·dependency 정합성·빌드 통과; Studio **2,572 PASS·2 skip** |
| 실제 브라우저 | AI 패널 94개·설정 20개 단언, Chrome package 실행, Firefox 다운로드 3회·편집·저장 통과 |
| workflow | Python 195개·Node CI impact 계약 334개·actionlint 통과 |

Cargo는 `/home/tsjang/rhwp/target/pr-review`를 순차 재사용했다. 전체 명령은
`cargo nextest run --locked --cargo-profile release-test --target-dir /home/tsjang/rhwp/target/pr-review --tests --test-threads 12 --no-fail-fast`다.
Rust 원 입력은 `36f255768fbfca2601a9a5499b1e92166212c0e1`, 최종 로컬 후보는
`d612e9f1710e6bf72f8bd21538820eeb41f9529c`다. 그 사이 Rust/Cargo/test 입력은 동일하다.
AI 응답은 가로챈 대역이며 실제 키나 외부 Anthropic API를 사용하지 않았다.
Firefox의 초기 Snap /tmp 프로필 접근 실패는 공유 가능한 TMPDIR로 보정한 실제 재실행에서 해소했다.

사용자 지시: “회귀 테스트가 끝나면 upstream/devel 위로 리베이스 충돌하는곳이 없으면 현재
회귀테스트를 공식인정. 다시 테스트 할 필요 없음.”

이 지시에 따라 `d612e9f...`를 최신 base `49bc935...` 위로 27개 commit 모두 충돌 없이
리베이스했다. 결과 `648ed728b8f0514cbf16cc7d5775b5a12787db59`와 그 뒤 증적 commit으로
제출 head를 만들었다. 들어온 #8278·#8280은 문서 전용 duration trigger와 검토 기록이다.
Rust·Cargo·기존 test·Studio/extension/shared package 입력의 전후 diff는 없다.
현재 검증을 공식 채택했으며 같은 로컬 회귀를 재실행하지 않았다.

## 조판 원칙과 검증 입력

제품 Rust source·조판 규칙·줄 구성·좌표·문서별 예외·회귀 baseline은 변경하지 않았다.
따라서 새 조판 분기나 값의 생산→측정→배치 전파 검토는 비해당이다. 직접 영향을 받는 글꼴
소비 경로는 `src/renderer/svg/webfont_subset.rs`이며 `src/renderer/svg.rs`의 cfg로 wasm32에서
제외된다. read-fonts/skera를 Canvas2D·CanvasKit·Native Skia text layout의 변경으로 확대하지 않는다.

실제 검증 입력은 [samples/issue2470/36382471_masked.hwpx](../../../samples/issue2470/36382471_masked.hwpx)다.
입력 SHA-256은 `43572dad5e17395aa02d1b0000b736b8467278931086604776ef30393dd0f54b`이며
검토 commit에 포함된 기존 입력과 일치했다. 새 원본/PDF나 50 MiB 이상 PDF는 추가하지 않았다.
실행한 Native debug binary의 SHA와 source anchor는
[SVG 증거](../assets/dependabot_20261026/svg-control.json)에 기록했다.

두 쪽 모두 실제 SVG의 부분 글꼴과 전체 face 출력을 브라우저에서 래스터화했다.
font payload/format 이외 SVG는 동일하다. 독립 FontTools로 사용 문자의 cmap·전진 폭·좌측 여백·
윤곽 좌표/플래그·공통 수직 메트릭의 보존을 확인했다. 두 쪽 변경 영역을 같은 배율로 확대해
문자·표·간격을 직접 판독했다.

| 쪽 | 색상 경계 보정 전 2px 실루엣 | 정수 미대응 | RGB 차이 |
| --- | --- | --- | --- |
| 1 | 100% | 0 / 0 | 325px |
| 2 | 100% | 0 / 0 | 8,974px |

이는 **부분 글꼴/전체 face 대조**다. 독립 한컴 Print 일치율이나 RGB 동일성을 뜻하지 않는다.
PNG 네 개와 의미 비교 결과를 [영구 증적](../assets/dependabot_20261026/svg-control.json)에 보존했다.
기존 sample16의 원본·Print·전쪽 TSV·기대값은 보존했다. 추가 Native 전체 배치와 로컬 fresh WASM/
384쪽 재산출 배치는 실행하지 않았으며 이전 전쪽 결과를 새 검증에 합산하지 않는다.
정확한 통합 head의 CI Frontend package job에서는 fresh WASM 빌드·실제 패키지 계약 검사가 성공했다.

## 정확한 head의 원격 CI와 병합

[CI #40323586840](https://github.com/edwardkim/rhwp/actions/runs/40323586840)의
builder와 worker 결과·artifact를 대조했다. 다음 수는 각 archive의 예상 수와 실제 실행 수가 같다.

| Archive | 예상 = 실제 통과 |
| --- | --- |
| A | 3,801 |
| B | 1,947 |
| C | 57 |
| D | 2,321 |
| E | 3,253 |
| 합계 | **11,379** |

Native Skia job은 15개 비어 있지 않은 검사 묶음의 **120 PASS**를 로그에서 확인했다.
Lint, Frontend package, [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/40323586696),
[Render Diff](https://github.com/edwardkim/rhwp/actions/runs/40323586711),
[Adapter](https://github.com/edwardkim/rhwp/actions/runs/40323586742),
[Proptest](https://github.com/edwardkim/rhwp/actions/runs/40323586766), Skill router도 성공했다.
총 38개 check가 완료되었고 success/skipped/neutral만 존재했다. CI Impact Policy는
`v=8;cv=8;mode=full;rfp=0`이며 `b=49bc935a149c20b942d3045c2f5c733aeccf57d0`로 성공했다.
같은 최신 base/head의 MERGEABLE/CLEAN과 merge-tree exit 0을 확인하고 head를 고정해 일반 merge했다.
[CI 수치와 gate 기록](../assets/dependabot_20261026/ci.json)을 보존한다.

## Merge 후 contributor PR comment 계획

별도 문서 PR의 fast-pass·merge와 devel 동기화 뒤 통합 PR 한 건과 원 봇 PR 21건에
통합 결과를 남긴다. 원 PR의 최신 head와 상태를 다시 조회하고 같은 merge/증적 comment는
중복 게시하지 않는다. 변경이 전부 포함된 OPEN 원 PR은 **체리픽 통합으로 종료**하며
원 PR 자체를 직접 merge했다고 표시하지 않는다. 봇 원격 head는 수정·삭제하지 않는다.

문안은 한국어 존댓말로 감사·통합 PR·merge SHA·실제 CI·검토 링크·보정 이유와 남은 범위를
명시한다. SVG 시각 근거는 [Visual Sweep 정본](../../manual/verification/visual_sweep_guide.md#github-merge-comment),
위 두 쪽의 실제 수치·직접 판독·대조군의 한계와 연결한다. 통합 PR에는 merge SHA 고정
`svg-subset-001.png`와 `svg-full-001.png`를 표시한다. 게시 후 API로 본문·링크를 재확인한다.

병합 뒤 [duration metadata 실행](https://github.com/edwardkim/rhwp/actions/runs/40328617344)이
성공했다. 정확한 PR/attempt의 B/C/D/E 실측 네 개를 확인해 46개 target을 갱신하고
metrics commit `9b071963`을 발행했다. 검증 CI는 재실행하지 않았다.
문서 반영·comment·원 PR 종료 후 이번 작업의 전용 브랜치만 exact SHA 조건으로 정리한다.
기본 작업공간·공유 target·다른 작업/과거 PR의 브랜치와 산출물은 임의로 정리하지 않는다.
필요한 실행 로그는 `/home/tsjang/rhwp-pr-evidence/dependabot-20261026`에 해시 대조해 보존한다.
output 전체의 소유·실행 중 사용을 확인하고 타 작업이 사용 중이면 삭제 보류와 경로를 기록한다.

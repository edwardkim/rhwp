---
kind: pr_review
status: completed
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-01
---

# PR #6562 검토 — 인쇄 판형 표준 치수 스냅

## 병합 결과

- PR #6562는 2026-09-01T08:38:57Z에 maintainer `edwardkim`이 정상 merge commit
  방식으로 병합했다.
- 최신 source head: `6ac4d298bef44528317e13618e325bf7bda4aa3d`
- source merge commit: `b33752594e5adaa85adc78481f171f81272aeb58`
- squash, rebase, `--admin` 우회는 사용하지 않았다.
- 이슈 #6561은 Close Issues on devel Push workflow로 2026-09-01T08:39:10Z에
  자동 종료됐다.
- source merge SHA의 CI, CodeQL, Proptest roundtrip, Adapter inter-diff와 issue-close
  workflow가 모두 성공했다.
- 이 archive와 오늘할일은 source merge 뒤 maintainer option M 운영 기록으로 반영한다.

## 결론

**승인.** contributor code commit `864cc39ff2273ffa32a32cc520028d46313af918`은
HWPUNIT 양자화와 페이지 정보의 소수 1자리 px 직렬화로 생긴 작은 오차를 인쇄 mm
경계에서만 표준 판형 치수로 스냅한다. 최신 `devel`을 합친 source head
`6ac4d298bef44528317e13618e325bf7bda4aa3d`와 local integration head의 tree가
일치했고, focused test, Studio 전체 unit test, CI unit TypeScript 검사, 실제 Chrome
PDF 계측과 최신-head GitHub Actions가 통과했다.

다만 이슈 #6561의 “PDF 바이트 페이지 크기가 595.28×841.89pt가 되어야 한다”는 완료
조건은 Chrome 출력 계약과 맞지 않는다. 직접 계측에서 수정 전·수정 후·Chrome의
`size: A4`가 모두 594.96×841.92pt였고 Poppler는 셋 모두 A4로 판독했다. 이 PR의 실제
효과는 PDF MediaBox 변경이 아니라 `PrintPage.widthMm/heightMm`과 `@page size`를
`210mm 297mm`로 복원하는 것이다. merge와 자동 issue close 뒤 이 정정을 기록한다.

이 문서의 `승인`은 검토 판정이며 GitHub approve, comment 또는 merge 실행 승인이
아니다. source branch update와 정상 merge는 각각 별도 승인을 받아 완료했다.

## 라우팅

- 기본 경로: `maintainer_general.md`
- 보조 경로: `intake_and_review.md`, `local_validation.md`,
  `visual_fixture_evidence.md`, `multi_pr_update_branch.md`, `post_merge.md`
- 작성자는 기존 기여자이므로 `first_time_contributor.md`는 적용하지 않았다.
- 같은 작성자의 #6564는 stacked PR이 아닌 별도 변경이다. #6562를 먼저 처리하고 최신
  `devel`을 만든 뒤 #6564의 충돌을 별도로 해결한다.

## 메타데이터와 범위

| 항목 | 값 |
| --- | --- |
| 원 PR / 작성자 | [#6562](https://github.com/edwardkim/rhwp/pull/6562) / @jeong-sik |
| 관련 이슈 | [#6561](https://github.com/edwardkim/rhwp/issues/6561) (`closes #6561`) |
| base / draft | `devel` / 아님 |
| contributor code commit | `864cc39ff2273ffa32a32cc520028d46313af918` |
| 최신 source head | `6ac4d298bef44528317e13618e325bf7bda4aa3d` |
| 검토 기준 devel | `c65fc42e851dcd4d65f2a32ac5182331d4808ef8` |
| local integration head | `dd35a0185766f162317be55c1e64aeb5b62ef0a7` |
| 변경 규모 | 2 files, `+63/-1`, 1 commit |
| current-base merge simulation | 충돌 없음, 결과 tree `9a3bb6ae5570156d15cf8616ad8a57b97629c7c8` |
| GitHub 참고 상태 | 최신 source head에서 `MERGEABLE`, `CLEAN`, required checks 성공; merge 직전 재확인 필요 |
| reviewer | `edwardkim` 요청 상태 확인 |

변경은 `rhwp-studio/src/command/print-pages.ts`의 인쇄 mm 환산과 그 계약 테스트에
한정된다. 편집 Canvas의 px 조판, Rust, WASM API, dependency와 lockfile은 바꾸지 않는다.

## 코드 검토와 보호 불변식

- 스냅은 `createPrintPage`가 사용하는 인쇄 mm 환산 경계에만 적용한다.
- 허용 오차는 0.05mm이며, 0.1mm 차이의 비표준 크기는 보존한다.
- A3~A5, JIS B4/B5, Letter, Legal, Tabloid의 변 길이를 방향과 무관하게 처리한다.
- 페이지별 named `@page`, 혼합 가로·세로 방향, `break-after`, `overflow: hidden`,
  `margin: 0` 계약은 유지한다.
- 실제 wire A4 값 793.7×1122.5px는 210×297mm로 복원된다.
- Chrome PDF MediaBox는 수정 전과 후가 동일하다. 이 결과를 “PDF 물리 크기 수정”으로
  과장하지 않고 CSS/API의 표준 판형 식별 복원으로 제한한다.

0.05mm 경계값 자체는 JavaScript 부동소수 표현에 따라 포함 여부가 달라질 수 있으나,
이번 원인 오차는 최대 약 0.018mm이고 공개 입력 단위의 비표준 크기는 0.1mm 간격이다.
검토한 범위에서는 이 경계 표현 차이를 blocker로 보지 않았다.

## 로컬 검증 결과

### 기준선과 GitHub CI

PR 생성 때의 GitHub merge ref `9fe63b8580721ca5ceb1ca99ea54af2a8db10bcd`는
부모가 `6d3fd65a...`와 contributor head였고, 현재 `devel`의
`c65fc42e...`를 포함하지 않았다. 따라서 당시 녹색 CI를 current-base 전체 검증으로
재사용하지 않았다.

- [CI run 33480000395](https://github.com/edwardkim/rhwp/actions/runs/33480000395):
  contributor head의 Frontend package gates와 Build & Test 성공
- 로컬 review branch에서 현재 `devel`을 충돌 없이 병합한 integration head
  `dd35a0185`를 별도로 검증했다.
- 승인된 update-branch API로 current `devel`을 source branch에 merge했다. 새 head
  `6ac4d298b`의 부모는 contributor code commit `864cc39ff`와 current `devel`
  `c65fc42e`이며, tree `9a3bb6ae...`는 local integration head와 동일하다.
- 최신 source head의 CI·Proptest·CodeQL·Render Diff·adapter workflow run
  [33487747519](https://github.com/edwardkim/rhwp/actions/runs/33487747519),
  [33487747504](https://github.com/edwardkim/rhwp/actions/runs/33487747504),
  [33487747556](https://github.com/edwardkim/rhwp/actions/runs/33487747556),
  [33487747357](https://github.com/edwardkim/rhwp/actions/runs/33487747357),
  [33487747839](https://github.com/edwardkim/rhwp/actions/runs/33487747839)가 모두
  성공했다. code diff가 바뀌지 않아 trusted post-merge reuse가 적용됐고,
  `Build & Test`와 모든 preflight가 성공했으며 재사용 대상 광범위 job은 정책대로
  skip됐다.

### Studio와 계약 테스트

| 검증 | 결과 |
| --- | --- |
| `git diff --check upstream/devel...HEAD` | 통과 |
| `node --test tests/print-pages.test.ts` | 통과: 11/11 |
| `npm test` | 통과: 1,341 pass / 1 skip / 0 fail |
| `npx tsc --project tsconfig.ci-unit.json --noEmit` | 통과 |

처음 실행한 raw `npx tsc --noEmit`은 새 worktree에 생성된 `pkg/rhwp.js`가 없어
`@wasm/rhwp.js`를 찾지 못했다. PR 코드 오류가 아니라 fresh WASM 파생물이 없는 환경
조건이므로, Rust/WASM을 바꾸지 않는 이 PR의 local current-base 검증은 CI unit용 stub을
쓰는 `tsconfig.ci-unit.json`으로 완료했다. contributor code commit의 GitHub Frontend
package gate는 fresh WASM package를 사용해 이미 성공했고, current-base source head는
동일 code diff에 대한 trusted post-merge reuse 검증을 통과했다.

`npm ci`는 기존 dependency에서 vulnerabilities 3건(낮음 1, 높음 2)을 보고했지만 이
PR은 dependency와 lockfile을 바꾸지 않는다.

## 실제 Chrome PDF 계측

Chrome for Testing 152.0.7977.54에서 실제 `page.pdf({preferCSSPageSize: true})`를
호출하고 Poppler `pdfinfo` 24.02.0으로 확인했다. 세 PDF 모두 1쪽이며 아래와 같았다.

| 입력 CSS | PDF page / MediaBox | Poppler 판독 |
| --- | --- | --- |
| 수정 전 `210mm 296.995mm` | `594.96×841.92pt` | A4 |
| 수정 후 `210mm 297mm` | `594.96×841.92pt` | A4 |
| Chrome keyword `size: A4` | `594.96×841.92pt` | A4 |

임시 계측 산출물과 SHA-256은 다음과 같다.

- `output/pr6562-review/before-296_995mm.pdf` —
  `de458cf61572b4798f2895a0b120d2130798bfe7874338bb0379f4abf1d12566`
- `output/pr6562-review/after-297mm.pdf` —
  `2d81234696755f8bf6dc3a3d8e3fa871ba1f11fabcfbcb7175cceb63f14f7b5f`
- `output/pr6562-review/keyword-a4.pdf` —
  `7d32b0b983b49181ec39a462e309b1e16cacab22ca6f8d3a027bb40184a69785`

이 PR은 HWP/HWPX fixture나 본문 렌더링을 바꾸지 않는다. 따라서 한컴 기준 PDF와
pixel visual sweep은 적용하지 않았고, 사용자-visible 판정은 실제 Chrome의 CSS page
계약·페이지 수·MediaBox 비교로 제한했다. 임시 PDF는 장기 증적이나 source PR 제출
파일이 아니므로 commit하지 않는다.

## 잔여 위험과 이슈 기록 정정

- 표준 판형 판정은 너비·높이 각각의 알려진 변 길이를 스냅한다. 판형 이름 enum을 새로
  도입하지 않으므로 혼합 방향과 사용자 정의 판형을 계속 허용한다.
- 0.05mm 안쪽의 의도적인 사용자 정의 값도 표준 변 길이로 스냅될 수 있다. HWP의 공개
  0.1mm 입력 정밀도와 현재 양자화 원인을 고려하면 허용 가능한 교환이지만, 더 높은
  정밀도의 외부 입력 경로가 생기면 재검토해야 한다.
- #6561의 PDF point 완료 조건은 Chrome의 A4 keyword 출력과도 맞지 않는다. merge 후
  issue comment에서 “MediaBox 수치 변경”이 아니라 “CSS/API 표준 치수 복원”이 완료된
  범위임을 계측값과 함께 정정해야 한다.

## 최종 판정

- 판정: **승인**
- 대상: 최신 source head `6ac4d298bef44528317e13618e325bf7bda4aa3d`
  (contributor code commit `864cc39ff2273ffa32a32cc520028d46313af918` 보존)
- 완료된 조건: current `devel` source update, 동일 integration tree 확인, 최신-head
  GitHub required checks 성공, `MERGEABLE`·`CLEAN` 확인
- merge 결과: source head `6ac4d298b`를 정상 merge commit
  `b33752594e5adaa85adc78481f171f81272aeb58`로 병합했다.
- post-merge 검증: CI·CodeQL·Proptest·Adapter·issue-close workflow 성공
- 남은 후속 조건: archive와 오늘할일을 `devel`에 반영한 뒤 #6561의 잘못된 PDF point
  완료 조건 정정 comment → contributor PR comment → worktree·branch 정리
- 원격 조치: 승인된 source update와 정상 merge 외에 GitHub approve 또는 comment는
  수행하지 않았다.

## Merge 후 contributor PR comment 계획

source merge SHA와 archive 기록 commit이 확정된 뒤 다음을 PR comment로 게시한다.

- 최신 source head와 정상 merge commit 연결
- focused 11/11, Studio 1,341 pass, TypeScript와 실제 Chrome 3조건 계측 결과
- 수정 전·수정 후·A4 keyword가 동일한 MediaBox를 만들며, 이번 수정 범위가 CSS/API의
  `210mm 297mm` 복원이라는 정확한 설명
- UTF-8 without BOM `--body-file` 게시 후 API로 한글·BOM·`??` 치환 검증

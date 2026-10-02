---
kind: plan
status: active
last_verified: 2026-10-02
---

# semanticist21 열린 PR 7건 누적 검토 계획

## 범위와 권한

작업지시자의 일괄 검토 시작 승인에 따라 #7487, #7491, #7493, #7497, #7498,
#7504, #7508을 검토한다. 원격 push·comment·PR 생성·merge·close는 아직 승인되지 않았다.
검토 브랜치는 `review/semanticist21-20261002`, 기준은 최신 devel
`e5098bc91be44a49367a7f2895a14fcd4f4c2c7f`이다. 기존 #7353 작업은 변경하지 않는다.

## 누적 순서

| PR | 고정 head | 변경 축 |
| --- | --- | --- |
| #7487 | 38c0af21a4370876da2fa34178f25d0ce0a782e0 | Enter 후 빈 페이지 소유 |
| #7491 | c4367ec03a28369cc6f26b17eca46553ac61514c | 편집 문단 들여쓰기 |
| #7493 | d29d483e4f5fc759c067e33766700bcad87133ed | 수정 모드·IME·Undo |
| #7497 | 64f76e37b31bad9c0dedcb9bde67cc7a8f70eb58 | HTML 인라인 그림 붙여넣기 |
| #7498 | 98133ad9c57c695fce2dcf8d684e801c8d431438 | 채우기 없음 조회 JSON |
| #7504 | 7dc340284bd84e2ee475da3b577146005b989500 | 커닝 글리프·캐럿·등록 글꼴 비용 |
| #7508 | b53d3621be516ed8f918b321a0e0b01f09f8ff19 | 첫 문단 복사 시 구조 컨트롤 슬롯 해제 |

원 author와 `-x` 출처를 유지해 기능 commit을 위 순서로 적용한다.
#7487의 `b28130e1`, 메인터너 보정 `45863eb2`는 함께 검토한다. 문서/asset commit
`38c0af21`은 누적 코드 검증에서 제외한다. 이 commit의 `mydocs/orders/20261002.md`
add/add 충돌은 원 기록을 덮어쓰지 않고 별도로 판정한다. 원 head의 증적은 직접 읽는다.
다른 6건은 현재 devel과 merge-tree 충돌이 없다. 누적 적용 충돌은 따로 기록한다.

## 검증 및 판정

1. 원 head 코드·본문·테스트·CI를 대조한다. 변경값의 실제 소비 경로와 편집/Undo 경계를 확인한다.
2. 공유 `target/pr-review`를 순차 사용해 6개 Rust 신규 사례와 관련 대조군만 먼저 실행한다.
   Studio 수정 모드는 실제 명령을 실행하는 테스트와 TypeScript 검사를 수행한다.
   generated integration suite는 검증 전용이며 stage하지 않는다.
3. 실행 결함, 코드 검토상 우려, 필수 증거 부족을 구분한다. 렌더링/페이지 변경의
   신규 회귀는 현행 독립 PDF 및 Native/fresh WASM 최저 90% 선행 조건으로 판정한다.
   합성 계약·원 PR의 CI 녹색을 독립 시각 증거로 간주하지 않는다.
4. 필요한 fresh WASM/시각 검증과 비용이 큰 누적 전체 검증은 선행 결과에 따라 결정한다.
   기존 exact-head CI를 이유 없이 반복하지 않는다. 아직 장시간 전체 회귀를 시작하지 않는다.
5. 원 PR별 검토 기록과 누적 검증 결과를 보고하고 필요한 다음 승인만 요청한다.

## 시작 상태

7건의 reviewer `edwardkim` 지정 완료. #7487은 문서 add/add 충돌,
나머지는 clean merge simulation이다. #7504·#7508의 CI는 최초 조회 때 진행 중이었다.
원격 게시나 통합은 수행하지 않았다.

## 누적 검증 결과

코드 candidate: `c6ef30ea943308c37e5d68c8304dfdabdd7b8f74`.
22개 기능 commit을 `-x`로 적용했고 author를 유지했다. 코드 충돌·메인터너 코드 보정 없음.
최종 수용 그룹을 따로 구성하지 않았으며 이 candidate 자체를 게시/통합하지 않는다.

| 검사 | 실행 결과 | 범위 및 제한 |
| --- | --- | --- |
| Rust focused | 61 PASS / 1 FAIL | 신규 6개 축 및 #4968 포함 62개, 1233개 필터 제외 |
| #6190 원 저장본 대조 | 1 PASS | #7491 저장 들여쓰기 대조 |
| TypeScript | PASS | Studio tsconfig --noEmit |
| Studio 전체 | 1813 PASS / 2 skip / 0 FAIL | 수정 모드 wrapper의 skip은 직접 runner로 보완 |
| overwrite runner | 25 PASS | 실제 TS 명령 + 작성자 mock core |
| 추가 수정 모드 반례 | 3 FAIL | astral 연속 입력 / scalar caret 뒤 명령 병합 Undo / deactivate 조각 잔류 |
| fmt · manifest · diff whitespace | PASS | 정책 base e509 고정 |
| fresh WASM build | PASS | host wrapper, dev profile, 별도 output pkg; Studio public 갱신 없음 |
| 실제 브라우저 WASM API | 3 PASS | #7497·#7498·#7508, 독립 한컴 시각 판정은 아님 |
| 원 PR 7건 exact-head CI | 모두 성공 | 누적 candidate의 Full CI로 간주하지 않음 |

### 실행 명령과 증적

로그 기본 경로: `output/pr-review/semanticist21-20261002/logs/`(ignored, 이번 로컬 진단).

```bash
cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review \
  --test regression_suite_005 --test regression_suite_009 \
  --test regression_suite_011 --test regression_suite_018 \
  --test regression_suite_019 --test regression_suite_022 \
  -E 'test(issue_7486_) | test(issue_7490_) | test(issue_7496_) | test(issue_7495_) | test(issue_7503_) | test(issue_7506_) | test(issue_4968_)'
cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review \
  --test regression_suite_017 -E 'test(issue_6190_)'
npm --prefix rhwp-studio exec -- tsc --project rhwp-studio/tsconfig.json --noEmit
npm --prefix rhwp-studio test
node --experimental-transform-types --no-warnings rhwp-studio/tests/support/overwrite-mode.runner.mjs
node --experimental-transform-types --no-warnings output/semanticist21-20261002-overwrite-boundaries.mjs
cargo fmt --all -- --check
node scripts/rust-test-suite-manifest.mjs --check --base-ref e5098bc91be44a49367a7f2895a14fcd4f4c2c7f
CARGO_TARGET_DIR=target/pr-review scripts/wasm-pack-locked.sh --target web --dev \
  --out-dir output/pr-review/semanticist21-20261002/pkg
node output/semanticist21-20261002-browser.mjs --mode=headless
```

`semanticist21-20261002-focused.log` 및 `-indent-control.log`에 Rust 결과,
`-studio-all.log`·`-overwrite-runner.log`·`-overwrite-boundaries.log`에 Studio 결과,
`wasm-build.log`·`browser.json`에 실제 WASM source SHA·package hash·API 결과를 보존했다.
`ci-heads.json`은 PR별 최신 head와 check URL의 조회 원문이다.

WASM SHA-256: `7397517c38cee810b31536c17366118fb2150f26d8d4890c270f107b786dfe4b`.
HeadlessChrome 152.0.0.0. 표준 Docker 경로 대신 host의 locked wrapper를 사용했다.
root pkg/Studio public을 변경하지 않았고 사용자 CDP 편집 화면에 접근하지 않았다.

### 실패 분류와 devel 대조

#7491의 실패 assertion은 표 호스트 앞 글자 입력 후 우변 710.6px > 본문 699.2px이다.
동일 샘플·API·편집 순서를 devel e509에서 실행해 동일 좌표를 관측했다.
`output/semanticist21-20261002-indent-probe.rs`를 release-test librhwp에 연결했다.
baseline 빌드는 `cargo build --locked --lib --profile release-test --target-dir target/pr-review`로
실행했고 뒤에는 누적 review branch로 돌아왔다. 두 결과는 `-indent-probe.log`와
`indent-probe-base.log`에 있다. 이는 기존 동작 재현이며 새 PR의 결함으로 단정하지 않는다.
독립 편집 후 한컴 기준이 없으므로 assertion도 임의 수정하지 않았다.

#7493 추가 반례의 두 번째 이름에는 redo가 있으나 Undo에서 먼저 실패했다. 실제 관측은
명령 병합 실패/Undo 잔여이며 Redo까지 실행한 증거는 아니다. production handler/command를
실행했지만 core는 작성자의 mock이라 실제 브라우저/WASM 재현과 구분한다.

### PR별 판정 및 남은 작업

- 수용 후보: [#7497](archives/pr_7497_review.md), [#7498](archives/pr_7498_review.md), [#7508](archives/pr_7508_review.md).
  기능 검토는 통과했으나 보류 건을 제외한 최종 통합 후보와 CI를 아직 만들지 않았다.
- 보류(최초 판정): [#7487](pr_7487_review.md), [#7491](pr_7491_review.md), [#7493](archives/pr_7493_review.md), [#7504](pr_7504_review.md). #7493의 대응 재검증·병합은 아래 후속 절에 기록합니다.
  실행 결함과 독립 시각 증거 부족을 PR별로 구분했다.
- 검토 중 신규 등록된 #7509(head `435f04507ba1cc5b43a12c39d73158ea9f01a872`)는
  입력/복사 논리 오프셋의 부분 수정으로 별도 접수했다. source/test diff를 읽었고 e509와
  merge-tree는 clean이다. 아직 누적에 적용하지 않았고 CI는 최초 조회 때 시작 단계였으므로 승인하지 않았다.

누적 전체 nextest·Native Skia 전체·누적 Clippy 3종·Visual Sweep은 실행하지 않았다.
선행 실패/증거 부족 판정 뒤 동일 전체 검증을 반복하지 않았으며 미실행을 PASS로 쓰지 않는다.
원 head CI는 원 head의 근거로만 남긴다. 선별 통합 head를 새로 게시하려면 해당 head에서
필수 lint·회귀·CI를 충족해야 한다. generated suite/manifest는 stage하지 않는다.

## 승인 후 댓글 게시 및 선별 검증

작업지시자가 보류 사유 comment 게시를 승인했다. #7487·#7491·#7493·#7504에
검토 head·실행 결과·미검증 범위·해제 조건을 게시하고 API로 본문을 재확인했다.
개별 review의 게시 기록에 URL을 남겼다. GitHub approve·push·PR 생성·merge·close는
이번 승인에 포함하지 않았다. 댓글 게시와 별개로 수용 후보 #7497·#7498·#7508만
e509 기반으로 선별한 로컬 통합 branch에서 최종 필수 검증을 진행한다.

### 선별 통합 후보

로컬 branch: `review/semanticist21-accepted-20261002`.
source head: `514d4933b01efa848eba7a37b98a79fe4ffc4736`.
review-only 문서를 옮긴 검증 checkout head: `f27661e63`.
전체 7건 진단 branch는 `review/semanticist21-20261002`와 기록 commit `9cfaf07b5`로 보존했다.
새 worktree나 target directory를 만들지 않았다. PR source branch를 rebase/force-push하지 않았다.

| PR | 원 commit → 선별 branch commit |
| --- | --- |
| #7497 | 756f8fdc → d27652457, b22887f8 → c4c32db75, 64f76e37 → 38da0b73c |
| #7498 | 98133ad9 → 5c700bcd3 |
| #7508 | b53d3621 → 514d4933b |

원 author와 `-x` 출처를 유지했고 선별 적용도 충돌이 없었다. 이 3건의 원 코드 외에
source/test/fixture/baseline 보정을 넣지 않았다. 이전 7건의 결과를 새 후보 결과로 재사용하지 않고
순차 Rust lint, manifest base 비교, release-test 전체 nextest, fresh WASM 실제 API를 확인한다.
독립 PDF 시각 판정으로 보고하지 않는다. 지원 범위는 입력 IR/Query/클립보드 계약이다.

검증 입력의 로컬 파일과 candidate의 `samples/`는 일치하며 sample 변경은 없다.
#7497·#7508 입력은 커밋된 테스트의 공개 API로 만든 합성 문서이고 독립 한컴 출력이 아니다.
#7498의 정상 저장본 SHA-256은 아래와 같다.

| 입력 | SHA-256 |
| --- | --- |
| samples/para-001.hwp | bab4561ceb02cdfa184a1689be9619c08e18d6021cdbc423486b848bc14d267e |
| samples/hwpx/para-001.hwpx | 3feded11906a573dda366b4299428bce2de573a153d11059fa8e1e26b6fc354d |
| samples/table-complex.hwp | 9b5334f0fc164a3168e41042ef5f535ab7180e563b8fdfb59b0c157c895bbc22 |
| samples/issue1937_rowbreak_footnote_overpagination.hwp | a3a075594994e4741a7fbe973bc230d95ae75fdee6578ac6017cec0d52844990 |
| samples/한글문서파일형식_5.0_revision1.3.hwp | f21edf2138e134702366f2fb6a2ab082b05c6dcb42216ec4fa2575ed292efd1d |

### 선별 후보의 최종 Rust 결과

검증 checkout은 `f27661e63bb4654f6cf89f200c4a96cec6ea731d`이며 source는 위 `514d4933b`와 같다.
문서 이외의 미커밋 변경은 없었다. 아래 명령을 공유 target에서 순차 실행해 모두 통과했다.

```bash
node scripts/rust-test-suite-manifest.mjs --prepare
cargo fmt --all
cargo fmt --all -- --check
cargo clippy --locked --target-dir target/pr-review -- -D warnings
cargo clippy --locked -p rhwp --lib --target wasm32-unknown-unknown --target-dir target/pr-review -- -D warnings
cargo build --locked --workspace --target-dir target/pr-review
cargo clippy --locked --workspace --all-targets --target-dir target/pr-review -- -D warnings
node scripts/rust-test-suite-manifest.mjs --check --base-ref e5098bc91be44a49367a7f2895a14fcd4f4c2c7f
cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review \
  --tests --test-threads 16 --no-fail-fast
```

전체 Rust 회귀: **10,243 PASS / 0 FAIL / 50 skip**, 78 binaries, 실행 527.996초,
테스트 빌드 4분 46초. 느린 검사 11개도 끝까지 통과했다. 기존 제외 50개를 새 ignore로 늘리지 않았다.
대상 host는 논리 CPU 16개, RAM 31GiB이며 Cargo 실행을 서로 겹치지 않았다.
`accepted-nextest.log` 및 `accepted-clippy-{native,wasm,workspace}.log`,
`accepted-workspace-build.log`, `accepted-fmt.log`, `accepted-manifest.log`에 결과를 보존했다.
nextest 0.9.137의 권장 버전(0.9.140) 경고와 JUnit `report-skipped` 미지원 경고는 별도로 남긴다.
실행된 검사 수와 종료 Summary를 확인했으며 이를 검사 실패나 누락으로 분류하지 않았다.

3건은 직접 renderer/paint/페이지네이션을 수정하지 않으므로 별도 Native Skia feature 전체 묶음과
독립 PDF Visual Sweep을 실행하지 않았다. 입력·Query·클립보드 동작 및 기존 전체 회귀의
무회귀와 한컴 시각 일치는 서로 다른 주장이다. source-side `#[cfg(test)]` 변경도 없어
unit-tier 증가 검사는 비해당이다. 파생 suite/manifest는 ignored이며 커밋에 포함하지 않는다.

원 PR 3건의 head는 최종 Rust 검사 후에도 접수 SHA 그대로 OPEN/MERGEABLE/CLEAN이었고,
upstream/devel도 e509 그대로였다. 이 상태는 merge 시점에 다시 확인한다.

### fresh WASM 및 최종 로컬 판정

선별 checkout `f27661e63bb4654f6cf89f200c4a96cec6ea731d`에서 아래 명령을 실행했다.

```bash
CARGO_TARGET_DIR=target/pr-review scripts/wasm-pack-locked.sh --target web --dev \
  --out-dir output/pr-review/semanticist21-20261002/pkg-accepted
node output/semanticist21-20261002-browser.mjs --mode=headless --accepted
```

WASM build PASS(42.29초), 실제 HeadlessChrome 152 API smoke **3/3 PASS**.
WASM SHA-256: `d3455486558c9dd9205498e6526e3bfc5a04409dddbbf0f963603a861b077547`.
원래 7건 진단 package와 JSON은 덮어쓰지 않았다. 별도 `pkg-accepted`,
`accepted-wasm-build.log`, `accepted-browser.json`에 source SHA·hash·결과를 보존했다.
Studio public 갱신이나 사용자 CDP 조작, 독립 한컴 시각 일치 판정은 하지 않았다.

최종 PR review 판정은 #7497·#7498·#7508 **승인**, #7487·#7491·#7493·#7504 **머지 보류**다.
승인은 기능 검토의 결론이며 원격 GitHub approve/merge 실행과 별개다.
이번 turn의 외부 변경은 승인받은 보류 댓글 4건뿐이다. 선별 후보의 push·통합 PR 생성·CI·merge는
아직 수행하지 않았다. 다음 승인 범위는 선별 후보 게시 및 통합 PR 생성이며 merge/원 PR close는
CI와 작업지시자의 후속 승인 뒤 별도로 진행한다.

## 선별 통합 PR 게시 승인

작업지시자의 “PR 처리를 진행하세요” 지시에 따라 수용한 #7497·#7498·#7508의
선별 branch push와 devel 대상 Open 통합 PR 생성을 진행한다. 통합 PR의 최신 head CI를
확인한 뒤 병합과 원 PR close는 별도 승인을 요청한다. 보류 4건과 신규 #7509·#7510은 포함하지 않는다.

게시 전 fetch 결과 upstream/devel은 검증 기준 e509 그대로이며, 수용한 3건의 원 head도
접수 SHA와 같다. source/test를 추가 수정하지 않았으므로 완료한 전체 회귀·lint·fresh WASM
결과를 재사용한다. #7487은 기여자가 devel e509를 병합해 head를 `f2f96733b`로 갱신했고,
remerge-diff의 수동 충돌 해소는 오늘할일 문서뿐이었다. 문서 충돌은 해소됐지만 독립 시각
증거 부족에 따른 보류는 유지한다. 이 변경은 선별 통합 source와 무관하다.

라우팅은 maintainer_general에 intake_and_review, local_validation,
multi_pr_update_branch, review_only_fast_pass를 적용한다. 기존 검토를 새로 반복하지 않으며,
게시 직전 고정 base/head의 merge-tree·공백·변경 문서 링크·오늘할일 기록 보존을 확인한다.

### 원격 게시 결과

선별 branch를 upstream에 push하고 [통합 PR #7511](https://github.com/edwardkim/rhwp/pull/7511)을
devel 대상으로 생성했다. 게시 head는 `39f0a27922ac5a1a14d6dd9dc69421d181f9bb31`이며,
정확한 upstream/devel e509와의 merge-tree는 `33b7158fbd6059d962257516b4253256e14246ea`이다.
merge-tree 종료 0, 실제 HEAD tree와 동일, 공백 검사 및 변경 문서 9개의 내부 링크 검사를 통과했다.
오늘할일의 기존 #7382/#7505 기록을 보존하고 별도 semanticist21 절만 추가했다.
push 직전 원격 base가 e509이며 해당 원격 branch가 없는 것을 확인했고 새 branch를 일반 push했다.
upstream tracking은 devel이 아니라 동일 이름의 원격 선별 branch로 설정됐다.

PR 본문은 UTF-8 파일로 게시했으며 API 재조회에서 원문 동일·BOM/`??` 없음과 정확한
head/base를 확인했다. [CI 실행](https://github.com/edwardkim/rhwp/actions/runs/36955832835)은
게시 뒤 시작됐으며 이 기록 시점에는 대기 중이다. CI 완료를 통과로 선기록하지 않는다.
이 게시 기록은 code candidate CI 성공 뒤 같은 PR의 trailing 문서 commit으로 반영한다.
원 PR approve/close, 통합 PR merge, 이슈 close는 아직 수행하지 않았다.

### 통합 code candidate CI 완료 및 문서 후속 처리

작업지시자의 CI 완료 확인 및 다음 절차 진행 지시에 따라 exact 게시 head
`39f0a27922ac5a1a14d6dd9dc69421d181f9bb31`의 결과를 API로 확인했다.
아래 workflow는 모두 completed/success이며 CI의 Build & Test, lint·Native Skia·네 Rust archive
검사도 성공했다. Frontend 등 비해당 skip과 GHAS CodeQL neutral은 실패로 세지 않는다.

| 검사 | 성공한 run |
| --- | --- |
| Full CI | [36955832835](https://github.com/edwardkim/rhwp/actions/runs/36955832835) |
| CodeQL | [36955832850](https://github.com/edwardkim/rhwp/actions/runs/36955832850) |
| Render Diff | [36955832554](https://github.com/edwardkim/rhwp/actions/runs/36955832554) |
| Adapter inter-diff | [36955832871](https://github.com/edwardkim/rhwp/actions/runs/36955832871) |
| Proptest roundtrip | [36955832919](https://github.com/edwardkim/rhwp/actions/runs/36955832919) |

재fetch한 upstream/devel도 e509로 동일하며 PR은 OPEN/MERGEABLE/CLEAN이었다.
앞서 로컬에만 보존한 게시 기록 `7ed98221f`와 이번 CI 기록은 모두 mydocs 한정 single-parent
후속 commit이다. source/test/fixture/workflow는 게시 head와 동일하다. 기록 반영을 위해
devel merge/rebase 또는 기존 로컬 전체 회귀를 반복하지 않는다.
기존 3개 원 PR review와 오늘할일을 같은 통합 PR에서 갱신하고 문서 후속 head를 push한다.
이 push의 최신 게이트를 확인한 뒤 병합 승인을 요청한다. 원 PR close·이슈 close·merge는
이번 후속 기록 승인으로 수행하지 않는다.

## 병합 및 승인된 후속 처리

작업지시자의 별도 승인으로 [통합 PR #7511](https://github.com/edwardkim/rhwp/pull/7511)을
2026-10-02 12:36:38 KST에 병합했다. 최종 head는
`f2a9341f7ed8f1fc2912b17dd54d2635fa760dc0`, merge SHA는
`c77ed685e21de0b2edca918efcec7c2ebbbf2bd5`다. merge 직전 MERGEABLE/CLEAN,
원 PR 3건의 head 불변 및 모든 최신 필수 게이트 완료를 확인했다.

| 최종 문서 head 검사 | 성공한 run 및 실행 방식 |
| --- | --- |
| CI | [36959554063](https://github.com/edwardkim/rhwp/actions/runs/36959554063), 검증한 code candidate의 fast-pass |
| CodeQL | [36959554001](https://github.com/edwardkim/rhwp/actions/runs/36959554001), 후보 검색 실패로 Full analysis 실행 후 성공 |
| Render Diff | [36959553768](https://github.com/edwardkim/rhwp/actions/runs/36959553768), 기존 성공 결과 재사용 |
| Adapter inter-diff | [36959554087](https://github.com/edwardkim/rhwp/actions/runs/36959554087), 기존 성공 결과 재사용 |
| Proptest roundtrip | [36959554030](https://github.com/edwardkim/rhwp/actions/runs/36959554030), 기존 성공 결과 재사용 |

로컬 devel을 merge SHA로 fast-forward했고 최종 head의 포함을 확인했다. devel push에는
CI 전체를 재실행하지 않고 이슈 종료·duration 갱신 두 workflow만 실행됐다.
[이슈 종료 run](https://github.com/edwardkim/rhwp/actions/runs/36960931326)은 성공했고
#7496·#7495·#7506 모두 CLOSED/completed를 확인했다.
[duration 갱신 run](https://github.com/edwardkim/rhwp/actions/runs/36960931332)도 성공했다.
code candidate `39f0a2792`의 Full CI `36955832835`에서 측정값을 가져와
`ci-metrics/nextest-target-durations`에 `b5bd440a`로 게시했다. 최종 문서 head가
fast-pass인 이유로 CI를 다시 실행하지 않았다.

후속 기록은 maintainer 직접 반영 경로로 개별 수용 review 3건을 archive로 이동하고,
이 일괄 기록과 오늘할일만 한 운영 문서 commit으로 devel에 반영한다.
원 PR #7497·#7498·#7508에는 merge SHA·확정 review 링크·검증 범위와 제한을 안내하고
통합 반영에 따른 superseded close를 수행한다. fork branch는 변경하지 않는다.

정리 대상은 병합한 동일 저장소의 `review/semanticist21-accepted-20261002` local/remote branch다.
병합 포함·원격 head 불변·동일 head의 다른 Open PR 부재를 확인한 뒤 제거한다.
보류 #7487·#7491·#7493·#7504는 계속 Open으로 유지하므로 일괄 구현계획, 진단 branch
`review/semanticist21-20261002`와 해당 output 증거는 보존한다. 기본 작업공간·공유
`target/pr-review`, 다른 작업의 branch/worktree/stash는 정리 대상이 아니다.
신규 #7509·#7510은 이번 수용 범위 밖이다. 최종 댓글·종료·정리 확정값은 GitHub에 남긴다.

## #7493 대응 재검증 및 단독 병합

기여자 대응 head `849955949f2f0d4a3f98e0006e727d6f3443333f`에서 최초 보류 사유를
해소한 것을 확인했습니다. [개별 review](archives/pr_7493_review.md)에 수정 전 음성 대조 6건 FAIL /
최신 코드 PASS, runner 31개 PASS, TypeScript PASS, Studio 전체 1,813 PASS / 0 FAIL / 2 skip,
실제 Chrome/WASM 편집 19/19 PASS와 기존 WASM 사용의 한계를 연결했습니다.
같은 head를 다시 검증하거나 누적 7건 branch를 통합하지 않았습니다.

작업지시자의 PR 처리 지시에 따라 [#7493](https://github.com/edwardkim/rhwp/pull/7493)을
2026-10-02 13:31:19 KST에 단독 병합했습니다. merge SHA는
`f536f15e750d9a32d33b013da7be079ecb10d1f9`이며 로컬 devel 포함·#7489 자동 종료를 확인했습니다.
공유 target과 다른 보류 PR의 누적 진단 branch는 보존합니다. 이번 response 전용 review branch만 정리합니다.
운영 문서 commit의 원격 push는 별도 승인 대기로 둡니다.

병합 후 이슈 종료 workflow는 성공했습니다. duration workflow도 성공했으나
`no-verified-pr-duration-measurements`로 자료 갱신을 보류했습니다. Studio-only CI에서
Rust worker가 skip됐으므로 측정 자료를 만들기 위한 전체 CI 재실행은 하지 않았습니다.
후속 문서 push 승인 뒤 #7489·원 PR 안내 및 response 전용 branch cleanup을 이어갑니다.

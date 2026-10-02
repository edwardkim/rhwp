# PR #7405 리뷰 — 공통 호스트 글꼴 공급과 CanvasKit·Canvas2D

## 최종 판정

**승인.** 화면용 호스트 글꼴 공급 계약과 명시한 검증 범위를 충족한다.
2026-09-25에 발견한 오늘할일 충돌은 문서 절 위치 조정만으로 해소했고,
후보 `9bf4451acd91d415505c7be1ffbefe132e498bdd`의 GitHub checks는
14 SUCCESS / 19 SKIPPED, 실패·대기 없음, `MERGEABLE / CLEAN`을 확인했다.
제품 소스·테스트·시각 자산은 기존 검증 head와 동일하다.
이는 작성자 self-review이며 GitHub approve나 실제 merge 승인이 아니다.
이 리뷰 기록을 포함한 최종 trailing head의 required CI·mergeability 재확인과 작업지시자의 merge 승인은 별도 조건이다.

## 2026-09-25 재검토

- 검토 head: `bf371e200cfbc343094b552169864ba6c9235a27`.
- 최신 fetch base: `97073606de2545f88540429e2f7f8312eefa9641`.
- 기본 경로: `collaborator_self_merge.md`. 보조 경로: `intake_and_review.md`,
  `local_validation.md`, `visual_fixture_evidence.md`, `rework_and_exceptions.md`.
  모 workflow·선택표 및 위 문서를 읽었다. 아래 접수 정보는 최초 제출 당시 기록이다.

### 최초 병합 차단 사항 — 아래 후속에서 해소

`git merge-tree --write-tree upstream/devel bf371e200cfbc343094b552169864ba6c9235a27`은
exit 1을 반환했다. 충돌 파일은 `mydocs/orders/20260924.md` 하나다.
최신 base의 #7368 검토 절과 이 PR의 M100/#7403 절이 같은 EOF에 추가되어 충돌한다.
두 절을 모두 보존해야 한다. 제품 소스의 텍스트 충돌은 없었다.
GitHub도 같은 head에 `CONFLICTING / DIRTY`를 반환했다.
기존 head의 CI 32 SUCCESS / 4 SKIPPED는 확인했지만 충돌 해소 후 head의 검증을 대신하지 않는다.

### 코드·검증 대조

`HostFontSource`의 세대/revision/face 소유권, `local-fonts`의 스타일 선택과 브라우저 목록 분리,
두 renderer의 자원 준비·해제, `WasmBridge`의 측정/paint scope와 portable SVG 분리,
`RendererSession` 및 VS Code nullable report 소비자를 대조했다.
이번 검토에서 새로 재현한 제품 결함은 없다. 조판 원칙의 적용 범위와 독립 증거는 아래 표를 유지한다.
분할·LineSeg·baseline 변경은 없으며 미실행 경로를 충족으로 확대하지 않는다.

집중 재실행 명령(검토 head의 `rhwp-studio`):

```sh
node --test tests/host-font-provider.test.ts tests/host-canvas-fonts.test.ts tests/renderer-session.test.ts
git diff --check 505661360..bf371e200
```

첫 명령은 **27 PASS / 0 FAIL / 0 SKIP**, 두 번째는 exit 0이었다.
전체 Studio 1,787개·fresh WASM·E2E는 아래 기존 source/head 증거를 재사용했으며 이번에 재실행한 것으로 세지 않는다.
기존 Native/fresh WASM/CanvasKit/Canvas2D 대표 review PNG와 WASM/두 화면 renderer overlay를
직접 열었다. 1쪽의 세 줄에서 누락·겹침·줄바꿈 변화는 보이지 않았고 가장자리·농도·screen guide
차이는 남는다. 이는 기존 산출물의 직접 재판독이며 새 base 통합본의 재캡처가 아니다.
입력 3개의 검토 head Git blob SHA-256은 아래 입력 표와 일치했다.

재판독한 review PNG의 SHA-256:

| 파일 (`mydocs/report/assets/issue7403/`) | SHA-256 |
| --- | --- |
| `native-review-001.png` | `6a4feb9a0b9f813a0d704cd72b0fe43052d04fcfc00fae76f6f034d4ae1cc5e1` |
| `wasm-review-001.png` | `3978b0e7e1ef8471d5ca50abb23313d152ca24029434fe4e119e67e9ca76bd76` |
| `host-canvaskit-review-001.png` | `15f40dd650de04f6c0568d002673ae01255e38f680c5ebde54de0de8a91bce33` |
| `host-canvas2d-review-001.png` | `dee707413b2fbc832e9686d86ca321dc387d09a837d8e234d4ce76ad65f3e854` |

### 다운스트림 증거와 잔여 범위

알한글 작업이 전달한 보고서 `mydocs/working/task_m020_567_pr7405_validation.md`
(알한글 소스 `ef7d638794f82b71a71c75ef2029bf4eb3d463db`)를 읽고 범위를 대조했다.
동일 upstream head에서 실제 IPC·WebKit Canvas2D·Apple GPU WebGL2를 포함한 격리 probe
29개 통과가 보고됐다. 이번 리뷰가 이를 재실행한 것은 아니다. WebGPU 요청은 실제 WebGL2로
fallback하므로 WebGPU 성공으로 세지 않는다. 전체 앱·실제 설치 권한·다쪽 출력·실제 프린터는 미검증이다.

현재 알한글 PDF의 다른 글꼴 사용과 별도 출력 WebView FontFace 공급 후 Regular/Bold 포함 결과는
문서화된 호스트 책임과 일치한다. 출력 snapshot·준비 대기·IPC queue·metadata 정규화·글꼴 메뉴는
다운스트림 후속이며 현재 증거로 upstream API 확대를 병합 필수 조건으로 요구하지 않는다.

### 보류 해제 조건

1. 양쪽 오늘할일 기록을 보존한 충돌 해소와 최신 base/head 병합 시뮬레이션 성공.
2. 통합으로 변경되는 코드 범위에 맞는 검증 및 시각 증적·PR 본문 SHA 갱신.
3. 새 head의 required checks와 `MERGEABLE / CLEAN` 확인, 작업지시자의 실제 merge 승인.

### 충돌 해소와 CI 확인

사용자가 충돌 해소·CI 확인·리뷰 문서 push·최종 판정을 지시한 뒤 진행했다.
`9bf4451acd91d415505c7be1ffbefe132e498bdd`는 이 PR의 M100 절을 문서 앞쪽으로 옮긴
single-parent 문서 전용 commit이다. 최신 devel 전체를 source branch에 병합하거나 다른 PR의 기록을
복사하지 않았다. 제품 코드·테스트·fixture·workflow·시각 자산 diff는 0이다.

push 전 base `a7458aa39ca4ac8636a52e4c7a607d55973bf3a6`에 대해
`git merge-tree --write-tree <base> <head>`가 exit 0을 반환했고,
merge tree는 `0089baaf9a81c1d8fcc3c30d323b0dc323e8c3f6`였다.
merge tree의 `git diff --check`, 최신 base 기록의 전수 보존, #7368/M100 절의 각 1회 존재,
오늘할일·review의 내부 링크 대상 10개 확인을 통과했다. 원격 base/head 유지도 push 직전 재확인했다.
검증 중 base가 전진했을 때는 새 base를 fetch하고 같은 검사를 다시 수행했다.

후보 head의 [CI](https://github.com/edwardkim/rhwp/actions/runs/36107076872)는 성공했다.
preflight의 `DETECTED_FAST_PASS=true`, `CANDIDATE_SHA=bf371e200cfbc343094b552169864ba6c9235a27`,
`DETECTED_REASON=build-and-test-green:success`와 최신 `Build & Test` success를 직접 확인했다.
[Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/36107076953)은 실제 회귀를 실행해 성공했다.
전체 checks는 14 SUCCESS / 19 SKIPPED, 실패·대기 없음이었다. 생략한 heavy job을 재실행한 것으로 세지 않는다.
최종 GitHub 상태는 `MERGEABLE / CLEAN`이었다.

이 기록과 오늘할일 갱신을 같은 PR의 후속 문서 commit으로 push한다. 후속 head도 동일한 병합·링크·기록
보존 검사를 거친 뒤 push하며 최신 head의 CI 집계를 다시 확인한다. PR 본문의 raw 이미지 URL은 최종 head로
갱신한다. code head가 바뀌지 않았으므로 기존 이미지와 source 검증을 재사용한다.
이번 승인 범위에는 실제 merge·issue close가 없으며 수행하지 않았다.

## 접수 정보

| 항목 | 값 |
| --- | --- |
| PR·작성자·base | [#7405](https://github.com/edwardkim/rhwp/pull/7405) / postmelee / devel |
| 관련 이슈 | [#7403](https://github.com/edwardkim/rhwp/issues/7403), 자동 종료 없이 참조 |
| 검증 source | `eb91a11f1052b88ae6efdfeb7e42b6233dbfde84` |
| 제출 candidate | `d8f65793be33890deeaef8f2e0ccbe019a02a147`, 최초 제출 상태 |
| 고정 base | `505661360e9a2d596f55300d0cb0c5222f0e14b4`, 제출 직전 최신 devel과 동일 |
| 최초 규모 | 55 files / +2,796 / -129, 코드·테스트·보고서·PNG 포함 |
| 작성 시점 참고값 | Open, MERGEABLE, CI 대기; reviewer는 self PR 규칙에 따라 지정하지 않음 |

- base route: `collaborator_self_merge.md`
- modifiers: `intake_and_review.md`, `local_validation.md`, `visual_fixture_evidence.md`, `review_only_fast_pass.md`, `rework_and_exceptions.md`(1,000줄 초과)
- loaded documents: `pr_review_workflow.md`, `pr_review/README.md`, 위 기본·보조 문서와 `review_template.md`, `github_operations.md`
- 원격 권한: push=true, admin=false. protected branch 설정 API는 404로 조회 불가였으므로 PR checks의 실제 결과를 사용한다.
- 대형 PR은 별도 검토·simulation·시각 검증을 거쳤다. 이번 작업에 merge/admin 우회는 포함하지 않는다.

## 변경과 검토 범위

목록·바이트 읽기·변경 구독을 `HostFontSource`에서 분리하고 `local-fonts`의 face 선택을 두 renderer가 공유한다.
연결 세대/revision/ID로 이전 데이터 재사용을 방지하고, 목록 조회 시 파일 바이트는 읽지 않는다.
CanvasKit Typeface와 Canvas2D FontFace는 각각의 문서 자원 경계에서 준비·정리한다.

Canvas2D의 보조 측정·paint는 동일 session의 동기 별칭 scope를 사용한다. 공급자 변경은 측정 캐시와
페이지 자원을 무효화하고 실제 갱신을 유발한다. `withPortableMetrics`의 scope 해제로 HWP/HWPX·SVG에
내부 이름이 유출되지 않도록 했고 실제 저장·재열기/교체 전후 SVG/undo·redo를 대조했다.
메타데이터와 로컬 폰트 CSS 사용 가능 판정·영속 저장은 분리돼 있다.

기존 E2E manifest 3건 누락의 등록 보완이 별도 commit에 포함돼 있다. 검증 도구·기준값을 완화하지 않았다.
네이티브 IPC·OS 권한·출력 환경 FontFace 공급·PDF/인쇄·출력 snapshot은 호스트 책임이며 Rust 조판 변경은 없다.

## 검증 입력과 결과

[최종 보고서](../../report/task_m100_7403_report.md)의 명령·계약별 관측값·독립 기대값·RED/GREEN을 재사용한다.
fresh web/Node dev WASM, TypeScript, Studio production build, **1,787 PASS / 0 FAIL / 0 SKIP**,
양쪽 renderer E2E, 기존 CanvasKit 글꼴 coverage, manifest 140/140, 합성 글꼴 재생성 hash 검사가 통과했다.
VS Code package compile도 후속 보정에서 확인했다. Rust/Cargo·Rust test/baseline helper를 바꾸지 않았고 새 글꼴은 TypeScript 브라우저 fixture이므로 Rust 전체 lint/integration은 비해당이다.
Native CLI는 Visual Sweep용으로 빌드했다. optimized release WASM을 검증했다고 주장하지 않는다.

실제 제품 API와 CanvasView에서 동명 글꼴 교체·read 실패 복구·해제·문서 전환을 검사했다.
abort를 무시하는 응답 및 reset 뒤의 FontFace.load 완료가 재등록·재paint하지 않는 반례도 통과했다.
Oblique-only family 회귀는 수정 전 실제 선택 실패와 수정 후 통과를 보존했다.
기존 머리말/꼬리말 소스 연결 검사의 `method!()` 인식 실패는 검사식 보정 뒤 통과했으며 제품 결함으로 집계하지 않는다.

### 검증 입력 커밋 확인 — 충족

실제 사용한 바이트와 제출 candidate의 Git blob을 직접 대조했다. 모두 기존 저장소 파일을 재사용했다.

| 입력 | 역할 | SHA-256 |
| --- | --- | --- |
| `samples/re-01-hangul-only-hancom.hwp` | 실제 한컴 원문 1쪽 | `61538931d2e2cf38f35050618ce7698960823938884d0d8977812c94587e85fd` |
| `pdf/re-01-hangul-only-hancom-2022.pdf` | 동일 원문의 한컴 기준 PDF | `ceb38d48887392ade56aef29be6bb65d77dd89e77a49c4e9b04a9b5323159684` |
| `samples/basic/issue2007_nested_cell_pagination_42065.hwp` | 실제 17쪽 문서의 10쪽 charOverlap | `bebd4ce3691246b0fb3ae332e1d40bc51d9035cddb9fc3d378466b6a8a2b5626` |

합성 TTC/TTF는 독창적인 outline·advance의 계약 입력이다. 한컴 기준 출력으로 취급하지 않는다.
HCRBatang 공급 파일은 PDF subset과 outline 동일성을 확인했으나 라이선스 바이너리는 PR에 추가하지 않았다.

### 조판 원칙 준수 검토

| 항목 | 판정 | 근거 |
| --- | --- | --- |
| 구현 근거·일반성 | 충족 | face ID·스타일·revision 계약으로 분기. 특정 문서/좌표 clamp 없음 |
| 측정·배치 일관성 | 충족 | Canvas2D 보조 측정과 paint에 같은 FontFace. 독립 TTF 픽셀 및 43.2/51.84/57.6px advance 대조 |
| 분할·이어받기, 줄 소속·점유 높이 | 비해당 | Rust 줄 구성·LineSeg·pagination·좌표 알고리즘 변경 없음 |
| 사례·증거 독립성 | 충족 | 합성 TTF 계약과 실제 한컴 원문/PDF를 분리. 실제 charOverlap 호출 검사 |
| baseline·golden·허용치 변경 | 비해당 | 변경 없음 |
| 주장과 범위 | 충족 | [기계 판독 증거](../../report/assets/issue7403/validation.json), 아래 미검증 범위 구분 |

## 시각 증적과 남은 차이

[Visual Sweep 정본](../../manual/verification/visual_sweep_guide.md#pr-body-visual-evidence)을 적용했다.
동일 원문·PDF 1쪽의 한글 3줄을 Native/fresh WASM/CanvasKit/Canvas2D로 산출하고 review·overlay를 직접 확인했다.
누락·겹침·줄바꿈 변화는 없고 가장자리·농도 차이와 screen guide가 남는다.
Native/WASM layout·text clip·cell boundary 후보는 0, 텍스트 차이는 0/0이었다.
표·그림이 없는 원문이므로 OVR은 비해당이다.

| 경로 | pixel match | 엄격 내용 픽셀 | 2px 실루엣 / gate |
| --- | ---: | ---: | --- |
| Native / fresh WASM | 99.42164% | 28.63272% | 100% / passed |
| CanvasKit | 99.38160% | 16.46720% | 100% / passed |
| Canvas2D | 99.37218% | 21.60762% | 100% / passed |

대표 PNG는 `mydocs/report/assets/issue7403/{native,wasm,host-canvaskit,host-canvas2d}-{review,overlay}-001.png`에 있다.
PR 본문은 head SHA 고정 raw 이미지로 네 경로를 직접 표시한다. 실루엣 점수를 전체 한컴 호환성으로 해석하지 않는다.
로컬 실행·중간 산출물은 `/private/tmp/rhwp-7403-validation/common-*`다.

다른 브라우저 엔진·WebGL/WebGPU·대규모 다쪽 성능, patch·머리말/꼬리말의 개별 호스트 글꼴
브라우저 수용 사례는 미검증이다. 같은 scope 연결의 코드 확인을 개별 실행 증거로 승격하지 않는다.

## CI와 병합 전 확인

최초 candidate의 [CI run](https://github.com/edwardkim/rhwp/actions/runs/35974531896)에서 VS Code nullable report 소비자 누락이 검출됐다.
`92b427c843edae758ec90f34f65d5257e1913059`에 기존 explicit 경로를 보존하는 guard를 추가했다. 수정 전 VS Code typecheck FAIL(TS18047),
수정 후 package compile PASS 및 RendererSession 13 PASS를 확인했다. 자세한 증거는 보고서의 CI 보정 절에 있다.
최초 source에는 consumer compile 회귀가 있었으며, 현재 승인은 이 보정을 포함한 범위다. Studio·Rust·fixture는 기존 검증 후 변경되지 않았다.
실패 candidate는 재사용하지 않고 보정·문서 commit을 함께 push해 새 CI를 실행한다.
최신 trailing head의 required checks와 mergeability를 별도로 확인한다. 완료한 로컬 검증을 반복하지 않는다.
문서 trailing commit은 최신 base와 merge-tree·공백·변경 문서 링크·오늘할일 기존 기록 보존 검사를 통과한 뒤 push한다.

## Merge 후 contributor PR comment 계획

본인 PR이다. merge가 승인되면 최종 merge SHA·CI URL과 [Visual Sweep 정본](../../manual/verification/visual_sweep_guide.md#github-merge-comment)을
연결하고 같은 네 경로의 PNG를 `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-sha>/mydocs/report/assets/issue7403/<파일>`로 표시한다.
위 1쪽·후보 0·지표·직접 판독 및 미검증 범위를 함께 쓰고 `--body-file` 게시 후 API로 본문을 대조한다.
이 계획은 현재 merge 또는 후속 comment 게시를 수행했다는 뜻이 아니다.

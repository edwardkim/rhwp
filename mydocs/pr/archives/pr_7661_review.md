---
kind: report
status: archived
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-10-15
---

# PR #7661 리뷰 — 셀·쪽 속성의 그림·그라데이션 배경 보존

## 최종 판정

**승인 — 메인터너 보정 통합 head `08a49f869c27c099c1dd9108b7a923d542c96b62`.** 로컬 검증, 사용자 시각 판정, 최신 head CI·CodeQL·Render Diff·CI Impact Policy가 모두 통과했다. 원 contributor head `f1da6189670357b4a2394c73d7106137ab4b38c4`와 보정 head를 구분한다. 작업지시자는 병합과 후속 처리를 승인했다. 2026-10-15에 merge commit `7b36eb7f0d2b36ae558ca50aea667c893ce05ff0`으로 병합했다. 아래 초기 보류와 단계별 결과는 당시 기록이며, 최종 결과 및 병합 절을 따른다.

## 접수 정보와 기존 맥락

- [PR #7661](https://github.com/edwardkim/rhwp/pull/7661), 작성자 `semanticist21`, base `devel`, non-draft, 원 head `f1da6189670357b4a2394c73d7106137ab4b38c4`.
- 원 head CI는 성공, 조회 시점 `MERGEABLE/CLEAN`. 2 commits / 3 files / +177 −2. assignee=author, milestone=v1.0.0, labels=bug/rust/api/roundtrip/table로 트리아지 완료 상태다. reviewer는 `edwardkim`을 지정하고 API로 재확인했다. 별도의 종료 대상 이슈 참조는 없다.
- 기본 경로 [maintainer 일반](../../manual/pr_review/maintainer_general.md), 보조 intake_and_review·local_validation·visual_fixture_evidence·post_merge와 review_template를 적용했다. 기존 기여자이므로 첫 기여자 경로는 적용하지 않았다.
- 이전 [누적 검토 진행78](semanticist21_20261008_integrated_review.md)은 읽기 전용 보류였다. 이번 사용자의 개별 처리 지시로 검증을 시작했으며 초기18개 통합에 새로 포함된 것으로 설명하지 않는다.
- 의존 [#7655](https://github.com/edwardkim/rhwp/pull/7655)는 개별 merge가 아닌 [#7882](https://github.com/edwardkim/rhwp/pull/7882) 통합으로 devel에 반영됐다. 현재 `border_fills_equal`은 `a.fill == b.fill`로 전체 Fill을 비교한다. 이 의존성을 새 미해결 조건으로 요구하지 않는다.
- 고정 최신 base `a5e88ad682c805e1d845502ddc9521b94ea8eccd`, clean merge tree `ef7751dbee09a0e2a5a8bfa23a492a441a1a8b72`, 로컬 통합 commit `8410f8179493a83866e5b45aa00f361700546662`. 원 PR과 base를 함께 담은 검토 후보이며 원격 head의 새 CI로 세지 않는다.

## 초기 검토: API 수정 효과와 실제 소비 경로

`helpers.rs::fill_json_values`는 기존의 단색·없음 판정을 유지하고 Gradient/Image를 같은 이름의 JSON 종류로 읽는다. 셀·표·쪽·글자·문단 조회가 이 결과를 사용한다. `html_table_import.rs::create_border_fill_from_json`은 `borderFillId`의 정의를 먼저 복제하고 gradient/image readback이면 기존 채우기를 보존한다. `none`은 계속 명시적 삭제이며 `solid`는 색/무늬 편집이다. 정의 재사용은 위의 Fill 전체 비교를 거친다.

이 수정은 문서 ID·고정 좌표에 맞춘 예외가 아니라 입력 종류·소유 정의를 보존하는 API 계약이다. 다만 getter의 새 종류를 실제 소비자가 어떻게 다시 쓰는지까지 확인해야 한다. API에서의 보존을 UI 해결로 확대하지 않는다.

### 실행으로 확인한 결함

현재 Studio의 `CharShapeDialog`와 `ParaShapeDialog`는 `populateFromProps()`에서 gradient/image를 `none` 선택값으로 표시한다. 이어 `collectMods()`가 원래 종류와 비교해 `fillType: "none"`을 생성하고, 아무것도 바꾸지 않은 `handleOk()`가 `onApply`를 호출한다.

Node 24의 TypeScript strip/transform으로 **제품 class의 실제 populateFromProps → collectMods → handleOk**를 실행했다. 가져온 번역 함수, 미리보기·언어 선택 처리, DOM 입력 컨트롤은 대역이다. 편집 대상 값 비교·배경 선택·변경 수집·적용 분기는 제품 메서드를 그대로 사용한다. 실제 브라우저/WASM 검증으로 보고하지 않는다.

| 대화상자 | API 종류 | UI 표시 | 아무 변경 없이 확인한 결과 |
| --- | --- | --- | --- |
| 글자 | gradient / image | none | 각각 `{fillType:"none"}` 적용 요청, 2건 |
| 문단 | gradient / image | none | 각각 `{fillType:"none"}` 적용 요청, 2건 |
| 글자·문단 대조군 | none / solid | 원래 종류 | 변경 없음, onApply 0회, 4건 |

실제 명령 연결 `rhwp-studio/src/command/commands/format.ts:260–291`은 이 mods를 `applyCharPropsToRange`·`applyParaPropsToRange`에 넘긴다. 조회는 `formatting.rs:693/775`의 공통 helper를 거치고, apply 경로는 같은 파일의 `create_border_fill_from_json`을 호출한다. 그 함수에서 `fillType:"none"`은 `Fill::default()`가 된다. callback 이후에 gradient/image를 복원하거나 이 키를 제거하는 분기는 없다. 최종 WASM 저장·재열기의 직접 재현은 미실행이다.

기존 devel getter는 gradient/image를 none으로 읽었으므로 이 무변경 UI 경로에서는 종류 차이가 없어 삭제 요청을 새로 만들지 않았다. PR의 올바른 readback과 기존 UI의 조합에서 생기는 새 회귀다. 셀·쪽 대화상자가 기존부터 none을 쓰는 별도 잔여 문제와 구분한다. 그라데이션/그림 편집기 전체를 새로 만들 필요는 없고, 미지원 원래 값의 유지와 명시적인 삭제 선택을 구분하면 된다.

## 완료한 검증과 재사용 범위

같은 공용 `/home/edward/mygithub/rhwp/target/pr-review`를 순차로 사용했다. 원 PR의 두 검사를 바꾸지 않고, 먼저 최신 base의 두 제품 파일만 복원한 음성 대조를 실행한 뒤 통합 후보의 원래 파일로 되돌려 다시 실행했다. 임시 제품 파일 변경은 전부 복원했다. 생성 suite/manifest는 PR source에 포함하지 않는다.

```text
cargo nextest run --locked --cargo-profile release-test   --target-dir /home/edward/mygithub/rhwp/target/pr-review   --test regression_suite_007   -E 'test(gradient_image_fill_readback::)' --no-fail-fast
```

| 범위 | 실제 결과 |
| --- | --- |
| 최신 devel + 원 PR 검사만 | 2 FAIL / 0 PASS, exit 100. 셀·쪽 조회 none, 편집 후 및 HWP/HWPX 재열기 Fill 손실로 실패. 환경·빌드 실패가 아님. |
| 최신 devel + 원 PR 구현 | 2 PASS / 0 FAIL, exit 0. 셀·쪽의 모델 Fill 보존 확인. 렌더링 일치 검사로 세지 않음. |
| 실제 UI 메서드 진단 | 새 배경 삭제 요청 4건 재현, 정상 대조 4건. DOM 대역이며 실제 브라우저/WASM 미실행. |
| fmt·manifest·Git | fmt check와 고정 base manifest check 통과. 1,465 sources / 28 suites + 20 exceptions. 자동 merge 충돌 없음·공백 정상, 임시 source 복원 확인. |

[원 head CI](https://github.com/edwardkim/rhwp/actions/runs/37667764031)의 실제 head SHA가 `f1da6189670357b4a2394c73d7106137ab4b38c4`이고 completed/success임을 재조회했다. archive a–d 검사·lint 성공 및 [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/37667763560), Adapter·Proptest 결과를 확인했다. 코드 보정 없는 검토이므로 녹색 CI의 광범위 검사를 중복 로컬 실행하지 않았다. CI의 Native Skia·WASM Build는 skipped이며 실제 fresh WASM 증거로 사용하지 않는다. 최신 base 통합 후보 전체 회귀의 성공을 주장하지 않는다.

## 검증 입력 커밋 확인

아래 5개는 실제 focused 검사 입력이며 통합 commit `8410f8179493a83866e5b45aa00f361700546662`의 blob과 실행 파일 바이트가 같다. 별도 개인 corpus는 사용하지 않았다.

| 저장소 경로 | SHA-256 |
| --- | --- |
| `samples/issue7235/156086935_none_image_fill.hwp` | `b12aa445776bef38a003e8679865a9a0ed2f729ecef18099625c976efe64d6ae` |
| `samples/hwpx/mel-001.hwpx` | `ec75dc24ade52e055c54ea0529af100a623027c465571fda7edaa97f6e736d8b` |
| `samples/hwpx/el-school-001.hwpx` | `e9636dccf3fdfed897f9cdc5d3a358d93093a6d2a509592898413493ae67a354` |
| `samples/basic/NewYear_s_Day.hwp` | `1be2736df6b21687f5c1627bac307cc689171928f6ceb0a235af0bc727eb6372` |
| `samples/issue2816/imgbrush_total_page_fill.hwpx` | `75d28e63f4610c751ea9cc56a41634c3f7a7761b817170a6b84acfa90511237f` |

원본 Print 후보는 `samples/issue7235/156086935_none_image_fill-2020.pdf`, `pdf/hwpx/mel-001-hwpx-2020.pdf`, `pdf/hwpx/el-school-001.pdf`, `pdf/basic/NewYear_s_Day-hwp-2020.pdf`, `pdf/issue2816/imgbrush_total_page_fill-hwpx-2020.pdf`에 이미 있다. 존재를 확인했지만 수정된 편집본과의 대응·출력 유효성은 미검증이다. 기존 기준을 먼저 재사용 검토하며, 원본 PDF를 여백이 변경된 편집본의 Print로 자동 간주하지 않는다.

## 조판 원칙과 증거 판정

| 항목 | 판정·근거 |
| --- | --- |
| 구현 근거·일반성 | API 계약 범위 충족: 실제 원본의 Fill 보존, 종류·정의 기준이며 sample 예외 없음. |
| 측정·배치 공통 결과, 분할·이어받기, 줄 소속 | 비해당: 해당 계산·기하·pagination 코드를 변경하지 않음. 채우기 paint 결과 영향은 아래 별도 미검증. |
| 최종 소비자·backend | 미충족: 기존 글자·문단 UI가 새 종류를 삭제 요청으로 바꾸는 경로 실행. Native PNG/Studio/WASM/SVG 실제 paint는 미검증. |
| 사례·기대값 독립성 | API 2건은 원본 Fill 및 저장 전 대조군을 기준으로 실제 FAIL→PASS. UI 진단은 synthetic 속성·DOM 대역 범위로 제한. |
| 기준값·회귀 | golden/baseline/허용치 변경 없음. 모델 Fill 검사 통과를 렌더링 위치의 정답으로 승격하지 않음. |
| 시각 선행·확대 판독 | 미검증: 동일 편집본 Print 및 최신 base/후보의 전쪽 Native/fresh WASM TSV·대표 PNG·직접 판독 없음. |
| 검증 입력 커밋 | focused 원본 5개 충족. 최종 편집본·시각 기준 대응은 아직 미검증. |
| 주장·검증 범위 | API 보존 효과만 확인. cellzone 잔여7곳은 기여자의 기존 보고이며 이번에 전수 재측정하지 않음. UI 전체 해결/시각 통과를 선언하지 않음. |

## 보완 요청과 해제 조건

1. 기존 글자·문단 대화상자가 미지원 gradient/image를 유지하도록 보완한다. 그대로 확인·다른 속성만 변경·명시적 채우기 제거를 구분하고, none/solid 정상 대조도 유지한다. 단순 문자열 가드나 API 검사만으로 UI 소비 검증을 대신하지 않는다.
2. 원래 API 편집 → HWP/HWPX 저장 → 재열기와 실제 Studio 동작을 fresh Docker WASM에서 확인한다. 기존 셀·쪽 대화상자 및 cellzone은 수용 범위와 잔여 범위를 명시한다.
3. 기존 원본 Print 후보를 먼저 검토하고, 검증하는 편집본에 대응하는 독립 Print와 Native/fresh WASM 전쪽 TSV·대표 review/overlay를 새 head로 제시한다. 90% 최소 및 최신 devel 대비 0.01%p 하락 gate·직접 판독을 모두 확인한다.

ignored `output/pr-review/pr7661-20261015/`에 메타데이터·focused-results·입력 manifest·진단 코드/JSON/로그·원본 Print 후보 목록을 보존한다. 기여자 댓글 초안은 `contributor-comment-ko.md`이며 아직 게시하지 않았다. 원 PR 브랜치나 원격 상태를 임의 보정하지 않고 작업지시자 승인 뒤 게시한다.

## 메인터너 보정 1: 대화상자 보존 계약

작업지시자의 “메인테이너 보정으로 진행” 승인 후 같은 `review/pr7661-20261015` branch에서 보정했다. collaborator 외부 PR 절차에 따라 검토 branch를 기본 작업공간으로 옮겼고, 임시 worktree만 해제했다. 현재 기록 경로는 이 문서이며 기존 임시 worktree 경로를 사용하지 않는다.

- 원 contributor head: `f1da6189670357b4a2394c73d7106137ab4b38c4`.
- devel 통합 commit: `8410f8179493a83866e5b45aa00f361700546662`.
- UI 보정 commit: `d18d59f8f2bdaea5be5f7463f822601f6d447d20`.
- 공용 target: `/home/edward/mygithub/rhwp/target/pr-review` 유지.

`fill-selection.ts`가 원래 gradient/image 종류를 “기존 … 유지” 선택지로 유지한다. none은 명시적 삭제이고 solid만 색·무늬 입력을 활성화한다. 글자·문단 대화상자는 미지원 종류를 유지할 때 단색 입력의 잔류값을 전송하지 않는다. 테두리/채우기를 바꾸면 원래 `borderFillId`를 함께 보내 전체 정의를 상속하며, 단색을 바꾸면 색·무늬 종류·무늬 색을 모두 전송한다. 다른 글자·문단 속성만 바꿀 때 채우기 payload를 새로 만들지 않는다.

실제 경로는 `getter Fill 종류/id → populateFillSelection → collectMods → 기존 format command/API → create_border_fill_from_json의 원래 정의 복제 → 전체 Fill 비교 → 저장·재열기`다. 이 단계에서 renderer·줄 높이·pagination은 변경하지 않았다.

| 검증 | 결과와 한계 |
| --- | --- |
| 실제 제품 대화상자 메서드 회귀 30건 | 수정 전 제품 class로 26 FAIL / 4 PASS, 수정 후 30 PASS. DOM·미리보기는 대역이며 실제 브라우저 증거는 아래 별도다. |
| Studio 전체 Node 검사·TypeScript | 최종 1,870 PASS / 0 FAIL / 2 skip. Docker wrapper로 오래된 pkg 선언을 갱신한 뒤 TypeScript 통과. |
| 실제 Chrome DOM + fresh Docker WASM | image/gradient × 글자/문단 × 무변경/다른 속성/none/solid의 16건. 실제 선택·확인 클릭, public format API, live/HWP/HWPX 재열기에서 종류와 테두리 보존. command 라우터 전체의 자동 UI 조작 검사는 아니다. |
| 원 PR API의 fresh WASM 대조 | 기존 공개 원본 5개, 셀 8곳·쪽 2곳을 live/HWP/HWPX에서 확인. 정규화한 전체 fillBrush XML도 보존. 편집본의 전체 Print 시각 검증을 대신하지 않는다. |
| Docker WASM | locked wrapper·공용 target으로 성공. pkg/public과 실제 Chrome 응답 SHA-256 `0ff83fdc16b7f3a73fc7b29070260b3b486663c1f2d537a95b6053bb94591b92`. UI만 바뀐 보정 commit에도 같은 Rust 산출물을 재사용했다. |
| Native Skia | `cargo build --locked --release --bin rhwp --features native-skia --target-dir /home/edward/mygithub/rhwp/target/pr-review` 성공. CLI SHA-256 `407725365232991276cd71f7cf11b3785935a94d594f85000e7095df6a1edad4`. |

상세 실행은 ignored `output/pr-review/pr7661-20261015/`의 `browser-verification.json`, `wasm-api-readback.json`, `logs/studio-tests-final.log`, `logs/typescript-final.log`, `maintainer-code-proof.json`에 보존했다. 초기 전체 binary 빌드는 불필요한 q 도구까지 포함해 중단한 실행이며 성공으로 세지 않는다. 이후 CLI 지정 빌드는 별도로 성공했다.

## 메인터너 보정 1의 독립 출력 비교

공개 `el-school-001.hwpx`의 기존 글자·문단 속성을 새 ID로 복제하고 원본 그림 채우기 또는 `mel-001.hwpx`의 그라데이션을 연결한 수동 UI 검증 입력이다. 텍스트·저장 LineSeg·쪽 속성·이미지 자원은 유지했다. 한컴 정상 생성 원본이라고 부르지 않는다. 두 입력과 실제 브라우저에서 글자 크기/문단 여백만 바꿔 저장한 네 HWPX 모두 한컴 2020에서 열어 Print PDF를 생성했다. actual engine `11.0.0.9136`, `pdf_print_method=0`, `hancom2020_pdf_driver_one_up`이며 각 1쪽이다. 생성 방법·해시·MCP job/profile은 fixtures manifest 및 sanitized status JSON에 남겼다.

같은 PDF·fonts·96 DPI·compat2022에서 고정 base `a5e88ad682`와 보정 1을 비교했다. cached base producer의 Rust source는 고정 base와 동일함을 `src/crates/Cargo` diff로 확인했고 새 캡처에는 현재 도구를 사용했다. 6개 문서 모두 각 backend의 base/head PNG RGBA가 동일하다. 따라서 이 보정에서 0.01%p 이상 하락한 페이지는 없다. **독립 PDF와의 일치 및 자동 gate는 별도다.**

| 1쪽 문서 | Native base = 보정 1 | fresh WASM base = 보정 1 |
| --- | --- | --- |
| image | 92.31721% | 91.94430% |
| image-char-unrelated | 88.07046% | 87.89580% |
| image-para-unrelated | 92.29444% | 91.94491% |
| gradient | 90.36146% | 90.58653% |
| gradient-char-unrelated | 86.49732% | 86.91110% |
| gradient-para-unrelated | 90.35778% | 90.57237% |

최신 head 대표 image/gradient 글자 편집 review PNG를 직접 열어, 문단의 그림·그라데이션 미출력 및 일부 글꼴 굵기 차이를 확인했다. 단순 텍스트 추출·쪽수·점수만으로 시각 통과를 선언하지 않았다. 자동 판정은 `re_review_required`이며 원래 5개 API 원본의 **편집본** 전쪽 Print 비교도 아직 미검증이다. 수동 입력 6개 통과/실패와 혼합하지 않는다.

자료는 `output/pr-review/pr7661-20261015/{native,wasm}-{base,head}/<key>/silhouette.tsv`, `visual-decision.json`, `visual-review/<key>/<backend>/review_001.png`에 있다. 복사 가능한 그림 내장 HTML은 `/mnt/e/tmp/pr7661-maintainer-review/visual-review.html`이며 같은 폴더에 입력과 PDF를 복사했다. 원본 TSV·로그·중간 JSON 및 미달 fixture/golden은 커밋하지 않는다.

## 승인된 후속 범위: 문단 배경 렌더링

작업지시자는 위 자료를 안내받은 뒤 **“문단 배경 렌더링까지 보정”**을 승인했다. 시각 예외 승인은 없으며 자동 gate는 유지한다.

위반 계약은 문단 BorderFill의 그림·그라데이션도 실제 문단 배경 상자에 그려져야 한다는 것이다. 독립 근거는 위 같은 입력의 한컴 Print이며 이 범위는 조판 흐름·문단 높이를 바꾸지 않는 paint 소비 수정이다.

값 추적: `resolve_border_style`의 `gradient/image_fill` 생산(`style_resolver.rs`) → `paragraph_layout.rs`의 실제 줄 상자·spacing·border inset 범위 수집 → `layout.rs::render_para_border_groups`의 연속 테두리 그룹과 최종 상자 → 현재 `fill_color`만 소비하는 Rectangle. 일반 본문 호출은 `layout.rs`, 셀 문단 호출은 `layout/table_layout.rs`에 있다. 셀 자체 배경은 이미 `render_cell_background`에서 같은 종류와 이미지 효과를 소비하므로 이를 공통 paint 결과로 재사용한다. 채우기 종류가 달라도 stroke가 같다는 이유로 첫 문단의 Fill을 뒤 문단에 복사하지 않으며, 채우기를 나누기 위해 연결된 테두리에 새 가로선을 만들지 않는다.

구현 전 반례는 연결 테두리 내 서로 다른 채우기/채우기 없음, 같은 채우기 연속 문단, 본문/셀 문단, 분할된 문단 조각이다. 실제 저장 속성과 독립 Print의 관측으로 기대를 정하고 영향 영역을 먼저 비교한다. 새 정식 배치 회귀는 Native/fresh WASM 90%와 직접 판독 이후에 추가하며 absolute px/전체 SVG 해시를 핀하지 않는다. 현재 대화상자/API 검증을 새 renderer 검증의 대용으로 삼지 않는다.

## 추가 승인: API 대조 문서 6쪽의 저장 좌표계 전환

작업지시자는 기존 6쪽 위치 차이도 이번 범위에서 보정하도록 승인했다. 같은 편집 HWPX와 독립 한컴 2020 Print의 14쪽 전량을 대조한 결과, 첫 표(문단 97) 뒤부터 약 5px 차이가 누적 없이 유지된다. 입력은 실제 HWP를 public API로 편집·저장한 것이며 임의 LineSeg 수정본이 아니다. 해당 표의 저장 줄 높이 10,377HU는 표 10,095HU와 위·아래 바깥여백 141HU씩을 포함한다. 다음 문단의 저장 시작 11,129HU는 이 높이와 후행 간격 752HU의 합이다. 표 호스트의 이전 쪽 저장 원점 67,373HU를 그대로 더하면 이 닫힌 새 쪽 좌표계를 인식하지 못하고 간격을 절반으로 줄인다.

추적 경로는 `composer의 저장 줄·후속 원점 → float_placement::stored_tac_line_successor_advance → typeset/controls/tac_reconcile::measure의 점유 상한 → layout::tac_next_line_full_spacing의 실제 후행 흐름`이다. 저장 원점 리셋의 판정은 온전한 단일 TAC 줄 상자와 다음 원점의 정확한 닫힘으로 설명해야 한다. 개체와 다른 줄이 섞인 문단·dirty/계산 LineSeg·닫히지 않는 후속 좌표에 이 판정을 확장하지 않는다. 절대 px 보정이나 문서별 예외는 추가하지 않는다. 구현 뒤 같은 원본 전쪽과 정상 대조군을 다시 확인한다.

## 메인터너 보정 2: 배경 소비와 실제 글줄

로컬 제품 head `801fc9a9d`에 다음을 구현했다. 이는 API·UI 보정 1 이후 별도 Rust 변경이므로 이전 lint/WASM 결과로 검증을 대신하지 않는다.

- `style_resolver → layout/fill_background::border_fill_background_nodes → Rectangle/Image → SVG/Canvas/Native Skia`로 채우기 종류·이미지 효과를 공통 소비한다. Native의 Rectangle·PageBackground 그라데이션 shader도 실제 색상 stop을 소비한다. 선형 그라데이션 중심 0/50/100%의 독립 Print 대조군으로 양쪽 램프를 확인했으며 SVG·Canvas·Native가 같은 stop 생산 함수를 사용한다.
- 문단 배경의 단위는 문단 전체가 아닌 **실제 물리 글줄**이다. 85개 명시적 줄바꿈을 가진 단일 문단의 3쪽 Print에서 줄마다 재시작함을 확인했다. 일반/셀 문단은 실제 줄 전진·줄 상자에서 밴드를 생산한다. NO_LS의 InlineFlowPlan 경로는 측정·fit에서 확정한 `rows`와 `end`를 사용한다. 저장 TAC 계획의 조기 반환도 수용한 pen/end를 같은 수집기에 전달한다. 기존 혼합 인라인 표 fallback의 별도 줄 구성은 재작성하지 않았으며 이 경로의 일반화된 배경 지원을 검증 완료로 주장하지 않는다.
- 수집 뒤 float 앵커가 옮겨지면 해당 노드 ID의 **최종 원점**으로 밴드를 이동한 뒤 paint한다. 최종 원점은 한 번 만든 ID map으로 조회하며 줄마다 전체 노드를 재탐색하지 않는다. 가로 조각은 기존 저장 물리 줄 판정으로 소유 줄을 공유한다. 테두리 연결과 채우기의 줄/문단 소유는 분리한다. 양쪽 문단의 연결 속성·서로 다른 채우기·없음·셀 배경 위 문단 배경을 독립 Print와 대조한다.
- 글자 채우기도 원래 단색 전용이던 일반 TextRun 경로에서 전체 Fill을 소비한다. 글자 배경·테두리는 선행 공백까지 원래 run 상자를 사용하고 글리프 배치는 바꾸지 않는다. 글자 배경만 지정한 별도 Print로 문단 배경과 다른 계약임을 확인했다.
- 6쪽의 좌표계 전환은 `composer::stored_tac_successor_delta_hu`를 측정·배치가 공통 소비한다. 단일 저장 TAC 상자·원점 리셋·정확한 후속 닫힘을 확인하며 계산/dirty 줄·다른 개체가 섞인 호스트는 확장하지 않는다. Native 선행 검증에서 6쪽 73.76408→98.80061%, 같은 입력 14쪽 모두 90% 이상, 다른 쪽 점수 하락 0쪽이다. 기존 WASM 기준은 71.38766%로 Native와 구분한다.

정식 회귀 추가 전 Native/fresh Docker WASM 전쪽 TSV·직접 판독을 진행 중이다. 선행 Native 관측은 글자 크기 변경 그림 92.67522%, 그라데이션 93.26290%, 긴 문단 전쪽 최저 99.96178%, 텍스트·두 TAC의 같은 줄/명시적 개행·공백 줄 98.79320%, 표만 있는 문단 99.63766%다. 선행 결과는 최종 head 전쪽 결과와 구분한다.

사용자 제공 `page-image.pdf`는 같은 요청 입력에 대응하는 한컴 2020 11.0.0.9166 Print 1쪽이며 SHA-256 `fb025238855f343f7ae46ffb4efd9d3b4fe52334961252c0a5a8f45b3e197862`다. 자동 변환의 매크로 거절과 수동 정상 출력은 구분한다. 오래된 MCP job 재조회에서 TTL 만료가 확인된 일부 대조군은 만료 전에 다운로드한 로컬 PDF를 보존한다. 재조회 상태를 새 변환 실패로 세지 않으며 PDF 해시·Creator·원문 생성 명령·대응 출력으로 출처를 기록한다.

글꼴은 실제 존재하는 Windows Fonts 및 한컴 TTF/All·TTF/Hwp를 공급한다. 휴먼명조 HMKMM.TTF는 Hwp 디렉터리에 있다. Native는 실제 font program의 한글 이름·weight를 연결한 로컬 fontconfig를 사용하고, WASM은 같은 font program들을 명시적으로 로드한다. fontconfig를 Chrome 프로세스에도 적용한 실패한 실험은 최종 WASM 검증에서 제외한다. 이 환경 변경 전 캡처를 같은 조건의 최종 기준으로 재사용하지 않는다.

## 직접 판독 보완: 1쪽 글상자 그림 채우기의 저장 모드 손실

작업지시자는 그라데이션은 통과로 판정했지만 issue7235 1쪽 글상자 그림의 반복 출력을 지적했다. 이어 저장 시 손상을 의심했고 `/mnt/e/tmp/issue7235.hwpx`에서 배치를 ‘크기에 맞게’로 바꾸면 정상임을 확인했다. 해당 수정본의 글상자는 `imgBrush mode="TOTAL"`이다. 이 사실을 새 renderer 결함으로 단정하지 않고 원본 입력과 저장 XML을 다시 대조했다.

원본 HWP `156086935_none_image_fill.hwp`의 글상자 및 두 BorderFill 그림은 공통 IR에서 `ImageFillMode::None`(HWP5 값 15)이다. 동일 원본을 한컴 2020 MCP로 편집 없이 HWPX로 저장하면 세 곳 모두 `ZOOM`이다(job `7ffa0e23-ead2-4c24-ab3d-24781317c731`, 출력 376,518 bytes). 반면 기존 rhwp API 편집 저장본은 세 곳 모두 `TILE`이고 재열기 IR도 `TileAll`이다. 따라서 동일한 저장본을 Native/WASM으로 그릴 때의 차이를 조사한 선행 결과만으로 렌더러를 원인으로 판정할 수 없다. **이번 반복 현상의 직접 원인은 HWP→HWPX 직렬화 시 모드 붕괴**다. 한컴으로 원본을 저장한 `ZOOM`과 사용자가 의도적으로 변경한 `TOTAL`은 구분한다.

값의 경로는 HWP `parser/doc_info::parse_fill`의 15→None → 원본 Shape/BorderFill IR → `serializer/hwpx/shape::write_fill_brush`의 미처리 enum을 TILE로 바꾸는 fallback → HWPX header/section parser의 TileAll → 실제 paint다. header의 BorderFill도 같은 writer를 사용한다. 해당 writer에서 `None`을 `ZOOM`으로 출력하도록 보정한다. 이미 정상인 `Zoom`, 실제 반복 모드 `TileAll`과 채우기 없음 `FillType::None`의 의미는 유지한다. renderer 원점·크기·clip을 덮어쓰지 않는다.

증거는 ignored `output/pr-review/pr7661-20261015/image-fill-roundtrip/`의 원본/저장 IR 진단, 한컴 무편집 HWPX와 사용자 수정본이다. 앞선 C10 26문서 전쪽 자동 점수가 통과해도 이 직접 판독 결함을 면제하지 않는다. 저장본의 독립 PDF도 이미 잘못된 TILE을 소비했으므로 그 PDF와의 높은 점수는 저장 보존의 정답 근거가 아니다. 보정 뒤 새 저장본과 원본/한컴 정상 저장본을 대조하고 영향 페이지를 다시 확인한다. 기존 UI/API 검사와 원래 입력을 덮어쓰지 않으며 새 산출물 경로를 사용한다.

역방향도 독립 한컴 저장으로 확인했다. 한컴의 ZOOM HWPX를 HWP로 저장하면 글상자와 두 셀 모두 HWP5 값 15(None)이며(job `b83a3e1e-fd41-41ce-bedc-af930f825030`), 사용자 수정 TOTAL 글상자를 HWP로 저장하면 값 5(FitToSize)다(job `0f57a22c-9faa-43c7-adb6-a0add151fd67`). 기존 HWP writer도 Zoom→5, Total→0으로 기록하므로 그대로 두면 다시 저장할 때 의미가 손상된다. HWP header/shape의 중복 표를 공통 함수로 합쳐 `Zoom/None→15`, `Total/FitToSize→5`를 함께 보존하고, HWPX의 FitToSize도 공식 `TOTAL`로 출력한다. 별도의 렌더링 동작은 변경하지 않는다. 교차 저장 의미를 확정하는 동안 첫 C11 Docker 빌드는 완료 전에 중단했으며 성공 증거로 세지 않는다. 최종 제품 코드가 정해진 뒤의 빌드로 새 산출물을 만든다.

### 보정 저장본의 재검증과 사용자 판정

작업지시자는 기존 Docker WASM에 보정 저장본만 다시 연 Studio 탭에서 **1쪽 글상자 그림 시각 판정 통과**를 회신했다. 앞서 그라데이션의 통과 판정도 유지한다. 같은 렌더러에서 저장본 교체만으로 반복이 해소되므로 저장 계층의 원인 판정을 뒷받침한다.

제품 source `bbd477b1f`의 Native와 fresh Docker WASM 저장본은 issue7235에서 **바이트까지 동일**하다(SHA-256 `cd8226dcc0706f30b8580928a986e659c4a0e2caf597b725ddcffeb5298a5e5f`). 선행 `07397acc5` Native 저장본도 같은 바이트다. 같은 저장본의 한컴 2020 독립 Print 14쪽(job `6d4f07e3-5043-4081-b6a7-9a6b2bd17589`)을 다시 확보했다. Native 전쪽 최저 95.22496%/1쪽 99.33702%, 새 Docker WASM 전쪽 최저 95.54386%/1쪽 98.62085%이며 14쪽 모두 측정됐다. Native PNG는 동일한 저장 바이트와 동일한 paint 코드인 선행 C11 실행을 재사용하고 실행 source를 구분한다. fresh WASM의 같은 1쪽을 Print와 확대 판독해 반복·누락·큰 로고 잘림이 해소됐음을 확인했다. 자료는 `image-fill-roundtrip/{c11-native,c12-wasm}/issue7235-corrected-save/`의 TSV/PNG와 `fresh-wasm-first-page-review.png`다.

최종 Docker 빌드는 성공(9m15s)했으며 WASM SHA-256은 `a41d6cb8e504bdb269420321a06d49ba8b93b035e554076ad6b38c447bc858b8`다. 루트 pkg·Studio public·cache/SW 우회한 브라우저 응답의 해시가 같다. 새 탭 `http://localhost:7801/?review=pr7661-c12-save-contract&lang=ko`에서 실제 새 WASM으로 원본 5개의 API 편집 후 HWP/HWPX와 HWPX→HWP 저장을 실행했다. issue7235의 3곳은 HWPX ZOOM, HWP 및 HWPX→HWP는 모두 None(15)이다. 나머지 mel/school/newyear/page-image의 새 API 편집 HWPX는 이전에 Print 검증한 파일과 바이트까지 동일하므로 기존 대응 PDF를 그대로 재사용한다. `image-fill-roundtrip/{c12-wasm-verification,c12-save-mode-results,unchanged-other-api-inputs}.json`에 기록했다. API·UI 구현을 바꾸지 않은 중복 UI 16시나리오 재실행은 하지 않았다.

정식 `issue_7661_image_fill_save_modes` 두 검사를 같은 compiled harness에서 CLI만 교체해 실행했다. 수정 전 C10 CLI는 두 검사 모두 의미 손실 assertion으로 FAIL(exit101), C12 CLI는 두 검사 모두 PASS(exit0)다. 글상자·BorderFill의 2회 HWP↔HWPX 저장 및 실제 TileAll/Total의 구분을 보호한다. 처음 untracked 신규 source를 준비하기 전 실행이 0건으로 끝난 결과는 검증 통과로 세지 않았고 stage 뒤 suite를 다시 준비해 실제 2건 PASS를 확인했다.

기존 26문서 96쪽 C10 Native/fresh WASM은 전쪽 90% 이상·base 대비 0.01%p 이상 하락 0쪽이다. 해당 paint 구현은 이후 저장 모드 보정에서 바꾸지 않았다. 다만 API issue7235 저장 입력의 TILE 손실은 높은 점수와 무관하게 위에서 별도로 고쳤으며, 잘못된 저장본의 독립 Print 통과를 보존 계약의 증거로 쓰지 않는다. 정식 배경 회귀의 입력 4개와 대응 Print는 `tests/fixtures/issue7661_backgrounds/`에 보존하고 source 소유/내용/포함/레이어 검사로 추가한다. 기존 #7235의 절대 px 핀도 고유 source 그림 치수→소유 render node→실제 SVG viewport/정렬 관계로 교체한다. 다른 문서의 검사는 확장하지 않는다.

Native Skia의 선행 lib 실패 1건은 실제 gradient를 첫 색상 단색 fallback으로 검사하던 기대값이었다. 사용자 그라데이션 판정과 독립 중심 0/50/100 Print를 근거로 상·하 원본 색 stop의 우세/변화 관계를 검사하도록 고친다. 기존 임계값만 완화하지 않으며 pattern/ellipse/path/line 검사는 유지한다. 이 source-side test 변경은 unit-tier base check와 최종 Native Skia lib 검사 대상으로 남긴다.


## 최종 제출 검증: 저장 보정·배경·후속 흐름

최종 제품 source는 `e167bf07ac0c84760a14de86ead849fe5beb3f4c`, 고정 base는 `a5e88ad682c805e1d845502ddc9521b94ea8eccd`다. 이후 `9e662936c`는 기존 배경 회귀의 빈 줄·후속 문단 assertion만 보강했으며 제품 source·입력·기준 PDF는 바꾸지 않았다. 같은 Native/WASM 산출물을 재사용하며 제품 재빌드나 전쪽 캡처를 반복하지 않는다.

| 최종 검증 | 관측과 증거 |
| --- | --- |
| Native / fresh Docker WASM | 각각 26문서·96쪽 전량. 최저 92.29460% / 90.98799%. 90% 미만·누락·쪽수 불일치·같은 입력/PDF의 base 대비 0.01%p 이상 하락 모두 0. |
| issue7235 API 편집본 | 14쪽 전량. 1쪽 Native 99.33702% / WASM 98.62085%, 6쪽 98.80061% / 98.84667%. 반복 그림 해소와 표 뒤 간격·뒤 문단을 직접 판독했다. |
| 저장·재열기 | 최종 실제 Docker WASM에서 원본 5개를 API 편집 후 HWP/HWPX 및 HWPX→HWP로 저장했다. 최종 HWPX 5개는 커밋된 검증 입력과 바이트가 같다. issue7235의 3곳은 HWPX ZOOM / HWP None(15)을 유지한다. |
| 전체 기본 Rust | `cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review --tests --no-fail-fast`: 10,304 PASS / 0 FAIL / 45 skip. 마지막 PDF JSON 검사는 완료됐으며 취소·재실행하지 않았다. |
| Native Skia | `cargo test --locked --profile release-test --target-dir target/pr-review --features native-skia --lib`: rhwp 3,859 PASS / 0 FAIL / 12 ignore. placeholder 2 PASS, direct PDF 4 PASS. |
| 배경 정식 회귀 | 최종 assertion 5건을 같은 compiled harness에서 CLI만 교체: 고정 base 5 FAIL(exit101) → 최종 제품 5 PASS(exit0). 실제 공백 줄의 양수 높이·앞뒤 순서, 85줄의 완전한 보존과 마지막 뒤 문단 1회 표시를 포함한다. |
| 저장 모드 정식 회귀 | 동일 harness의 수정 전 C10 CLI 2 FAIL → 모드 보정 CLI 2 PASS. 실제 반복 모드와 크기 맞춤을 구분하고 두 번의 왕복 저장을 검사한다. |
| 최종 lint·정책 | assertion 보강 뒤 fmt/check, native·WASM lib·workspace all-targets의 세 Clippy, workspace build, 고정 base manifest 및 unit-tier check 전부 PASS. 별도 native-skia feature Clippy도 PASS. |
| Studio UI | 제품 UI source가 같은 보정 1의 30건·전체 Node 1,870 PASS/2 skip·TypeScript·실제 브라우저 16건을 재사용했다. 새 실행으로 표시하지 않는다. 바뀐 저장 계층은 위 최종 WASM API 실행으로 별도 확인했다. |

Native CLI SHA-256은 `49ac6d308e7665ff0ac2bc1d9d700ac04c10bed5b72f30d35dc486a087ae7542`, 최종 Docker WASM은 `2fa9ffcd57ced1f90112de79867fb7048ac7bf8c8500be7032c03b9d4d91d817`다. 루트 pkg·Studio public 및 브라우저 직접 응답의 해시를 대조하고 캐시/SW를 우회한 새 탭 `http://localhost:7801/?review=pr7661-final-save-contract&lang=ko`에서 실제 API와 14쪽 로드를 확인했다. 대표 PNG도 이 최종 산출물로 생성했다.

전쪽 명령은 ignored `output/pr-review/pr7661-20261015/run-background-visuals.py`에 보존했다. `RHWP_BG_COMMITTED_INPUTS=1`로 `verification-inputs.json`의 21쌍과 기존 공개 원본/PDF 5쌍을 고정하고 `RHWP_BG_CLI`·`RHWP_BG_PKG`에 위 최종 바이너리를 전달해 Native/fresh WASM을 각각 실행했다. 각 문서의 TSV·입력/PDF hash·쪽수·비교 환경은 `background-render/final-{native,wasm}/<key>/`에 있다. 집계 `background-final-strict-gates.json`은 26개 키·96쪽·누락·하락·같은 입력/PDF 대응을 검사했다. 실루엣 전용 실행의 `not_evaluated`를 자동 PR 승인으로 바꾸지 않고, 전쪽 TSV 조건과 직접 판독·사용자 판정을 별도로 확인했다.

입력과 독립 Print는 `tests/fixtures/issue7661_backgrounds/verification-inputs.json`에 재현 가능한 저장소 경로·SHA로 보존했다. 신규 21쌍의 42파일은 실행본과 Git blob이 같다. 정상 원본에 편집 속성을 연결한 수동 대조군은 실제 생성 방식을 표시했으며 한컴 생성 원본으로 부르지 않는다. `format-oracles/`의 네 파일은 독립 한컴의 ZOOM↔15, TOTAL↔5 **저장 enum 근거**이며 별도 픽셀 baseline이나 추가 전쪽 통과 문서로 세지 않는다.

같은 페이지·배율의 직접 판독은 1쪽 로고의 반복·잘림 해소, 6쪽의 세 사례 표와 뒤 문단 간격, 줄별 그라데이션의 재시작, 명시적 개행/너비가 충분한 두 TAC의 소속·실제 공백 줄, 이어지는 문단을 확인했다. 기존 일부 글꼴 굵기 차이는 남으며 완전한 픽셀 동등성을 주장하지 않는다. Native 비선형 그라데이션의 모든 종류와 기존 혼합 인라인 표 fallback의 일반화된 배경 배치는 미검증이며 이번 수용 범위에 포함하지 않는다.

기존 #7235 검사 중 수정한 두 함수는 같은 문서의 고유 원본 그림 치수→소유 render node→SVG viewport·비율 관계로 고정 px 영역을 교체했다. 원본 치수의 계약과 그림 누락/중복 검사는 유지한다. 다른 문서의 검사는 바꾸지 않았다. Native gradient 검사도 독립 Print와 원본 stop의 상하 색상 관계로 교정했으며 기존 단색 fallback 기대를 허용치 완화로 유지하지 않았다.

증거 위치(모두 ignored 실행 자료): `logs/final-{nextest,skia-lib,skia-placeholder,skia-direct-pdf}.log`, `logs/submission-final-lint-sequence.log`, `logs/final-clippy-native-skia-feature.log`, `logs/final-boundary-{before,after}-formal.log`, `background-probe/final-formal-before-after.json`, `image-fill-roundtrip/{final-wasm-verification,final-saved-inputs-match,formal-before-after}.json`, `final-review-assets/manifest.json`. 공개 대표 review/overlay는 [assets/pr7661](../assets/pr7661)에 16개 PNG로 보존한다. TSV·중간 JSON·로그·생성 suite·로컬 환경 파일은 커밋하지 않는다.

원격 #7661은 재조회 시 여전히 원 head `f1da6189670357b4a2394c73d7106137ab4b38c4`, OPEN, maintainer 수정 가능이다. 최신 base와 local 통합 충돌 없음. 원 기여 commit을 보존하는 보정본 push·한국어 본문 갱신·새 head Full CI/CodeQL 실행 승인을 받은 뒤 원격 반영한다. 원 head의 기존 CI를 보정본의 CI로 대신하지 않는다.


## 최종 CI와 병합 전 확인

- 검증 head `08a49f869c27c099c1dd9108b7a923d542c96b62`, 최신 devel `a5e88ad682c805e1d845502ddc9521b94ea8eccd`. merge tree 충돌 없음, 원 contributor commit 포함, 최신 검사 실패·대기 없음, MERGEABLE/CLEAN.
- [CI](https://github.com/edwardkim/rhwp/actions/runs/38682005770), [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/38682005792)와 GHAS CodeQL, [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/38682005766), [Adapter](https://github.com/edwardkim/rhwp/actions/runs/38682005786), [Proptest](https://github.com/edwardkim/rhwp/actions/runs/38682005784) 성공.
- CI Impact Policy 실패는 GitHub PR base가 과거 `60efc4b7`에 머물러 devel의 기존 workflow 변경까지 포함한 2,824파일을 반환한 것이 원인이었다. 원래 base 이름 devel을 다시 지정한 뒤 최신 base의 96파일·17커밋으로 정상화됐고 로컬 PR diff와 일치했다. enforcement 파일의 제품 변경은 없다. [initial Controller attempt 2](https://github.com/edwardkim/rhwp/actions/runs/38682001100/attempts/2)와 [완료 감사 attempt 2](https://github.com/edwardkim/rhwp/actions/runs/38683585696/attempts/2)가 성공했다. 제품 CI를 중복 실행하거나 정책·branch protection을 완화하지 않았다.
- 메인터너 일반 경로의 병합 후 운영 기록 직접 반영을 적용한다. 성공한 제품 head는 유지하고, CI 확정 결과·archive 이동·오늘할일·merge SHA만 후속 문서 commit으로 반영한다.

## Merge 후 contributor PR comment 계획

원 기여의 셀·쪽 Fill 조회/보존 개선에 감사하고, 실제 대화상자 소비 과정의 배경 삭제·문단 배경 누락·HWP/HWPX 그림 배치 enum 손실·6쪽 저장 좌표계 전환을 보정한 이유와 근거를 설명한다. 원 기여 commit을 보존한 merge SHA와 위 최신 CI 링크를 사용한다. Native/fresh Docker WASM 각각 26문서·96쪽, 최저 92.29460%/90.98799%, 쪽수 차이·미달·기준 대비 0.01%p 이상 하락 0쪽 및 메인터너의 직접 시각 통과를 명시한다. 사용자 판정과 자동 실루엣 보조값을 구분한다.

실제 devel에 포함된 `assets/pr7661/api-issue7235-p01-wasm-review.png`와 `api-issue7235-p01-wasm-overlay.png`를 merge SHA의 원본 저장소 raw URL로 표시한다(해당 쪽 98.62085%). [Visual Sweep 가이드](../../manual/verification/visual_sweep_guide.md#github-merge-comment)를 연결하고 원본·독립 Print는 검증 manifest를 안내한다. 기존 글꼴 굵기 차이, 비선형 그라데이션 전 종류 및 혼합 인라인 표 fallback의 미검증 범위는 전체 해결로 확대하지 않는다. 별도 종료 이슈 참조는 없으며 관련 없는 이슈를 닫지 않는다.


## 병합·후속 운영 기록

2026-10-15 KST, [PR #7661](https://github.com/edwardkim/rhwp/pull/7661)을 [merge commit 7b36eb7f0d](https://github.com/edwardkim/rhwp/commit/7b36eb7f0d2b36ae558ca50aea667c893ce05ff0)으로 통합했다. 부모는 최신 devel `a5e88ad682`와 검증 head `08a49f869`이며 원 기여 `f1da61896`의 이력을 보존한다. [정식 승인 review](https://github.com/edwardkim/rhwp/pull/7661#pullrequestreview-5528587489)는 검증한 동일 head에 게시했다.

운영 기록 반영은 maintainer 일반 경로의 option M으로 archive review와 오늘할일만 devel에 직접 반영한다. 제품 source·test·workflow·fixture·baseline을 포함하지 않는다. 별도 종료 이슈 참조는 없으므로 issue close를 추가하지 않는다. contributor 결과 안내는 앞 절의 계획과 실제 merge SHA의 asset으로 게시한다.

기본 작업공간은 devel로 전환했다. `rhwp-studio/public/rhwp.js`의 기존 미커밋 생성물은 SHA-256 `f377c6be818c64ffd20752e29271000ff5053ea8c7c984f65abb8fe74f17eddd` 그대로 유지하며 운영 기록에 stage하지 않는다. 공유 `target/pr-review`·루트 pkg·Studio public은 보존한다. 원시 TSV·실행 로그는 AGENTS.md의 ignored 증적 보존 지침에 따라 남긴다. 별도 PR worktree는 앞 단계에서 이미 해제했으며 로컬 review branch만 후속 안내 뒤 정리한다. contributor fork의 `fix/cell-fill-readback`은 소유 범위 밖이므로 보존한다.

[병합 후 검사 시간 메타데이터 갱신](https://github.com/edwardkim/rhwp/actions/runs/38687414031)은 성공했고 실측 수집과 data branch 갱신 step이 모두 실행됐다. 병합 후 제품 CI·CodeQL·Render Diff를 새로 실행하지 않았다.

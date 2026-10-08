---
kind: snapshot
status: archived
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-10-08
---

# PR #7630 검토 — 폭 0 표 앵커 뒤 텍스트 문단의 중복 줄 전진

## 최종 판정

**승인 — 코드·focused 검사·Native/fresh WASM 시각 대조를 충족하고 작업지시자가 시각 통과와 병합을 승인했다.** 생산 코드 보정은 없다. 2026-10-08 본문을 현재 head로 갱신하고 원 증적 생성 시점을 구분했다. API 본문 일치·현재 head PNG 6개의 정상 응답·실제 PR 화면 표시를 확인했다.

- 최신 head `e4cbce28cbdefcb9da56660302f8152666add069`, CI 성공과 MERGEABLE/CLEAN을 재확인했다. 현재 devel `74faf435a`와의 merge-tree는 exit 0, tree `29c52cf5ae153f085abb021ef43015b4227eead2`다.
- 사용자 시각 판정 및 병합 승인: 2026-10-08. 병합 뒤 확정값은 후속 처리 절에 기록한다.
- 수용 범위: #7621의 **저장 후 재열기**, 폭 0·빈 host·오프셋 0·문단 기준 TopAndBottom 표 뒤의 텍스트 문단. 데스크톱 한/글 전반의 호환이나 모든 TAC 전환·표 분할의 완료를 주장하지 않는다.

## 접수와 경로

| 항목 | 값 |
| --- | --- |
| PR / 작성자 / 관련 이슈 | [#7630](https://github.com/edwardkim/rhwp/pull/7630) / sacru2red / closes [#7621](https://github.com/edwardkim/rhwp/issues/7621) |
| base / 검토 head | devel / `e4cbce28cbdefcb9da56660302f8152666add069` |
| 현재 로컬 기준 devel | `74faf435a0195ffb936e320563d05bd618f618ee` |
| 규모 | 15파일, 생산 코드 +17줄, 새 회귀 +131줄, 생성기·설명·manifest·HWP 2개·PDF 2개·PNG 6개 |
| reviewer | edwardkim을 지정하고 API로 확인했다. #7608의 기존 기여자이므로 first-time 경로는 비해당이다. |
| 작성 시점 참고 상태 | OPEN / non-draft / MERGEABLE / CLEAN / check 실패·진행 중 없음; merge 전 재확인 필요 |

- base route: `maintainer_general`.
- modifiers: `intake_and_review`, `local_validation`, `visual_fixture_evidence`, `post_merge`.
- loaded documents: `pr_review_workflow.md`, `pr_review/README.md`와 위 자식, `review_template.md`, 개발 환경·시각 검증 정본.
- branch/worktree: `review/pr7630-20261008` / `/tmp/rhwp-pr7630-review-20261008`. 검토·승인 대기 중이므로 유지한다. 기본 devel과 기존 검증 Studio는 변경하지 않았다.

## 구현 근거와 실제 소비 경로

문제는 글자가 있는 다음 문단 자체의 줄 높이가 아니다. `stored_empty_anchor_band_host_line_advance_hu`가 생성 HWP의 `next.vpos - host.vpos == lh + ls`를 **표 아래에 별도 host 줄이 있다는 증거**로 해석한 것이다. 폭 0 host 줄은 기존 #7470 계약상 TopAndBottom 띠 안에 흡수되므로 이 사다리 등식을 우선 적용하면 줄 진행을 중복 계상한다.

새 guard는 기존 `empty_host_line_absorbed_by_topbottom_float`를 재사용한다. 저장 정보 유효성·빈 문자열 가드를 유지한 뒤, 실제 개체의 비TAC·TopAndBottom·Para·세로 오프셋 0, 단일 저장 줄·폭 0·implementation tag 없음으로 판별한다. 문서 ID·특정 표 크기·임의 허용치를 추가하지 않았다. 폭이 남는 줄, 양수 오프셋, host 글자/공백, 다중 줄, TAC·다른 wrap은 기존 경로를 유지한다. Table뿐 아니라 Picture/Shape에 같은 기존 흡수 술어를 적용한다.

변경 값 하나인 **host line advance**의 연결:

1. `src/renderer/float_placement.rs:3411–3463`: 흡수된 host는 `None`을 반환해 별도 줄 꼬리를 만들지 않는다. 기존 마지막 개체 소유·저장 사다리 판정은 다른 입력에서 유지된다.
2. `src/renderer/typeset/table/host_spacing.rs:258–279`: 동일 helper 결과를 `stored_empty_anchor_host_line_tail`로 읽어 `after`와 `after_for_fit`에 반영한다. 바깥 아래 여백의 흐름/fit 차이는 기존 계약이며 이 PR은 바꾸지 않는다.
3. `src/renderer/typeset/table/block/entry.rs:223`: `before + after_for_fit`를 표 요구 높이에 소비한다. 예약 간격과 후속 흐름도 이 HostSpacing 결과를 쓴다.
4. `src/renderer/layout.rs:13796–13856`: 동일 helper를 호출해 표 lane의 실제 하단에 host 꼬리를 더할지 선택한다. 흡수된 입력은 꼬리 분기를 거치지 않고 기존 빈 float 흐름으로 간다. 공통 helper 뒤의 lane 원점 선택까지 확인했으며 재현·대조군에서 최종 TextLine 원점도 대조했다.

## 조판 원칙 준수 검토

| 항목 | 확인 근거 | 판정 |
| --- | --- | --- |
| 구현 근거와 일반성 | 기존 #7470 흡수 계약과 독립 한컴독스 Print PDF. 문서별 조건·clamp·출력 은폐 없음. #6147의 폭이 남는 host 대조 검사 통과 | 충족 |
| 측정·배치 일관성 | 위 생산→HostSpacing→fit→최종 lane 흐름 연결. 두 backend 모두 다음 문단 195.3px, 표 아래 바깥 여백과의 관계 일치 | 충족 |
| 줄 소속·점유 높이 | 비어 있는 앵커 줄과 뒤의 실제 문단을 구분. 빈 다음 문단 대조군의 줄 점유·뒤 문단 순서를 보존 | 충족 |
| 분할·이어받기 | 컷/rowspan/유닛 종료 구현은 변경 없음. 요구 높이는 공통 HostSpacing을 소비한다. 기존 #6950 fit·split/defer 경계와 #7470 그림 이월 검사 통과. 새 guard가 발동하는 다쪽 표 조각의 독립 PDF 검증은 하지 않았으며 완료 범위에 넣지 않는다 | 컷 변경 비해당 / 기존 경계 충족 / 신규 다쪽 경로 미검증 |
| 독립성과 음성 대조 | 생성 입력은 수동 LineSeg 변경 없이 rhwp API로 생성된 저장본. 한컴독스의 독립 인쇄본과 대조했다. 수정 전 실제 WASM에서 같은 기하 관계 FAIL/PASS, 수정 후 Rust 정식 검사 2 PASS 및 fresh WASM 관계 PASS | 충족 |
| 기준값 변경 | baseline·golden·래칫·허용치 변경 없음. 새 검사 기대값은 절대 화면 위치가 아닌 `표 하단 + 바깥 아래 여백 = 다음 문단 윗변` 관계 | 비해당 / 관계 검사 충족 |
| 주장과 증거 범위 | #7621 두 입력 전쪽·Native/fresh WASM 직접 판독. 기여자의 155문서/6,157쪽 tree 동일 주장은 이번 세션에서 전수 재현하지 않았고 reviewer의 전수 한컴 피델리티 증거로 쓰지 않는다 | 이번 수용 범위 충족 / 전수 주장 미검증 |

## 입력·기준 PDF와 커밋 확인

새 입력은 `samples/issue7621/generate.mjs`를 devel `d9749c5a1` web WASM에서 실행한 생성 대조군이다. 한컴 저장본 일반과 구분한다. 두 기준은 한컴독스 6.3.6 / 한컴오피스 Web v2 한글 `20260812021742`의 **파일 → 인쇄** 서버 PDF이다. Producer cairo 1.15.12, PDF 1.5, A4 595×842pt, 각 1쪽이며 직접 열어 원문 대응을 확인했다. 정상 독립 Print 출력이 있어 재변환하지 않았다. 데스크톱 Print 대조는 미검증이다.

| 실제 저장소 입력 | SHA-256 |
| --- | --- |
| `samples/issue7621/b-text-after-float.hwp` | `8af2ecfe54878968ffb78634a4075dfed6ce733d10e16322c7123d28a0e7cafa` |
| `samples/issue7621/control-float.hwp` | `81d61a11efd5070e64767b8cd32491ba8af93ec212351607694e10aece0a9480` |
| `pdf/issue7621/b-text-after-float-hancomdocs.pdf` | `1b4a56da1cfe1fc38cf7337f159713217f1f87473a1d70068b927ec3a1ebf35d` |
| `pdf/issue7621/control-float-hancomdocs.pdf` | `83da2a4b27b50aad35ac4a502e9acb521e1f222ece95bfa15485568bde52f9f1` |

**검증 입력 커밋 확인: 충족.** 위 파일은 HEAD `e4cbce28c`의 blob과 실행 파일·fixture manifest가 바이트 및 해시로 일치했다. 관련 focused 검사에 사용된 기존 저장소 입력도 같은 head blob과 대조했다. 전체 경로·해시는 로컬 `output/pr-review/pr7630-20261008/all-input-commit-check.json`, 테스트 입력 출처는 각 정식 case에 기록되어 있다. 실행 로그·JSON·TSV는 ignored output에만 보존한다.

## 실행한 검증

증적 루트: `/home/edward/mygithub/rhwp/output/pr-review/pr7630-20261008/`.

| 검증 | 실제 결과 |
| --- | --- |
| current-base `git merge-tree --write-tree` | exit 0, tree `29c52cf5ae153f085abb021ef43015b4227eead2`; GitHub CLEAN과 구분 |
| `git diff --check upstream/devel...HEAD`, 새 설명 문서 링크 | 통과 |
| derived manifest `--prepare` / `--check --base-ref 74faf435a…` | 통과; 파생 source PR 변경 없음 |
| `node scripts/run-rust-test.mjs issue_7621_text_after_float_only_host -- --cargo-profile release-test --target-dir /home/edward/mygithub/rhwp/target/pr-review` | 2 PASS / 0 FAIL; 241 skipped는 필터 밖 검사 |
| 관련 #6147·#7470·#6888·#6950·#6312 및 새 sample 보안 검사 | 41 PASS / 0 FAIL, 859 필터 밖 skipped. 새 HWP 2개를 `RHWP_SECURITY_SWEEP_SAMPLES_JSON`으로 명시 전달 |
| Native `cargo build --locked --profile release-test --bin rhwp --target-dir /home/edward/mygithub/rhwp/target/pr-review` | 통과, 2m13s |
| fresh WASM root wrapper `--target web --out-dir pkg` | 최적화 포함 통과, 7m35s. 공유 target 사용; 로컬 wrapper 검증이며 Docker 릴리즈 패키징으로 주장하지 않음 |
| Studio public 동기화 | JS·WASM 각각 pkg/public SHA-256 일치. 이번 실제 Chrome 실행은 Sweep이며 별도 Studio 편집 E2E는 하지 않음 |
| Native/fresh WASM Sweep | 두 입력 각 전1쪽, 인쇄 프로필·96dpi; TSV와 대표 review/overlay를 새로 생성하고 직접 판독 |

WASM SHA-256: `da267652e968e1f2192b7452ad06fed07f6d1fba3708c80e942ae7d3121316ae`; JS `70cde06a369fa7fd4fc8bc8f3d6acaee158596ba1a116a2c72159002b0b5654e`. Native 및 각 실행 시 실제 exporter hash는 run manifest와 `review-provenance.json`에 기록했다. 글꼴 디렉터리 `/mnt/c/Windows/Fonts`의 존재·입력 face 공급을 확인했다. Chrome 154 Linux headless/webfont, 인쇄 프로필로 실행했다.

### 수정 전 음성 대조

기존 검증 WASM source `93322623d8092f01fe3353cb261f31dffe683bae`의 production은 현재 devel `74faf435a`와 동일하다. 해시 `0b68fda69a28b83d1d719acedd4a139e2f8ea9bdd95c5c2735f155adb5b0459c`를 확인하고 실제 HwpDocument로 동일 HWP를 열었다. 기대 관계는 독립 PDF와 새 정식 검사의 표 하단·여백 관계이며, 제품 helper를 복제하지 않았다.

- 글자 다음 문단: 표 하단191.6px + 여백3.7733px = 기대195.3733px, 실제216.7px → **관계 FAIL**, 21.3267px 과대 전진.
- 빈 다음 문단 대조군: 실제195.3px → **관계 PASS**.
- 수정 후 Native/fresh WASM: 두 입력 모두195.3px → **관계 PASS**. 다음 문단 및 뒤 문단·셀 텍스트가 한 번씩 보존되고 순서·쪽 소속도 유지했다.
- 음성 대조는 수정 전 WASM 제품 실행이다. 새 Rust nextest를 수정 전 소스로 다시 빌드·실행했다고 쓰지 않는다. 기여자가 제시한 수정 전 Rust FAIL 기록과 별도 증거다.

### CI 재사용과 제한

원 code candidate `f3b26a6cf303589737ddc233bac915802421517b`의 [Full CI 37586494550](https://github.com/edwardkim/rhwp/actions/runs/37586494550)는 완료·성공했다. 네 archive worker, Native Skia, fmt·native/WASM32/workspace lint, 별도 CodeQL 3언어·Render Diff·Adapter·Proptest 성공을 확인했다.

최신 [CI 37637680059](https://github.com/edwardkim/rhwp/actions/runs/37637680059)는 preflight에서 원 candidate 성공과 base merge-tree를 검증해 heavy lane을 **SKIPPED** 처리하고 Build & Test를 성공 처리했다. 이 skipped를 최신 head의 Full 실행 성공으로 세지 않는다. latest preflight 로그의 candidate `f3b26a6cf…`와 current-base-tree 검증을 대조했다. PR 고유 생산 코드·테스트·입력·증적은 두 source head 사이에서 동일하다.

검토자가 code/test/fixture/baseline 보정을 추가하지 않았으므로 기존 Full CI를 광범위 회귀의 근거로 재사용했다. 로컬 release-test 전체와 Native Skia 전체·3종 lint의 중복 실행은 하지 않았고, focused 43건·fresh WASM·시각 증적·current-base 충돌/정책 검사를 별도로 수행했다. source 보정이 생기면 이 생략 판단을 다시 적용할 수 없다.

## 직접 시각 대조

| 입력 / backend | 전체 쪽수 PDF / rhwp | 최저 실루엣 / 원값 | 미달·누락 / 후보 | 사람 판독 |
| --- | --- | --- | --- | --- |
| b-text-after-float / Native | 1 / 1 | 100.00000% / 100.00000% | 없음 / 0 | 표 외곽과 두 후속 문단 위치·순서 일치 |
| b-text-after-float / fresh WASM | 1 / 1 | 100.00000% / 100.00000% | 없음 / 0 | Native와 raster bytes 동일 |
| control-float / Native | 1 / 1 | 100.00000% / 100.00000% | 없음 / 0 | 빈 다음 문단의 줄 공간과 뒤 문단 보존 |
| control-float / fresh WASM | 1 / 1 | 100.00000% / 100.00000% | 없음 / 0 | Native와 raster bytes 동일 |

이진화 경계 조정 픽셀은 모두0. 일반 모드 gate는 모두 `passed`, 구조 후보0이다. 실루엣100%를 색상·획 모양의 완전 일치로 주장하지 않는다. 엄격 내용 픽셀/visual proxy는 글자·선의 색상 및 raster 차이로 b-text6.40706%, control5.13544%이며 전체 pixel match99.53637%/99.62295%다. 표 선·문단 시작은 같은 좌표에 있고 획 두께·색상 차이는 남는다.

- TSV: `native-scores/{b-text-after-float,control-float}/silhouette.tsv`와 `wasm-scores/...`.
- 이번 최신 head 재출력의 영구 증적: [Native 재현 review](../assets/pr7630-review-20261008/b-text-after-float-p001-linux-native-review.png), [Native overlay](../assets/pr7630-review-20261008/b-text-after-float-p001-linux-native-overlay.png), [fresh WASM review](../assets/pr7630-review-20261008/b-text-after-float-p001-linux-wasm-review.png), [fresh WASM overlay](../assets/pr7630-review-20261008/b-text-after-float-p001-linux-wasm-overlay.png). 정상 대조군은 같은 asset 디렉터리의 `control-float-p001-linux-{native,wasm}-{review,overlay}.png`에 보존했다. Native/fresh WASM 제품 raster는 동일하며, backend별 표시 문구를 포함한 review/overlay는 각각 보존했다.
- 정상 대조군: `native-review/control-float/`, `wasm-review/control-float/`의 review/overlay도 직접 확인했다.
- 쪽 대응과 내용의 누락·중복: `structure-verification.json`에 기록했다. 글자 모양 차이와 배치 해결 여부를 구분했다.

## 본문 갱신 준비와 병합 후 기여자 코멘트 계획

기존 PNG 6개는 `mydocs/pr/assets/issue_7621_text_after_float_only_host/`에 있으며, 이전 candidate와 현재 head의 blob이 동일함을 확인했다. PR 본문이 이전 head `f3b26a6cf…`를 현재 head로 기재하고 있어, 원 생성 시점을 보존하면서 URL을 `sacru2red/rhwp/e4cbce28c…`로 갱신하는 초안을 `output/pr-review/pr7630-20261008/pr-body-update-draft-ko.md`에 준비했다. 기존 PNG를 새로 캡처한 증적으로 표시하지 않는다. 이번 재출력 결과는 별도로 구분해 연결한다. 생산 코드 변경은 없다.

작업지시자의 승인 후 본문을 갱신하고 API와 실제 PR 화면에서 본문·이미지를 확인한다. 병합 후에는 검토 문서 보관본·필요한 대표 asset·오늘할일 기록을 정식 후속 절차로 devel에 반영한다. 한국어 존댓말의 기여자 코멘트에는 다음 내용을 남긴다.

- 기여 코드가 해결한 #7621의 현상, 최신 head CI 집계 성공과 재사용한 Full CI, focused 검사 43건 통과.
- [Visual Sweep 정본](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment), 두 입력 각 1쪽의 Native/fresh WASM 최저 실루엣 일치율 100%, 해결한 배치와 남은 획·선 색상 차이.
- 대표 review/overlay를 `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/issue_7621_text_after_float_only_host/<file>.png`의 실제 Markdown 이미지로 표시한다. 이번 재출력 이미지를 후속 커밋에 저장한다면 해당 asset 커밋 SHA를 명시하고, 병합 시점에 존재했던 PNG와 구분한다.
- #7621의 종료 상태를 확인한다. 관련 #7606/#7470 등은 이번 제한된 수정만으로 자동 종료하지 않는다. `--body-file`로 게시하고 API로 UTF-8/BOM·본문 일치·SHA 고정 링크를 재확인한다.

병합 후에는 정식 후속 순서로 운영 기록·이슈·기여자 코멘트를 처리하고 전용 worktree·branch를 정리한다. 공유 `target/pr-review`와 기존 Studio는 보존한다.

## 병합과 후속 처리 확정값

- [원 PR #7630](https://github.com/edwardkim/rhwp/pull/7630)은 2026-10-08 07:03:14 UTC에 devel로 merge 완료했다. merge SHA `12c3c579e8444b6fd01b076ff46fbc11be0b2a07`, 최종 contributor head `e4cbce28cbdefcb9da56660302f8152666add069`.
- 작업지시자의 시각 통과와 명시적인 병합 승인에 따라 GitHub approve·admin merge를 수행했다. source/test 보정이나 contributor branch push는 없다.
- 후속 문서 처리 결정: **maintainer 직접 반영**. 이 archive·최신 head 재출력 PNG 8개·오늘할일만 한 운영 기록 commit으로 반영한다. 원 기여자의 PNG 6개는 코드 merge에 그대로 포함되어 있고 생성 시점을 구분한다.
- 기본 devel은 원 코드 merge로 fast-forward했다. 검증 CI·CodeQL·Render Diff·Adapter·Proptest를 병합 후 재실행하지 않았다. duration·이슈 자동 종료 결과는 아래에 기록한다.
- 이번 review 전용 clean worktree `/tmp/rhwp-pr7630-review-20261008`와 local branch `review/pr7630-20261008`는 운영 기록·기여자 안내를 완료한 뒤 제거한다. 이 PR만의 ignored 로그·중간 산출물은 영구 요약과 위 PNG 보존 후 정리한다. 기본 작업공간·공유 target·다른 검토·실행 중인 Studio와 contributor fork branch는 보존한다.
- [duration run 37741100603](https://github.com/edwardkim/rhwp/actions/runs/37741100603)은 성공했다. 수집 결과 `ready:false`, `reason:no-verified-pr-duration-measurements`로 데이터 갱신은 보류됐다. 로그는 최신 CI candidate `37637680059`의 worker b 실측을 수용하지 못했다고 기록했다. trusted 재사용으로 heavy worker가 skipped된 CI 성공과 실제 실행 시간의 가용성을 구분하며, 이 사유로 CI를 재실행하지 않았다.
- [#7621](https://github.com/edwardkim/rhwp/issues/7621)은 2026-10-08 07:03:30 UTC에 자동 종료됐다. 종료 workflow도 성공했다. 운영 기록 반영 후 동일 merge·증적의 기존 코멘트가 없음을 확인하고 issue/PR에 한국어 안내를 게시한다.

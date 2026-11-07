# PR #8711 검토 — 선행 정의 슬롯과 HWP 저장 재열기

## 판정

**수용·MERGED — 새 head CI와 self-review 완료.** 작업지시자가 fast-forward push·본문 보충·새 head CI·self-review·merge·#8226 대체 수용·감사 댓글·mydocs 운영 기록 반영·동기화·종료 정리를 승인했다. 검증 head `cbe5a6f4dfbc2775913af10ee42098137b27e088`를 [승인 리뷰](https://github.com/edwardkim/rhwp/pull/8711#pullrequestreview-5809579616) 후 merge commit `e724c6caff896a9478dc537cab4e99a57cf879cf`로 병합했다(2026-11-07 11:45:33 KST). 원 기여·협업자 commit을 보존했다. 로컬 검증과 원격 검증의 head·실행 범위는 아래와 [원격 수용 기록](../assets/pr8711/remote/completion.md)에서 구분한다.

## 접수·출처·검증 head

| 구분 | 값 |
| --- | --- |
| 대상 | [PR #8711](https://github.com/edwardkim/rhwp/pull/8711), postmelee(COLLABORATOR), 원 기여 semanticist21의 [#8226](https://github.com/edwardkim/rhwp/pull/8226)을 대체하는 통합 PR |
| 원 source head | `57f897a3bd844aadd883d7afb31def4904acb8ca`, upstream `review/pr-8226-save-recheck` |
| 제출 당시 base | `a7feb849e11642b1438b7e9ee460ff4c8a7c946e` |
| 최신 검토 base | `13775409e57a9550cfb31866d40bdf319ab54bac` |
| clean merge 제품 검증 head | `a657ac0936e324f9648ab628862062b5b9fddbd8` |
| 최초 주석 보강·전체 기능 검증 | `4e5a8d1565543daa999c776668d491fcc0beade5` |
| 측정 기록 정정·최종 lint/focused | `929f0f5ca537fe83b9947131d79b05e3f108c512` |
| reviewer | edwardkim 지정 및 requested_reviewers API 확인 |
| 경로 | maintainer_general + intake_and_review, local_validation, visual_fixture_evidence, multi_pr_update_branch, rework_and_exceptions(대형 증거 묶음), post_merge |
| 검토 공간 | `review/pr-8711-leading-definition-slots`, `/home/edward/mygithub/rhwp-worktrees/pr-8711-leading-definition-slots` |

원 #8226의 코드 `d2a7376aa…`와 증거 `b97b418a…`는 각각 `d77cb886…`·`064c51cb…`로 체리픽 출처를 보존했다. postmelee의 Storage 호환 `03300db4…`, 본문 줄 계획 보정 `24bb81d0…`, 관계 회귀 `effcb0d6…`, 증거 head `57f897a3…`도 보존한다. 상세 대응은 [기존 협업자 검토](pr_8226_review.md)이며 그 실행을 메인테이너 실행으로 소급 표시하지 않는다.

## 원인과 공통 소비 경로

구역·단 정의의 보강이 머리말·누름틀 등이 이미 점유한 선행 8-unit 슬롯을 예약 공간으로 오인했다. 또한 기존 저장 정책이 무효한 LineSeg를 제거한 뒤 머리말 정의 때문에 표·글줄 계획을 거부해 본문 글자 상자가 잘렸다. 잘못된 캐시를 유지하거나 paint에서 숨기는 방식으로 고치지 않았다.

| 값/계약 | 생산·측정 → 실제 소비 | 검토 |
| --- | --- | --- |
| 제어 슬롯·필드 소유 | `model/paragraph.rs:1203`의 `insert_leading_definition` → `converters/hwpx_to_hwp.rs:1508`·`serializer/body_text.rs:145` | 기존 gap, 필드 끝·제목 마커, UTF-16 위치와 field control index·병렬 records를 함께 보존한다. Storage reserve 뒤 논리 상태를 바꾸며 writer는 복제 문단만 수정한다. |
| 본문 밖 정의와 줄 계획 | `inline_flow.rs:235`·`inline_flow/table_rows.rs:45` → `typeset/inline_flow/plan.rs:123` | 머리말·꼬리말은 본문 밖 프레임의 정의다. 같은 문단의 표·텍스트 재조판을 막지 않고 기존 공통 plan을 사용한다. |
| fit·실제 배치 | `typeset/inline_flow.rs:108`의 `occupied_end()`·plan commit → `layout/paragraph_layout.rs:3086`, `layout.rs`의 `inline_flow_plans`·`plan.end` | 측정과 paint가 같은 rows·위치·높이를 소비한다. backend별 좌표 보정이나 문서 ID/화면 수치에 따른 분기를 추가하지 않았다. |
| 분할·이어받기 | 기존 `typeset/controls/paragraph_flow.rs`의 단독 분할 owner·예산 실패 경로 유지 | diff는 컷·소유 유닛·누적 예약·종료를 바꾸지 않는다. 원 1쪽은 전체 fit이며 새 분할 경계가 없다. 최근 그림 편집과 편람을 정상 대조군으로 확인했다. |

새 회귀의 실제 문서 배치는 내용·문단/개체 소유·앞뒤 순서·포함·누락/중복으로 검사한다. 고정형은 독립 Print의 전체 쪽수 1이며 절대 px 좌표를 추가하지 않았다. 원본 경로의 기존 검사는 이 파일뿐임을 확인했다. 나머지 세 검사는 합성 스트림 계약이며 다른 문서의 시각 점수를 그 계약의 실측으로 쓰지 않는다.

## 메인테이너 직접 검증

[입력 blob·producer·실수신 WASM·글꼴 원장과 재현 코드](../assets/pr8711/README.md), [전쪽 TSV](../assets/pr8711/tsv/)에 실제 실행을 보존했다. 기준은 같은 공개 HWPX와 독립 Mac 한컴 Print 1쪽, 2022 호환·Print profile·96dpi다. 로컬 Windows HANBatang 프로그램을 양쪽에 명시했다. 협업자가 쓴 다른 프로그램과 구분하고 글꼴 프로그램·진단 SVG는 커밋하지 않았다.

| 검증 | 최신 base | 후보 |
| --- | --- | --- |
| 원본 Native / fresh WASM | 100.00% / 100.00% | 100.00% / 100.00% |
| HWP 저장 재열기 Native / fresh WASM | 28.46% / 28.94% | 100.00% / 100.00% |
| 두 번 저장 후 WASM | 별도 시각 측정 없음 | 1쪽, 100.00%, 내용·순서 보존 |
| 저장본 실제 Native / WASM SVG | 별도 시각 측정 없음 | 각각 1쪽·100.00% |

위 수치는 색상 경계 보정 전 2px 이웃 raw 실루엣이며 전쪽 수는 모두 독립 PDF와 같다. 전체 분석은 Native/WASM 각각 flagged 0/1·gate passed다. Native pixel match는 99.66377%, `visual_accuracy_proxy_percent`는 29.04142%이며 WASM은 각각 99.62015%·21.63350%다. [상세 지표](../assets/pr8711/repro-visual-metrics.json)의 낮은 자동 잉크 보조값도 생략하지 않고 [같은 영역 확대](../assets/pr8711/figures/content-zoom.png)를 직접 판독했다. Mac 기준과 실제 공급한 Windows 프로그램·래스터 차이의 glyph 농도/가장자리 차이는 남지만, 머리말·표 셀·표 앞 문장 → 표 → 뒤 문장의 의미·순서·배치와 잘림 해소를 확인했다. 해당 영역에는 수식·첨자가 없으며 raw 100%를 픽셀 완전 일치나 사람의 정확도로 표시하지 않는다.

브라우저의 최초 WASM HTTP 응답 `f5bf3679…`가 고정 최적화 package와 같음을 확인했다. `exportHwp()`·`exportHwpWithReport()`·실제 글꼴 프로그램 준비 후 저장은 Native CLI와 같은 `729ed729…` HWP를 만들며 content-loss count 0이다. 반복 저장의 바이트 동일성을 요구하지 않고 실제 재열기 내용·쪽수·출력을 확인했다. base는 세 경로가 모두 같은 잘못된 `cb1fec65…` 파일을 만들었다.

CLI `--verify-pages`는 후보 1쪽으로 통과했다. 엄격한 `--verify`는 base·후보 모두 `section[0] paragraph[0] linesegs: count: expected=1 actual=0` 한 건으로 exit 3이다. 기존 무효 저장 줄 캐시 제거의 같은 진단을 PASS로 세지 않았다. 실제 의미와 배치는 위 재열기·독립 Print·회귀로 구분했다. 이 진단을 없애려고 잘못된 저장 LineSeg를 유지하거나 검사 기대값을 바꾸지 않는다.

## 최신 base 기능 보존

최근 #8566·#8595·#8632 입력은 각각 4·5·4쪽이며 같은 입력·독립 Print를 기존 원장에서 재사용했다. Print의 HCRBatang과 [기존 검증 전용 매핑](../assets/kang-t-20261106/print-reference-font-environment.json)을 적용해 Native/fresh WASM을 새로 산출했다. 기본 fallback 진단도 base/head 차이가 없었지만 최종 제출값은 올바른 Print 매핑 조건이다.

- Native 전 13쪽 최저 98.63%, fresh WASM 98.53%. backend별 base/후보 PNG [26쌍이 바이트 단위로 같다](../assets/pr8711/latest-feature-preservation.json).
- 제목·칸 정렬, 그림 옆 글줄, 34→35번 행 이어받기, 3쪽 그림과 9/10번 글줄·셀 외곽을 같은 영역/배율로 직접 대조했다. 내용 누락·중복·겹침 악화가 보이지 않는다.
- 실제 브라우저 셀 글자 삽입→삭제→HWP 저장·재열기에서 본문 4,442자와 목표 600/1200 HWPUNIT 및 정렬 2의 111개 기준선을 보존했다. base/후보 저장 바이트도 같다. [실행·Native 재파싱 관측](../assets/pr8711/cell-edit-preservation.json).
- 편람 383쪽 [정식 Native gate](../assets/pr8711/handbook-gate.json)는 최저 99.96%·기준 대비 후퇴 0쪽으로 통과했다.

## 로컬 게이트와 주석 보강

[전체 명령·head·결과](../assets/pr8711/rust-validation-summary.json), [실제 환경](../assets/pr8711/environment.json), [최소 글꼴 목록](../assets/pr8711/minimal-font-inventory.txt)을 따른다. 공용 `target/pr-review`를 순차 사용했으며 회귀에서는 사용자/상용 글꼴 경로를 숨기고 라이선스가 확인된 5개 fallback만 노출했다. 독립 Print 글꼴을 명시한 시각 실행과 별개다.

| 게이트 | 결과 |
| --- | --- |
| focused | 76건 통과: 선행 슬롯·필드 소유·Storage 예산·최근 그림 편집 |
| fmt·Native/WASM Clippy·workspace build/all-target Clippy | 통과 |
| suite·unit-tier 정책 | 통과, 파생 suite는 stage하지 않음 |
| release-test 전체 nextest | 12,782 passed / 0 failed / 기존 지정 제외 45건 |
| Native 공식 lib | 4,040 passed / 0 failed / 지정 제외 12건 |
| Native 그림 누락 표시 / 직접 PDF 출력 | 2 / 22 passed, 공식 합계 4,064 passed |
| 최종 기록 정정 후 | 모든 Rust lint·정책 및 해당 focused 4건 재통과 |
| fresh WASM·브라우저·시각·편람 | 위 실제 결과로 통과 |

신규 회귀 파일 첫머리의 유형·시각·실측·증거 경로를 `4e5a8d1565`에서 보강했다. 원자료를 대조하다 잘못 옮긴 환경/래스터라이저 표기를 발견해 `929f0f5ca5`에서 추정 플랫폼을 제거하고 실제 `pypdfium2`로 정정했다. 제품과 테스트 본문·assertion·기대값·fixture는 같다. [보강 patch](../assets/pr8711/test-provenance-header.patch), [정정 전후 본문 해시·제품 source 동등성](../assets/pr8711/metadata-correction-equivalence.json)을 보존한다.

전체 nextest와 Native 공식 실행의 정확한 head는 `4e5a8d1565`다. 최종 `929f0f5ca5`에서는 위 동등성을 입증하고 Rust lint와 영향 focused를 재실행했으며, 이전 실행을 새 SHA에서 실행했다고 표시하지 않는다. 초기에는 녹색 source head의 Full CI 재사용을 계획했지만 Rust test 주석 보강 후에는 그 GitHub 예외를 적용하지 않았다. 새 head의 GitHub required check는 push 뒤 아래 원격 최종 검증에서 별도로 확인했다.

`svg_snapshot`은 전체 nextest에 포함돼 통과했으며 golden/기대값을 바꾸지 않았다. Studio UI·공개 API/type·npm/package·CI 정의는 변경하지 않아 해당 TypeScript/npm/CI 운영 보정 검사는 비해당이다. 실제 Rust API의 브라우저 저장과 Canvas·SVG 소비 경로는 위와 같이 실행했다.

## 원격 최종 검증과 병합

원 source head의 성공은 [기존 원장](../assets/pr8711/source-head-ci.json)에 구분해 보존한다. 새 제출 head `cbe5a6f4dfbc2775913af10ee42098137b27e088`의 [CI](https://github.com/edwardkim/rhwp/actions/runs/42327643464)·[CodeQL](https://github.com/edwardkim/rhwp/actions/runs/42327643465)·[Render Diff](https://github.com/edwardkim/rhwp/actions/runs/42327643448)·[Adapter](https://github.com/edwardkim/rhwp/actions/runs/42327643447)·[Proptest](https://github.com/edwardkim/rhwp/actions/runs/42327643511)와 [최종 정책 감사](https://github.com/edwardkim/rhwp/actions/runs/42330675981)가 모두 success다. Archive 실제 합계는 **12,588 passed·기존 지정 제외 45건**이며 CI의 선택 Native 검사는 192건이다. 로컬 전체 12,782·Native 공식 4,064와 실행 범위를 혼동하지 않는다. [작업별 실제 로그 요약](../assets/pr8711/remote/ci-test-results.json), [exact-head check](../assets/pr8711/remote/exact-head-checks.json)를 보존했다.

병합 직전 fetch한 base는 `13775409e57a9550cfb31866d40bdf319ab54bac`, clean merge-tree는 `adf85e7bed4899fdb69033b90e8da977ba3ba5e5`다. 동일 head의 MERGEABLE/CLEAN과 전체 check 완료를 다시 확인하고 `--match-head-commit`으로 merge commit 병합했다. [선행 증명](../assets/pr8711/remote/premerge-proof.json), [실제 merge 결과](../assets/pr8711/remote/merge-result.json), [공개 self-review](https://github.com/edwardkim/rhwp/pull/8711#pullrequestreview-5809579616)를 따른다. 수동 CI 재실행·취소·보안 경고 재분류·workflow 변경은 0건이다. 원격 본문 이미지 4개의 HTTP·해시와 실제 GitHub 표시도 확인했다.

후속 기록은 메인테이너 직접 반영 경로다. source·test·workflow·golden은 포함하지 않는다. 운영 기록 push와 devel 동기화 뒤 #8226 대체 수용 댓글·close와 #8711 감사 댓글을 진행하며, 기존 #4680·#4687의 CLOSED 상태는 보존한다. 최종 댓글·정리 상태는 [운영 기록](../assets/pr8711/remote/completion.md)에 갱신한다.

## Merge 후 contributor PR comment 계획

[Visual Sweep 정본](../../manual/verification/visual_sweep_guide.md#github-merge-comment)을 따른다. #8711의 실제 merge SHA와 devel의 asset 존재를 확인한 뒤 해당 SHA로 고정한 원 문서 저장본 1쪽의 review·overlay를 Native/fresh WASM 각각 첫 댓글부터 표시한다. HTTP 응답·형식·해시와 게시 뒤 실제 표시를 확인한다.

- 각 1쪽·flagged 0/1·raw 100.00%, Native pixel match 99.66377% / 보조값 29.04142%, WASM 99.62015% / 21.63350% 및 위 직접 판독 결론을 함께 적는다. 자동 보조값을 사람의 정확도로 표시하지 않는다.
- 대표 asset: `mydocs/pr/assets/pr8711/figures/candidate-native-saved-review.png`, `candidate-native-saved-overlay.png`, `candidate-wasm-saved-review.png`, `candidate-wasm-saved-overlay.png`. 다른 문서·임시 output을 대신 쓰지 않는다.
- #8711에는 postmelee의 최신 base·Storage·본문 줄 계획 통합/검증과 semanticist21의 원 기여에 감사한다. 메인테이너 보강이 주석임을 설명한다.
- #8226에는 위 체리픽 대응을 설명하고 실제 통합 뒤 승인된 대체 수용 완료 댓글·close를 진행한다. 자동 MERGED를 가정하지 않는다. #4680·#4687은 현재 CLOSED이며 상태를 바꾸거나 이 PR로 전체 해결을 새로 주장하지 않는다.
- archive·오늘할일·운영 증거를 반영하고 devel을 동기화한 뒤 게시한다. 협업자가 만든 기존 upstream head와 원 contributor fork는 보존한다. 이번 local review branch/worktree/fetch ref와 종료된 루트 `output/` 전체·공용 debug만 정본대로 정리한다. 기본 작업공간·사용자 Vite·외부 `pr-8296-seal-pua`는 보존한다.

## 후속 처리 완료

검토 기록과 운영 증거를 mydocs 전용 commit `5251501a63`으로 devel에 반영·동기화한 뒤 #8226 대체 수용 댓글·close와 #8711 감사 댓글을 완료했다. 기존 이슈 #4680·#4687의 CLOSED 상태와 기존 협업자 upstream head·원 fork를 보존했다. 공개 승인 리뷰·댓글의 대응 이미지 총 12개를 각 게시물 안에서 확인했다. 소유 worktree·local branch·fetch ref와 루트 output 전체·공용 debug도 실제 사용 종료 뒤 제거했다. 약 21.50 GiB를 확보했다. [최종 상태·댓글·정리 영수증](../assets/pr8711/remote/final-receipt.json)을 따른다.

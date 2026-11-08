# PR #8696 검토 — 목록형 원본 글꼴의 관측 오류와 정상 TTF 대체

## 최종 판정

**승인 — #8687 해결 범위의 로컬 검증과 정확한 code head CI를 완료했다.**
검토한 code head는 `46790458ceaad2abd5c86168161798cb01f0473e`다.
2026-11-06 22:24 KST 조회에서 성공 34개·정상 생략 5개, `MERGEABLE / CLEAN`이었다.
이 문서를 담는 trailing head의 CI·정책 판정과 병합 직전 SHA 확인은 다음 단계다.
작업지시자는 `그냥 두고 cI 완료되면 후속처리.`, 이어서 `CI 완료됨 후속처리`를 지시했다.
100% 추가 조판 보정은 중단하고 현재 코드·검증 자료로 후속 절차를 진행한다.

## 접수와 경로

- [PR #8696](https://github.com/edwardkim/rhwp/pull/8696), 작성자 `jangster77`, base `devel`, 관련 [이슈 #8687](https://github.com/edwardkim/rhwp/issues/8687).
- base route: `collaborator_self_merge.md`.
- modifiers: `intake_and_review.md`, `local_validation.md`, `visual_fixture_evidence.md`, `review_only_fast_pass.md`, `post_merge.md`.
- loaded documents: `pr_review_workflow.md`, `pr_review/README.md`와 위 기본·보조 문서.
- reviewer 요청은 없으며 소스·테스트·workflow를 trailing 기록에 추가하지 않는다.
- base `45684640be5eddcf5972b063260250345b34d928`, 제품 source `87c073fa7094603f6915c43851da7ed66697565b`.

## 변경과 동작 검토

잘못된 원본/대체 이름이 관측 치환 목록에 들어가 전체 `FontEnvironment`를 거부하는 문제를
공통 `valid_name`으로 막았다. 원본 TTF 레코드의 미해소된 쉼표 이름은 내부 CSS 체인으로
분해하지 않고, 같은 레코드에 선언된 유효한 비내장 TTF 대체를 공통 style 결과로 전달한다.
정상 단일 face·이미 해소된 별칭·HFT·내장 프로그램과 명시적 환경 우선은 보존한다.

선행 원 해결 PR #8565와 실제 통합 `5551091085418c17d8321c0fee2626365911481b`,
현 base의 같은 요청 생산자를 대조했다. 별도 원인의 #8692를 해결했다고 주장하지 않는다.
[계획과 수정 전 FAIL](../../plans/issue_8687_font_fallback_names.md),
[구현·검증 보고서](../../report/task_m100_8687_report.md)에 실제 경로와 RED/PASS를 연결했다.

공통 조판 원칙 검토는 다음과 같다.

| 항목 | 판정과 근거 |
| --- | --- |
| 원본 계약·독립 기준 | 충족. FontFace의 단일 레코드와 구조화된 TTF 대체를 구분하며, 한컴 2024 Print에서 HCRDotum을 확인했다. 원본 저장 정보·환경 validator 허용치는 바꾸지 않았다. |
| 공통 생산 결과 | 충족. `try_lookup_font_name_decision → normalized_face/css_family_chain → char style·메트릭 → TextStyle → Native/Canvas/SVG`가 하나의 선택을 소비한다. 문서 ID나 backend별 수치 보정은 없다. |
| 좌표·높이·분할·이어받기 | 비해당. 해당 계산·소유·예약 높이·pagination·LineSeg는 변경하지 않았다. BOX의 기존 차이는 유지한다. |
| 회귀·대조군 | 충족. 설치 글꼴 없이 API 6건으로 정상/잘못된 이름 공존, trim, 환경·원본 보존, 실제 표시 체인과 정상 원본 우선을 검사한다. 수정 전 실패·수정 후 통과와 정상 control 보존을 확인했다. |
| px 기대값·기존 검사 변경 | 비해당. 새 테스트는 합성 FontFace의 관계형 API 계약이다. 실물 위치·쪽수·golden 핀이나 기대값 완화는 추가하지 않았다. |
| 실제 출력·직접 판독 | 충족. Native/fresh WASM, SVG·직접 PDF·Studio Canvas2D/CanvasKit을 독립 Print와 대조했다. 글자 모양·장평·기준선·자간·세 줄 간격을 같은 배율로 확인했다. 입력에는 수식·첨자가 없다. |

## 로컬 검증과 승인된 rebase 재사용

실제 실행 source는 `e4aefbcce7bf3db783ec39f3223e79b53082fbc1`, 초기 base는
`08963f6825956984d38b0d9392f44777db7107ab`이다. 최소 글꼴 격리에서 전체 nextest
12,758 PASS·0 FAIL, Native Skia lib 4,040 PASS·placeholder 2 PASS·직접 PDF 22 PASS,
집중 검사·fmt·Native/WASM/workspace all-target Clippy·workspace build·manifest를 완료했다.
실제 Studio 글꼴 E2E 11건도 통과했다. [명령·결과](../assets/issue_8687/validation-results.json).

최신 base로 rebase할 때 충돌 0건·두 patch의 range-diff `=`·제품 글꼴 파일과 테스트 blob 동일을
확인했다. 작업지시자의 `rebase, 충돌없으면 현재 테스트 자료 그대로 사용해도됨.`에 따라
자료를 재사용했다. 들어온 PR #8694의 표·개체 점유 변경을 새 source로 다시 출력한 것으로
표시하지 않는다. [승인 범위와 파일 대조](../assets/issue_8687/rebase-reuse.json).
수정 전 별도 WASM 빌드는 중단했으며 기준 WASM TSV는 없다. 이 한계를 숨기거나 새 측정으로
대체하지 않고 승인된 Native 기준/후보와 후보 Native/WASM의 독립 Print 측정을 사용한다.

## 검증 입력과 시각 증적

**검증 입력 커밋 확인: 충족.** [입력 목록·SHA-256·독립 Print job](../assets/issue_8687/input-provenance.json)의
control/repro/used HWPX와 대응 PDF는 code head `46790458ce`의 실제 Git blob과 실행 파일이 같다.
편람 HWPX와 4,692,840바이트 PDF도 같은 commit의 기존 경로를 재사용했다.
50 MiB 이상 PDF는 없으며 [93개 증적 해시](../assets/issue_8687/sha256sums.txt)를 확인했다.

| 문서 | 전체 쪽수 | Native raw 최저 | fresh WASM raw 최저 |
| --- | ---: | ---: | ---: |
| repro | 1 | 91.21255% | 91.09617% |
| used | 1 | 91.21255% | 91.09617% |
| 정상 control | 1 | 91.10204% | 91.12426% |

96 DPI·Print·compat 2024·raw 2px 실루엣이며 마스크는 없다. 90% 미만·누락·쪽수 불일치는 0이다.
[전쪽 TSV와 review/overlay/compare](../assets/issue_8687/README.md),
[본문 확대](../assets/issue_8687/used-font-backends-body-3x.png),
[내용·영역 소유와 순서](../assets/issue_8687/content-relations.json)를 직접 확인했다.
used의 guard-only Native 88.76426%가 최종 91.21255%로 개선됐고 정상 control Native PNG는
기준과 byte-identical이다. 전체 차이는 BOX의 기존 높이·기준선에 남는다.
사용자가 현재 상태 유지와 CI 이후 후속 처리를 지시했으므로 100% 완료라고 쓰지 않는다.

편람은 최소 글꼴·자동 웹폰트 공급에서 기준/후보 383쪽 전부 PNG가 byte-identical이며,
양쪽 strict gate가 통과했다. 최저 raw 99.96465%인 130쪽은 작업지시자의 명시 PASS다.
[편람 전쪽 결과](../assets/issue_8687/handbook/results.json). `정부상징 부처명_16040911.ttf`는
현재 호스트·ubuntu-ted·ubuntu-ted2에서 제거하고 캐시 갱신·부재를 확인했다.

## 정확한 code head GitHub CI

[원본 metadata·check·Archive별 수량](../assets/issue_8687/github-code-ci.json)을 보존했다.
로컬 전체 수량과 CI의 실행 묶음을 구분한다.

| workflow | 실제 결과 |
| --- | --- |
| [CI](https://github.com/edwardkim/rhwp/actions/runs/42221320953) | success. Archive A/B/C/D/E 각각 3,801/2,192/2,308/2,074/2,197 PASS, 합계 12,572 PASS·기존 45 SKIP. Lint·Native·frontend package도 success. |
| [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/42221320734) | success. 실제 Canvas 비교와 Native/CanvasKit provenance·backend·bundled-font guard 통과. |
| [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/42221320917) | success. JS/TS·Python·Rust·Actions 분석과 GHAS CodeQL check 성공. |
| [Proptest](https://github.com/edwardkim/rhwp/actions/runs/42221320918) / [Adapter](https://github.com/edwardkim/rhwp/actions/runs/42221320961) | 모두 success. |
| [최종 정책 감사](https://github.com/edwardkim/rhwp/actions/runs/42225048884) | exact code head의 `CI Impact Policy=success`, Full 실행·base SHA 일치. |

## 후속 실행과 Merge 후 PR·이슈 comment 계획

이 문서·오늘할일·CI 근거만 trailing commit으로 반영한다. push 전 최신 base/head의
merge-tree·공백·실제 merge tree의 Markdown 링크·기존 오늘할일 보존을 확인한다.
trailing head의 required CI·정책·mergeability와 SHA를 다시 확인한 뒤 후속 병합을 진행한다.

병합 후에는 merge SHA·duration 갱신 결과·devel 포함을 확인한다. 새 검증 CI를 시작하지 않는다.
이슈 #8687의 close 여부를 확인하고 같은 merge·증적의 댓글 중복을 검사한다.
원 PR과 이슈에 실제 CI·로컬 결과, 원본 HWPX/PDF, 잔여 BOX 차이와 130쪽 수용을 연결한다.
[Visual Sweep 정본](../../manual/verification/visual_sweep_guide.md#github-merge-comment)을 링크하고,
`https://raw.githubusercontent.com/edwardkim/rhwp/<merge-sha>/mydocs/pr/assets/issue_8687/{native,wasm}/{used,repro,control}-{review,overlay}-p001.png`
중 주장에 대응하는 이미지를 사용한다. raw 값과 보조 proxy를 구분한다.
merge SHA의 응답·PNG·해시 확인 후 UTF-8 `--body-file`로 게시하고 API 본문·이미지 표시를 재확인한다.
마지막으로 이번 소유 branch만 정리하고 루트 output 전체·공유 검토 debug를 종료 규칙에 따라 삭제한다.
단순 self PR이며 별도 보정·체리픽이나 선택 분기가 없어 review_impl은 이 실행 순서로 대신한다.

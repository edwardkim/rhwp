# PR #8658 검토 — 표 칸 그림의 겹침 방지와 분할 행 소유

## 최종 판정

**승인.** 검증 head `d440ddc370026ba89ac79a6f6bc2385b24195973`의 실제 GitHub checks 38개와 CI Impact Policy success, 작성자 self-review 및 최신 base merge-tree를 확인해 [수용 PR #8694](https://github.com/edwardkim/rhwp/pull/8694)를 merge `67adf83672eedd0672a4a1b0cb81db7ef7e85574`로 병합했다. 원 [#8658](https://github.com/edwardkim/rhwp/pull/8658)도 간접 MERGED를 확인했다. [원격 검증·후속 기록](../assets/pr8658/remote-completion.md)을 따른다.

**로컬 수용 검증 완료.** 메인테이너 보정과 필수 검사를 모두 통과했다. 원 사례·정상 대조·선언 높이·남은 공간 경계 입력 8개 10쪽은 Native/fresh optimized WASM에서 모두 raw 100.00%다. 실제 브라우저 SVG는 전 10쪽 최저 99.98%다. 최신 devel 기능 67쪽도 두 경로 각각 base와 PNG가 동일하다. 전체 입력 36개 77쪽의 최저 raw는 Native 91.16%·WASM 91.83%이며 90% 미달·누락이 없다. 기존 차이의 수용 예외는 필요하지 않다.

집중 회귀 8건·전체 회귀 12,760건·Native 공식 3종 4,064건과 모든 lint가 통과했다. [최종 검증 대장](../assets/pr8658/final/validation.md)에 실제 명령·출처·미실행과 제외한 진단을 구분했다. 초기 승인은 로컬 보정·검증에 한정됐다. 이후 작업지시자가 원격 처리와 merge·후속 처리를 별도로 승인해 수용 PR을 처리했다. CodeQL #500의 false positive 분류도 별도 명시 승인을 받았고 보안 check가 자동 success로 갱신됐다. workflow 재실행은 없었다.

## 접수·고정한 source와 승인

| 항목 | 확인 값 |
| --- | --- |
| 원 PR·기여자 | [#8658](https://github.com/edwardkim/rhwp/pull/8658), `hyunmyungJaneLee` |
| 연결 이슈 | [#8576](https://github.com/edwardkim/rhwp/issues/8576) |
| 원 fork·branch | `kidsnote/rhwp`, `task/8576-cell-square-no-overlap` |
| 원 head | `218c8856ad0a338fd1820c43e62b40dbcdcb86bb` |
| 고정한 최신 base | `08963f6825956984d38b0d9392f44777db7107ab` |
| 로컬 통합 | `759f35f2fbccc2682356dc22cb31b1068b57e178`, 원 기여 SHA 보존, 충돌 없음 |
| 메인테이너 제품 보정 | `19de2f3b54`, 공통 행 높이 수용 조건 한 곳 |
| 관계형 회귀·경계 입력 | `08cd5ab833` |
| 로컬 branch·worktree | `review/pr-8658-cell-square-no-overlap`, `/home/edward/mygithub/rhwp-worktrees/pr-8658-cell-square-no-overlap` |
| route | `maintainer_general` + `intake_and_review`, `local_validation`, `visual_fixture_evidence`, `post_merge` |

원 head의 CI 성공을 확인했지만 최신 base의 보정 후보 CI로 사용하지 않는다. 접수 당시 원 PR은 OPEN이고 head가 유지됐으며, 원격 devel도 고정 base와 같음을 다시 조회했다. reviewer `edwardkim` 배정을 확인했다. 앞선 기여의 수용 이력과 GitHub의 오래된 FIRST_TIME_CONTRIBUTOR 표시는 구분한다.

2026-11-06 17:41 KST에 [보정 계획](pr_8658_review_impl.md)의 **메인테이너 보정·필수 로컬 검증**을 한 단위로 승인받았다. 이후 원격 처리·merge·후속 처리의 명시 승인을 받았다. fork의 현재 API 권한은 `push=false`다. 이번 PR에서 fork push를 시도하지 않았으며, 원 기여를 보존하는 upstream 수용 PR 경로를 사용했다.

## 선행 해결·원인과 보정

원 문제는 같은 문단의 allowOverlap=0 Square 그림 6장이 겹치고, 분할 행에서 빈 줄을 소유하지 않은 그림 문단이 어느 조각에도 그려지지 않는 것이다. 최신 base의 실제 Native/최적화 WASM에서 모두 재현했다. 원 사례의 base raw는 Native 34.98–100%·WASM 34.99–100%다. 그림 배치·소유를 고친 원 기여를 그대로 보존했다.

기여 후보의 선언 높이 경계는 Native 83.83%·WASM Canvas 83.81%였고, 아래 괘선·END가 약 3.8px 위로 올라갔다. 그림 6장의 높이 합 42522HU와 칸 선언을 같게 하면 측정은 이미 실제 그림 566.96px + 안 여백 1.88px × 2 = 570.72px로 정확하다. 최종 배치의 선언 신뢰 함수가 **빈 문자열 그림 호스트를 빈 텍스트 칸으로 취급해** 측정 행을 선언 566.96px로 덮은 것이 원인이다.

메인테이너는 저장 줄 없는 그림·도형·표·수식·양식 호스트를 순수 ViewText 텍스트 격자의 증거로 사용하지 않도록 공통 수용 조건을 보정했다. 실제 저장 줄과 순수 텍스트 격자의 기존 계약은 유지한다. 특정 문서·그림 개수·3.8px 상수·출력 숨김·새 clamp를 추가하지 않았다. 실제 표 h=567.0→570.7px, END y=714.8→718.5px이며 그림 6개의 원점은 유지된다.

선행 #3386 → `81d208159`·`a60ac3974d`, #7160 통합 `505661360e9`, 후속 `08958748dec`를 추적했다. #8599 → [통합 #8683](https://github.com/edwardkim/rhwp/pull/8683)의 선행 공통 배치 계약도 확인했다. 자세한 생산→요구 높이→예약→최종 배치, 상대/물리 좌표 계약과 실제 컷은 [소비 경로](../assets/pr8658/correction-consumers.md)에 기록했다.

## 입력과 독립 Print

원 HWPX·CENTER·선언 높이·겹침 허용·가로 비충돌·남은 공간 대조는 한컴 2024 Print, 원 split HWP는 한컴 2020 Print를 사용했다. HWPX는 실제 마지막 저장 metadata에 맞춘 2024 조판 세대, 2020 출력은 2022 compat다. 원본들은 공개 재현물의 합성본이며 원본 생성 방식과 한컴의 독립 인쇄를 구분한다.

2024는 HOffice 13.0.0.3901, `pdf_print_method=0 / frame_print_to_pdf_ex_one_up`, 2020은 11.0.0.9136, `pdf_print_method=0 / hancom2020_pdf_driver_one_up`이다. `pdf/issue8576/*-2024.pdf` 및 `split-row-picture-only-para-2020.pdf`를 보존했다. 기존 웹한글기안기 Print도 삭제하지 않았다.

입력 36개와 대응 기준 PDF 36개, 총 72개 레코드의 실제 SHA-256·bytes와 Git blob 일치를 [입력 대장](../assets/pr8658/final/validated-inputs.tsv)에 기록했다. 원 글꼴은 검증용 파일을 명시적으로 공급했고 글꼴 프로그램·내장 글꼴 SVG를 커밋하지 않는다. 한컴 원본/기준 출력·공개 재현물과 비공개 코퍼스 집계를 혼동하지 않았다.

## 전쪽 시각 결과와 직접 판독

| 범위 | 전체 쪽수 | Native raw 최저 | fresh WASM raw 최저 | 확인 |
| --- | --- | --- | --- | --- |
| 작은 그림 / 큰 그림 | 각 1 | 100.00% | 100.00% | 6장의 소유·순서·비겹침, 칸 성장·END |
| split-row HWP | 2 | 100.00% | 100.00% | 1쪽 그림 2개, LINE 1–18 / 2쪽 LINE 19–40·END |
| CENTER | 1 | 100.00% | 100.00% | 묶음 전체의 위아래 여유·후속 본문 |
| 선언 높이와 그림 합이 같은 경계 | 1 | 100.00% | 100.00% | 두 안 여백·마지막 그림·아래 괘선·END |
| 겹침 허용 / 가로 비충돌 | 각 1 | 100.00% | 100.00% | 의도한 겹침 또는 나란한 원점 보존 |
| 선언 높이만 fit하는 남은 공간 | 2 | 100.00% | 100.00% | BEFORE는 1쪽, 그림 6개·칸·END는 2쪽 |
| 현재 devel 기능 대조 | 67 | 91.16% | 91.83% | 두 경로 각각 base/후보 PNG 67쌍 동일 |

전체 전쪽 TSV는 [Native](../assets/pr8658/final/corrected-native)·[WASM](../assets/pr8658/final/corrected-wasm), 별도 실제 SVG 경로는 [SVG](../assets/pr8658/final/corrected-svg/primary)에 있다. [기준/후보 전쪽 집계](../assets/pr8658/final/visual-summary.json)·[양쪽 base PNG 일치](../assets/pr8658/final/actual-base-preservation-comparison.json)를 함께 본다. base의 남은 공간 대조군은 잘못된 1쪽이므로 기준 2쪽과 쪽수도 다르며, 이를 후보의 성공으로 합산하지 않는다.

96dpi·색상 경계 보정 전 2px raw 실루엣이다. RGB overlay는 정본의 threshold 32를 사용했다. 원 사례·경계 전 10쪽의 실제 `analyze_page` 후보는 Native/WASM 각각 0/10이며, 전체 페이지 직접 비교 및 선언 높이의 괘선·END 확대 판독을 완료했다. 수식이 없는 재현물이며 분할 전후 글줄·그림·칸 바깥·뒤 본문·빈 쪽·누락·중복을 직접 확인했다. raw 100%를 RGB 픽셀 완전 일치나 사람 판정 정확도로 표현하지 않는다.

### 원 PR의 주장별 페이지·보조 지표

**Native**

| 입력·쪽 | 후보 | pixel match | visual_accuracy_proxy | raw |
| --- | --- | --- | --- | --- |
| small p1 | 0/1 | 99.68581% | 67.76831% | 100.00000% |
| large p1 | 0/1 | 99.66875% | 76.51600% | 100.00000% |
| split p1 | 0/1 | 98.26495% | 30.27022% | 100.00000% |
| split p2 | 0/1 | 99.03925% | 20.21068% | 100.00000% |

**WASM**

| 입력·쪽 | 후보 | pixel match | visual_accuracy_proxy | raw |
| --- | --- | --- | --- | --- |
| small p1 | 0/1 | 99.68435% | 67.63724% | 100.00000% |
| large p1 | 0/1 | 99.67088% | 76.63373% | 100.00000% |
| split p1 | 0/1 | 98.27662% | 31.29727% | 100.00000% |
| split p2 | 0/1 | 99.05384% | 22.98063% | 100.00000% |

## 회귀와 검증 출처

기존 테스트는 그림 포함·순서·겹침 관계가 중심이고, 이번에 선언 높이+안 여백·CENTER·겹침 허용·가로 비충돌·남은 공간 5건을 보강했다. split 검사는 ROW 1–6, LINE 1–40, END의 순서·누락·중복 및 마지막 조각 아래의 END도 검사한다. 첫머리에는 실제 Native/fresh WASM 전쪽 값·측정 시각과 직전 Git 파일 변경 SHA/시각을 기록했다. 새 px 좌표·SVG 해시 핀은 없다. 원본 속성에서 온 선언 높이와 독립 Print 쪽수는 내용 보존과 함께 유지한다.

선언 높이의 같은 관계 검사 `마지막 그림 바닥 + 원본 아래 안 여백 ≤ 칸 바닥 + 0.5px`는 수정 전 FAIL/수정 후 PASS다. [기하 음성/양성](../assets/pr8658/final/exact-fit-red-green-geometry.json)은 실제 두 render tree의 검사이며, 수정 전 라이브러리에서 새 Rust 파일을 실행했다고 주장하지 않는다. 새 Rust 정식 집중 검사는 최소 글꼴 환경에서 8건이 실제 통과했다.

[Native 편람 383쪽](../assets/pr8658/final/handbook-native.tsv)은 최저 99.96%·기준 하락 0쪽으로 통과했다. [편람 gate](../assets/pr8658/final/handbook-gate.json), [입력 8개의 보안 검사 24건](../assets/pr8658/final/new-input-security.json), [producer·package/public 출처](../assets/pr8658/final/producer-identities.json)를 보존한다. 전체 회귀·Native 공식 3종·Rust lint는 모두 통과했으며 [최종 검증](../assets/pr8658/final/validation.md)을 따른다.

공용 target에서 첫 기준 WASM이 이전 후보 산출물을 재사용한 것은 실제 기능·SHA 대조로 검출해 기준 증거에서 제외했다. source bytes를 바꾸지 않고 기준 패키지를 재컴파일했고, 원래 문제의 실제 base WASM raw 34.99%/60.50%와 선언 경계 45.58%를 재확인했다. 후보 빌드도 실제 Rust compile·wasm-opt·최상위 exit 0을 확인했다. Node가 격리 경로에 없었던 최초 집중 실행의 127은 테스트 시작 전 환경 오류이며, 실행 파일만 읽기 전용 노출한 뒤 실제 8 PASS로 재검증했다. 성공한 정책 검사는 재실행하지 않았다.

## Merge 후 contributor PR comment 계획

수용 PR #8694의 merge와 원 PR의 간접 MERGED를 확인했다. 원 fork/branch와 원 기여 SHA는 보존했다. 개정된 post-merge 순서로 이 archive·assets·오늘할일 운영 기록을 반영하고 최종 devel을 동기화한 뒤 아래 계획대로 댓글을 게시한다. 첫 운영 기록 반영·devel 동기화 뒤 [감사 댓글](https://github.com/edwardkim/rhwp/pull/8658#issuecomment-6684200684)을 게시했고 [API·실제 화면](../assets/pr8658/github-contributor-comment-display.json)의 16개 이미지를 확인했다.

첫 댓글부터 원 주장 3입력의 p1/p1/p1–2에 대응하는 실제 review·overlay를 넣는다. 아래 각 `corrected-wasm` 자산을 실제 asset을 포함한 merge SHA의 raw URL로 고정하며, Native 전쪽 TSV와 위 실제 보조 지표 표도 연결한다.

- small p1 review: [review](../assets/pr8658/final/corrected-wasm/primary/small/review/review_001.png)
- small p1 overlay: [overlay](../assets/pr8658/final/corrected-wasm/primary/small/overlay/overlay_001.png)
- large p1 review: [review](../assets/pr8658/final/corrected-wasm/primary/large/review/review_001.png)
- large p1 overlay: [overlay](../assets/pr8658/final/corrected-wasm/primary/large/overlay/overlay_001.png)
- split p1 review: [review](../assets/pr8658/final/corrected-wasm/primary/split/review/review_001.png)
- split p1 overlay: [overlay](../assets/pr8658/final/corrected-wasm/primary/split/overlay/overlay_001.png)
- split p2 review: [review](../assets/pr8658/final/corrected-wasm/primary/split/review/review_002.png)
- split p2 overlay: [overlay](../assets/pr8658/final/corrected-wasm/primary/split/overlay/overlay_002.png)

[Visual Sweep의 merge comment 정본](../../manual/verification/visual_sweep_guide.md#github-merge-comment)을 direct link로 남기고, 위 실제 후보 수·pixel match·visual_accuracy_proxy·raw를 구분한다. 수치는 자동 보조값이고 사람의 최종 판정 정확도가 아님을 밝힌다. 원 기여의 그림 쌓기·분할 복원 성과를 먼저 인정하고 메인테이너 선언 높이 보정의 이유를 존댓말로 설명한다. 게시 전 raw 이미지 형식·bytes·SHA-256, 게시 후 API 본문·이미지 표시를 확인한다. 원격 승인과 실제 merge SHA는 확정됐다. 운영 기록 반영·최종 devel 동기화 뒤 이 계획대로 게시하고 실제 화면까지 확인했다.


## 최종 GitHub 검증·승인된 보안 분류

- [작성자 self-review](https://github.com/edwardkim/rhwp/pull/8694#pullrequestreview-5799585117): 검토한 head의 공통 조판·분할·입력·실제 출력 소비와 기존 기능 보존을 확인했다. 새 구현 blocker는 없다.
- [정확한 head 검증](../assets/pr8658/github-final-validation.json): CI·CodeQL 4언어·Render Diff·Adapter·Proptest와 최종 정책 감사 모두 성공했다. 최초 실행 결과를 유지했다.
- [GitHub Archive 실제 로그 집계](../assets/pr8658/ci-archive-actual-results.json): 12,566 PASS·기존 45 skipped. 로컬 전체 12,760 PASS 및 Native 공식 4,064 PASS와 구분한다.
- CodeQL workflow와 별도의 보안 check를 구분했다. #500은 이미 존재하던 opt-in 숫자 좌표 로그다. [source/sink 도입·4개 flow·기존 3건 검토](../assets/pr8658/security-bot-review.json), [현재 SARIF 정규화](../assets/pr8658/alert500-normalized-flows.json)를 확인했다.
- 작업지시자의 별도 승인으로 [#500 분류](../assets/pr8658/alert500-classification-result.json)만 false positive로 변경했다. [동일 보안 check failure→success](../assets/pr8658/codeql-security-check-resolution.json)가 자동 반영됐으며 기존 #478·#483·#484는 OPEN 유지, CodeQL 재실행·탐지 설정 변경은 없었다.
- 후속 기록은 maintainer 직접 반영 옵션 M이다. source·test·workflow·golden·기존 sample을 바꾸지 않고 archive·assets·오늘할일만 반영한다. 이후 이미 자동 close된 #8576의 근거 댓글·원 PR 감사 댓글·작업 공간 정리를 순서대로 완료한다. #8579는 OPEN을 유지한다.

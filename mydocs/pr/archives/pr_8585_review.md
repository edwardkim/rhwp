# PR #8585 검토

## 최종 판정

**메인터너 보정 후 수용 가능.** 원 기여의 여러 줄 그림 배치를 보존하면서 말미 공백 때문에
마지막 사진 줄만 왼쪽으로 밀리던 차이를 공통 정렬 계약에서 보정했다. 후보
`c3e28d2c6e44baa9714832e1521f5a733101a5fd`의 필수 로컬 검증을 완료했다.
작업지시자가 통합 PR 경로와 CI·self-review·merge·후속 처리를 승인했다.
[통합 PR #8601](https://github.com/edwardkim/rhwp/pull/8601)의 같은 code head에서 Full CI와 최종 정책 감사가
모두 성공했다. 2026-11-05 KST self-review에서 구현·관계 회귀·독립 출력·실행 경로와 전체 검증 증적을
다시 대조했으며 머지 차단 사항은 없다. 이 문서 commit 이후의 최신 head 검사와 충돌 확인은 별도 최종 조건이다.
원 PR의 fork head는 원 기여 `7e7a86fef914fa021ad2ba511d88908c3f28f6ba`로 보존했다.

## 접수와 source 구분

| 항목 | 값 |
| --- | --- |
| PR·작성자 | [#8585](https://github.com/edwardkim/rhwp/pull/8585), `hyunmyungJaneLee`, `FIRST_TIME_CONTRIBUTOR` |
| 원 fork·branch | `kidsnote/rhwp`, `task/7524-textbox-tac-picture-rows` |
| 원 head | `7e7a86fef914fa021ad2ba511d88908c3f28f6ba` |
| 로컬 시각 검증 devel 고정값 | `7fdd5441ff1e77c083818e920a48089f294de35e` |
| 구현 보정 commit | `a55c53a393` |
| Full CI를 통과한 code head | `c3e28d2c6e44baa9714832e1521f5a733101a5fd` |
| 원 head merge simulation | clean, tree `df3c1e2b824f4a5a4502e92e6189ecfd1a6c5dd4` |
| 보정 후보 merge simulation | clean, tree `81ef14dcf5263bf3928d53968322dc995c50645b` |
| 원 PR reviewer·상태 | `edwardkim`, OPEN·non-draft·maintainer_can_modify=true; 실제 fork push=false |
| 전달 PR | [#8601](https://github.com/edwardkim/rhwp/pull/8601), upstream의 동일 검토 branch, Open·non-draft·self-review |
| 제출·CI base | `3a53945439862712be86764af73714827b15a44e`, clean merge tree `eaed8d42dd52a7ba2229d0b6fcc14b0b6fb6ca86` |
| 관련 이슈 | [#7524](https://github.com/edwardkim/rhwp/issues/7524), 원 PR의 `Closes` 대상 |
| 경로 | maintainer 일반 처리 + 첫 외부 기여자·로컬·시각 증적 절차 |
| 작업 위치 | 기본 작업공간의 `review/pr-8585-textbox-picture-rows` |

원 기여의 구현 `ecfd76527c`, 회귀·HWP `c312ea2c87`, 웹한글 Print PDF `d7fcabb85b`,
시각 자료 `7e7a86fef9`를 모두 보존했다. 원 기여 commit을 rebase·amend·force-push하지 않았고,
관련 없는 devel 이력을 fork branch에 병합하지 않았다. 로컬 검토 기록 `cea9ec9636`, 구현 보정
`a55c53a393`, 측정 주석·쪽수 원장·증적 `c3e28d2c6e`를 원 head 위에 분리했다.
PR API의 base 값 `1924d371...`와 실제 검증 base `7fdd5441...`를 혼동하지 않는다.

## 원 주장이 현재 devel에서도 유효한가

동일 공개 HWP를 최신 devel에서 열면 본문 글줄은 이미 세 줄인데 별도 그림 배치는 한 줄에 모인다.
사진 6개 중 4개의 bbox가 글상자 범위 밖으로 나가 클리핑된다. 따라서 핵심 문제는 아직 해결되지 않았고
이 기여를 수용할 이유가 있다. 과거 이슈의 45자 합성 줄바꿈 설명을 현재 원인 전체로 재인용하지 않는다.

선행 계약으로 #6465의 개체 소유 줄, #6651의 첫 줄 오프셋, #5820·#6173의 마지막 개체 뒤 공백 제외,
#8122의 저장 줄 없는 글상자 세로 정렬을 확인했다. #8122의 세로 정렬 보정은 이미 base에 있으며,
표 셀 안 그림의 별도 문제 #8576·#8579는 원 PR이 구분한 범위를 유지한다.

원 head는 사진을 2열·3줄로 복원했다. 다만 각 줄의 텍스트 폭에 말미 공백까지 포함해 그림의
가운데 정렬 원점을 별도로 계산하므로, 공백이 4개인 마지막 줄이 공백 1개인 앞 두 줄보다 10px 왼쪽으로 밀렸다.

| 줄 | 독립 PDF 왼쪽 사진 x | 원 head x | 보정 Native x |
| --- | ---: | ---: | ---: |
| 첫째 | 151.68 | 148.4 | 151.8 |
| 둘째 | 151.68 | 148.4 | 151.8 |
| 셋째 | 151.68 | 138.4 | 151.8 |

96 DPI의 PDF image object bounds와 실제 render tree 관측값이다. 이 좌표를 회귀 기대값으로 고정하지 않았다.

## 공통 규칙과 실제 소비 경로

`shape_layout::layout_textbox_content`의 composed line·control text position → 원 기여의
`tac_line_plan` 줄 소속 → 줄별 정렬 폭·시작 x/y → 컨트롤 순회에서 `inline_x`·`inline_obj_y` 선택 →
Image/Shape/Equation bbox → 실제 Native Skia·WASM Canvas·SVG 소비 경로를 확인했다.

메인터너 보정은 `paragraph_layout::center_alignment_excludes_trailing_space`로 본문의 기존
가운데 정렬 적용 조건을 공유한다. 그림 배치도 같은 소유 줄의 마지막 인라인 개체 위치를 넘겨
기존 `trailing_space_width_after_last_inline_object`를 소비한다. 마지막 개체 앞의 공백은 개체 사이
내용으로 남기고, 가운데 정렬의 밑줄 공백과 저장 줄/fresh 조판의 기존 조건도 유지한다.
본문의 기존 조건을 그대로 추출했으며, 문서 ID·공백 개수·관측 좌표를 이용한 보정 상수는 추가하지 않았다.

실제 SVG export의 image 속성에서도 세 줄의 대응 열 x가 같음을 확인했다.
[SVG 소비 결과](../assets/pr8585/corrected-svg-photo-attributes.json)는 별도 SVG 출력 관측값이며
Native/fresh WASM의 PNG 비교를 대신하지 않는다. 이번 변경은 pagination·rowspan·continuation의 컷이나
예약 높이를 바꾸지 않는다. 그림의 y와 높이·글상자 본문 점유·후속 문단은 그대로 유지됐다.

기존 관계 검사에 독립 Print의 2열·3줄 개수와 대응 열 정렬을 보강했다. 원 head의 실제 native library에
강화한 동일 검사를 연결했을 때 포함 검사는 PASS, 열 정렬 검사는 FAIL이었다. 보정 후 두 검사 모두 PASS다.
[수정 전 실패](../assets/pr8585/original-head-new-relations.log.txt),
[수정 후 통과](../assets/pr8585/corrected-test-issue_7524_textbox_tac_picture_rows.log.txt)를 보존했다.

## 입력·글꼴·실행 경로

| 구분 | 입력 | 독립 기준 PDF |
| --- | --- | --- |
| 원 사례 | `samples/issue7524/textbox_tac_picture_rows.hwp` | `pdf/issue7524/textbox_tac_picture_rows-webhwp-print.pdf` |
| 정상 대조군 #8122 | `samples/issue8122/textbox_valign_no_lineseg.hwp` | `pdf/issue8122/textbox_valign_no_lineseg-webhwp-print.pdf` |

네 입력은 모두 후보 commit에 포함되어 있고 각 HWP/PDF는 전 1쪽이다.
[경로·바이트 수·SHA-256](../assets/pr8585/input-manifest.json)을 확인했다.
원 사례 HWP의 SHA-256은 `ffc0a66e92b10ec2ea10167bca2a2721c68a4207aed4acf5b381ab5e7ca18d36`,
PDF는 `212f1c608c99a1a427aa5302046d8479afb0a16fecc19c2fcf62afb7212653a5`다.
PDF는 동일 공개 익명화 입력을 한컴 웹한글에서 Print한 작성자 제공 자료이며 Cairo 1.15.12, A4 전 1쪽이다.
rhwp가 만든 PDF를 기준으로 쓰지 않았다. 사내 4,428건 비교는 작성자 참고 주장으로 구분하고 독립 통과 근거로 쓰지 않는다.

비교는 print profile·compat 2022·96 DPI·PDFium·색상 경계 보정 전 2px 내용 실루엣으로 통일했다.
`gulim.ttc`, `HBATANG.TTF`, `hamchod-r.ttf` 세 로컬 프로그램을 base/후보와 Native/WASM에 동일하게 공급했다.
사용할 수 없는 `한컴 윤고딕 230`은 실제 공급한 `HCR Dotum`으로 명시적으로 대체했다.
[글꼴 환경](../assets/pr8585/font-environment.json)과 각 provenance에 프로그램 해시를 남겼고 폰트 파일은 추가하지 않았다.
전체 사유 글꼴 디렉터리 공급 때의 Native 글자 누락은 환경 진단으로 분리했으며 최종 수치에 섞지 않았다.
정식 글꼴 예외는 사용하지 않았다.

기준 Native·최적화 WASM은 고정 devel에서 직접 빌드해 별도 보관했고, 후보도 직접 빌드했다.
WASM은 `wasm-pack-locked.sh --target web`과 `wasm-opt`를 완료한 fresh 패키지를 Chromium 1660786에서
실제 Canvas로 출력했다. 기존 Studio 배포 패키지를 검증용으로 대체하지 않았다.

실행 코드 build source는 `a55c53a393`, 최종 검사 source는 `c3e28d2c6e`다. 그 사이는 측정 주석·
원장 행 위치·검토 증적만 달라졌고 runtime source 2,149개와 테스트 assertion·원장 값이 같음을 확인했다.
[실행 코드 동등성](../assets/pr8585/runtime-equivalence.json)에 실제 build/validation SHA를 구분했다.
정상 대조군을 실행한 작업공간의 Git context SHA와 보관 binary의 build SHA도
[대조군 결과](../assets/pr8585/control-8122-results.json)에 별도로 기록했다.

## 전쪽 수치와 직접 판독

| 문서·source | Native raw | fresh WASM raw | 쪽수 |
| --- | ---: | ---: | --- |
| #7524 고정 devel | 48.37363% | 48.34812% | 각각 1/1 |
| #7524 원 head | 96.84744% | 96.85057% | 각각 1/1 |
| #7524 보정 후보 | **99.95893%** | **99.96404%** | 각각 1/1 |
| #8122 devel | 99.55124% | 99.93233% | 각각 1/1 |
| #8122 보정 후보 | 99.55124% | 99.93233% | 각각 1/1 |

원 사례는 devel 대비 Native +51.58530%p, WASM +51.61592%p다. 대조군은 0.00%p이며
[실제 픽셀 비교](../assets/pr8585/control-8122-pixel-comparison.json)도 두 경로 모두 동일하다.
현재 검증한 두 문서의 전쪽 최저는 Native 99.55124%, WASM 99.93233%다. 90% 미만·누락·쪽수 불일치·
devel 대비 2%p 초과 하락은 없다. 평균값으로 미달 쪽을 상쇄하지 않았다.

[보정 Native TSV](../assets/pr8585/corrected-native.tsv),
[보정 WASM TSV](../assets/pr8585/corrected-wasm.tsv),
[Native PNG](../assets/pr8585/corrected-native-p001.png),
[WASM PNG](../assets/pr8585/corrected-wasm-p001.png)를 보존했다. base·원 head·대조군 전쪽 TSV와 PNG도 같은 폴더에 있다.

독립 PDF와 base·원 head·보정 Native/WASM을 같은 영역·배율로 확대해
[마지막 사진 줄](../assets/pr8585/zoom-last-photo-row.png),
[제목](../assets/pr8585/zoom-title.png), [후속 본문](../assets/pr8585/zoom-following-body.png)을 직접 판독했다.
대응 열의 정렬, 그림 간격·행 간격·테두리·앞뒤 본문·문자 보존을 확인했다.
정상 대조군도 [3배 확대](../assets/pr8585/zoom-normal-textbox.png)로 행 소속과 세로 여백을 대조했다.
입력에 수식은 없다. Native/PDF의 기존 래스터 경계·힌팅 차이는 보정 전후 같으며 새 배치 결함은 발견하지 않았다.
전체 원본은 각각 1쪽이므로 모든 쪽을 직접 판독했다. px 좌표 고정 assertion을 추가하지 않았다.

## 필수 로컬 검증

검사 source는 모두 `c3e28d2c6e44baa9714832e1521f5a733101a5fd`다.
[명령·소요 시간·exit](../assets/pr8585/local-gates-results.json)와
[완료 기록](../assets/pr8585/local-gates-complete.json)을 보존했다.

| 검사 | 실제 결과 |
| --- | --- |
| fmt·root Clippy·WASM32 Clippy·workspace all-target Clippy | 모두 PASS |
| workspace build·base SHA 지정 suite manifest 검사 | PASS |
| release-test 전체 nextest | **12,426 PASS, 45 skip, 0 FAIL**, 55 binaries |
| Native 공식 1: `--features native-skia --lib` | 2 PASS, 0 FAIL |
| Native 공식 2: `issue_2225_missing_picture_placeholder` | 2 PASS, 나머지 suite 279개는 module filter로 제외 |
| Native 공식 3: `render_p37_direct_pdf_export` | 22 PASS, 나머지 suite 263개는 module filter로 제외 |
| 원 사례 강화 관계 검사 | 원 head 1 PASS/1 FAIL → 보정 2 PASS |
| 기존 계약 focused | #8122 4개, #6173·#6465·#6651 각 1개, 합계 7 PASS |
| 신규 sample 보안 | 명시적 JSON 입력 1개로 hidden text·injection·unicode deception 검사 PASS |
| 전쪽 Native/최적화 WASM·실제 SVG 소비 | 위 결과와 직접 판독 완료 |
| `git diff --check` | PASS |

일반 Rust 검증은 bwrap 최소 글꼴 환경에서 실행했다. 시스템·사유 글꼴을 숨기고 저장소의 OFL 프로그램
5개만 노출했다. 시각 측정용 사유 글꼴 공급과 회귀 실행 환경을 구분했다.
nextest는 0.9.137이고 저장소 권고는 0.9.140이다. 경고는 실행하지 않은 `ci-duration-observation` profile의
JUnit `report-skipped` 키에 관한 것이며, 선택한 default profile의 실제 실행 수와 skip을 위와 같이 기록했다.
source-side `#[cfg(test)]`는 수정하지 않아 unit-tier 정책 증분 검사는 적용 대상이 아니다.

새 HWP의 자동 수집 래칫은 전체 회귀에서 통과했다. 독립 Print 전 1쪽과 `printMethodImpliesNup=false`를 확인해
oracle 원장에 `1 → 1` 행을 명시 등록했다. clipping controlset에는 이 원본의 행이 없어 그 외부 controlset을
검사한 것으로 쓰지 않는다. 실제 글상자 포함·겹침·내용 보존은 정식 관계 검사와 두 backend 출력으로 확인했다.
`layout-anomaly`는 base/후보 모두 offCanvas·textOverlap·overflow 0이므로 이 결함의 시각 통과 근거로 쓰지 않았다.
원장 끝에서 생긴 추가 행 충돌은 새 행의 위치를 옮겨 해소했다. 최신 devel의 기존 원장 행을 모두 보존했고,
[merge 보존 확인](../assets/pr8585/merge-preservation.json)에서 추가된 행은 #7524 한 개뿐이다.

## GitHub CI와 다음 조건

원 head의 [Full CI](https://github.com/edwardkim/rhwp/actions/runs/41802783098),
[Render Diff](https://github.com/edwardkim/rhwp/actions/runs/41802783027),
[CodeQL](https://github.com/edwardkim/rhwp/actions/runs/41802783113), Adapter·Proptest와
[최종 CI Impact Policy](https://github.com/edwardkim/rhwp/actions/runs/41811035805)를 확인했다.
[원 head check snapshot](../assets/pr8585/ci-snapshot.json)은 success 31·neutral 1·skipped 6이다.
GHAS neutral은 JS/TS·Python configuration 두 개 누락 안내이며 전체 언어의 경고 없음으로 바꾸어 쓰지 않는다.

원 head를 그대로 검토할 때는 Full CI 재사용 조건으로 중복 전체 회귀를 생략했다.
메인터너 source/test/baseline 보정 뒤에는 재사용을 종료하고 위 필수 로컬 검사를 새 후보에서 모두 실행했다.
보정 code head `c3e28d2c6e`는 같은 통합 PR #8601에서 다음 Full 실행을 완료했다.

- [CI](https://github.com/edwardkim/rhwp/actions/runs/41848506895): Lint, Frontend package, Native Skia, Archive A–E build·worker와 aggregate 성공
- [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/41848506810): Canvas visual diff 성공
- [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/41848507303): actions·Python·JavaScript/TypeScript·Rust Analyze 및 후속 GHAS `CodeQL` check 성공
- [Adapter](https://github.com/edwardkim/rhwp/actions/runs/41848506984)·[Proptest](https://github.com/edwardkim/rhwp/actions/runs/41848507024) 성공
- [최종 CI Impact Policy 감사](https://github.com/edwardkim/rhwp/actions/runs/41853560286): `AUDIT_CONCLUSION=success`, `audit:stage3-5-truth-table`, 발행 status success

[같은 head의 전체 check·workflow·policy snapshot](../assets/pr8585/corrected-code-ci.json)을 보존했다.
이는 `mode=full;rfp=0`, 실제 base `3a539454...`이며 원 fork의 녹색 결과를 재사용한 것이 아니다.
최신 devel에서 추가된 #8285의 자동 번호 앞 분할 보정과 회귀 파일은 clean merge tree에서 각각 byte 단위로
보존되며 이번 그림 정렬 경로와 겹치지 않는다. 새 PR의 Full CI에서도 최신 base와 함께 검증했다.

## 원격 경로 전환과 self-review

원 fork의 실제 push 권한이 false여서 기존 Git 및 gh credential dry-run 모두 HTTP 403으로 거절됐다.
원 fork는 변경하지 않았다. 작업지시자가 승인한 대안에 따라 같은 후보를 본 저장소의 동일 검토 branch에 push하고
Open 통합 PR #8601을 생성했다. 원 기여 commit의 SHA를 모두 보존하므로 merge commit 방식으로 수용하며,
merge 뒤 원 PR #8585의 간접 MERGED 상태를 API로 확인한다. squash·rebase merge는 사용하지 않는다.

self-review에서 공통 정렬 조건과 줄 소유 결과의 소비, 말미 공백과 개체 사이 공백 구분, 밑줄 공백 예외,
Native/fresh WASM 직접 판독, 전체 회귀·최소 글꼴·새 입력 보안·기존 #8122 대조군을 확인했다.
측정 주석·쪽수 원장·증적과 runtime의 출처를 구분했으며 source/test/baseline 추가 변경은 없다.
이번 trailing 변경은 검토 문서·증적·PR별 오늘할일뿐이다. code head의 성공 후 작성했고, 최신 base의 다른
오늘할일을 source에 복사하지 않는다. 새 head fast-pass의 실제 판정·required aggregate·clean merge 확인 뒤 통합한다.

## Merge 후 contributor PR comment 계획

원 PR #8585에 한국어 존댓말로 첫 기여 환영과 감사를 게시한다. 원 기여가 사진의 줄 소속을 복원한 성과와,
메인터너가 말미 공백 정렬·관계 회귀·측정 근거·원장을 보완한 이유를 구분한다. 실제 통합 PR·merge SHA와
원 PR의 간접 MERGED 확인, code Full CI·최신 trailing 검사, 로컬 전체 12,426 PASS/45 skip·Native 공식 3종을 연결한다.

독립 WebHWP Print 전 1쪽의 Native 99.95893%·fresh WASM 99.96404%와 정상 #8122의 픽셀 보존을 적는다.
이는 내용 픽셀 중심 자동 일치율 보조값이며 사람의 판정 정확도가 아님을 설명한다. 실제 integration merge SHA에
고정한 `corrected-native-p001.png`, `corrected-wasm-p001.png` 두 이미지를 Markdown 이미지로 포함한다.
[Visual Sweep의 comment 정본](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment),
독립 PDF·전쪽 TSV·확대 판독 기록을 함께 연결한다. 게시 전 raw 이미지 존재와 해시를 확인한다.

이슈 #7524의 실제 close 상태를 조회하고 같은 merge·시각 증적의 후속 comment를 남긴다.
#8576·#8579는 별개 범위로 유지한다. contributor fork branch는 보존하고, 승인된 후속 기록·devel 동기화 후
이번 upstream 임시 branch와 전체 `output/`·사용을 마친 `target/pr-review/debug`를 정리한다.

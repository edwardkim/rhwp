# PR #8585 검토

## 최종 판정

**머지 보류 — 원 head의 검증을 완료했고 정렬 폭 보정을 진행한다.** 사진의 줄바꿈 복원은 확인했으나, 같은 줄 구성의 텍스트와
그림이 서로 다른 가운데 정렬 폭을 사용해 마지막 사진 줄이 왼쪽으로 밀린다. Native와 fresh optimized WASM 모두 같은 차이를 확인했다. 공개 댓글·push·merge는 아직 수행하지 않았다.

## 접수와 검토 범위

| 항목 | 값 |
| --- | --- |
| PR | [#8585](https://github.com/edwardkim/rhwp/pull/8585), 글상자 그림의 여러 줄 배치 |
| 작성자 | `hyunmyungJaneLee`, `FIRST_TIME_CONTRIBUTOR`, fork `kidsnote/rhwp` |
| 원 head·현재 검토 code head | `7e7a86fef914fa021ad2ba511d88908c3f28f6ba` |
| 고정한 최신 devel | `7fdd5441ff1e77c083818e920a48089f294de35e` |
| current-base merge simulation | clean, tree `df3c1e2b824f4a5a4502e92e6189ecfd1a6c5dd4` |
| reviewer | `edwardkim` 지정 후 API 재조회 |
| 관련 이슈 | [#7524](https://github.com/edwardkim/rhwp/issues/7524), 접수 시 OPEN |
| 검토 경로 | maintainer 일반 처리 + 첫 외부 기여자·시각 증적 절차 |
| 가시성 branch | `review/pr-8585-textbox-picture-rows`, 원 head에서 시작 |
| 이번 승인 범위 | 접수·로컬 검토·검증. 원격 댓글·push·merge는 별도 승인 대상 |

원 기여의 구현 `ecfd76527c`, 회귀·HWP `c312ea2c87`, 웹한글 인쇄 PDF `d7fcabb85b`,
시각 자료 `7e7a86fef9`를 보존한다. 원 기여 커밋의 rebase·amend·force-push는 하지 않았다.
최신 devel을 fork branch에 섞거나 제품 코드를 수정하지 않았다.

## 원 주장과 기존 해결의 대조

최신 devel도 동일 HWP의 사진 6개를 한 줄에 배치하고 4개를 글상자 밖으로 보낸다.
따라서 원 이슈의 핵심 현상은 미해결이며 이 PR의 수정 필요성은 유효하다.
현재 devel의 본문 글줄 자체는 이미 세 줄이다. 현재 결함은 글줄 구성 결과를 별도 그림 배치가
첫 개체의 줄 하나로 소비하는 계층에 있다. 과거의 45자 합성 줄바꿈 설명을 현재 원인 전체로
그대로 재인용하지 않는다.

- #6465의 개체 소유 줄 보존과 #6651의 첫 줄 수직 오프셋 계약을 확인했다.
- #5820·#6173의 마지막 인라인 개체 뒤 공백 제외는 공통 helper에 구현되어 있다.
- #8122의 저장 줄 없는 글상자 세로 가운데 정렬 보정은 이미 base에 있다.
- 원 PR에서 명시한 표 셀 안 그림의 별도 문제 #8576·#8579는 이번 사진 줄 배치의 해결과 구분한다.

## 확인한 수정 효과와 남은 정렬 차이

생산·소비 경로는 `layout_textbox_content`의 composed line과 control text position →
새 `tac_line_plan`의 줄별 시작 x·y → 컨트롤 순회에서 `inline_x`·`inline_obj_y` 선택 →
Image/Shape/Equation 노드 bbox → Native Skia·Canvas·SVG 출력이다.
새 계획은 여러 줄일 때만 활성화되며, 한 줄 개체 경로는 기존 처리를 유지한다.

다음 좌표는 96 DPI의 독립 PDF 이미지 object bounds와 실제 Native render tree에서 읽었다.
검토용 관측값이며, 98% 미만 페이지의 절대 좌표를 회귀 기대값으로 추가하지 않았다.

| 사진 줄 | 독립 PDF의 왼쪽 사진 x | 원 head의 왼쪽 사진 x | 관측 |
| --- | ---: | ---: | --- |
| 첫째 | 151.68 | 148.4 | 약 3.3px 왼쪽 |
| 둘째 | 151.68 | 148.4 | 첫째 줄과 같은 열 |
| 셋째 | 151.68 | 138.4 | 앞 두 줄보다 10.0px 왼쪽 |

원 head는 사진 6개를 2열·3줄로 복원하고 글상자 밖 사진을 0개로 줄였다.
그러나 마지막 줄의 두 사진이 함께 왼쪽으로 밀린 것을 실제 PNG에서도 확인했다.
첫째·둘째 줄의 마지막 개체 뒤 공백은 1개, 마지막 줄은 4개다.

새 계획의 `total = w + text_w`는 이 말미 공백을 정렬 폭에 포함한다.
본문의 `paragraph_layout`은 가운데 정렬에서 마지막 개체 뒤 공백을 기존 공통
`trailing_space_width_after_last_inline_object`로 제외하고 밑줄 공백의 예외도 처리한다.
그 결과 텍스트의 열 원점은 유지되지만 그림의 원점만 줄마다 달라진다.
공백 개수에 맞춘 위치 상수나 문서 예외가 아니라 같은 줄의 정렬 폭 계약을 공유하는 보정이 필요하다.

기여 테스트 2개는 포함·겹침·여러 줄 여부를 보호하나 `tops.len() >= 2`만 확인하므로 이 열 정렬 차이를
잡지 못한다. 보정 뒤에는 독립 출력의 2열·3줄 구조와 대응 열의 정렬 관계를 함께 보호해야 한다.

## 검증 입력과 글꼴

| 입력 | 경로·해시 |
| --- | --- |
| 공개 익명화 HWP | `samples/issue7524/textbox_tac_picture_rows.hwp`, SHA-256 `ffc0a66e92b10ec2ea10167bca2a2721c68a4207aed4acf5b381ab5e7ca18d36` |
| 독립 한컴 웹한글 Print PDF | `pdf/issue7524/textbox_tac_picture_rows-webhwp-print.pdf`, SHA-256 `212f1c608c99a1a427aa5302046d8479afb0a16fecc19c2fcf62afb7212653a5` |

두 입력은 원 PR commit에 포함되어 있다. PDF는 Cairo 1.15.12, 2026-11-04 생성, A4 전 1쪽이다.
HWP를 rhwp로 변환한 PDF가 아니라 작성자가 동일 공개 입력을 한컴 웹한글에서 인쇄한 자료다.
사내 4,428건 비교는 작성자의 주장으로만 구분하며 이번 독립 검증 근거로 사용하지 않는다.

시각 비교에는 로컬 `gulim.ttc`, `HBATANG.TTF`, `hamchod-r.ttf` 세 프로그램을 base/head,
Native/WASM에 동일하게 공급한다. 사용할 수 없는 `한컴 윤고딕 230`은 실제 공급한 `HCR Dotum`으로
명시적으로 대체한다. 전체 사유 글꼴 디렉터리를 한꺼번에 공급했을 때의 Native 한글 누락 결과는
환경 진단으로 남기고 최종 비교와 분리했다. 글꼴 파일은 저장소에 추가하지 않는다.

## 실제 검증

| 검사 | 결과·범위 |
| --- | --- |
| base Native/fresh optimized WASM | 별도 실제 빌드 완료. 기존 Studio 배포 JS/WASM을 사용하거나 덮어쓰지 않음 |
| 원 head Native | `--locked --profile release-test --features native-skia --bin rhwp`, 성공. 전 1쪽 raw 96.84744% |
| 신규 관계 검사 | 동일 기여 테스트 원문 2개: 최신 devel 0 PASS/2 FAIL → 원 head 2 PASS |
| 기존 계약 focused | #8122·#6173·#6465·#6651, 기존 검사 7개 PASS |
| 새 sample 보안 | 명시적 `RHWP_SECURITY_SWEEP_SAMPLES_JSON` 입력 1개로 세 탐지기 검사 PASS |
| layout-anomaly | base/head 모두 offCanvas·textOverlap·overflow 0. 글상자 밖 사진 배치 자체를 검출하지 못하므로 시각 검증을 대신하지 않음 |
| 전쪽 수 | HWP와 독립 PDF 모두 1쪽. 누락 없음 |
| fresh WASM 원 head | 최적화 빌드 성공(785.89초)·실제 Canvas 전 1쪽 raw 96.85057% |

관계 검사는 자동 수집 경로 `regression_suite_018::issue_7524_textbox_tac_picture_rows`를 확인한 뒤,
동일 test source를 해당 head의 실제 native library에 연결해 실행했다. 최소 글꼴 환경은 bwrap으로
시스템·사유 글꼴을 숨기고 저장소의 OFL 글꼴 5개만 노출했다. 검사를 조용히 건너뛰지 않았다.

원 code head의 [Full CI](https://github.com/edwardkim/rhwp/actions/runs/41802783098),
[Render Diff](https://github.com/edwardkim/rhwp/actions/runs/41802783027),
[CodeQL](https://github.com/edwardkim/rhwp/actions/runs/41802783113), Adapter·Proptest와
[최종 CI Impact Policy](https://github.com/edwardkim/rhwp/actions/runs/41811035805)의 성공을 확인했다.
source·test·fixture 보정이 없고 current-base merge가 clean이므로 `local_validation` 4.3.0에 따라
전체 release-test와 Native Skia 광범위 검사를 중복 실행하지 않았다. 새 code 보정이 생기면 이 재사용은 종료된다.
GHAS의 neutral은 JS/TS·Python configuration 2개 누락 안내이며, 전체 언어의 보안 경고가 없다는 뜻으로 쓰지 않는다.

## 제출 전에 보완할 증적

원 PR의 신규 회귀 파일 첫머리는 관계형 유형·갱신 시각·직전 파일 commit·Native/fresh WASM 실측값과
정식 증거 경로를 갖추지 않았다. 직전 파일 commit은 `c312ea2c87cddbb8608f649a0d742eac1ab6107c`,
committer 시각은 `2026-11-04T17:50:30+09:00`다.
또한 독립 PDF를 추가했지만 `oracle_page_count_baseline.tsv`에 새 HWP의 행이 없다.
일반 신규 발생 래칫의 0건 결과와 별도로, 1쪽 정답지와 대응 내용 보존을 공식 회귀에 등록해야 한다.

## 후속 처리 조건

정렬 폭 보정 → Native/fresh WASM 전쪽 90% 이상과 같은 영역 직접 판독 → 관계형 검사·측정 근거·
쪽수 기준 보완 → 보정 head의 필수 로컬 검증 순서로 진행한다. 현재 원 head를 승인 또는 merge 준비 완료로
판정하지 않는다. 기여의 핵심 개선과 남은 작은 보정 범위를 구분해 안내하며, 공개 댓글·보정 push·새 head CI·
merge는 작업지시자의 승인 범위를 확인한 뒤 수행한다.

[원 head·base 전쪽 TSV와 PNG](assets/pr8585/)에 Native/WASM 각 1쪽, 실제 글꼴 공급·source·바이너리 해시를 보존했다.
[로컬 보정 범위](pr_8585_review_impl.md)에 따라 원 기여를 유지하며 한 단위로 보정·재검증한다.

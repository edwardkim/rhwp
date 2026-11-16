# PR #9063 검토 — 제한 개체 이월과 부동 앵커 앞 꼬리 공백

## 최종 판정

**메인터너 보정 후 수용 가능.** 검증한 수용 후보 `fafc84f71187aa1edec73d5204d501abaa0aa50a`를
[PR #9069](https://github.com/edwardkim/rhwp/pull/9069)로 제출하고, 해당 head의 전체 CI 성공과
[self-review](https://github.com/edwardkim/rhwp/pull/9069#issuecomment-6872355751) 뒤 merge했다.
실제 merge는 `5423ce12b1553af7352c38c24fad663d3b04fcde`, 2026-11-14 14:25:19 KST다.

원 기여의 열기 개선을 보존하면서 편집 후 이월 해제·중복 예약·합성 저장 줄 원점의 경계를 보완했다.
보정 없는 원 head를 별도 승인한 것으로 해석하지 않는다. 작업지시자의 원격 처리·머지·후속 처리 승인을
확보했으며 fork branch는 변경하지 않았다. 원 PR #9063의 간접 MERGED와 #8660·#9059의 CLOSED를 API로 확인했다.

[보완 계획](pr_9063_review_impl.md), [실행 원장](../assets/pr9063/evidence.md),
[승인된 원격 처리안](../assets/pr9063/remote-action-plan.md)을 따른다. 이 archive는 원 코드 PR의
완료 기록이며, 아래 contributor 댓글 계획에 따라 devel에 존재하는 실제 증거로 후속 댓글을 게시한다.

## 접수와 검토 범위

- PR: https://github.com/edwardkim/rhwp/pull/9063
- 작성자: `hyunmyungJaneLee`, 원 fork `kidsnote/rhwp`.
- 원 head: `b43050954c10512705d714114f5af23f1ee98ac6`.
- 최신 devel: `dc3d0ff15e6e7cdb39138c72a5466994dea0e0d4`.
- 공통 조상: `dd7f453898776c262c00e862e10f47d22a4d6ceb`.
- reviewer: 사용자 지시에 따라 `edwardkim`을 요청하고 REST API로 재확인했다.
- 역할: 메인테이너 일반 검토와 승인된 보완·필수 검증·원격 수용·후속 처리. 공통 이월 보완은 로컬 커밋에 기록했고, 원격 처리 승인을 받아 upstream 수용 PR에서 CI·self-review·merge를 완료했다.
- 연결 이슈: #8660, #9059. 수용 PR merge 뒤 두 이슈의 CLOSED를 API로 확인했다.

첫 metadata 조회에 본문을 포함해 reviewer 지정 선행 순서를 지키지 못했다. 이를 인지한 뒤
reviewer 지정과 API 재확인을 마치고 소스 fetch 및 로컬 검토를 시작했다.

원 PR의 여섯 기여 커밋은 보존했다. 원 head를 최신 devel에 병합하는 `merge-tree`는 충돌 없이
완료했으며 tree는 `2d5f2f6a4b5625cd033f7a78b63eaf73c5834512`다. 로컬 검증용 병합
`bc8e562aebc9e4f2f8c170728d84bb468a869268`도 같은 tree다. 원 fork branch는 변경하지 않았다.

## CI와 선행 해결 추적

원 head의 CI·CodeQL·Render Diff·Adapter·Proptest와 정책 감사는 성공했다. 이는 기여자의
원 head에 대한 원격 결과이며, 아래 메인테이너의 데스크톱 Print·실제 편집 검증을 대신하지 않는다.

| 검사 | 원 head 실행 |
| --- | --- |
| CI | https://github.com/edwardkim/rhwp/actions/runs/43510933200 |
| CodeQL | https://github.com/edwardkim/rhwp/actions/runs/43510933178 |
| Render Diff | https://github.com/edwardkim/rhwp/actions/runs/43510932965 |
| Adapter | https://github.com/edwardkim/rhwp/actions/runs/43510933273 |
| Proptest | https://github.com/edwardkim/rhwp/actions/runs/43510933057 |
| 최종 정책 감사 | https://github.com/edwardkim/rhwp/actions/runs/43515664755 |

#2032의 해결 PR #2033 및 #2075의 해결 PR #2076은 개체가 쪽 아래로 소실되는 경우 하단을
제한했다. 이번 변경은 그리기 단계의 제한보다 앞선 조판 단계에서 다음 쪽 소유를 정한다.
기존 검토 기록의 제한 범위와 셀 내부 개체 경로를 구분해 확인했다.

#8639의 원 해결·메인테이너 수용은 [PR #8662 검토](pr_8662_review.md)와
[원 SHA 대응](pr_8662_review_impl.md)에 기록되어 있다. 현재 devel에는
`37f51c4e78`의 저장 줄 없는 TAC 줄 높이·공백 처리와 `1562db7133`의 변경 추적 저장 줄 보존이
포함되어 있다. 이번 수정은 부동 앵커만 뒤에 남은 공백 분류와 줄 끝 걸기에 추가 적용된다.

## 독립 기준

원 PR이 명시한 웹한글기안기 Print PDF는 데스크톱 한컴 Print 판정 기준에서 제외했다.
같은 공개 HWP 17개를 Windows 데스크톱 한컴 2020 `11.0.0.9136`으로 실제 인쇄했다.
서비스 `engine=2020`, `pdf_print_method=0`, `hancom2020_pdf_driver_one_up`을 확인했다.

입력의 `info --json`은 모두 `format=hwp5`, `lastSavedWith.product=hancom-office-2010`,
`version=8.0.0.466`이다. 이 값은 저장 메타데이터이며 합성 입력의 실제 작성 제품을 증명하지 않는다.
현행 engine 선택 규칙에 따라 2020을 사용했다. 입력 및 PDF SHA-256, 비동기 작업 ID,
terminal 상태와 내려받은 파일의 client/server 크기·SHA-256 일치는
[Print 원장](../assets/pr9063/evidence.md)에 기록했다.
원본 PDF는 [개체 경계](../../../pdf/issue8660)·[꼬리 공백](../../../pdf/tac_space_tail)의 canonical 경로에,
실제 편집 파생본은 [추가 증적 경로](../../../pdf/pr9063)에 보존한다. 글꼴 프로그램은 추가하지 않는다.

## 보정 결과와 로컬 검증

최신 devel에서도 원 문제는 남아 있었다. #9059의 세 입력은 독립 Print 대비 raw
52.75%·52.75%·61.27%였고, #8660의 초과 그림 세 입력은 devel 1쪽/Print 2쪽이었다.
[수정 전 Native 전쪽 TSV](../assets/pr9063/base-native)를 보존했다. 원 PR의 기본 열기는
17문서·22쪽 모두 개선됐으나 일반 편집 후 이월 유지가 빠져 있어 이번 승인 범위로 보완했다.

제품 source `e2d7fb08b5`, 회귀 보강 `5b028b3f03`. 실제 Native producer와 Docker 최적화 WASM의
제품 입력은 같고, `src`·Cargo·toolchain·build.rs·locked wrapper의 Git object를 대조했다.
Native와 WASM에는 같은 HCR 환경·실제 프로그램을 공급했다. 글꼴 파일은 커밋하지 않는다.

| 검사 | 실제 범위와 결과 |
| --- | --- |
| 원본·실제 저장 파생본 | 39조합·Native/fresh WASM 각 58쪽, 전쪽 raw 100.00%와 Print 쪽수 일치 |
| Native 실제 편집·반복 저장 | 12개 × live·두 형식의 첫째/둘째 재열기 = 60상태·100쪽, raw 100.00% |
| fresh WASM live·반복 저장 | live 12개·20쪽 raw 100.00%, HWP/HWPX 두 번씩 재열기 48상태의 쪽 소유·본문 순서/내용 보존 |
| 실제 출력 소비 | SVG 6개·10쪽 raw 100.00%, Studio 실제 키보드·Canvas2D 4개에서 이월/fit 유지 |
| 최신 devel 기능 보존 | #9042·#9066의 표·빈 머리말/꼬리말·한컴 재저장본 4개, Native/fresh WASM 각 8쪽 raw 100.00% |
| 정본 보호 | Native/fresh WASM 합계 3,542쪽, 공식 두 자리 표시값 하락 0·누락 0 |
| 편람 필수 gate | Native HWPX 383쪽 passed, 기준 하락 0 |
| 최소 글꼴 전체 회귀 | 13,524 passed / 0 failed / 45 config-skipped |
| Native 공식 3종 | lib 4,035 passed / 12 ignored, placeholder 2 passed, direct PDF 36 passed, 합계 4,073 passed |
| 집중 회귀 | 개체 12·공백 4·기존 표 성장/Enter 5·편집 들여쓰기 20, 합계 41 passed |
| 새 입력의 보안 검사 | 원본·파생본 43개 × hidden-text/injection/unicode = 129회, 모두 clean |
| 새 독립 Print 쪽수 등록 | canonical PDF 17개와 신규 원장 17행, 실제 쪽수 17/17 일치·기존 행 변경 없음 |

Rust lint·workspace build, 명시적 새 샘플 보안과 최종 쪽수 원장 회귀도 모두 exit 0이다.
실제 명령·개수·초기 환경 오류와 복구는 [실행 원장](../assets/pr9063/evidence.md)에 기록했다. 원 head의 성공 CI를
보정 후보의 CI로 재사용하지 않았다. 보정 후보는 수용 PR #9069에서 정확한 head의 CI를 통과해 통합됐다.

### 공통 조판 원인과 실제 소비

HWP5 [CommonObjAttr bit 13](../../tech/한글문서파일형식_5.0_revision1.3.md)은 문단 기준 개체의
쪽 영역 제한 속성이다. 문서의 편집 상태가 그 속성을 해제하는 조건은 아니다.

`section.rs`의 문단 흐름 원점 → `restricted_object_host_top` → 보통/표 호스트 controls의
`overflow_para_object_next_page_placement` → `defer_stored_frame` → transition의 확정 앵커와
`next_flow_y` 예약 → layout의 최종 배치를 추적했다. 편집 시 현재 유효 흐름 원점을 사용하고,
실제 저장 줄은 기존 계약을 유지하되 구현용 합성 LineSeg의 문단 내부 vpos를 쪽 좌표로 재사용하지 않는다.

자리차지 그림은 기존 선행 fit이 개체 높이를 호스트에 더해 문단까지 먼저 옮기는 경계가 있었다.
`prepare_fit_budget_with_storage`가 같은 공통 이월 계획을 소비해 별도 쪽 소유가 있는 개체의 높이를
중복 예약하지 않는다. 다음 쪽에서 TopAndBottom은 실제 개체 점유 끝 다음에 본문을 배치하며,
InFront는 본문 흐름을 밀지 않는 기존 속성을 유지한다. 단순 helper 이름만으로 공유를 주장하지 않고
선행 fit·queue·전이·최종 원점까지 대조했다. 다단·caption·비제한 개체 등의 기존 적용 경계도 보존한다.

문서 ID·임의 높이 상수·clamp·출력 숨김은 추가하지 않았다. 빈 글줄/개체 가시성과 흐름 예약을
같은 상태로 취급하지 않았다. 저장 줄 없는 합성 입력이라는 사실과 실제 한컴 Print 관측을 구분했다.

### 시각 증거와 회귀의 독립성

- [Native 확대 비교](../assets/pr9063/correction/native-detail.png),
  [WASM 확대 비교](../assets/pr9063/correction/wasm-detail.png),
  [첫 쪽 본문](../assets/pr9063/correction/native-page1.png),
  [실제 Studio 편집](../assets/pr9063/correction/studio-keyboard.png).
- [Native 전쪽 TSV](../assets/pr9063/correction/native),
  [fresh WASM 전쪽 TSV](../assets/pr9063/correction/wasm),
  [보호 원장 3,542행](../assets/pr9063/correction/protected-comparison.tsv).
- [수정 전 실패와 원인](../assets/pr9063/edit-boundary/README.md),
  [Print·SHA·글꼴·실제 실행 영수증](../assets/pr9063/evidence.md).

그림 윤곽·사각형 선, 첫 쪽 ANCHOR/AFTER00~09, 다음 쪽 그림 아래 AFTER10/11의 순서·간격·누락·겹침을
같은 영역/배율로 직접 판독했다. 이 입력에는 수식이 없어 수식 검증을 주장하지 않는다.
raw 100.00%는 RGB 픽셀 완전 일치라는 뜻이 아니다.

시각 선행 조건을 완료한 뒤 기존 두 회귀 파일에 실제 편집·반복 저장 관계 검사 4개를 보강했다.
절대 페이지 px 핀은 추가하지 않았다. TAC 상대 이동은 웹 Print의 과거 값 대신 독립 Desktop Print의
TEXT 잉크 bbox에서 얻은 3.1966 / 18.2201 / 15.0235px를 사용하며 기존 0.6px 여유는 유지했다.
같은 원본의 기존 검사를 함께 확인했고 정상 검사는 보존했다. 새 예외나 기준 하락은 적용하지 않았다.

### 원격 검증과 수용

| 검사 | 실행/실제 결과 |
| --- | --- |
| 제품 CI | [43558842756](https://github.com/edwardkim/rhwp/actions/runs/43558842756), 전체 22개 job·모든 step 완료, success |
| 기본 회귀 Archive A–E | 3,796 + 2,046 + 2,467 + 2,515 + 2,506 = 13,330 passed / 0 failed |
| Native CI 선택 검사 | 실제 로그의 25개 요약 합계 243 passed. 로컬 공식 4,073건과 실행 범위를 구분한다 |
| CodeQL | [43558842804](https://github.com/edwardkim/rhwp/actions/runs/43558842804), 전체 success |
| Render Diff | [43558842738](https://github.com/edwardkim/rhwp/actions/runs/43558842738), 실제 Canvas/PDF 검사 success |
| Adapter / Proptest | [43558842599](https://github.com/edwardkim/rhwp/actions/runs/43558842599) / [43558842615](https://github.com/edwardkim/rhwp/actions/runs/43558842615), success |
| 최종 정책 | [43561181677](https://github.com/edwardkim/rhwp/actions/runs/43561181677), success |

GitHub check는 성공 33개·정상 skip 5개이며 실패는 없다. 실제 CI job·step의 종료 상태와 로그를 확인했다.
최신 devel `dc3d0ff15e6e7cdb39138c72a5466994dea0e0d4`와의 merge-tree는 exit 0이고,
실제 PR merge ref 및 최종 merge의 tree `18c2f8053ec711a649c646585cdcecb4babb2f01`과 같다.
`--match-head-commit`으로 검증한 head를 지정해 정상 merge했으며 보호 설정·검사 기준의 예외는 적용하지 않았다.

fork `kidsnote/rhwp`는 `push=false`여서 본 저장소 수용 branch를 사용했다. 원 head와 여섯 기여 commit은
실제 merge의 조상이며 원 PR은 2026-11-14 14:25:22 KST에 간접 MERGED로 확인됐다. 연결 이슈 #8660·#9059도
14:25:44/45 KST에 닫혔다. 완료·감사 댓글과 종료 정리는 승인된 후속 범위다.

## Merge 후 contributor PR comment 계획

- 대상: [원 PR #9063](https://github.com/edwardkim/rhwp/pull/9063). 원 기여를 먼저 인정하고,
  편집·반복 저장 보완이 필요했던 원인과 실제 검증 결과를 존댓말로 설명한다.
- 실제 merge: `5423ce12b1553af7352c38c24fad663d3b04fcde`. 수용 PR은 #9069이며 fork branch는 유지한다.
- 개체 원본 8문서·13쪽과 실제 편집/저장 경계는 `correction/native-detail.png` 및
  `correction/wasm-detail.png`로 표시한다. 그림/사각형의 다음 쪽 상단과 그림 뒤 본문을 확인했다.
- 공백 원본 9문서·9쪽은 `native-detail-comparison.png`의 앞 네 대조 영역과
  `wasm-detail-comparison.png`로 연결한다. `at_table_n2/n3`, `after_1fit_n2`, `after_3fit_n4`의 1쪽 TEXT·표 간격을 직접 대조했다.
  이 이미지는 원 기여 접수의 비교이며 같은 원본의 최종 재검증 TSV도 전쪽 raw 100.00%다.
- 전체 최종 원본/실제 저장본 39조합·Native/fresh WASM 각 58쪽은 raw 100.00%, 보호 3,542쪽의 공식값 하락 0이다.
  수치가 자동 내용 실루엣 보조값이며 RGB 완전 일치나 사람 판정 정확도가 아님을 설명한다.
- 게시 전 위 네 PNG의 merge SHA raw HTTP 응답이 PNG이며 해당 commit의 파일과 SHA-256·바이트가 같은지 확인한다.
  게시 후 API의 실제 UTF-8 본문과 브라우저 이미지 표시를 확인한다. Visual Sweep 정본 링크와 전쪽 TSV·검토 기록 링크를 함께 넣는다.
- issue #8660·#9059에는 각 증상에 해당하는 완료 근거와 비교를 따로 게시한다. 원 PR과 이슈 상태를 API로 재확인한 뒤 정리한다.

## Merge 후 실행 확인

[Duration data 갱신](https://github.com/edwardkim/rhwp/actions/runs/43561454031)은 success, 실제 `ready:true`·`published:true`이며 게시 commit `15d0b35a7113ba1312110ad1545668f95575ce98`을 확인했다. merge 뒤 제품 CI를 시작하거나 재실행하지 않았다. 댓글 이미지 네 개의 merge SHA raw PNG 응답은 Git blob과 동일함을 확인했다. archive·증거가 devel에 반영된 뒤 승인된 완료·감사 댓글을 게시한다.

## 완료 댓글과 게시 검증

- [8660 결과 댓글](https://github.com/edwardkim/rhwp/issues/8660#issuecomment-6872474094), 본문 SHA-256 `1b010a9d7afda651feeee067f34fc95b513977e44acb691188eccb2b1c102d14`.
- [9059 결과 댓글](https://github.com/edwardkim/rhwp/issues/9059#issuecomment-6872474568), 본문 SHA-256 `03c7de748a9bce25bf1c0408f78b0351555aa20b3472ca46e22a1e630f1f4308`.
- [9063 결과 댓글](https://github.com/edwardkim/rhwp/pull/9063#issuecomment-6872475032), 본문 SHA-256 `97d852984e1f29e214e5f3378b870b9c9d21242cac5a7cb09a7ed1858d4fb2c7`.

API에서 세 댓글의 실제 UTF-8 본문이 준비한 원문과 같은지 확인했다. 원 PR #9063과 수용 PR #9069는 MERGED, #8660·#9059는 CLOSED/completed다.

Chromium `152.0.7946.0`의 실제 GitHub 페이지에서 정확한 alt·merge SHA 원본 URL로 이미지 5개를 구분했다. 아바타는 제외했고 Native/WASM은 1260×1000, 꼬리 공백 비교는 1250×1165의 실제 원본 크기로 로드됐다. 댓글별 첫 비교 이미지가 화면 안에 표시되는 좌표도 확인하고 직접 캡처를 판독했다. React 재렌더링 중 분리된 DOM 캡처나 아바타만 확인한 초기 probe는 완료 결과로 계상하지 않았다. 브라우저·임시 profile은 닫았고 최종 원시 화면·API 로그는 output 종료 정리 대상이다.

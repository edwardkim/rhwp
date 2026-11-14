# PR #9063 검토 — 제한 개체 이월과 부동 앵커 앞 꼬리 공백

## 최종 판정

**메인테이너 보정 후 수용 가능.** 원 기여의 열기 개선은 유효하며,
편집 후 제한 개체가 앞쪽으로 돌아오던 경계를 공통 조판 경로에서 보완했다.
원 head와 보정 후보의 결과를 구분한다. 필수 로컬 검증을 모두 완료했으며 원격 수용은 별도 승인을 기다린다.

2026-11-14 KST 작업지시자가 승인한 범위는 로컬 보완·필수 검증이다.
공개 댓글·승인 리뷰·push·새 PR·머지는 수행하지 않았다. #8660·#9059는 OPEN을 유지한다.
[보완 계획](pr_9063_review_impl.md), [실행 원장](assets/pr9063/evidence.md),
[원격 처리안](assets/pr9063/remote-action-plan.md)을 따른다.

## 접수와 검토 범위

- PR: https://github.com/edwardkim/rhwp/pull/9063
- 작성자: `hyunmyungJaneLee`, 원 fork `kidsnote/rhwp`.
- 원 head: `b43050954c10512705d714114f5af23f1ee98ac6`.
- 최신 devel: `dc3d0ff15e6e7cdb39138c72a5466994dea0e0d4`.
- 공통 조상: `dd7f453898776c262c00e862e10f47d22a4d6ceb`.
- reviewer: 사용자 지시에 따라 `edwardkim`을 요청하고 REST API로 재확인했다.
- 역할: 메인테이너 일반 검토와 작업지시자가 승인한 로컬 보완·필수 검증. 공통 이월 보완은 로컬 커밋에 기록했고, 공개 댓글·push·승인 리뷰·머지는 수행하지 않았다.
- 연결 이슈: #8660, #9059. 해결 증거는 준비했으며 실제 통합 전에는 OPEN을 유지한다.

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

#8639의 원 해결·메인테이너 수용은 [PR #8662 검토](archives/pr_8662_review.md)와
[원 SHA 대응](archives/pr_8662_review_impl.md)에 기록되어 있다. 현재 devel에는
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
[Print 원장](assets/pr9063/evidence.md)에 기록했다.
원본 PDF는 [개체 경계](../../pdf/issue8660/)·[꼬리 공백](../../pdf/tac_space_tail/)의 canonical 경로에,
실제 편집 파생본은 [추가 증적 경로](../../pdf/pr9063/)에 보존한다. 글꼴 프로그램은 추가하지 않는다.

## 보정 결과와 로컬 검증

최신 devel에서도 원 문제는 남아 있었다. #9059의 세 입력은 독립 Print 대비 raw
52.75%·52.75%·61.27%였고, #8660의 초과 그림 세 입력은 devel 1쪽/Print 2쪽이었다.
[수정 전 Native 전쪽 TSV](assets/pr9063/base-native/)를 보존했다. 원 PR의 기본 열기는
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
실제 명령·개수·초기 환경 오류와 복구는 [실행 원장](assets/pr9063/evidence.md)에 기록했다. 원 head의 성공 CI를
보정 후보의 CI로 재사용하지 않았다. 보정 후보는 아직 원격에 제출하지 않았다.

### 공통 조판 원인과 실제 소비

HWP5 [CommonObjAttr bit 13](../tech/한글문서파일형식_5.0_revision1.3.md)은 문단 기준 개체의
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

- [Native 확대 비교](assets/pr9063/correction/native-detail.png),
  [WASM 확대 비교](assets/pr9063/correction/wasm-detail.png),
  [첫 쪽 본문](assets/pr9063/correction/native-page1.png),
  [실제 Studio 편집](assets/pr9063/correction/studio-keyboard.png).
- [Native 전쪽 TSV](assets/pr9063/correction/native/),
  [fresh WASM 전쪽 TSV](assets/pr9063/correction/wasm/),
  [보호 원장 3,542행](assets/pr9063/correction/protected-comparison.tsv).
- [수정 전 실패와 원인](assets/pr9063/edit-boundary/README.md),
  [Print·SHA·글꼴·실제 실행 영수증](assets/pr9063/evidence.md).

그림 윤곽·사각형 선, 첫 쪽 ANCHOR/AFTER00~09, 다음 쪽 그림 아래 AFTER10/11의 순서·간격·누락·겹침을
같은 영역/배율로 직접 판독했다. 이 입력에는 수식이 없어 수식 검증을 주장하지 않는다.
raw 100.00%는 RGB 픽셀 완전 일치라는 뜻이 아니다.

시각 선행 조건을 완료한 뒤 기존 두 회귀 파일에 실제 편집·반복 저장 관계 검사 4개를 보강했다.
절대 페이지 px 핀은 추가하지 않았다. TAC 상대 이동은 웹 Print의 과거 값 대신 독립 Desktop Print의
TEXT 잉크 bbox에서 얻은 3.1966 / 18.2201 / 15.0235px를 사용하며 기존 0.6px 여유는 유지했다.
같은 원본의 기존 검사를 함께 확인했고 정상 검사는 보존했다. 새 예외나 기준 하락은 적용하지 않았다.

### 후속 범위

fork `kidsnote/rhwp`는 실제 `push=false`다. 승인받은 경우 upstream의 새 Open 수용 PR에 원 기여
여섯 commit과 보정을 함께 제출하고, 그 head의 CI·self-review 후 merge한다. 별도 원격 승인은
[실제 처리안과 댓글 초안](assets/pr9063/remote-action-plan.md)을 기준으로 요청한다.
merge 전에는 원 PR·연결 이슈를 닫지 않는다.

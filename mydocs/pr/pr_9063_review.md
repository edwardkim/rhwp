# PR #9063 검토 — 제한 개체 이월과 부동 앵커 앞 꼬리 공백

## 최종 판정

**머지 보류.** 기본 열기의 개선은 유효하지만, #8660의 개체 이월이 일반 편집 후 유지되지 않는다.
첫 문단 `L00`에 `X` 한 글자를 넣으면 그림·사각형이 2쪽 상단에서 1쪽 하단으로 돌아가며,
저장 후 다시 열면 다시 2쪽이 된다. Native와 최적화한 fresh WASM의 실제 Chromium에서 모두
재현했다. 기준 문단 원점은 같고 독립 한컴 Print는 편집 전후 모두
2쪽을 유지한다. [실행 결과·원인·보완 제안](assets/pr9063/edit-boundary/README.md)을 따른다.

이는 기존 devel 대비 새 악화라는 판정이 아니라, PR의 열기 개선이 편집 경로에 적용되지 않는
미해결 경계다. 높은 기본 열기 일치율이나 기존 테스트 통과로 이 경계를 면제하지 않는다.
`Closes #8660`으로 완료하기 전에 편집·미편집의 같은 쪽 영역 제한 규칙을 공통으로 적용하고,
그림·도형·fit·제한 off·자리차지 뒤 본문을 Native/fresh WASM에서 재검증해야 한다.

2026-11-14 KST 작업지시자가 **메인테이너 보완·필수 검증**을 승인했다.
[보완 계획](pr_9063_review_impl.md)에 따라 로컬 제품 보정과 검증을 진행한다.
공개 코멘트·승인 리뷰·push·머지는 수행하지 않았다. #8660·#9059는 OPEN을 유지한다.

## 접수와 검토 범위

- PR: https://github.com/edwardkim/rhwp/pull/9063
- 작성자: `hyunmyungJaneLee`, 원 fork `kidsnote/rhwp`.
- 원 head: `b43050954c10512705d714114f5af23f1ee98ac6`.
- 최신 devel: `dc3d0ff15e6e7cdb39138c72a5466994dea0e0d4`.
- 공통 조상: `dd7f453898776c262c00e862e10f47d22a4d6ceb`.
- reviewer: 사용자 지시에 따라 `edwardkim`을 요청하고 REST API로 재확인했다.
- 역할: 메인테이너 일반 검토. 로컬 검토만 승인되었으며 제품 보정·공개 댓글·push·승인 리뷰·머지는 아직 수행하지 않았다.
- 연결 이슈: #8660, #9059. 검토 중이며 두 이슈의 완료 여부를 아직 판정하지 않았다.

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
[Print 원장](assets/pr9063/desktop-print-index.json) 및 개별 영수증에 기록했다.
PDF는 [정식 증적 경로](../../pdf/pr9063/)에 보존한다. 글꼴 프로그램은 추가하지 않는다.

## 로컬 검증

| 검증 | 결과 |
| --- | --- |
| 최신 devel 재현 | #9059의 3문서가 raw 52.75%·52.75%·61.27%. #8660의 그림 3문서는 1쪽/독립 Print 2쪽 불일치. 이월 뒤 본문 사례도 차이가 남는다 |
| 후보 Native, 같은 공개 입력 17문서·22쪽 | 전쪽 raw 100.00%, 독립 Print 쪽수 일치. [전쪽 원본 TSV](assets/pr9063/native/), [점수 원장](assets/pr9063/head-native-score-index.json) |
| 최소 글꼴 집중 회귀 | 공식 자동 수집 suite 2개에서 해당 12건 전부 PASS. [실제 로그](assets/pr9063/focused-tests.log), [글꼴 격리](assets/pr9063/minimal-font/README.md) |
| Native 편집 API | 그림·도형의 2→1→2쪽 변화 재현. fit 대조군은 1→1→1쪽 유지 |
| 편집본 독립 Print | 그림·도형 모두 2쪽. 원 입력과 편집본의 2쪽 96 DPI RGB가 동일하며 `ANCHOR` bbox도 같다 |
| fresh WASM, 같은 공개 입력 17문서·22쪽 | 최적화 포함 빌드 성공. Native와 같은 HCR 환경·프로그램을 명시한 최종 전쪽 raw 100.00%. [전쪽 원본 TSV](assets/pr9063/wasm/), [점수 원장](assets/pr9063/head-wasm-score-index.json) |
| WASM 편집 API | 실제 Chromium `152.0.7946.0`에서 그림·도형 2→1→2쪽 재현. fit 대조군 1→1→1쪽 유지. [실제 결과](assets/pr9063/edit-boundary/wasm-results.json) |
| diff 및 문서 링크 | `git diff --check` 통과. 최종 검토 문서 4개 내부 링크 검사 통과 |

광범위 회귀·Native 공식 전체 검사는 원 exact head의 성공한 Full CI를 확인했고 이번 로컬 검토에서
반복하지 않았다. 새 제품 보정을 수행하면 그 후보에서 4.3의 필수 로컬 검증을 다시 완료해야 한다.
원 head와 최신 devel의 자동 병합에는 source 충돌이 없으며 원 기여 여섯 커밋은 보존했다.

Native 검증 source와 문서 commit 사이의 `src`·`tests`·Cargo 입력·WASM wrapper의 Git object는
[모두 동일](assets/pr9063/source-identity.json)하다. 빌드 source와 파일 SHA는 각 실행 파일 영수증에
구분했다. source를 바꾸지 않은 문서 기록을 제품 보정 SHA로 표기하지 않는다.

WASM은 Docker Compose에서 기본 작업공간의 같은 물리적 `target/pr-review`를 `/review-target`에
연결해 locked wrapper·최적화를 완료했다. Rust 1.93.1, wasm-pack 0.15.0이며 pkg와 해당 worktree의
Studio public JS/WASM 해시가 같다. [빌드 영수증](assets/pr9063/fresh-wasm-receipt.json)을 따른다.
브라우저는 CI의 고정 Chromium build `1660786`을 명시했다. 편집 재현은 브라우저의 실제
`HwpDocument` API 검사이며 Studio UI 클릭 E2E와 구분한다.

첫 WASM 실행은 명시적 환경이 없어 Noto Serif KR을 선택했다. 그 결과는 진단용으로 분리하고
최종 TSV에 사용하지 않았다. 최종 검증은 Native의 `HCR Dotum`→`HCRDotum`,
`함초롬바탕`→`HCRBatang` 환경과 같은 프로그램 파일을 공급했다. 실제 WASM loaded SHA와
Native 공급 파일 SHA, 공개 source URL은 [글꼴 원장](assets/pr9063/font-supply.json)에 연결했다.
글꼴 파일 자체는 Git에 추가하지 않았다.

## 조판 원칙과 직접 판독

HWP5 [CommonObjAttr bit 13](../tech/한글문서파일형식_5.0_revision1.3.md)은 문단 기준 개체의
세로 위치를 본문 영역으로 제한하는 속성이다. 저장 줄 캐시의 신뢰 여부와 개체의 쪽 영역 제한은
같은 상태가 아니다. 편집되지 않은 다른 문단의 원점이 유효한데도 전체 edited 플래그만으로
이월 판정을 중단하는 경계가 실제 재현되었다.

기준 문단 원점 생산 → `restricted_object_host_top` 선택 → 두 controls 호출자의
`overflow_para_object_next_page_placement` → `defer_stored_frame`의 다음 쪽 소유 →
`layout_column_shapes_pass`의 확정 앵커 소비를 추적했다. edited 경로에서는 첫 helper가
`None`을 반환해 이 연결이 끊기며 기존 paint 하단 제한으로 돌아간다.

[같은 배율의 Native 확대 대조](assets/pr9063/native-detail-comparison.png)와
[fresh WASM 확대 대조](assets/pr9063/wasm-detail-comparison.png)에서 공백 뒤 본문,
부동 표의 위치와 글자 크기, 그림·사각형의 다음 쪽 상단 배치를 직접 판독했다. 그림은 작은
윤곽 이미지이며 빈 2쪽으로 오인하지 않았다. 확대판은 96 DPI 원본 픽셀을 같은 영역으로
자르고 바깥 라벨만 추가했으며 원본·기준의 확대·왜곡·내용 변경은 하지 않았다.
이 공개 샘플에는 수식이 없어 수식 일치 검증을 했다고 주장하지 않는다.

# PR #9063 검토 — 제한 개체 이월과 부동 앵커 앞 꼬리 공백

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

## 검토 상태

최신 devel에서 #9059의 공백 처리 차이와 #8660의 개체 이월 미반영을 재현했다.
후보의 Native/fresh WASM 전쪽 시각 대조와 실제 편집·저장 경계 검증을 진행 중이다.
`restricted_object_host_top`이 편집된 세션 전체를 제외하는 조건은 실제 실행 결과를 확인한 뒤 판정한다.
소스 관찰만으로 회귀라고 확정하지 않았다.

최종 판정은 아직 보류다. 미실행 검증을 완료한 것으로 기록하거나 머지 준비 완료로 판단하지 않는다.

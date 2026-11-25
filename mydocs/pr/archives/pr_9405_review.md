# PR #9405 리뷰 — MCP PDF 출력 모드와 글꼴 이름 보정 응답

## 최종 판정

**승인 — 운영 시험·문서 검증과 최신 head의 CI를 통과했으며 작업지시자의 병합·후속 처리 승인을 받았습니다.**

제출 head와 검토 기록 head `449e291f92fc30b4733694b09fb7f1ea3b9bb915`의 CI가 성공했습니다.
2026-11-25 작업지시자의 승인으로 최종 판정을 확정했습니다. 승인 기록 commit의 CI와
mergeability를 다시 확인한 뒤 병합과 `post_merge.md` 후속 처리를 수행합니다.

## 접수와 변경 범위

| 항목 | 값 |
| --- | --- |
| PR·작성자·base | [#9405](https://github.com/edwardkim/rhwp/pull/9405) / jangster77 / devel |
| 검증한 제출 head | `b42770d3f5503b797396caf49441485cdfab8e5a` |
| 기준 devel | `fb98e8bff300ffe6aeb15119ee9ed280871e8abd` |
| 관련 이슈 | [#9263](https://github.com/edwardkim/rhwp/issues/9263), 이미 닫힌 출처 확인 이슈 참조 |
| 역할·reviewer | collaborator 본인 PR, 작성자 self-review, reviewer 미지정 |
| 선택 절차 | collaborator_self_merge, intake_and_review, local_validation, review_only_fast_pass |

[MCP 사용 매뉴얼](../../manual/mcp_hwp2024Convert_usage.md#출력-모드와-글꼴-이름-문법-보정-확인)에
2020·2024의 한 쪽씩 인쇄와 PDF 대체 저장 모드, `pdf_print_method`, 보정 응답 예시를 추가했습니다.
`comment`가 있어도 글꼴 이름 문법만 보정한 인쇄본은 Print 모드를 유지하므로, 출력 모드와 함께
판단하도록 명시했습니다. 기존 대체 저장본의 출처 확인과 전쪽 시각 검증 조건은 유지합니다.

변환기 구현·배포는 별도 `hwp-convert-2024`의 `2099bb00e31a554f718ae6696112824abc0d7cf8`에서
완료했습니다. 이번 rhwp PR에는 매뉴얼과 후속 검토 기록만 포함합니다. client `0.10.0`은
기존 `comment` 전달 기능을 그대로 사용하므로 archive 교체는 필요하지 않았습니다.

## 로컬 검증과 운영 시험

제출 head의 `git diff --check`, 변경 문서의 상대 링크·메타데이터·UTF-8 검사와 JSON 예시의
실제 응답 대조를 통과했습니다. 본인 열린 PR이 없는 상태에서 최신 base와 병합 시뮬레이션을
수행해 exit 0, tree `8acf84508512d15248eff8dd397ce73efdde4a47`을 확인했습니다.

2026-11-25 14:23 KST 운영 서비스 업그레이드 후 재시작·`Running`·인증 health `ok`, 설치
소스·worker 해시 일치를 확인했습니다. 토큰·런타임 프로필과 rollback을 보존했습니다.
새 변환 직전마다 `queue-status: idle`을 확인하고, 실제 배포 서비스에 npx client의
`start → status → download`를 실행했습니다.

| 시험 대상 | 엔진 | 성공 건수 | 각 PDF 쪽수 | 최종 출력 |
| --- | --- | ---: | ---: | --- |
| 소형 문서 | 2020·2024 | 2 | 3 | Print |
| 오류 재현 원본 | 2020·2024 | 2 | 66 | 글꼴 이름 보정 Print |
| 오류 재현 부록 | 2020·2024 | 2 | 54 | 글꼴 이름 보정 Print |
| 오류 재현 표 부분 | 2020·2024 | 2 | 44 | 글꼴 이름 보정 Print |
| 진단 문서 | 2024 | 1 | 1 | Print |
| 제주 문서 | 2020·2024 | 2 | 313 | 같은 엔진 PDF로 저장하기 |

11건의 다운로드 바이트 수·SHA-256·쪽수와 입력 보존을 확인했습니다. 별도의 QPDF 구조·참조
검사와 총 961쪽의 Poppler 24 DPI 렌더링에서 경고·오류는 0건입니다. 제주 48·49쪽의
144 DPI PNG를 두 엔진 모두 직접 확인했으며 차트가 표시됩니다. 이는 PDF 기술 검사와
지정 영역의 판독이며 원본과의 모든 시각적 일치를 보증하는 결과는 아닙니다.

매뉴얼의 보정 예시는 부록 2024 job `4b28605e-3d49-463e-805a-bf459180c4cf`의 실제 응답과
필드별로 일치합니다. 제주 2020 job `fb7dcd27-7fd7-4bc2-a17b-7189800fcf11`은 449초,
2024 job `c451a943-97b5-4002-9a99-dacf97e9e4e0`은 478초에 각각 313쪽으로 성공했으며,
각 엔진의 `*_pdf_save_as` 모드와 재저장 설명을 반환했습니다. 마지막 대기열은 `idle`이고
서비스도 `Running`이었습니다.

변환기 자동 검사는 총 102개 중 100개 통과, 설치 환경 조건부 2개 제외, 실패 0개였습니다.
운영 응답·최종 PDF·독립 검사·해시는 관리자 Downloads의 `hwp-pdf-failure-fix-20261125`에
보존했습니다. `test-summary.tsv`, `analysis.md`, `production-fixed/verification-*.json`에서
실제 배포 후 시험과 배포 전 출력 재입력 시험을 구분할 수 있습니다.

조판 원칙·검증 입력 커밋 확인·Cargo/WASM·renderer Visual Sweep은 **비해당**입니다.
rhwp source·test·fixture·baseline·CI 정책을 바꾸지 않으며, 별도 서비스의 운영 시험 자료를
새 rhwp 렌더링 회귀나 기준 PDF로 등록하지 않았습니다. 기존 시각 검증 기준도 변경하지 않았습니다.

## 제출 head의 CI와 남은 병합 조건

[CI](https://github.com/edwardkim/rhwp/actions/runs/45556282352)의 preflight와 `Build & Test`는
정확한 제출 head에서 성공했습니다. 로그의 판정은
`fast_pass=true reason=all-review-only-no-code-impact`이며 heavy worker는 범위에 따라 건너뛰었습니다.
[CodeQL](https://github.com/edwardkim/rhwp/actions/runs/45556282318),
[Adapter](https://github.com/edwardkim/rhwp/actions/runs/45556282348),
[Proptest](https://github.com/edwardkim/rhwp/actions/runs/45556282321)와
[CI Impact Policy](https://github.com/edwardkim/rhwp/actions/runs/45556333831)도 성공했습니다.
제출 head의 mergeability는 확인 당시 `MERGEABLE`·`CLEAN`이었습니다.

검토 기록 head의 [CI](https://github.com/edwardkim/rhwp/actions/runs/45556677251)도
`all-review-only-no-code-impact` 경로로 성공했으며 `Build & Test`와
[CI Impact Policy](https://github.com/edwardkim/rhwp/actions/runs/45556741631)를 확인했습니다.
승인 기록 commit에서도 required check와 실제 fast-pass를 다시 확인한 뒤 병합합니다.
후속 기록은 이미 이 PR에 포함하므로 별도 기록 PR을 만들지 않습니다.
병합 뒤 devel 동기화, 관련 이슈 상태와 PR 후속 설명, 소유·미사용을 확인한 작업 브랜치와
`output/`·`target/pr-review/debug` 정리를 [종료 절차](../../manual/pr_review/post_merge.md)에 따라
수행합니다. 병합 뒤 확정되는 값과 실제 정리 결과는 후속 설명·저장소 밖 로그로 남깁니다.

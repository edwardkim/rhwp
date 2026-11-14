# #7727·#7729·#7730·#7731·#7732 분석·검증 증적 안내

상세 원인 분석은 [단계별 분석·보정 보고서](../../../working/task_m100_7727_stage1.md)에
기록한다. 이 문서는 그 보고서와 커밋된 증적을 연결하는 진입점이다.

## 현재 확인한 결과

검증 대상은 CBTA 204쪽·권고안 22쪽·영어 시험지 8쪽·API 105쪽·각주 표 50쪽이다.
번호 문단 보정 커밋 `77f6bb74183a40ce957f043b5f35cbd2fc95700f`의 제품 소스에 대해
Native와 fresh WASM **각 389쪽 모두 98% 이상**을 확인했다.
생산자는 parent `fd9207f300`과 실제 소스 패치 SHA로 고정되어 있다.
아래 결과는 전체 회귀나 PR 제출 준비 완료를 뜻하지 않는다.

| 문서 | 쪽수 | Native 최저 | fresh WASM 최저 | 최신 TSV |
| --- | ---: | ---: | ---: | --- |
| CBTA | 204 | 98.11% (2쪽) | 98.08% (2쪽) | [Native](marker-partition/native-cbta.tsv) · [WASM](marker-partition/wasm-cbta.tsv) |
| 권고안 | 22 | 98.69% (7쪽) | 98.68% (7쪽) | [Native](marker-partition/native-recommendation.tsv) · [WASM](marker-partition/wasm-recommendation.tsv) |
| 영어 시험지 | 8 | 99.49% (3쪽) | 99.49% (3쪽) | [Native](marker-partition/native-exam.tsv) · [WASM](marker-partition/wasm-exam.tsv) |
| API | 105 | 99.55% (97쪽) | 99.55% (97쪽) | [Native](marker-partition/native-api.tsv) · [WASM](marker-partition/wasm-api.tsv) |
| 각주 표 | 50 | 98.58% (37쪽) | 98.58% (37쪽) | [Native](marker-partition/native-footnote.tsv) · [WASM](marker-partition/wasm-footnote.tsv) |

같은 원본·독립 한컴 Print PDF·정상 설치 글꼴 조건에서 96 DPI, compat2022,
Visual Sweep raw 2px·픽셀 차이 문턱 32로 측정했다. 기준 PDF 바이트와 판정 기준은
바꾸지 않았다. 회귀 검사는 별도의 Mac 최소 글꼴 환경에서 실행했다.

## 후속 주석 줄 보정의 현재 범위

각주37쪽의 `24)`·`25)`가 겹치는 추가 결함을 [분석·보정](../../../working/task_m100_7727_stage1.md#추가-판독-각주-표-37쪽의-주석-전용-줄-누락)했다.
새 제품 소스 Native 전389쪽은 모두98% 이상이며, 각주37쪽98.75%·39쪽99.92%로
두 쪽이 개선되고 나머지387쪽은 같다. [Native 증적](note-only-line/native-after.json)과
[review](note-only-line/after-native-p037-review.png)·[overlay](note-only-line/after-native-p037-overlay.png)를 보존했다.
위 양쪽 전쪽 표는 앞선 번호 문단 소스의 결과다. 새 주석 소스의 fresh WASM은 아직이다.

최소 글꼴 전체 회귀도 실행했으며 보정 전 code head에서 [13,555 PASS / 4 FAIL](full-nextest-before-note-repair.json)이다.
여백 원점·목차 오른쪽 탭은 기존 검사를 그대로 유지한 최소 글꼴 독립 실행에서 통과했다.
남은 두 실패는 번호 문단 갱신 단계부터 발생함을 [단계별 대조](full-regression-repair/full-failures-producer-bisect.json)로 확인했다.
네 실패의 보정과 새 소스 전체 검증을 완료하기 전에는 PR 준비 완료로 판단하지 않는다.

## 원인별 분석과 실제 보정

| 대상 | 확인한 원인과 보정 | 독립 근거·실행 증적 |
| --- | --- | --- |
| CBTA 96쪽 번호 문단 | 마커 높이와 줄 수가 같으면 현재 프레임의 새 문자 경계를 버리던 캐시 수용 조건을 보정했다. 원본 세로 슬롯은 보존한다. Native 97.00%→99.72%, WASM 96.99%→99.74%. | [원인·생산자·회귀](marker-partition/cause.json), [389쪽 전후 비교](marker-partition/page-comparison.json) |
| CBTA 162·166쪽 종료 행 | 근거 없이 남은 쪽 공간을 완료 행에 배분하던 처리를 제거하고, 실제 병합 칸이 소유한 종료 점유만 소비한다. 두 경로 모두 99.94%·99.95%. | [원인·전쪽 결과](complete-row-band/cause.json), [PDF 종료 선 좌표](marker-partition/cbta-pdf-complete-row-close-lines.json) |
| 권고안 4쪽 배경 | RGB 최종값 반올림과 독립 Print의 채널 변화량 절삭 차이를 공통 보간에서 보정했다. 두 경로 모두 97.44%→99.96%. | [원인·독립 인쇄 표본](gradient-channel/cause.json), [MCP 출력 영수증](gradient-channel/gradient-band-color-print-status.json) |
| 영어 시험지 낱말 공백 | 영문 상대 크기를 반각 공백의 기준 em에도 적용하던 경로를 분리했다. 원본 8쪽의 양쪽 최저 99.49%. | [원인·독립 인쇄 표본](half-space-origin/cause.json) |
| 영어 시험지 밑줄 | 글자 위치 변화와 줄 기준선의 밑줄 원점을 분리했다. 원본 장식 run은 보존한다. | [독립 한컴 Print와 전후 검증](underline-baseline/cause.json) |
| CBTA 그림 캡션·API 표·권고안 단 시작 | 소유 그림과 테두리의 동시 이동, 표 앞뒤의 중복 점유, 빈 줄과 다음 물리 프레임의 경계를 공통 소비 경로에서 보정했다. | [상세 분석 보고서의 원인별 규칙](../../../working/task_m100_7727_stage1.md#원인별-보정-규칙) |

위 표의 앞선 단계 결과는 해당 단계 생산자의 증적이다. 현재 전체 쪽수와 최저값은
`marker-partition/`의 최신 TSV를 따른다. 과거 측정을 새 head의 실행으로 바꾸어 쓰지 않는다.

## 같은 쪽의 review와 overlay

| 보정 영역 | Native | fresh WASM |
| --- | --- | --- |
| CBTA 96쪽 문자 경계 | [review](marker-partition/native-p096-review.png) · [overlay](marker-partition/native-p096-overlay.png) | [review](marker-partition/wasm-p096-review.png) · [overlay](marker-partition/wasm-p096-overlay.png) |
| CBTA 162쪽 종료 행 | [review](complete-row-band/native-p162-review.png) · [overlay](complete-row-band/native-p162-overlay.png) | [review](complete-row-band/wasm-p162-review.png) · [overlay](complete-row-band/wasm-p162-overlay.png) |
| CBTA 166쪽 종료 행 | [review](complete-row-band/native-p166-review.png) · [overlay](complete-row-band/native-p166-overlay.png) | [review](complete-row-band/wasm-p166-review.png) · [overlay](complete-row-band/wasm-p166-overlay.png) |
| 권고안 4쪽 배경 | [review](gradient-channel/native-p004-review.png) · [overlay](gradient-channel/native-p004-overlay.png) | [review](gradient-channel/wasm-p004-review.png) · [overlay](gradient-channel/wasm-p004-overlay.png) |
| 영어 시험지 2쪽 공백 | [review](half-space-origin/native-p002-review.png) · [overlay](half-space-origin/native-p002-overlay.png) | [raw PNG](half-space-origin/wasm-p002-raw.png) · [전쪽 TSV](marker-partition/wasm-exam.tsv) |

## 회귀 검사와 남은 작업

Mac 최소 글꼴 집중 검증은 **23 PASS / 0 FAIL**, 4.110초다. 선택 밖 2,778건이며
전체 회귀 결과가 아니다. [검증 범위·격리 프로필·로그 SHA](marker-partition/cause.json)에
실제 실행 근거를 기록했다. 새 번호 문단 반례는 보정 전 1 PASS / 1 FAIL, 보정 후 2 PASS다.
CBTA 118·162·166쪽에는 양쪽 98% 이상과 독립 PDF 판독에 근거한 ±2 CSSpx 보호도 추가했다.

CBTA 2쪽의 목차 쪽번호 위치는 저장 RIGHT 탭 전진과 정지점 재계산을 분리하여 분석 중이다.
아직 확정 원인이나 완료 결과가 아니며, 이를 해결 불가능한 글꼴 차이로 분류하지 않았다.
최종 전체 nextest·필수 lint/build·정본 보호 TSV·편람 gate·실제 backend 소비 검증이
남아 있으므로 **PR 제출 준비는 진행 중**이다.

임시 진단기·실행 파일·로그·MCP 대기 자료는 저장소의
`output/pr-review/issues-7727-7732-20261114/`에 둔다. 확정된 분석과 제출 증적은
이 디렉터리와 단계별 보고서에 커밋하고, 임시 산출물과 비공개 글꼴은 커밋하지 않는다.

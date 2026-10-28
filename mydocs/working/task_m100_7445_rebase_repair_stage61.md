# #7445 리베이스 보정 61단계: 빈 저장 줄의 종료 간격과 가운데 정렬

## 원본·계보·원인

기존 가맹사업 보고서와 동일 입력의 독립 한컴 2020 Print 142쪽을 사용했다. 원본 및 #5966·
#6718의 선행 해결 계보는 59·60단계와 같다. 중첩 컷의 정렬 계산은 기존
`b56b6b50f3`가 원본 유닛 모듈로 옮긴 공통 helper다. #7870의 중첩 atom 종료 여백 계약도
함께 읽고 현재 유닛·측정·실제 배치를 대조했다.

70쪽 표 650의 행1·열1은 문단 7부터 이어받아 문단 14의 빈 줄에서 이번 조각을 닫는다.
다음 문단 15는 저장 vpos가 0으로 재시작하는 다음 쪽 소유자다. 문단 14의 원본 줄 높이 600HU와
뒤 간격 360HU가 한 유닛 12.8px로 계상돼 있다. 마지막 줄이 `empty_spacer`라는 이유만으로
정렬용 뒤 간격 분리를 건너뛰어 내용 높이에 4.8px를 더하고 전체 본문을 2.4px 위로 옮겼다.

`nested_cut_terminal_line_spacing`의 실제 저장 줄·단일 줄 범위·원본 출처·유닛 높이와 저장
전진량 일치 조건을 사용한다. 빈 문자열이라는 이유로만 이 공통 분리를 막지 않도록 보정했다.
600HU의 빈 줄 상자·유닛 수용·문단 소유는 유지하며360HU의 다음 줄 전진만 정렬 높이에서
분리한다. 문서ID·쪽 번호·새 물리 여유값은 구현에 추가하지 않았다.

## 실제 전쪽 검증

Print/compat 2022, 96DPI, raw 2px 실루엣, 같은 명시적 Human TrueType·Haansoft fallback과
PDFium 153 조건이다. Mac fresh WASM은 locked web/pkg/no-opt 로컬 대체 빌드다.

| 출력 | 전쪽 수 | 최저 raw | 미달 | 68쪽 | 70쪽 |
| --- | --- | ---: | --- | ---: | ---: |
| Native | 142/142 | 92.75649% | 0쪽 | 100% | 100% |
| fresh WASM | 142/142 | 92.75394% | 0쪽 | 100% | 100% |

같은 backend의 고정 base 전 142쪽도 대조했다. Native 0.06510%p / WASM 0.01892%p가 최대 하락이며
0.1%p 이상 후퇴는 양쪽 모두 0쪽이다. 고정 base Native의68·70쪽 미달과 WASM의50~54·68쪽
미달을 숨기거나 서로의 TSV로 대체하지 않았다.

Native는 전 142쪽 중70쪽 PNG만 바뀌고 TextRun의 내용·순서는 모두 같다. 양쪽 backend의70쪽
review·overlay를 직접 판독했다. 68쪽은 직전 직접 판독 PNG와 두 backend 각각 SHA가 같아
producer/hash 근거로 판독을 재사용했다. 수치100%는 RGB byte 동일성을 뜻하지 않는다.
루트pkg·Studio public·frozen pkg의 JS/WASM SHA도 일치한다. 실제 browser 초기화 응답과
최종 소비 경로 전체 검증은 아직 남아 있다.

## 중요한 위치의 회귀 보호와 검사 자체 검증

두 backend 전쪽 90% 이상 및 두 쪽 각각 100%·직접 판독 이후, 기존 Native integration 모듈에
두 보호 검사를 추가했다. 원본 첫 저장 앞 띠와 현재 조각의 마지막 빈 줄 상자 소유를 관계로
검사하고 독립 Print의68쪽 796.959961px·70쪽 155.84px 기준선을 ±2 CSSpx로 보호한다.
공통 공개 RenderTree를 사용하며 source-side test나 새 test-only API는 추가하지 않았다.

새 검사에는 전쪽 비교와 같은 명시적 글꼴 환경을 지정했다. 기존 6건의 기본 대조 환경은 별도
캐시에 보존했다. CLI의 같은 두 셀 TextLine 원점·높이가 실제 Native PNG 생산자의 RenderTree와
동일함을 확인했다. 현재 조각 끝을 셀 전체의 마지막 문단으로 오인한 검사 초안은 수정했고,
원본 본문 내용과 다음 저장 재시작으로 조각 끝을 정한다.

새 두 검사를 같은 test binary에서 변경 전 frozen Native로 직접 실행해 각각 실제 FAIL을
재현했다(exit101,1.122초/1.154초). 실패는 앞·뒤 띠 관계이며 CLI 실패가 아니다.
수정 후 현재 Native의 모듈 8건은 nextest에서 8 PASS, 2.784초다. prepare 후 suite는 022로
현재 매핑에서 다시 도출했다. 실행 중 배정이 바뀐 과거 target 번호를 재사용하지 않았다.
초기 진단의 환경 누락·지원하지 않는 CLI 옵션·잘못 고른 전체 셀 끝은 최종 실패/통과 근거에서
제외하고 오류 로그도 ignored output에 보존했다.

## 잔여와 제출 범위

새 검사 추가 전의 집중 23건은 8 PASS / 15 FAIL이었다. 새 8건 통과를 최신 전체 회귀 통과로
보고하지 않는다. 남은 15건 보정·최신 전체 nextest·필수 lint/build/Skia/실제 소비 경로 확인 후
PR을 진행한다. HWP 보정 대상을 확대하지 않았다.

정식 전쪽 TSV·대표 PNG는
`mydocs/pr/assets/issue7445/completed-20261009/rebase-605ccfd9-20261022/empty-closing-line-alignment/`다.
source·입력·PDF·글꼴·빌드 hash, base/후보 전쪽 비교, 실제 before/after 검사 및 로그는 ignored
`rebase-8155-20261022/stage61-empty-closing-line-alignment/`에 보존한다.

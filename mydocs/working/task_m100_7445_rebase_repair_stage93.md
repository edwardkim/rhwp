# Stage93: 다단의 독립 저장 축과 간격 복원 불가 판정 분리

Issue: #7445. 부모 HEAD `2c0e61e8cb247f1b2e244090f89652be0fca8593`, 고정 base
`d9fd85ed5bc3c4d9ce6b2981422e48bf9bb90867`이다. 기존 50조건의 보정 범위를 확대하지 않았다.
단축키와 시험지는 이 브랜치에서 이미 보호하던 완료 문서의 기능 보존 대조군이다.

## 전체 회귀가 검출한 실제 후퇴

Stage91 전체 nextest는 **11756 실행·11730 PASS·26 FAIL·45 skip**, 실제 검사 875.204초,
명령 전체 1376.521초·exit100으로 종료했다. 실패는 단축키의 7→9쪽 및 시험지 계열의
쪽수 증가였다. 최종 목록에 다시 출력한 FAIL을 중복 합산하지 않았다. 이전 source의
전쪽 캡처와 대기 중인 실제 소비 경로 검증은 소유 PID를 확인해 중단했고, PNG·TSV·로그는
그대로 보존했다. 전체 검사는 중단하지 않고 실패 목록까지 수집했다.

단축키는 원 이슈 #702·PR #710과 후속 7쪽 계약을 추적한
[Stage21](task_m100_7445_rebase_repair_stage21.md)을 다시 대조했다. 원 PR의 과거 8쪽 결과로
현재 독립 `pdf/basic/shortcut-2022.pdf`의 7쪽 검사를 완화하지 않았다. 시험지의 기존 원본·PDF
해시와 21/22쪽 계약도 유지했다. 새 회귀나 fixture·골든·예외를 추가하지 않았다.

## 공통 원인과 수정

Stage87이 `spacing_restore_unavailable`을 다단에서도 소비하게 하면서, 실제 빈 float lane의
복원 차단과 **전역 본문 축이 미확립인 상태**를 같은 이유로 취급했다. 그러나 다단 fit은
단별 저장 글줄 축을 사용한다. 단축키의 204회, 시험지 원본의 410회 전진에서 단 수 2·전역
page/lazy 축 None·dirty false인 상태를 관측했다. 단축키는 fit 13.3px 대신 20px씩 누적했다.

`paragraph.rs`의 다섯 소비 지점이 같은 `spacing_restoration_unavailable` 질의를 쓰게 했다.
전역 축 미확립은 단단에서만 복원 차단으로 사용하고, dirty 사다리·다음 빈 float lane·후속
스냅 철회는 계속 보존한다. metrics의 유효 저장 글줄 판정과 합성 사다리의 full advance도
유지한다. KTX에 필요한 실제 간격을 버리지 않고 일반 다단의 fit 장부를 복구한다.
수용 전진·원점 역산·atomic overflow·빈 줄 분할의 다섯 경로가 이 공통 판정을 소비한다.
실제 paint의 단별 저장 좌표를 임의 이동하거나 문서명·쪽 번호별 분기를 추가하지 않았다.

## 집중 검사와 독립 Print 전쪽 비교

집중 **23 PASS / 0 FAIL**: 기존 다단 5건, KTX/합성 대조 3건, 단축키 편집 8건, 원본 2건,
시험지 원본·변형 5건이다. 기존 26실패 중 11건의 동일 assertion이 통과했으며 정상 12건도
유지했다. 나머지 실패와 전체 정상 범위는 새 head 전체 회귀에서 다시 확인해야 한다.
21쪽 HWP/HWPX뿐 아니라 원래 22쪽이어야 하는 HWP/HWPX 두 변형도 통과했다.

| 기존 보존 문서 | Native 쪽수·최저 raw | fresh WASM 쪽수·최저 raw |
| --- | ---: | ---: |
| KTX | 1/1·99.36494% | 1/1·99.38975% |
| 단축키 | 7/7·96.75859% | 7/7·96.74949% |
| 3-11월 시험지 HWP | 21/21·97.52483% | 21/21·97.55326% |
| 같은 시험지 HWPX | 21/21·97.58048% | 21/21·97.61948% |
| 과학 시험지 | 4/4·95.84398% | 4/4·95.78187% |

양 backend 각54쪽·총108쪽에서 미달·누락·쪽수 불일치가 없다. 현행 96DPI·PDFium smooth RGB·
raw2px90%를 유지했다. 과거 192DPI 수치를 현재 수치와 직접 증감 비교하지 않았다.
단축키 7쪽의 두 단 꼬리·단축키·장식 번호, 시험지 HWP/HWPX 9쪽의 문제 29/30·수식·그림·
보기 상자, 과학 3쪽의 분수 셀·뒤 본문을 Native/fresh WASM review/overlay에서 직접 판독했다.
얇은 획·글리프 가장자리·일부 미세 기준선 차이는 남으며 엄격 RGB 동일성을 주장하지 않는다.

[전쪽 TSV·입력/PDF/생산자·검사·대표 review/overlay](../pr/assets/issue7445/completed-20261009/rebase-d9fd85ed-20261023/multicolumn-axis-preservation/validation.json)를 보존했다.
Native 빌드 111.271초, fresh WASM 205.792초는 exit0이다. Native SHA는
`abff5056fe395c6527dc6d8528c22c42281c9947d5f4bf6bc3e3ccc62fca06a6`, WASM SHA는
`4af5842b5d9c612d4b30caea794a6666c0a32c728d442ddcd68693a81b567ce4`다.
WASM은 Mac locked wrapper web/pkg/no-opt 로컬 대체 빌드이며 루트 pkg·Studio public과
hash가 같다. Docker 최적화나 실제 브라우저 초기화 응답 검증을 대신하지 않는다.

## 다음 검증

이번 집중 복구를 새 head의 전체 통과로 확대하지 않는다. 기존 50조건 전쪽 Native/fresh WASM,
전체 nextest·필수 lint/build/Skia·실제 소비 경로를 마친 뒤 PR·CI·merge·후속 처리를 진행한다.
기대값·검사·지침·원본·PDF·사용자 초안은 변경하지 않았다. 종료된 과거 빌드의 동일 바이트만
hardlink로 중복 제거했고 입력·PNG·TSV·로그·공유 target은 보존했다.

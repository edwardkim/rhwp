# #8153: 원본 일반 행의 여러 내용 프레임과 물리 잔여

이 단계는 두 편람의 HWP 317·327·353쪽에서 각각 남은 9픽셀을 보정한다. 원 해결
[#7764](https://github.com/edwardkim/rhwp/issues/7764), 실제 통합 `5d22772bf1`와
[#8191](https://github.com/edwardkim/rhwp/pull/8191)의 추적은
[이어받기 선행 분석](issue_8153_ordinary_row_continuation_frame_20261026.md)을 따른다.
첫 상자의 별도 보정 `cb8d552a7d`와
[그 계획](issue_8153_ordinary_row_opening_frame_20261026.md)도 대조했다.
첫 상자 수용과 그 뒤의 물리 잔여 생산은 다른 단계다. 원본·기준 PDF·측정값은 바꾸지 않는다.

## 원본과 실제 호출 증거

고정 Native `0fcbd1e42e`·표준 최적화 fresh WASM `70e060edad`는 두 문서 각각 383쪽이다.
전쪽 최저 raw는 각각 99.96446%·99.96402%이며 두 원본의 관련 쪽 모두 98% 이상이다.
HWP 317·327·353쪽의 세로선은 가로선 전에 끝나지만 HWPX와 독립 Print는 연결된다.
Native·독립 Print의 양 끝 확대와 실제 정수 미대응 좌표에서 확인했다.

section 11 / para 3의 103행×2열 원본 표를 HWP 압축 record와 HWPX XML에서 직접
대조했다. 206개 셀의 크기·span·여백·문단 수·모든 저장 LineSeg 필드가 같다.
parsed model의 원본 크기도 같다. 형식별 raw 보존 메타정보가 다른 것을 기하 차이로
설명하거나, HWPX의 높은 점수만으로 HWP를 완료 처리하지 않는다.

기존 `RHWP_DIAG_NESTED_FRAME`·`RHWP_DIAG_ROW_TAIL`로 실제 Print의 312·317·327·353쪽을
추적했다. 진단 출력 PNG는 동결 전쪽 PNG와 RGB 전체가 같았다. 내용 컷도 두 형식에서 같다.
317쪽의 같은 `row12, start=[4,4], end=[17,27]`은 다음처럼 전달된다.

| 원본 | 이어받은 높이 | 실제 끝 행 상자 | 반복 머리행 |
| --- | --- | ---: | ---: |
| HWP | `None` | 483.106666…px | 60.64px |
| HWPX | `Some(1985.573333…)` | 489.88px | 60.64px |

HWP의 직전 `cursor11→end13, cut=[4,4]`에서 다음 물리 잔여는 `None`이며,
HWPX는 1985.573333…px다. 따라서 paint의 선만 늘릴 문제가 아니라 잔여 생산에서
같은 source 최소·수용 원장을 이어받는지 확인해야 한다. 독립 Print의 표 끝
644.263997px나 위 차이를 제품 보정 상수로 넣지 않는다.

## 공통 값의 생산·소비와 보정

`원본 셀 최소·저장 내용 컷 → scan의 수용 row box → emit의 pending_row_box 누적 →
stored_ordinary_row_remainder → continuation.start_row_height_override → 다음 scan의
stored_ordinary_row_end_frame → end_row_height_override → table_partial의 실제 행·세로선`
을 같은 원장으로 연결한다.

현재 잔여 helper는 **남은 내용이 한 저장 프레임 안에 있는지**를 물리 잔여 전달 전체의
입구 조건으로 사용한다. 원본 일반 셀·저장 컷이 유효하고 수용 상자를 실제로 차감할 수
있어도 내용이 여러 프레임이면 거절한다. 첫 상자의 `cb8d552a7d`와 같은 잘못된 가정이
별도의 잔여 생산자에 남아 있다.

단일 프레임 조건은 독립적인 **종료 텍스트 프레임** 선택에 유지한다. 여러 내용 프레임의
일반 행에서는 검증된 선언 최소 또는 이어받은 물리 상자에서 같은 행의 실제 수용분만
차감한다. 종료 텍스트 프레임·마지막 줄간격 흡수·큰 독립 최소·새 행의 소유는 유지한다.
문서·쪽·행 번호나 형식별 예외를 제품 조건으로 쓰지 않고 공통 helper를 보정한다.

## 검증과 완료 조건

먼저 정식 `tests/cases/issue_8153_ordinary_row_physical_remainder.rs` 원문을 동결 이전
library에서 실행해 HWP의 독립 ±2px 끝점 FAIL과 HWPX 대조군 PASS를 확보한다.
앞뒤 내용 컷·행/셀 소유·원본 HU·누락/중복·세 열 경계도 함께 보호한다.
보정 후 같은 원문과 기존 opening·continuation·그림 자원 검사를 실행한다.

312·316~320·326~328·344~345·352~354쪽과 앞뒤 닫힘을 실제 Native로 먼저 대조한다.
한 쪽의 종료선만이 아니라 조각 상자·다음 내용·전체 383쪽·빈 쪽·내용 누락과 중복을 확인한다.
후퇴가 없을 때 Native 전쪽과 표준 최적화 fresh WASM을 재검증한다. 미실행은 완료가 아니다.
현재 1,532조건 raw 100%·미대응 0의 목표와 최종 전체 회귀·필수 검증은 유지한다.

증거는 ignored `output/pr-review/issue8153-handbook-raw100-20261022/remaining-raw100-diagnostic/hwp-only-row-ends/`
의 `source-cell-size-comparison.json`, `source-line-record-comparison.json`, parsed model,
`actual-paint/`, `unmatched-source-endpoints.json`, `p317-endpoints-review.png`에 있다.
전체 source/backend 원본 TSV는 그림 자원 단계의 `combined-whole-result.json`에 연결한다.
upstream 동기화·원격 GitHub 변경 없이 PR 준비까지만 진행한다.

## 전쪽 반례와 물리 프레임 소유의 구분

`1c7f742a9e`는 정식 nextest 14건과 집중 Native 60조건을 통과했지만, 전쪽 HWPX
223쪽에서 새 회귀가 확인됐다. raw 100%·미대응 0에서 75.15644%·37,120픽셀로
하락했다. 같은 내용 컷 `[1,10,1] → [1,44,1]`의 물리 잔여가
1066.253333…→1069.973333…px, 이어 329.066666…→330.52px로 늘었다.
마지막 행 상자의 1.453333…px 증가가 후속 빈 문단 수용을 바꾸어 본문을 14.786666…px
밀었다. 이 후보는 완료·제출 가능 상태가 아니며, 14건 통과를 전쪽 무회귀로 대신하지 않는다.

원본은 section7 / para239 / 3행×7열 표다. 일반 행의 가운데 칸은 열 1~5를 병합했지만
rowspan은 1이고, 저장 60줄의 재시작은 문단 7→8, 28→29의 경계다. 각 새 문단은
200HU에서 시작한다. 셀의 97,729HU 선언과 이 문단 경계 사이의 전진 합은 같은
물리 공간 원장이 아니다. 반면 법령 대조표의 `[4,4]`, `[7,7]`, `[10,10]` 컷은 같은
원본 문단의 줄이 0HU로 재시작하는 경계다.

다중 프레임의 새 잔여 발행은 남은 프레임 수가 아니라 **현재 컷의 원본 물리 프레임 소유**로
인증한다. `stored_in_paragraph_unit_is_original_restart`가 검증하는 원본 줄·문자 증가·
문단 내부의 양수→0 재시작을 재사용한다. 문단 사이 로컬 좌표를 선언 전체에서 차감하는
근거로 쓰지 않는다. 이미 수용한 물리 잔여의 전달과 기존 단일 종료 프레임 계약은 보존한다.
페이지 번호·열 수·colspan·문자열을 제품 분기로 사용하지 않는다.

전쪽 반례 PNG와 변경 tree·실제 emit 로그는 ignored `hwp-only-row-ends/p223/`,
`p223-new-regression-review.png`, `p223-render-tree-changes.json`에 보존했다.
이 반례의 독립 좌표·내용 순서 검사는 먼저 output의 제안 파일로 실행하며, 실제 보정과
Native/fresh WASM 시각 선행 조건을 확인한 뒤 정식 `tests/cases/`에 반영한다.

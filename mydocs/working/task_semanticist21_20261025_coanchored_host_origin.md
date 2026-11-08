# 같은 호스트에 앵커된 표의 글줄 원점 재검토

## 독립 근거와 원인

#1510 → PR #1518(`42d7f6bc2cce2ab3169892367669154e7a0291aa`),
#1535 → PR #1548(`350b46f80edcf5c5f3fe92b4270eaae97cbb1b81`)의 본문·논의·diff를
확인했다. 원 검사는 형제 표의 순서·배제를 보호하며, 새 한컴 Print는 제목과 후속
본문도 앞선 표가 점유한 흐름 아래에 보존해야 함을 보여 준다.

고정 base `9418ed0c7cf289c5c4ac2b3bb8cdc18a1b637a9a`와 후보
`317241176d7186eaae5dbf89b6e7f34a3ece2ff0`의 비교에서 #1510 첫 쪽 raw는
89.51780% → 70.11639%, #1535는 97.19373% → 85.82939%로 하락했다.
같은 입력의 현재 fresh WASM은 각각 69.73147%, 85.33595%다.
base 미달을 이유로 현재 회귀를 이관하지 않고 구현을 보정한다.

`from_computed_host / from_computed_head_host → ParagraphFloatPlacement.stored_host_origin →
layout.rs의 글줄 원점 적용`이 확인할 소비 경로다. 양수 오프셋 표 하나의 위에
제목이 들어간다는 사실이 다른 앞쪽 형제 표 위에도 들어간다는 증거는 아니다.
현재 후보는 앞 형제 표가 흐름을 전진시킨 뒤 제목을 처음 원점으로 되돌린다.
표의 앵커와 글줄 전체의 확정 원점을 분리하고, 같은 문자 위치에 형제 표가 있는 제어문자의
단독 프레임으로 전체 글줄 원점을 인증하지 않는다. 실제 줄 위치는 형제 표를 모두
소비한 기존 흐름 결과가 정한다. 문서 이름·고정 좌표 보정은 추가하지 않는다.

## 진행과 검증

작업 로그와 전쪽 TSV는 ignored
`output/pr-review/semanticist21-20261019/remaining-visual-3842/`에 있다.
구현 뒤 기존 #1510·#1535 관계 검사와 정상 단일 표·복수 양수 오프셋·글 끝 앵커
대조군을 확인하고, Native/fresh WASM 전쪽·대표 review/overlay를 재산출한다.
90% 선행 조건을 충족하기 전 새 렌더링 회귀를 등록하지 않는다.
#6133의 현재 전쪽 raw 100%와 별개인 용지 경계 진단은 독립 PDF와 따로 대조한다.

## #6133 축약본의 의도된 경계와 래칫 원장

원 해결 PR #6169와 통합 #6191의 원본 4쪽 계약을 유지한다. 별도 IR 축약본은
단 정의가 없어 한컴이 기본 용지로 여는 2쪽이며, 새 Print의 괘선 자체도 종이
오른쪽 밖까지 이어진다. 독립 PDF의 실제 벡터 연산에서 96 DPI 환산 종이 오른쪽은
793.33333px, 제목 괘선 오른쪽은 796.21200px다. 현재 렌더의 off-canvas 한 건은
같은 제목 표의 오른쪽 경계이며 글자를 새로 숨긴 회귀가 아니다. 두 번째 쪽은 원
기여자가 이미지를 1×1 흰색으로 바꾼 빈 그림 쪽이다. 이 원본 개체의 선언 높이가
기본 본문 높이보다 큰 것도 source 계약으로 유지하고, 보이지 않는 그림의 픽셀
원점을 PDF에서 읽었다고 주장하지 않는다.

새 한컴 Print와 현재 Native/fresh의 전쪽 raw는 모두 100%다. 기존 정식
`verified_mixed_float_tac_host_line`은 제목·그림의 유일한 소유, 독립 2쪽 수,
같은 크기의 유색 그림 대조군과 원본 4쪽의 포스터도 보호한다. 따라서 이 입력을
제외하거나 출력 좌표를 맞추지 않고 body-overflow와 off-canvas 원장에 실제 원본
상태인 1건씩을 명시한다. 현재 실측값만 보고 원장을 올리는 처리가 아니라
독립 Print·원본 개체·관련 정식 검사를 연결한 의도된 변화 판정이다. 다른 원장
행과 진단 공차는 유지한다.

- 입력 SHA-256: `1d2ecde2a4b34bf9abc5cfab8551f9b6bfc417d9013504dcdbbb07a41a62e4e8`
- Print PDF SHA-256: `b9935c0f2e3fd70802c29f5f4694576aa64b840497c577af4d3442100fd481e3`
- 입력: `samples/issue6133/156483831_poster_title_above_offset_float.hwp`
- Print: `pdf/issue6133-mixed-host-line/original-slice-2020.pdf`
- 전쪽 TSV: `remaining-visual-3842/6133-candidate-{native,wasm}/silhouette.tsv`
- 독립 연산: `remaining-anomalies-3841/6133-independent-pdf-geometry.json`

이는 #1510·#1535의 실제 글자 겹침과 구분한다. 그 두 입력은 원장을 올리지 않고
구현을 고치며, 보정 뒤 현재 head의 필수 검증을 계속한다.

### 실제 제어문자 위치 확인과 첫 가설의 기각

첫 Native233 후보는 글 앞 제어문자만 처리해 출력이 바뀌지 않았다. 이를 수용하지
않았다. 실제 IR을 다시 읽은 결과 두 입력의 제목은 23자이며 표 3개의 제어문자는
모두 글 끝 위치 23에 있다(`logs/coanchored-metadata-3846.log`). 원인은 글 앞/뒤
자체가 아니라 같은 위치의 여러 표 중 하나로 전체 글줄 원점을 인증한 것이다.
두 경로가 공유하는 `from_computed_host_at`에서 같은 논리 앵커의 소유를 판정하도록
보정하며, 다른 문자 위치·TAC·overlay의 소유까지 기계적으로 묶지 않는다.

### 공통 앵커 보정의 실제 결과

Native234 빌드는 exit 0이다. 같은 원본의 글자 겹침은 #1510의 3건과 #1535의
1건이 모두 0건이 됐다. 두 입력은 2쪽을 유지한다. 전쪽 Native 최저 raw는
#1510 89.53401%, #1535 97.23162%, #6133 축약본 100%다.

정식 focused nextest는 **113개 PASS / 0개 FAIL**, exit 0이다. #1510·#1535의
기존 관계 검사, #6950의 단독/글 끝 앵커, 형제 표 여백, #6133의 18개 정식
계약과 body·off-canvas·text-overlap 전수 partition을 포함했다.
`logs/coanchored-origin-focused-3850.log`와 `remaining-visual-3849-result.json`을
연결한다. 이 결과는 전체 Rust와 최종 fresh WASM 검증을 대신하지 않는다.

#1510은 아직 90% 미만이므로 PR 제출이나 새 렌더링 회귀 등록을 하지 않는다.
다음 보정은 비양수 오프셋 형제 표 사이의 흐름이다. 현재 측정과 배치의 이전 분기는
호스트 글줄의 줄간격을 표 사이에 예약하는데, 같은 입력의 Print는 각 표의 바깥
아래·위 여백을 한 번씩 소비한다. 단일 계산 글줄·같은 논리 앵커의 비양수 형제는
기존 `NextLine` 프레임으로 실제 표 상자와 여백을 확정하고 측정·배치가 이를 함께
소비하도록 조사한다. 여러 글줄의 혼합 호스트는 그 자체의 앞줄/뒤줄 소유를 유지한다.

### 뒤쪽 helper가 프레임을 다시 덮어쓴 경로

Native235에서 새 프레임의 표 상단은 전달됐지만, 최종 `FLOW_ORIGIN` 로그의
flow는 `NextLine`이 아닌 `Exclusion`이었다. 생산 함수 이름으로 결과 공유를
판정하지 않고 실제 `from_computed_host_at → with_tail_line_space →
HostPlacementConstraint → record_paragraph_float_placement → paint`를 추적했다.
남은 가로 폭 helper가 이미 확정한 흐름을 무조건 배제로 재분류해 측정은 예전
줄간격을 계속 예약했다. 다른 소유 규칙이 확정한 흐름은 보존하고, 가로 폭은
기하 배제 프레임을 흐름으로 전환할 때만 사용하도록 공통 helper를 보정한다.
단독 글 끝 표의 폭 부족 판정은 그대로 보호한다.

이 문제가 확인된 WASM067 빌드는 성공으로 세지 않고 중단했다(exit 143).
Native235 raw 83.67852%도 실패 증거로 보존하며 제출 TSV로 사용하지 않는다.

### 글줄 소유가 다른 흐름의 구분

Native236 후보의 focused 검사는 104 PASS / 9 FAIL이었다. 기존 글 뒤 표에
일반 `NextLine`의 하단을 즉시 소비하도록 한 paint 변경이 #6950과 정상 대조군을
후퇴시켰다. 이 결과를 기준값 갱신으로 숨기지 않았다.

기존 `NextLine`은 글줄 뒤 개체가 남은 가로 폭에 들어가지 않는 계약이다. 새로
다루는 경우는 같은 계산 글줄의 형제 표 본체가 호스트 글줄보다 먼저 공간을
차지하는 계약이다. 같은 이름으로 양쪽의 글줄 소유를 추정할 수 없어
`CoanchoredTable`로 표현한다. 생산한 프레임이 측정의 표 사이 전진과 실제
배치, 호스트 문단 닫기에 전달되고, 기존 `NextLine`의 소비 시점은 유지한다.
여러 조건을 더해 특정 문서만 고르는 대신 다른 소유 규칙을 명시적으로 구분한다.
재검증 전에는 이 구분이 기능을 보존했다고 판정하지 않는다.

Native237에서 기존 9개 회귀는 해소됐으며 112 PASS / 1 FAIL이었다. #1510의
filler 30이 독립 Print와 달리 첫 쪽에 남는 실패와 전쪽 raw 68.55556%를 확인했다.
첫 쪽은 95.87937%였지만 둘째 쪽의 내용 이동을 평균으로 상쇄하지 않았다.
실제 표 사이 위·아래 여백은 Print에 맞아졌으나, 양수 오프셋 표의 배제 밴드가
본체 위끝에서 시작해 위 바깥여백을 누락했다. 본체 직전의 한 줄이 잘못 들어갔다.
같은 프레임의 `exclusion_top`에서 원문 위 여백을 포함한 점유 위끝을 만들고
측정과 배치가 이를 함께 소비하도록 보정한다. 글줄을 숨기거나 고정 좌표로
다음 쪽에 보내지 않는다.

### 원본 줄 메트릭과 인쇄 경로별 유지 계약

HWP 원본의 새 MCP 2020 Print도 1쪽이며, filler 07은 표 위에 있다. 실제 한컴은
11.0.0.9136, `pdf_print_method=0`, `hancom2020_pdf_driver_one_up`이다. 새 PDF의
SHA-256은 `106b80dee1437144245206c3bc69ebcaca1e883f7cc5031504bddab338bd6aa0`이다.
이 기준과 고정 base의 raw는 88.43907%, 앞 후보 Native234는 88.45205%, Native238은
86.58306%였다. base 미달이라는 이유로 현재 하락이나 정확한 기존 검사를 숨기지 않는다.

원본을 바꾸지 않은 메모리 내 대조에서 다음을 확인했다.

| 대조 | HWP 최저 raw | HWPX 최저 raw | 해석 |
| --- | ---: | ---: | --- |
| 원본 그대로 | 86.58306% | 97.41163% | 실제 CLI와 같은 제어 출력 |
| clone만 다시 로드 | 86.58306% | 97.41163% | clone·재구성 자체의 효과가 아님 |
| 호스트 줄 메트릭만 명목 글자 높이로 투영 | 100% | 97.41163% | HWP 제목 줄의 과거 strut이 뒤 흐름을 밀었음 |
| 셀 줄 메트릭만 투영 | 84.78886% | 100% | HWP 셀의 저장 물리 프레임을 HWPX와 같게 바꾸면 안 됨 |
| 호스트·셀 모두 투영 | 97.81215% | 100% | 형식별 셀 계약을 구분해야 함 |

이는 원본 입력의 최종 통과 선언이 아닌 원인 분리 실험이다.
`logs/line-metric-probe-score-3867.log`에 실행과 결과를 보존한다.

공통 render-only 정규화에서 원본 문자 크기·언어 상대 크기와 Percent 줄간격이
같은 명목 높이를 증명할 때만 초과된 단일 저장 줄 메트릭을 투영한다. 글자처럼
취급하지 않는 표만 가진 호스트는 표가 글줄의 문자 높이를 증명하지 않는다.
HWPX의 plain 셀 캐시도 같은 메트릭 함수를 소비하되, HWP 셀의 저장 strut은 유지한다.
명시적 개행·여러 줄·실제 인라인 개체·번호·장식 글자·고정 줄간격 등 다른 높이
소유자는 유지한다. 원본 IR·저장 바이트와 줄의 문자 소속·너비·원점은 바꾸지 않으며,
같은 투영 문단을 측정·페이지 수용·실제 배치가 소비한다.

### 메트릭 투영의 첫 실제 검증과 안전성 보완

Native239에서 입력을 바꾸지 않고 CLI로 다시 출력한 4개 원본의 전쪽 raw는 모두
100%였다. HWP #1510 1쪽, HWPX #1510 2쪽, HWPX #1535 2쪽, HWP #6133 2쪽으로
각 독립 Print의 쪽수와도 일치했다. `remaining-visual-3869-result.json` 및 각
`silhouette.tsv`를 연결한다. 집중 정식 검사는 113 PASS / 0 FAIL, exit 0이었다
(`logs/coanchored-text-metrics-focused-3868.log`). fresh WASM의 최종 통과는 아직 아니다.

새 HWP Print는 정식 경로
[`pdf/issue1510/issue1510_coanchored_float_tables-hwp-2020.pdf`](../../pdf/issue1510/issue1510_coanchored_float_tables-hwp-2020.pdf)에
보존한다. 입력 SHA-256은
`66399b3ff7fbe8410a1996788702208be3b732b8ec96aff6b0ec8160a66e2d55`, PDF는 위
`106b80d…` 전체 해시와 같다. 14,757바이트, 1쪽, MCP job
`3fb866a3-045b-4370-a573-b5c3ada8249f`이며 원본 내용은 수정하지 않았다.

첫 보정 뒤의 코드 검토에서 다음 경계도 함께 보완한다. 여러 글자모양의 주소는
문자별 원본 주소가 완전할 때만 수용하고, 글자 크기의 단위 환산은 스타일을 실제로
해소한 DPI를 사용한다. 편집 여부는 문서 전체가 아닌 해당 문단의 저장 줄 유효성으로
판단해 무관한 문단을 편집할 때 다른 유효한 줄의 투영을 잃지 않는다. 셀 메트릭만
바뀐 문단은 기존 본문 조판을 재사용하고, 실제 본문 메트릭이 바뀐 문단만 다시 조판해
무관한 본문 번호 투영을 버리지 않는다. 이 보완은 `logs/coanchored-normalization-focused-3870.log`에서
재검증한다. 원본 IR과 저장 캐시는 보존한다.

안전성 보완 뒤의 Native240도 같은 4개 원본·7쪽의 전쪽 raw 100%와 PDF 쪽수
일치를 유지했다(`remaining-visual-3872-result.json`). 집중 정식 검사는 다시
113 PASS / 0 FAIL, exit 0이다(`logs/coanchored-normalization-focused-3870.log`).
기존 정확한 검사나 허용 오차를 완화하지 않았다. 같은 코드의 표준 release fresh
WASM068 빌드는 `logs/rebase-candidate-wasm068-build-3873.log`에서 진행하며, 완료와
브라우저 실제 출력 확인 전에는 최종 통과나 PR 준비 완료로 판정하지 않는다.

본문 줄 메트릭의 생산·소비는
`stored_text_metrics::normalize → compute_render_normalized의 투영 문단과 필요한 본문 compose →
TypesetEngine의 수용·배제 프레임 → LayoutEngine의 같은 문단·프레임 배치`로 이어진다.
`CoanchoredTable`의 실제 점유 하단은 표 사이 전진과 문단 종료가 함께 소비하고,
양수 오프셋 표의 위 바깥여백은 `exclusion_top` 한 함수에서 측정·paint 양쪽에 전달한다.
기존 글 뒤 `NextLine`의 호스트 줄 소유는 바꾸지 않는다. 정상 대조군은 #6950의 글 끝
앵커·단일 표·여러 줄·폭 부족과 #6133 원본·축약본이며 위 113개 실행에 포함했다.

### fresh WASM의 완료와 새 이름의 정식 회귀

코드 `ca4420fad6d974931aafdfbc1a803d729611967c`의 표준 release WASM068은
실제 wasm-opt 최적화까지 exit 0으로 완료했다. 빌드 중 source hash는 불변이며
그 코드 commit의 1,600개 런타임 파일과 일치한다. Native240과 fresh WASM068은
4개 원본·7쪽 전쪽 raw 100%, 독립 Print와 쪽수 일치를 각각 확인했다.
`logs/after-wasm068-3875.log`와 `remaining-visual-3875-result.json`을 연결한다.

#1510 HWP/HWPX와 #1535의 글자·표 경계·여백·이어받기 영역을 직접 대조했다.
초기 Native239와 안전성 보완 뒤 Native240의 7쪽 PNG는 바이트 단위로 같다.
[선행 증거](../pr/assets/issue1510/coanchored-host-flow-20261025/README.md)에
원본 TSV와 대표 review·overlay를 보존했다. 이 자료는 뒤의 #6970·#7158 보정에 대한
최종 시각 검증을 대신하지 않는다.

`tests/cases/verified_coanchored_host_flow.rs`에 새 이름의 정식 검사 4개를 추가한다.
형제 표의 원문 여백 일회 소비·본문 순서·누락·겹침, HWP 호스트의 저장 IR 보존,
이어받기 첫·마지막 문단을 보호한다. 독립 PDF의 글자 원점을 96 DPI로 환산한
핵심 위치만 ±2 CSSpx로 검사하며, 같은 페이지의 양쪽 raw는 100%다.

동일한 새 검사 source를 보정 전 Native234와 보정 후 Native240의 보관 라이브러리로
실행했다. 두 빌드는 모두 성공했다. 전자는 2 PASS / 2 FAIL, 후자는 4 PASS였다.
전자의 실패는 원문 여백 566HU를 883HU로 중복 소비한 관계와 독립 PDF 제목 기준선이다.
이미 통과하던 2개는 정상 동작 보존 검사로 기록하며 결함 검출 증거로 부풀리지 않는다.
`logs/verified-coanchored-{before-native234,after-native240}-3891.log`를 연결한다.

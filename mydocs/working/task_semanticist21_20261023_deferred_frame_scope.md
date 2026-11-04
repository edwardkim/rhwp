# 저장 쪽 프레임의 그림 지연과 줄 슬롯 수용 보정

## 원인·독립 근거와 범위

앞 단계 `ff77e83f25` 이후 #4138 편집 저장본은 여전히 197쪽이며 독립 Print는 196쪽이다.
첫 두 쪽만 개선됐고 13쪽부터 본문 경계가 다르다. 전체 검증 완료가 아니다.
입력과 기준 PDF의 SHA는 [앞 단계](task_semanticist21_20261023_stored_line_slot_cut.md)에 있다.

`delay_empty_anchor_topandbottom_flow_units_before_hard_break`는 빈 앵커의 그림 점유만
다음 가시 hard break 앞까지 옮긴다. 원 도입 `06fde211bd0`과
[Task 1999 Stage 5](archives/task_m100_1999_stage5.md)는 정상 원본 115쪽의 14쪽 본문과
15쪽 그림을 함께 보존하기 위한 처리다. 이 계약을 제거하지 않는다. 다만 현재 API 저장본은
페이지별 저장 원점이 없는 단조 사다리인데 문단 310의 그림 띠를 유닛 886의 앵커에서
6769 뒤로 옮긴다. 같은 입력을 한컴이 저장한 보조본에서도 빈 문단의 실제 저장 쪽 경계를
지나, 그림 띠를 유닛 1024로 미룬다. 임의의 먼 hard break는 같은 저장 프레임의 증거가 아니다.

`issue4138-hancom-saved-native143-units-2269`의 trace에서 보조본 24쪽은 원점 0의
문단 307부터 문단 320까지다. 다음 문단 321은 저장 프레임 재시작이지만 빈 문자열이어서
기존 pending flush가 건너뛴다. 문단 310의 그림 띠 235.706667px가 뒤로 빠져 수용량과
실제 소스 좌표가 달라진다. 이 보조본은 메타데이터 분석용이며 원 196쪽 Print를 대체하지 않는다.

뒤 보정 `a64840c344a` / 원 커밋 `fdee27aa366f1d6f4b6699d11f6b285280decb30`은
#7840의 156465345 문서에서 저장 사다리가 이미 품은 host 글줄을 그림 띠에 중복 계상하지
않도록 했다. 같은 조건의 중복 차감은 유지하고 해당 문서·정상 원본을 대조군으로 확인한다.

혼재 수식의 별도 문제도 확인했다. 문단 111·155의 본문+근호 수식은 source 개체 높이
1300HU·기준선 85%이고 한컴 보조 저장 줄도 1300/1105HU지만 현재 API 줄은 1200/1020HU다.
`publish_regenerated_rows`는 텍스트가 있는 문단을 제외하여 재조판 뒤 본문과 수식의 공통
기준선 합집합을 저장에 발행하지 않는다. 수식 전용/혼재를 같은 composition으로 발행한다.
정상 저장 캐시의 수용 조건을 완화하거나 문단 번호·화면 px로 줄을 키우지 않는다.

## 적용 경로와 반례

그림: 원본 CellUnit 생성 → 같은 저장 프레임의 지연 소유 → 컷/높이 예약 →
`cell_cut_owns_deferred_picture_band`와 실제 paint가 같은 유닛 순서·개체 ID를 소비한다.
실제 저장 프레임 경계(빈 문단 포함)에서 pending을 닫고, 인증된 다음 저장 프레임 없이
임의의 먼 explicit hard break만 있는 사다리에서는 앵커 점유를 이동하지 않는다.
원본의 정상 지연·host 중복 차감은 반례로 유지한다.

수식: 재조판 줄·소속 → `refresh_rows`의 본문/수식 어센트·하강부 합집합 →
발행 LineSeg → 저장/재열기·셀 컷·paint. 개체 크기를 다시 추정하거나 저장본을 기준 PDF로
바꾸지 않는다. 새 API 출력은 입력 SHA가 바뀌므로 새 버전별 한컴 Print를 확보해 대조한다.

## 검증 계획

Native144를 공용 `target/pr-review`, 빌드 잡 8개로 한 번 빌드한 뒤 기존 입력과 새 API
저장본의 초반·수식·그림 경계를 대조한다. 편집 전 115쪽 원본과 156465345를 함께 확인한다.
큰 위치 차이가 남으면 전체 검증보다 원인을 먼저 보정한다. 새 렌더링 회귀나 #4138 기대값
변경은 전쪽 Native/fresh WASM 90%·쪽수 일치·직접 판독 뒤에만 진행한다.

## 원본 셀 프레임과 자식 reset의 구분

`issue4138-source-frame-authentication-2280.json`에서 모든 후보의 같은 셀 원본 인접
LineSeg를 대조했다. 현재 API 저장본의 `stored_frame_break_before` 2개는 부모 저장
원점의 되감김이 아니다. 첫 값은 문단 2280의 자식 표에서 전파됐고, 둘째는 문단 2291의
상태 플래그다. 부모 원점은 모두 앞으로 진행한다. 한컴 보조 저장본은 195개 모두 같은
셀의 실제 저장 원점 되감김이다. 따라서 플래그만으로 지연 경계를 정하는 초안 Native144는
최종 후보가 아니다. 같은 셀의 원본 인접 줄에서 비합성·비음수 원점 되감김까지 확인하는
Native145를 사용한다. 부모의 다음 프레임으로 자식의 reset을 승격하지 않는다.

Native144의 빌드는 종료 코드 0·235초다. 소스가 빌드 중 바뀌지 않았음을 확인했고 이
생산자는 혼재 수식 발행 변화의 API 저장 관측에만 사용한다. 최종 후보 검증으로 재명명하지
않는다. 소유 인증 보완은 Native144 종료 후에 적용했다.

## 수식 발행 후보 철회

혼재 수식의 발행 제외만 제거한 Native144에서 두 공개 분할 API의 HWP 바이트는 서로
같았지만 문단 111·155는 여전히 1200/1020HU였다. live 개체도 1300HU를 유지했다
(`issue4138-native144-candidate-2277-result.json`, `...2285-result.json`). 명시적 2024
compat 선택도 같은 바이트였다(`...2283-result.json`). 단순한 혼재 발행 누락이 이 두
줄 차이의 충분한 원인이라는 가정은 틀렸으므로 해당 실행 코드 변경을 철회했다.
수식 차이는 보류·계속 분석하며 Native145에 검증되지 않은 발행 변경을 섞지 않는다.

진단용 새 Print 요청 스크립트의 최초 복제에서 입력 디렉터리 치환이 누락됐다. 해당
job `16f69654-6edc-4dca-9aa6-8de93d09113a`는 기존 입력 9ae...의 성공 출력이며 새
입력 45ab...의 증거가 아니다. 로컬 SHA assertion에서 검출되어 완료 기록은 생성되지
않았다. 잘못된 요청 로그는 `mcp-issue4138-native144-candidate-2278`에 보존했다.
수정한 요청은 별도 `...2284`에 입력 경로·SHA·target·engine을 요청 전에 봉인했다.
철회 후보의 출력은 진단용으로만 보존하며 최종 구현의 검증으로 사용하지 않는다.

## 12쪽의 수용량과 실제 끝 상자 재대조

수식 높이 추정과 별개로 12쪽의 원본 슬롯 합을 계산했다. Print가 수용한 문단 145의
다섯 번째 줄부터 문단 165의 첫 줄까지는 현재 API 저장본 979.466667px, 한컴 보조본
980.8px다. 다음 한 줄을 더하면 각각 1005.066667px·1006.4px로 내용 예산
1001.613333px를 넘는다. Native143은 문단 165의 마지막 줄까지 받아 9.6px를 빼
995.466667px로 만든다. 따라서 수식 줄 높이만 고쳐야 한다는 앞 추측은 충분하지 않다.

`row_physical_band::cut_trailing_advances`는 문단 끝이라는 이유로 원본 슬롯의
following 간격을 capacity에서도 뺀다. 이 가정은 현재 브랜치 `bc4af26560d`에서 추가됐고,
`3a9fc5a1d63`은 같은 가정에 문단 뒤 간격을 더했다. 새 프레임에 실제 내용이 들어가는
용량과 수용 뒤 그려야 할 끝 상자를 분리한 취지는 유지한다. 다만 일반 문단 끝만으로
capacity를 완화하지 않고, 독립적으로 인증된 source-frame trim만 수용량에 사용한다.
occupied 쪽의 기존 말미 제외는 그대로 두며 컷 선택·최종 요구가 같은 capacity를 소비한다.

Native145는 그림 소유 보정만의 방향을 보는 생산자다. 같은 셀의 실제 저장 원점 인증과
일반 문단 끝 슬롯 보정을 합친 Native146에서 영향을 함께 검증한다. 이 변경도 고정 px·
문서/문단 ID 분기를 추가하지 않는다.

## Native146에서 실행한 결과와 현재 한계

- 그림 지연의 원 해결 PR은 #2005, merge `d0bfda4d585b727128a7111d1f7a73d9af661ba3`다.
  원문은 `pr2005-deferred-band-provenance-2310.json`에 읽어 보존했다. 과거의 배경 픽셀
  점수를 현재 raw 일치율로 승격하지 않고 원래 보존하려던 그림/본문 소유의 근거로 사용했다.
- Native145(그림 경계 소유만 보정)는 편집 저장본 196/196쪽이지만 13쪽 raw 46.99105%다.
- Native146(문단 끝 capacity 보정 포함)은 13쪽 raw 100%로 개선됐고 12쪽은 99.25787%다.
  하지만 전체는 197/196쪽이고 24쪽은 30.28914%다. `re_review_required`를 유지한다.
  `issue4138-native146-critical-review-2304`의 13·24쪽 review/overlay를 직접 판독하고
  사용자에게 각각 함께 표시했다. 24쪽은 그림이 위로 올라가고 본문이 아래로 밀린다.
- Native146의 두 공개 분할 API 저장본은 SHA 9ae...로 기존 입력과 바이트가 같다
  (`issue4138-native146-candidate-2292-result.json`). 따라서 원래 독립 196쪽 Print를
  그대로 대조했다. 철회한 혼재 발행 후보 45ab...의 Print로 바꾸지 않았다.
- 편집 전 원본은 115/115쪽이며 선택한 1·2·14·15·16·115쪽 raw 최저 99.15773%다.
  전체 115쪽 완료를 주장하지 않는다. #7840의 156465345는 9/9쪽·전쪽 raw 최저
  93.43362%다. 이번 변경 전 Native143과 Native145/146의 전쪽 raw가 모두 같았다.
  최신 고정 upstream과의 2%p 비교를 마쳤다는 뜻은 아니다.
- 기존 표/경계 대조군 35개·66쪽은 Native146에서 전부 raw 100%, 쪽수 차이 0이다
  (`native146-matrix-2300-summary.json`, `logs/native146-matrix-batch-2300.log`).
  fresh WASM은 아직 이번 변경을 빌드하지 않았으므로 예전 WASM048 결과를 재명명하지 않는다.
- 앞 단계의 글자겹침·캐럿·SVG 계약 세 모듈 38건은 같은 동결 라이브러리에서 38 PASS다
  (`reviewed-contract-tests-2303-result.json`). 검사 소스와 생산자 소스 해시를 대조했다.
- 과거 실패 47건의 Native146 재실행은 11 PASS / 36 FAIL / 누락 0이다
  (`current-previous-failures-2291-result.json`). Native142의 9 PASS와 달라진 두 건은
  #4138의 과거 197쪽 핀이 우연히 맞은 결과다. 독립 Print가 196쪽이고 실제 그림/본문
  차이가 있으므로 이 두 PASS를 결함 해결이나 완료 증거로 인정하지 않는다. 기대값은 바꾸지 않았다.

현재 후보의 첫 source 컷 분기는 15쪽 끝이다(`issue4138-native146-cut-comparison-2307.json`).
한컴 저장 프레임의 글줄 상자 끝은 196쪽 모두 최대 996px지만 Native의 내용 예산은
1001.613333px다. 이 관측만으로 새 상수를 정하지 않는다. 일반 행이 전체 문서의 다른
중첩 개체 때문에 물리 예산을 쓰지 않는 경로와 실제 바깥 여백·끝 띠의 소유를 계속 대조한다.
문단 끝이라는 이유의 용량 완화도, 모든 저장 슬롯을 온전히 요구하는 후보도 아직 충분한
일반 규칙으로 입증되지 않았다. 현재 capacity 변경은 중간 후보로 보존하며 완료로 판정하지 않는다.

한컴의 보조 저장본 문단 310→311의 원점 차이 19473HU는 두 그림 중 둘째 그림의
`13517(height)+2202(vertical offset)+852+852(margins)+850(caption spacing)+1200(caption line)`
와 정확히 같다. 현재 편집 사다리는 host 줄 전진만 발행한다. 이 독립 등식은 다음 분석에서
그림/캡션의 공통 점유 결과와 저장·컷·paint 소비를 대조하는 근거이며, 특정 문단의 숫자를
구현 조건으로 쓰지 않는다. `issue4138-control-metrics-2272-*-saved.json`에 원본 속성이 있다.

새 회귀·fixture/쪽수 핀 갱신·PR 생성은 계속 보류한다. 본 단계는 미해결 후보와 실제
검증 근거를 일반 커밋으로 보존한 뒤 최신 devel을 리베이스하고 원인 보정을 이어간다.

## 최신 devel 리베이스와 새 기능 보존

중간 보정 `f5bd4b8976b25cc42448af097c102264b009773e`를 보존하고
`upstream/devel d9fd85ed5bc3c4d9ce6b2981422e48bf9bb90867`(#8181) 위로 리베이스했다.
현재 검증 code head는 `fdf17630f76f615ae94095148443e68330ea196b`다. 상태 조회와 겹친
일시적 index.lock 오류 뒤 잠금 파일이 없고 작업 트리가 깨끗함을 확인해 재개했다.
`rebase-upstream-d9fd-2312-result.json`과 `logs/rebase-upstream-d9fd-2312-continue01.log`에
종료 코드 0이 있다. `range-diff`의 211개 패치는 모두 `=`다
(`logs/rebase-patch-preservation-2313.log`). 백업은
`backup/review-semanticist21-before-d9fd-native146-220556`에 보존했다.

#8181 원문과 실제 통합 범위는 `pr8181-preservation-provenance-2301.json`에서 확인했다.
Native147은 공유 target·잡 8개·241.66초·종료 코드 0으로 빌드했다. 라이브러리의 실제
Cargo fingerprint도 생산자 영수증에 봉인해 후속 진단이 다른 feature의 최근 fingerprint를
잘못 사용하는 일을 방지했다.

| 실행 | 실제 결과·범위 | 증거 |
| --- | --- | --- |
| source 사전 검사 계약 | 18 PASS / 0 FAIL | `pr8181-source-checks-2314-result.json`, `logs/pr8181-source-checks-2314.log` |
| 현재 head의 8개 패키지 source | source check PASS. Cargo package/registry 게시 판정은 아님 | `pr8181-source-preflight-2314/source-preflight.json` |
| 공개 오류 API + 기존 세 계약 모듈 | 39 PASS / 0 FAIL. 새 `HwpError` 전파/downcast를 실제 실행 | `reviewed-contract-tests-2315-result.json` |
| 글꼴 생성 도구 | 공용 target/debug·dev-tools·잡 8개로 빌드 후 5 PASS / 0 FAIL | `pr8181-font-metric-generator-2318-result.json` |
| CLI 입력·오류 계약 | 일반 HWP/HWPX/HWP3 12개, 잘못된 입력·암호 정상/오입력/미지정 4개, PDF info 1개: 17/17 | `pr8181-cli-preservation-2322/summary.json`, 같은 폴더 `results.json` |
| 실제 페이지 출력 | 선택한 편집 저장본 11쪽·원본 6쪽·#7840 전 9쪽의 raw·쪽수 결과가 리베이스 전과 같음 | `issue4138-native147-boundary-batch-2317-result.json` |
| 픽셀 보존 직접 대조 | 같은 26개 Native PNG의 RGBA 바이트가 모두 동일 | `rebase-native147-pixel-preservation-2321.json` |

CLI 대조는 동일 cwd·입력·인자로 Native146과 Native147의 exit/stdout/stderr 전체를
비교했다. 기존 #8181의 원격 registry 소비 실행을 이번에 새로 했다고 표시하지 않는다.
글꼴 도구의 임시 canary가 정식 source를 바꾸지 않은 것도 clean 작업 트리로 확인했다.

따라서 이 리베이스에서 관측한 후퇴는 없다. 다만 26쪽 보존을 전체 #7969/#8064 또는
19문서 835쪽 검증 완료로 확대하지 않는다. #7969의 원문·기능 범위도
`pr7969-preservation-provenance-2319.json`에 재확인했으며 최종 필수 검증에 유지한다.

다음 분석에서는 단일 sqrt의 source 캐시 1300HU와 실제 render-tree Equation bbox의
반올림 표기 15.0px를 구분한다(`issue4138-native147-equation-tree-2320`). bbox를 줄 수용
높이로 동일시하거나 자연 glyph 크기로 정상 source 프레임을 무조건 축소하지 않는다.
실제 `flow_metrics_hwp` 생산 → 재조판/저장 줄 발행 → 컷 소비 → paint 상자를 추적한다.
Native144 혼재 발행 후보의 공개 composition 관측은 `issue4138-native144-candidate-2316-result.json`
에 있다. 이것은 철회 후보의 진단이며 최종 구현·시각 통과 증거가 아니다.

# 이슈 #7642 — 저장된 빈 Square 그림 호스트의 줄 예약

이전 단계: [그림의 공통 재조판 흐름](task_m100_7642_stage4.md).
시작 커밋: `98d952455`. 전체 실패 회귀·원본 24개·시험지 16개 범위를 유지한다.

## 독립 근거와 위반된 규칙

`samples/hwp3-sample16-hwp5-2018.hwp`의 빈 첫 문단은 종이 기준 Square 부동 그림과
실제 저장 줄을 가진다. 줄 높이 2000HU, 간격 1200HU이고 다음 빈 문단의 저장 원점은
3200HU다. 독립 한컴 Print의 표지 내용도 한 줄을 점유한다. 글자 없음과 줄 공간 없음을
같게 취급한 `resolve_line_metrics`의 Square 빈 줄 제거 때문에 측정 높이가 0이 된다.
paint의 빈 Shape 호스트 사다리 질의도 다음 가시 문자열을 요구해 같은 저장 줄을 버린다.

## 공통 결과와 실제 호출 경로

1. 정상 저장된 단일 물리 줄과 다음 저장 원점의 델타를 줄 높이·간격·호스트 뒤 간격·
   다음 앞 간격과 대조한다. 서로 다른 column_start를 가진 같은 vpos 조각은 한 줄로
   합친다. 합성 태그·편집 dirty·여러 물리 줄·쪽/단 reset·명시적인 다음 쪽 경계는 제외한다.
2. 이 읽기 전용 결과를 `InlineFlowPlan`의 저장 호스트 줄로 생산한다. 다음 문단의
   앞 간격은 이번 호스트가 소유하지 않는다. 원본 IR·저장 LineSeg는 변경하지 않는다.
3. section 본문 경로의 format 결과가 같은 plan의 줄 높이·간격·문단 간격을 사용한다.
   기존 fit·예약 뒤 실제 수용한 전체 문단의 동일 plan을 column metadata에 보존한다.
4. `layout_inline_flow_plan`은 그 줄 결과를 재구성하지 않고 소비하며, 개체 출력은 기존
   Shape 항목이 소유한다. Shape 뒤 vpos 기준 초기화가 줄 예약을 지우지 않도록 실제
   호스트의 수용 끝을 전달한다. 캡션·다른 컨트롤 소유자는 기존 경로에 남는다.

이 단계는 저장 Square 그림/도형 호스트와 실제 후속 저장 줄의 예약 증언을 다룬다.
NO_LS 그림 재조판, 표 호스트·TopAndBottom의 다른 물리 흐름을 이번 통과로 대신하지 않는다.

## 검증 순서

- 두 원본 표지의 직접 Print 대조와 빈 후속 줄·그림 뒤 원점부터 확인한다.
- 정상 단일 줄/좌우 조각, 실제 예약이 없는 wrap guide, 편집 stale, 단 예산/이월 경계를
  독립 한컴 출력으로 확인한다. 자료 없는 경로는 미검증으로 기록한다.
- 원본 전체 쪽과 정상 대조군을 Native/fresh WASM으로 재출력한다. 모든 관련 쪽 90%와
  확대 판독 전에는 해당 기존 px/hash 회귀 기대값을 변경하지 않는다.
- 실행 로그·전쪽 TSV·진단 JSON·다음 실행 계획은 ignored
  `output/pr-review/issue7642/failed-inputs-20261008/`에 보존한다.


`dump-pages`의 기존 재측정 helper는 이번 section 적용 plan을 읽지 않아 빈 호스트를
여전히 높이 0으로 보고할 수 있다. 진단도 실제 pagination의 저장 호스트 plan을
소비하도록 연결한다. 진단의 숫자만 바꾸거나 plan을 다시 추정하는 방식은 쓰지 않는다.


## 첫 Native 결과

읽기 전용 source/실행 파일 해시는 `stored-empty-host-trial-provenance.json`에 고정했다.
빌드는 4분 14초, exit 0이며 `rhwp-stored-empty-host-trial`로 원본 2018의 1·3쪽을
재출력했다. 쪽수는 양쪽 64쪽이고, 1쪽은 39.75383%에서 65.07881%로 개선됐다.
표지 제목·날짜·하단 로고의 세로 원점이 복원됐지만 제목 글꼴·바깥 테두리 차이가
남는다. 3쪽은 40.03361%다. `pr_review_gate`는 re_review_required이며 review를 직접
표시했다. 90% 미만이므로 이 원본의 기존 회귀/fixture 기대값은 변경하지 않는다.

표지의 선언 face 중 한양신명조는 설치된 fontconfig face로 직접 해결되지 않는다.
휴먼명조는 공식 HMKMM.TTF로 해결되지만 독립 2020 PDF는 Type3/HFT 출력도 포함한다.
이 관측만으로 해결 불가능한 글꼴 예외로 판정하지 않으며 배치 차이를 계속 보정한다.


최종 Native 생산 source에서는 후속 줄의 양수 높이를 확인하고, 문단 간격의 부호를
원본대로 보존했다. 숨긴 빈 문단에는 공통 plan metadata를 게시하지 않는다.
진단도 실제 pagination의 동일 plan을 읽어 첫 호스트를 `lh=26.7px + gap=16px`로
보고한다. 이는 저장된 2000HU·1200HU의 표시 단위이며 테스트 기대값으로 고정하지 않는다.
전체 64쪽 Native TSV는 최저 24.02432%, 미달 62쪽으로 원본 전체 기준을 아직 충족하지
않는다. 후속 내용의 원점 회복을 전체 문서 해결로 보고하지 않는다.


## 정상 저장 반례와 직접 판독

기존 정상화 대조군에서 그림을 종이 기준 Square 부동 개체로 설정하고 한컴으로
정상 저장했다. 빈 호스트 뒤에 빈 문단이 오는 입력과 본문이 바로 오는 정상 대조군이다.
기준은 해당 저장본의 Print PDF이며, engine 2020 / 11.0.0.9136 / PrintMethod 0 /
one-up을 server 결과에서 확인했다. 각 1쪽 Native/fresh WASM의 일치율은 모두 100%다.
review를 직접 표시하고 그림·본문의 원점과 간격을 확인했다.

- [입력·Print PDF와 SHA](../../tests/fixtures/issue_7642/empty-square/README.md)
- [빈 후속 줄 WASM review](../pr/assets/issue7642_stored_empty_square/empty-square-empty-follower-wasm-p001-review.png)
- [직접 후속 본문 WASM review](../pr/assets/issue7642_stored_empty_square/empty-square-text-follower-wasm-p001-review.png)

빈 후속 줄 입력은 수정 전 `rhwp-picture-logical-font`에서 89.13472%다. review를 직접
표시했으며 뒤 본문이 위로 올라온다. 수정 후 두 경로 100%와 직접 판독을 확인한 뒤
`tests/cases/issue_7642_stored_empty_square_host.rs`에 관계 검사 2건을 추가했다. 독립
저장 줄의 원점·높이 비율과 실제 문단 소유·내용 누락/중복·빈 줄 순서를 검사한다.
직접 본문 대조군은 기존 정상 경로의 보존 검사이며 결함 검출 검사로 혼동하지 않는다.

출처 계약 보정 전 중간 WASM 빌드는 15분 32초, exit 0이며 bg SHA-256은
`36d8d8fb7923dab295bc3210606cd16639fdd5d3daeaba1737225e3aff19be14`다. pkg/Studio SHA도
같다. 최종 Native는 빌드 로그가 완료된 뒤 exec가 143을 반환해 cached build를 다시
확인했고 exit 0(0.26초), 보존 실행 파일과 같은 SHA를 확인했다. 실행 실패를 성공으로
추정하지 않았다. `stored-empty-host-provenance.json`에 source·산출물 해시를 연결했다.

원본 2018/2024 각 64쪽의 Native 최저는 여전히 24.02432%다. 2018 원본의 fresh WASM
64쪽도 같은 최저이며 미달 62쪽이다. 대조군의 통과를 원본 전체 통과로 대신하지 않는다.
WASM 정상 대조군 중 exam-12에서 Chromium WS 기동 30초 timeout이 발생했다. 렌더
결함과 구분하고 이미 완료한 5개·37쪽을 보존한 채 미완료 범위만 재개한다.


## 편집 대조에서 검출한 새 회귀

첫 정식 확대 검증은 20건 중 16 PASS / 4 FAIL이었다. 새 저장 호스트 관계 2건과
기존 그림/p122 8건은 통과했으나, 기존 #2069의 HWP/HWPX Enter·재입력 4건이 실패했다.
초기 전체 회귀에서 이 4건은 모두 PASS였으므로 기존 실패로 분류하지 않는다.
편집 후 남은 저장 줄을 정상 저장 증언으로 적용해 본문 줄 피치가 두 번 전진했다.
`session_edited()`인 경로는 저장 호스트 plan을 생산하지 않고 기존 편집 재조판 소유자에
남긴다. 원래 테스트·기대값은 변경하지 않았으며 보정 후 동일 20건을 재실행한다.
이 코드 변경 전 시각 결과는 중간 증거로 보존하고 최종 Native/fresh WASM을 재산출한다.


`session_edited()`의 출처 봉인은 HWP5 raw stream 계약이다. 이 조건만으로 HWPX의
편집 여부까지 확인했다고 가정한 재실행은 18 PASS / HWPX 편집 2 FAIL이었다.
이번 정상 저장 호스트 경로의 입력 계약을 출처 봉인이 있는 HWP5 컨테이너와 미편집
세션으로 명시한다. 변환 HWP5는 포함하지만 원본 HWP3/HWPX는 출처 봉인 없이 같은
증언을 쓰지 않고 기존 소유자에 남긴다. 포맷별 원점 출처를 확인하지 않은 캐시 수용이
잘못된 가정이었다. 문서 ID·OLE 종류·좌표로 대상을 좁히지 않았다.
HWPX 저장 호스트의 같은 보정은 이 단계에서 검증/완료로 주장하지 않는다.


출처 계약 보정 뒤 같은 20건은 20 PASS / 0 FAIL, exit 0(컴파일 5분 1초)이다.
새 저장 호스트 2건, 기존 그림/p122 8건, 기존 #2069 10건을 포함한다.
`stage5-sealed-origin-nextest.log`에서 확인한다. 원래 검사/기대값은 유지했다.
정식 저장 호스트 suite의 CLI만 이전/보정 실행 파일로 바꾼 결과는 1 PASS / 1 FAIL →
2 PASS다 (`stage5-formal-counter-before-after.json`). 본문 직접 후속은 정상 대조군이고
빈 문단 후속이 실제 결함 검출 검사임을 구분한다.

미편집 출처 계약을 추가한 최종 source는 `stored-empty-host-sealed-provenance.json`에
고정한다. 이전 WASM 대조군 19개·83쪽 통과는 중간 source의 증거다. 최종 Native·fresh
WASM을 다시 산출하고 같은 범위를 재검증하며 이전 PNG를 최종 head 캡처로 쓰지 않는다.


최종 미편집 HWP5 출처 계약의 Native 대조군은 19개 입력·83쪽, 최저 90.61634%로 쪽수
일치와 90% 수치 기준을 유지했다. 별도의 저장 호스트 2개·2쪽도 각각 100%, 대표
review/overlay gate passed다. 새 Source의 Native PNG로 자산을 갱신했다. 전쪽 TSV의
not_evaluated와 대표 PNG의 직접 판독·passed를 구분한다.


## 출처 계약 보정 후 최종 검증

최종 Native와 fresh WASM 모두 정상 대조군 19개·83쪽에 저장 호스트 반례 2개·2쪽을
더한 총 21개·85쪽을 비교했다. 모든 실행은 exit 0, 누락 쪽 없음, 최저 90.61634%다.
전쪽 TSV만 산출한 정상 대조군의 gate는 `not_evaluated`이며 자동 승인 증거가 아니다.
저장 호스트 두 반례는 두 경로 각각 100%, 대표 review/overlay gate `passed`다.
최종 자산으로 갱신하고 그림·본문 소속과 원점·빈 줄 간격을 직접 판독했다.

최종 WASM wrapper는 16분 14초, exit 0이다. `rhwp_bg.wasm` SHA-256은
`0980f6a92fb4fc360c3cbff4bb0ac5534ecb5108fc28e8ddf39595723c20397f`이며 루트 pkg와
Studio가 일치한다. Native SHA-256은
`3f9957913e6c19c0ffd2984fc2e32bf7deb69386bc18e45d11b93c428e3581d8`다.
`stored-empty-host-sealed-provenance.json`의 생산 source 7개 해시가 검증 후에도 일치한다.

| 검증 | 결과 | 로그 |
| --- | --- | --- |
| 관련 정식 회귀 20건 | 20 PASS / 0 FAIL, exit 0 | `stage5-sealed-origin-nextest.log` |
| fmt | PASS | `stage5-sealed-fmt-check.log` |
| Native Clippy | PASS, 1분 12초 | `stage5-sealed-clippy-native.log` |
| WASM lib Clippy | PASS, 1분 6초 | `stage5-sealed-clippy-wasm.log` |
| workspace build | PASS, 2분 9초 | `stage5-sealed-workspace-build.log` |
| workspace all-targets Clippy | PASS, 2분 | `stage5-sealed-clippy-all-targets.log` |
| integration manifest | PASS, 1489 sources / 48 targets | `stage5-sealed-manifest-check.log` |
| source unit tier 정책 | PASS, 4199 tests / 298 modules | `stage5-sealed-unit-tiers-check.log` |

위 정책 비교 base는 `b3c3047db575d146dff9d39098e6de98c4630b3e`다. 이후 확인한
upstream/devel `60efc4b7b380b2a183c20c3ee0ab24a60bc2d832`에는 #7643의 다른 조판
수정 등이 포함돼 있다. 최신 base 위의 rebase 뒤 해당 변경과 소비 경로·정책 비교를
다시 확인한다. 이전 base의 통과를 새 base/head 검증으로 보고하지 않는다.

원본 2018의 최종 Native/fresh WASM 직접 비교는 1쪽 65.07881%, 3쪽 40.03361%로
미달이다. 전체 원본의 기존 회귀·기대값을 갱신하지 않았다. 최초 전체 회귀의 다른
실패와 시험지·수식 확대 판독은 여전히 해결/검증 중이며 전체 검증·PR 준비 완료로
판정하지 않는다. 로그·TSV·진단 JSON은 앞서 지정한 ignored output에만 보존한다.

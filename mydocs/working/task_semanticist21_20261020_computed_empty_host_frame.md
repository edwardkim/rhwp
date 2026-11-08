# #7559 현재 API의 계산 빈 호스트와 표 프레임

## 원 해결 추적과 독립 근거

현재 API의 새 HWP/HWPX4개를 같은 한컴2020·11.0.0.9136·PrintMethod0·1-up으로 인쇄했다.
영수증은 ignored `output/pr-review/semanticist21-20261019/mcp-save-origin-originals-475`에 있다.
원 API 테스트의 입력 구성은 바꾸지 않았으며 native HWP와 adapter HWP 바이트가 같음을 확인했다.
현재 HWP의 live/재열기 table top은176.2666667, HWPX는180.04다. 독립 Print는 두 형식의
표 테두리가 같은 위치다. Native049의 HWP raw5.24242%, fresh023의 HWP5.27488%이며
같은 입력 HWPX와 padding 대조는 양쪽100%다. 이 수치는 보정 전 결과다.

원 #7559는 #7265를 참조하며 실제 메인터너 통합 #7430의 위 여백 계약을 유지해야 한다.
#7538의 원점 논의와 실제 후속 보정도 읽었다. #7538 자체는 closed·unmerged다.
추가 원인인 `reflow_empty_table_host`의 LineSeg 계약은 원 #7518→통합 #7570의
`3d9239eeec`→`a9dfc4e664` 보정까지 추적했다. 계산 줄을 저장 줄 부재로 간주해 없애면
원래 보정의 줄 점유 계약을 깨뜨린다. #7601의 `0a305d51a8`은 입증된 저장 원점만 수용한다.
`generated-host-original-pr-trace-481.json`과 기존 `pr_7518_review_impl.md`에 근거가 있다.

## 값의 생산과 소비

- 입력의 계산 LineSeg는 synthetic tag를 유지한다. HWPX writer는 이 줄을 저장 사다리로 내보내지 않는다.
  이전 fixture012와 현재 public API 모델의 차이도 이 tag다. 이전 저장본100%를 현재 실패의
  해결 증거로 재사용하지 않는다.
- `host_spacing::resolve`가 이미 위·아래 바깥 여백을 산출한다. `FormattedParagraph.total_height`는
  계산 줄의 실제 format 결과다. 줄이 보이지 않는다고 공간을0으로 취급하지 않는다.
- `block::computed_host_frame::resolve`는 빈 글줄과 표 하나가 같은 호스트에 속한 경우만,
  같은 원점에서 표 앞 여백·양수 오프셋·표 높이·뒤 여백과 계산 줄 점유의 합집합을 만든다.
- `block::entry`의 기존 명시 저장 프레임과 NO_LS 계획이 없을 때 이 결과를 수용한다.
  기존 HostPlacementConstraint가 앞 형제·점유 띠를 적용한 뒤 같은 상자를 fit에 사용하고 기록한다.
- `place_table_with_text`는 기록한 occupied_bottom으로 흐름을 전진시키고,
  layout의 resolved_table_origin은 같은 table_top을 쓰며 별도 offset·여백을 재계산하지 않는다.
  뒤에서 원점 덮어쓰기나 paint-only 이동을 추가하지 않는다.

실제 저장 줄, visible/공백 호스트, 둘 이상의 줄 참여 개체, 음수 오프셋·절대 기준·다른 세로 정렬은
이 새 경로에 들어오지 않는다. 구역·단 설정과 줄에 참여하지 않는 그림·도형은 기존 공통
`control_leaves_table_anchor_empty` 판정을 공유한다. 문서 ID·좌표 상수·row 수 조건은 없다.
계산 줄의 공간을 보존하므로 #7518의 `reflow_empty_table_host` 수용 조건은 바꾸지 않는다.

## 검증 순서

1. Native050 동결 빌드 뒤 실제 원 API2검사와 현재 API4입력의 전쪽 Print 대조를 먼저 확인한다.
2. 같은 입력의 fresh WASM 전체 페이지와 직접 review/overlay 판독을 완료한다.
3. 양쪽90% 이상이 확인된 뒤 원 API 저장 전후 불변 검사와 줄·표·다음 내용의 관계 검사를 보강한다.
4. 원 #7518 계산 줄, #7430 위 여백, 기존 형제 표·분할·positive offset 대조와 #8064를 함께 검증한다.
   원본 저장 줄의 처리는 변경하지 않는다. 넓은 전체 회귀는 영향 경계 확인 이후 수행한다.

아직 수정 후 빌드·시각 검증 전이며 PR 제출 조건이 충족된 상태가 아니다.

## Native050 관측

공유 target·잡8개 Native 빌드는236.07초에 exit0으로 끝났다. 원 API 입력 구성과 원점 기대값을
바꾸지 않은 `table_hwpx_save_origin_contract`의2검사가 모두 통과했다.
현재 API의 HWP adapter/HWPX·offset/padding4개 새 Print 대조도 각 전체1쪽 Native100%다.
수정 전 HWP offset5.24242%에서100%로 개선됐다.

strict rustc로 실제70검사를 실행해0FAIL을 확인했다: 원점2, 보충 측정21, 세션11, 강조점3,
형제 표7, 원 #7518 계산 줄·물리 프레임22, 저장 도형 호스트4다. 원 #7518의 LineSeg 수용 조건과
검사 기대값은 변경하지 않았다. 전체 Cargo 회귀 완료로 표현하지 않는다.
`char-overlap-native-build-050-result.json`, `emphasis-metric-current-failure-focus-484.json`,
`save-origin-current-api-488-computed-host-fixed-native/results.json`에 실제 명령·SHA·결과를 보존했다.
fresh WASM, 직접 판독과 넓은 대조는 다음 검증 단계이며 아직 전체 PR gate가 통과한 상태는 아니다.

## 원 PR·실제 통합의 추가 대조와 반례

추가 조회에서 원 #2101은 #2108(`a12b42ab643b6a166f57d286b02b9abd26b69476`),
원 #2109·#2133은 #2135(`bb87b367a546b7f9b3482aa772effbc0abfd3d71`)로 통합됐음을 확인했다.
#2123은 #2109를 충돌 때문에 제외한 자료 보강 PR이다. 원 PR의 연결 이슈만 보고 실제 통합으로
분류하지 않는다. #2101의 선언-fit은 선언 높이가 들어가는 None 표의 과분할 방지이며,
선언 자체가 본문보다 큰 표를 잘라야 한다는 근거가 아니다. #2109는 광역 반증 뒤 범위를 좁혔고,
#2135에서 기존 None 게이트와 충돌을 해소했다. #2512의 실제 저장 호스트·반복 여백 계약도
유지한다. 원문·커밋 조회는 ignored `none-table-integration2108-516.json`,
`rowbreak-integration2123-516.json`, `rowbreak-actual-integration-517.json`,
`rowbreak-integration2135-518.json`에 보존한다.

Native050/fresh024에서 원 API4입력은 양쪽100%이며, 기존116조건은 Native112/116,
fresh113/116이 수치 gate를 충족했다. 이것은 직접 판독·base 무회귀나 전체 PR 완료 판정이 아니다.
미달에는 원 #8055 None 입력과 CELL 진단, #7694 편집 차트가 남아 있다.

공개 API로 호스트 글자 크기·양수 offset을 바꾼12입력을 독립2020 Print로 출력했다.
Native/fresh 모두90% 이상이지만, 작은2.5pt 호스트의 양수 offset에서는 뒤 문단이 표 위 공간에
들어가는 Print와 달리 표 아래로 밀렸고,40pt HWPX는 빈 줄의 점유를 잃어 뒤 문단이43.6px
앞당겨졌다. 원 입력의 통과만으로 계산 빈 호스트 일반 계약이 입증되지 않는다.
`computed-host-height-geometry-audit-509.json`과 양쪽504/505 TSV가 수정 전 근거다.

### 다음 보정의 공통 결과와 소비 경로

- `computed_host_frame::resolve`가 실제 저장 줄과 계산 줄/줄 부재를 구분한다. 계산 경로는
  기존 NO_LS 복원보다 먼저 format 결과를 사용한다. 유효한 원본 저장 줄은 이 경로에 넣지 않는다.
- `ComputedEmptyHost.line_advance`는 호스트 줄의 흐름 끝, `table_top/occupied_bottom`은 물리
  표 상자다. 같은 호스트에 있어도 뒤 문단의 자연 줄 원점과 표 점유 하단을 같다고 가정하지 않는다.
- 통째 표 확정 뒤 `place_table_with_text`는 계산 줄 끝을 소비하고 표 상자를 배제 영역에 등록한다.
  `resolve_computed_float_first_line`은 문단 앞 간격을 포함한 자연 첫 줄 원점과 실제 줄 높이를
  표 상자와 비교한다. 앞 간격 자체를 잉크로 간주하지 않고, 들어가는 줄은 표 위 공간을 사용한다.
- 첫 줄의 문단 원점과 앞 간격은 `FloatPushedFirstLine`으로 단 결과에 보존한다. 기존 #7330의
  간격 기록은 원점을 강제하지 않는 상태를 유지한다. 전체/첫 부분 문단 출력은 이 확정 원점을
  사용하고, 계산 호스트의 문단 완료·lane 끝도 같은 `paragraph_end`를 소비한다.
- 이 수정에는 좌표 상수·글자 크기·문서 ID·행 개수 분기가 없다. 기존 저장 줄의 범위는 유지한다.
  분할 표의 마지막 원점·여백은 별도 원래 계약으로 대조하며 통째 보정의 통과로 대신하지 않는다.

Native051 빌드와 원70검사, 새12입력·정상 대조군을 먼저 실행하고, fresh 재빌드·직접 판독 뒤
정식 회귀를 추가한다. 현재 이 추가 보정은 검증 진행 중이다.

### 대조군의 실제 저장 속성 확인

모델에서 `flow_with_text`만 바꾼506 진단 입력은 HWP와 HWPX의 같은 속성을 보장하지 않았다.
HWP의 CTRL_HEADER bit13은 두 경우 모두false, HWPX는 설정대로false/true였다.
`unsplit-wire-flow-519.json`이 실제 바이트 근거다. 이 차이를 한컴 형식별 배치 규칙 차이로
해석하지 않는다. 공개 `restrictInPage` 편집 명령으로 의미 값과 raw를 함께 갱신한 대조군을
다시 만들고 저장 비트 확인 후 독립 Print를 수행한다. 앞선 원본·PDF는 실패 증거로 보존한다.

## 계산 줄 보완 뒤 Native052 확인

Native051의 수치 검사는 원70검사와18입력을 통과했지만, 작은 글자 HWP의 뒤 문단 위치는
여전히 달랐다. HWP에 남은 구현 태그 LineSeg는 글자 크기250HU와 달리 옛1000HU 높이를
가졌다. 이를 유효한 저장 높이로 사용할 수 없다. `computed_empty_table_host`의 동일한
판정을 formatter와 개체 프레임 생성이 공유하고, 계산 빈 줄은 현재 글자·문단 속성으로
메트릭을 다시 구성한다. `reflow_empty_table_host`의 줄 부재 계약은 유지한다.

Native052 빌드는217.35초 exit0, 변경하지 않은 원70검사0FAIL이다. 원 API4개·글자 크기/offset
반례12개·앞 간격 대조2개의 전쪽18개는 모두 raw100%다. 작은 글자/양수 offset 뒤 문단은
두 형식 모두 y158.9로 재개하고,40pt 호스트 뒤 문단은 y238.9로 재개해 Print 잉크 위치
y159.044/y239.116과 같은 줄에 놓였다. 잉크 bbox와 줄의 font box는 동일한 값이라고
간주하지 않는다. `computed-host-height-geometry-audit-536.json`,
`emphasis-metric-current-failure-focus-530.json`, `computed-line-boundaries-531-computed-line-final-native/results.json`이
실제 관측 기록이다. 전체116조건 Native 대조는 별도로 진행하며 기존 미달을 완료로 바꾸지 않는다.

공개 `restrictInPage` 명령으로 다시 만든522 입력6개는 HWP bit13과 HWPX flowWithText가
모두true임을 확인했다. 독립 Print6개도 모두 성공했다. 큰 None 표는 두 형식 모두 다음 쪽으로
통째 이동하고 뒤 본문은 앞쪽에 남는다. 모델만 수정한506 입력의 형식 차이는 저장 속성
불일치였으며, 렌더러의 형식별 예외 근거로 쓰지 않는다. `unsplit-public-wire-flow-525.json`,
`unsplit-public-print-geometry-529.json`이 근거다. 큰 None 표와 RowBreak 분할 보정은 아직 남아 있다.

이 commit은 Native에서 확인한 구현 단계의 고정이며 최종 PR 승인이 아니다. 같은 소스의
fresh WASM·직접 review/overlay 판독·고정 devel 양쪽 전쪽 비교 뒤 정식 회귀를 추가한다.

## fresh025와 정식 회귀

제품 commit은 `bc649064faaa2193380c1ab5b2567b4470f2472e`다. Native052의 빌드 시작 시점은
부모 commit·dirty였고, 이후 commit의 실제 소스 바이트와 일치함을 `native052-committed-source-equivalence-537.json`으로
확인했다. fresh025는 같은 commit·같은 소스 지문으로752.1초에 최적화·Studio public 동기화를
마쳤다. 18입력의 전쪽1쪽은 Native/fresh 모두 raw100%다. 고정 devel의 양쪽36조건도 실제
재출력했고0.1%p 이상 하락은 없다. 기존116조건410쪽 Native는 이전cf7e3e1d22와 raw가 모두 같다.
별도 기존116조건의 fresh 대조는 계속 진행한다. 이 결과를 전체 PR 완료로 바꾸어 적지 않는다.

대표 작은 글자/양수 offset·큰 빈 줄·앞 간격 대조의 review와 standalone overlay를 양쪽에서
직접 열어 표 외곽·앞뒤 글줄·gap과 겹침을 확인했다. 격자와 줄 위치는 일치하며 얇은 선의
raster 농도 차이는 남는다. 원본0.36pt 선을 점수를 위해1px로 바꾸지 않았다.

`mydocs/pr/assets/computed_empty_host_line_20261020`에 새 회귀의14입력·독립 Print PDF14개,
양쪽 후보/고정 devel TSV56개·글꼴 공급 기록·원본/PDF/TSV SHA·대표 review/overlay12개를
보존했다. 공개 API의 기존 원점4입력은18조건에 함께 검사했고, 새 회귀 입력14개와 구분한다.
HWP의 스크립트 비활성화 전처리와 HWPX의 전처리 없음은 provenance에 기록했다.

새 이름 `tests/cases/computed_empty_host_line_and_float_frame.rs`의5검사는 표/문단 소유,
행열·내용 순서/누락/중복, gap에 들어가는 줄·큰 빈 줄·표 배제·앞 간격 중복 방지 관계를 보호한다.
완료100%쪽은 독립 Print의 격자 선 중심·뒤 텍스트 잉크 bbox를96DPI 종이 원점±2px로 추가
보호한다. 같은 정식 파일의 strict rustc 검증은 Native050에서 의도한4FAIL·정상1PASS,
Native052에서5PASS다. 빌드 실패나0개 실행을 결함 재현으로 세지 않았다.
명령·정식 소스 SHA·실제 로그는 `emphasis-metric-current-failure-focus-553.json`에 있다.

큰 None 표, RowBreak 컷/여백, 현대 편집 차트와 다른 전체 Rust 실패는 여전히 남아 있다.
원 PR과 실제 통합 근거를 따라 보정을 계속하며, 로컬 PR 필수 검증이 끝나기 전 원격으로 제출하지 않는다.

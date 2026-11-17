# PR #9149 리뷰 — 저장 줄 없는 직선 띠 앵커

## 최종 판정

**검토 및 code CI 충족 — 문서 trailing 최신 head 검사 확인 후 병합 가능.** 원 기여와 필요한 메인터너 보정을 구분해 검증했다.

## 변경과 검토 범위

작성자는 `hyunmyungJaneLee`, 원 head는 `6e9f7fc98f78e8c6a180fb2e46c2cf57cc0a68da`다.
관련 이슈는 [#9138](https://github.com/edwardkim/rhwp/issues/9138)다.
원 기여의 HWP5 변환 경로 집중 검사는 통과했으나 HWPX 원본은 58.23%였다. 저장 근거를
질의하는 경로가 합성 vpos로 흐름을 되감는 공통 오류를 보정해 100.00%로 개선했다.
특정 문서·쪽 예외가 아니라 실제 저장 줄과 합성 줄의 출처를 구분한다.
[원인·보정 범위](../assets/open_prs_20261117/band-cause.md)를 따른다.

## 공식 PDF와 실제 시각 결과

원 PR의 **웹한글기안기 PDF는 공식 비교 자료에서 제외**했다. 같은 원문을 MCP
`--engine 2024`로 새로 출력했다. 실제 제품은 데스크톱 한컴 2024 `13.0.0.3901`,
**Hancom PDF Print**이며 Microsoft Print to PDF·PDF 저장/내보내기가 아니다.
입력 전처리는 `none`, Print method는 `0`, 출력 모드는
`frame_print_to_pdf_ex_one_up`다. 입력의 저장 버전과 실제 출력 버전을 구분했다.
[공식 PDF](../../../pdf/pr-review-open-prs-20261117/hwp_band-2024.pdf)와
[MCP·해시 기록](../assets/open_prs_20261117/reference-pdfs.json)을 보존했다.

같은 원문 1쪽·PDF·96DPI·compat2024·2px raw 기준 Native 공식 표시값은
base **36.37%** → 원 기여 **58.23%** → 보정 후보 **100.00%**다.
쪽수 불일치는 없으며 review와 overlay를 직접 열어 줄·직선·본문 소속을 판독했다.
이는 허용 반경의 실루엣 보조값이며 RGB 픽셀 완전 동일성을 뜻하지 않는다.

- [base TSV](../assets/open_prs_20261117/cases/band/native/base/silhouette.tsv) · [후보 TSV](../assets/open_prs_20261117/cases/band/native/candidate/silhouette.tsv)
- [후보 review](../assets/open_prs_20261117/cases/band/native/candidate/review-001.png) · [후보 overlay](../assets/open_prs_20261117/cases/band/native/candidate/overlay-001.png)
- [Native 원문 PNG](../assets/open_prs_20261117/cases/band/native/candidate/raw-001.png) · [한컴 PDF PNG](../assets/open_prs_20261117/cases/band/native/candidate/pdf-001.png)

## 접수와 통합 신원

리뷰어는 `jangster77`이며 지정 결과를 API에서 확인했다. 검토 브랜치는
`review/integration-20261117-open-prs`, base는 `1e2488ebd432deb7c326be532f8a91945f260231`,
현재 보정 source는 `2c5e3fe38726d23bec1c970ba3f92a49e15cae34`다. #9144 → #9149 → #9152의 11개 커밋을
원 SHA를 기록하며 체리픽했고 개별·쌍별·누적 시뮬레이션 9건은 충돌 없이 종료했다.
원 fork에는 push하지 않았다. [선행 분석](../assets/open_prs_20261117/intake-analysis.md)을 따른다.

## 최종 로컬 검증과 후속 조건

[통합 검증 정본](../assets/open_prs_20261117/README.md)과 각 원본 TSV·PNG를 확인했다.
최소 글꼴 환경 focused 7건과 전체 nextest **13,708 PASS / 0 FAIL / 45 skip**,
fmt·Native/WASM/all-target Clippy·workspace build·manifest/unit-tier, Native Skia **본체 lib 3,852 PASS·12 ignored, workspace 멤버 182 PASS·별도 2·36 PASS**을 통과했다.
원본 글꼴 일반 환경의 정본 보호 18종·1,771쪽은 Native/fresh WASM 모두 하락·상승 0쪽이다.
명시적 편람 Native 383쪽(최저 99.97%)과 기여자 공통 보호 8건도 통과했다.
[회귀 집계](../assets/open_prs_20261117/rust-validation.json),
[보호 Native](../assets/open_prs_20261117/protected-native-validation.json)·
[보호 WASM](../assets/open_prs_20261117/protected-wasm-validation.json),
[실제 소비 경로](../assets/open_prs_20261117/consumer-validation.json)를 따른다.

[통합 PR #9156](https://github.com/edwardkim/rhwp/pull/9156)의 code head `69f0deaa468f0914c29afa0e9697424a75fb10ae`에서
[Full CI](https://github.com/edwardkim/rhwp/actions/runs/44079335323)·CodeQL·Render Diff·Adapter·Proptest와 최종 정책이 성공했다.
[실제 head·job 영수증](../assets/open_prs_20261117/code-ci.json)의 조건부 생략은 실행 통과와 구분한다.
오늘할일·검토 보완만 같은 PR의 문서 전용 trailing으로 반영한다. Native Skia 본체/멤버의 집계 표기 오류도
원 실행 로그에 따라 정정했으며 제품·테스트·기준 자료는 바뀌지 않았다.
trailing 최신 head 검사·병합과 원 PR/실제 해결 이슈 close는 아직 실행 전이다.
원 fork를 수정하지 않으며, 통합 병합 뒤 검토 결과와 확정 SHA를 원 PR에 안내한다.

## 후속 코멘트 계획

통합 병합 뒤 원 기여의 성과와 메인터너의 PDF 재산출·필요한 보정 범위를 분리해 설명한다.
웹한글기안기 PDF를 판정에서 제외한 사실과 실제 데스크톱 제품·Print 경로를 코멘트에도 적는다.
해당 PR의 같은 쪽 raw·PDF·review·overlay를 merge SHA 고정 이미지로 직접 표시한다.

## 최종 실제 출력과 회귀 보강

원본 1쪽의 Native/fresh WASM과 실제 Studio Canvas2D·CanvasKit compat/default,
Native SVG·WASM SVG가 독립 Print 대비 각각 **100.00%**다.
같은 원문·쪽의 review·overlay를 직접 판독했으며 허용 반경 내 안티앨리어싱·획 차이는
RGB 완전 동일성을 뜻하지 않는다. 실제 Studio는 원본 함초롬바탕 TTF를 공식 host-font API로 공급했다.
CanvasKit software 실제 backend·typeface 준비·fallback 없음과 초기화 JS/WASM 응답 해시를 확인했다.
[소비 경로 TSV·raw/PDF/review/overlay](../assets/open_prs_20261117/consumer-validation.json)를 따른다.

새 원본 HWPX 회귀는 독립 PDF 좌표 ±2px와 라벨 누락·중복·순서를 함께 검사한다.
[동일 검사 실제 RED/GREEN 및 공유 target 재사용 오류 제외](../assets/open_prs_20261117/band-red-green-validation.json)를 확인했다.

## 조판 원칙 준수 검토

| 검토 항목 | 실제 확인 근거 | 판정 |
| --- | --- | --- |
| 제출 전 로컬 검증과 PR 상태 | 고정 base1e2488eb·제품2c5e3fe3·회귀c2ec0621, 필수 로컬 gate·Open 통합 PR을 확인했다. 원격 CI·후속 기록은 별도 병합 조건으로 관리한다. | 충족 |
| 기존 이슈·해결 PR 선행 분석 | 원 이슈#9138 → 원 PR#9149의 고정 head·11개 누적 적용 → 독립 Print·현재 원본 출력을 [선행 분석](../assets/open_prs_20261117/intake-analysis.md)과 대조했다. | 충족 |
| 구현 근거와 일반성 | NoLS·빈 문단·문단 기준 TopAndBottom·직선만·비 Word 조건과 독립 desktop Print를 대조했다. 원본 HWPX 58.23%의 합성 vpos 되감기를 보정했으며 특정 파일·쪽 조건이나 출력 clamp는 없다. | 충족 |
| 측정·배치 일관성 | `hwp_topbottom_band_host_metrics` → 공용 anchor metrics/height cursor와 최종 renderer의 저장 출처 질의를 대조했다. 합성 vpos로 재조판 y를 되감지 않고 같은 줄 구성을 Native/Canvas/SVG에서 확인했다. | 충족 |
| 리베이스 반영 PR의 기능 보존 | 최신1e2488eb에서 시작했으며 이후 fetch도 같은 head다. base의 #9147 실제 대체 관측·말미 SP 계약을 기존 회귀와 원본 보호18종 양쪽1,771쪽 무하락으로 확인했다. 현재 batch에서 재차 리베이스한 이력은 없다. | 충족 |
| 최종 출력 소비 계약 | 원본 1쪽 Native/fresh WASM PNG와 실제 Studio Canvas2D·CanvasKit software 두 경로 및 양쪽 SVG가 각각 100%다. 초기화 JS/WASM 실제 응답·원본 typeface·runtime readiness·fallback0을 확인했다. | 충족 |
| 분할·이어받기 계약 | 표/셀 조각의 컷·continuation·행 높이 배분 로직을 변경하지 않는다. 공용 문단 흐름과 줄 소속은 아래 및 전쪽 보호로 별도 확인했다. | 비해당 |
| 줄 소속과 점유 높이 | 앵커 줄과 겹친 후속 줄의 전진, 실제 저장 줄과 합성 줄 출처를 구분했다. 원본 라벨 여섯 개의 소속·순서·누락/중복과 독립 좌표를 확인했다. | 충족 |
| 사례와 증거의 독립성 | 원본 HWPX와 HWP5 변환 대조를 구분했다. 같은 새 검사로 원 기여 source5264d6f1의 의도한 좌표 FAIL → 보정 source2c5e3fe3 실제 PASS를 확인했다. 실제 archive 뒤 잘못 재사용된 after 시도는 제외했다. | 충족 |
| 기준값 변경 | 기존 HWP5 상대 간격은 WebHWP가 아닌 독립 Print 5789.6727/5490/5490 HU와 대조하고 기존 60 HU 여유를 유지했다. 새 원본 좌표는 ±2px다. 보호 floor·90%·측정 임계값 변경은 없다. | 충족 |
| 회귀 추가·변경의 선행 조건 | 관련 원본 전체 1쪽의 Native/fresh WASM100/100% 및 직접 판독 후 회귀를 보강했다. 해당 페이지 양쪽98% 이상으로 독립 좌표 ±2px를 허용하며 소속 관계도 함께 보호한다. | 충족 |
| 90% 통과 후 확대 판독 | 같은 쪽 review·overlay에서 BAND/AFTER 라벨·세 직선과 후속 END를 대조했다. 작은 안티앨리어싱/획 차이는 RGB 동일성으로 주장하지 않는다. 수식 변경 주장은 없다. | 충족 |
| 주장과 검증 범위 | source/producer/input/PDF/hash·쪽수·실제 CLI/소비 경로·최종 회귀를 개별 증적에 연결했다. 원 기여·메인터너 보정·원격 대기와 실패/무효 실행 이력을 구분한다. | 충족 |

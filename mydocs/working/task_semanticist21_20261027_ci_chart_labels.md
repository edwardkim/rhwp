# PR #8319 — CI 차트 라벨 검사의 글꼴별 위치 오판 분석

## 원 해결과 현재 실패

head `d1e9f27148884d24bdc96650883c39f68d40d801`의
[Native Skia job](https://github.com/edwardkim/rhwp/actions/runs/40485025707/job/122124967686)에서
#7950의 네 검사는 통과했다. 다음 #2293의 `chart_png_renders_text_labels`는
`카테고리 라벨 밴드 잉크 0 (title=332)`으로 실패했다.

원 해결은 PR 번호가 없는 커밋 `10cad483e8065836f4d7f25605d3c795e9ef9f19`다.
[#2293](https://github.com/edwardkim/rhwp/issues/2293)의 논의와
[원 보고서](../report/archives/task_m100_2293_report.md)를 읽고, 조달 경로를 공통화한
[#2898](https://github.com/edwardkim/rhwp/pull/2898)과 번들/시스템 우선순위를 복구한
[#3310](https://github.com/edwardkim/rhwp/pull/3310)도 대조했다.
핵심 계약은 RawSvg의 제목·카테고리 글자가 최종 Native PNG에서 사라지지 않는 것이다.

## 실제 재현과 계층 판정

원 입력은 기존 `samples/chart/라인/표식이있는꺽은선형.hwp` 한 개다.
DejaVu와 저장소 번들만 공급한 읽기 전용 bubblewrap 환경의 같은 실행 파일에서
동일한 `title=332 / category=0` 실패를 재현했다. 실제 PNG에는 네 카테고리가 모두 보였다.
원래 하단 6% 영역은 글자의 아래쪽 일부만 검사하며 대체 글꼴의 획이 그보다 위에서 끝난다.
SVG의 카테고리는 `dominant-baseline="text-after-edge"`와 글자 크기로 배치되므로,
차트 bbox의 고정 94% 선을 가시 글자의 공통 경계로 사용할 근거가 없다.

뒤이어 실행될 Native 검사 여섯 묶음도 같은 최소 글꼴 환경에서 모두 통과했다.
동일 CI에서 먼저 통과한 검사와 새로 드러난 실패를 구분하며, 범위를 다른 실제 HWP로 확대하지 않는다.

## 보정할 검증 계약

제품 renderer·글꼴 조달·차트 배치는 바꾸지 않는다. 원본과 새 한컴 2020 MCP Print를
Native/fresh WASM의 전체 한 쪽에서 먼저 비교하고 직접 판독한 뒤 검사만 보정한다.

공개 `DocumentCore::render_page_png_native`가 소비하는 같은 layer tree에서 원본의
제목·눈금·카테고리·범례 12개가 각각 최종 출력에 기여하는지 검사한다. 특정 라벨 하나만 제외한
대조 layer를 같은 Skia 경로로 그려 원 출력과 비교한다. 다른 텍스트·선·마커는 그대로 두므로
격자 잉크가 누락된 글자의 존재를 대신하지 못한다. 원본 라벨의 SVG 존재도 각각 확인한다.
고정 px·bbox 비율 영역·특정 글리프 농도 대신 **개별 텍스트와 가시 출력의 관계**를 보호한다.

원본 배치의 정확성은 독립 Print·전쪽 TSV·review/overlay로 확인한다. 소비 단계의 SVG를
조판 위치의 독립 정답지라고 표시하지 않는다. 글꼴을 전혀 공급하지 않은 음성 대조에서는
같은 검사가 실제 글자 소실로 실패해야 한다. 기존 산점도 검사는 변경하지 않는다.

## 실제 보정·검증 결과

제품 source는 바꾸지 않고 `tests/issue_2293_chart_png_text.rs`의 고정 영역 검사만
위 관계형 검사로 바꿨다. 변경 전 하단 6% 밴드와 그 좌표 계산·잉크 helper를 제거했다.
원본의 라벨 하나만 제외한 대조를 실제 Skia로 출력해 그 라벨의 가시 기여를 확인한다.
차이 픽셀 개수나 특정 위치·패턴·획 농도를 기대값으로 고정하지 않는다.

- 새 MCP Print: engine 2020·실제 11.0.0.9136·PrintMethod 0·한 쪽.
  job `b8ee495b-7687-4ef2-af4a-a08f568bba2c`.
- 고정 base Native/fresh WASM: **85.57830% / 78.70130%**.
  후보: **97.45943% / 96.57949%**. 변경 전 미달은 비교 자료로 남기고 후보의 양쪽
  90% 이상·쪽수 일치와 직접 review/overlay 판독을 확인한 뒤 검사를 변경했다.
- **97% 수준의 쪽이므로 px·고정 영역 회귀를 추가하지 않았다.**
  원래 위치의 독립 정답은 Print로 확인하고 테스트는 개별 텍스트의 가시 출력 관계를 보호한다.
- 정상 환경 **2 PASS**, CI 최소 글꼴 환경 **2 PASS**. 같은 새 검사에서 원본 라벨 12개가
  각각 실제 출력에 기여했다. 글꼴 부재 음성 대조는 `가시 출력에 기여하지 않는다`로
  **FAIL(exit 101)**이었다. 기존 산점도 검사는 그대로 통과했다.
- 파생 suite 재생성이 빠져 0건으로 끝난 첫 실행은 근거에서 제외했다.
  재생성 후 실제 두 검사 실행과 종료 코드 0을 확인했다.
- fmt·Native/WASM32/workspace all-target/해당 Native Skia Clippy·workspace build·suite
  정책 검사를 공유 `target/pr-review`, Cargo 8 jobs로 순차 실행해 모두 통과했다.

[전쪽 TSV·원본/Print·같은 페이지의 review/overlay·source 증거](../pr/assets/s21_20261027/ci_chart_labels/README.md)를
정식 경로에 보존했다. 원문 로그는 ignored `output/pr-review/semanticist21-20261019/logs/`의
`ci-chart-label-relational-4824.log`, `ci-chart-label-{minimal,empty}-4825.log`,
`ci-chart-basic-validation-4826.log`다. 기본 검사 뒤 추가 commit을 push하라는 사용자 지시에 따라
같은 PR에 반영한다. 새 head의 CI·병합은 아직 완료로 기록하지 않는다.

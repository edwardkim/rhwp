# 개인정보 41·42쪽의 닫힌 컷과 완전한 이어받기 정렬

Issue: #7445. 보정은 기존 `samples/task2097/75544_pii_bunseok.hwpx`에 한정한다. 새 HWP 보정 대상을 추가하지 않았다. 2025 행정업무운영 편람 242·277쪽은 완료 범위의 보존 검사이며 이번 보정 대상이 아니다.

- 입력 SHA-256: `f429768ec02e5a849d08c06678cbee029fdc8618ca9e2bdcdbba485087fe71f1`
- 독립 기준: `pdf/task2097/75544_pii_bunseok-hwpx-2020.pdf`, 66쪽, SHA-256 `0fef38f44b3825f1c97869c2720112536504eaf30cec2a2d2af6d4399ffe8770`
- 고정 base: `f7abae4a8e70c062c378db06ad64d2d2eacb88cd`
- 실행 소스: parent `d4f0a28eceed68d9121d6940ef0946006e7dbdc8` + Runtime patch `d4242f4696a5c0e29b7626d29384447610a586a299756e890259b58595e7f291`. 이후 정식 회귀만 추가했고 Runtime hash가 동일함을 확인했다.
- 같은 선언 글꼴 환경·실제 713개 공급 파일 hash, 96dpi, compat2022, Print, PDFium smooth_rgb_v1, raw 2px·90% 문턱. Native는 실제 Skia PNG, WASM은 실제 Canvas2D 출력이다.

## 원인과 공통 소비

1. 행 prefix를 뺀 예산 `23.333333333333258px`와 같은 닫힌 셀의 점유 `23.333333333333336px`를 엄격 비교해 한 줄을 이월했다. `CellCutFrame::fitting_height`의 결과를 비교할 때 행 오프셋과 유닛 누적 연산의 f64 오차 규모만 구분한다. HU/px 여유나 문서별 조건을 넣지 않았다. 선택한 컷의 같은 물리 높이가 예약과 paint로 전달된다.
2. 그 첫 줄을 앞쪽에 복구하자 42쪽에는 남은 두 라벨 줄만 속한다. 정렬은 행 서수·첫 논리 행만 보아 블록 컷을 Top으로 바꿨다. 내용 선택과 동일한 `cell_cut_window` 및 `is_split_start_row`를 소비해 완전히 선택한 일반 텍스트 suffix의 원본 Center 정렬을 보존한다. 이 소유를 composition window와 마지막 `effective_align`/`text_y_start`까지 전달한다. 실제로 더 잘리는 내용과 개체 셀의 기존 처리는 유지한다.

## 실제 시각 결과

| 쪽 | Native 전 | Native 후 | fresh WASM 전 | fresh WASM 후 |
|---|---:|---:|---:|---:|
|41|97.93657|99.99530|97.90569|99.99527|
|42|97.94921|99.82579|98.11879|99.82542|
|65|100.00000|100.00000|100.00000|100.00000|
|66|100.00000|100.00000|100.00000|100.00000|

양쪽 전 66/66쪽, 90% 미만 0쪽. 최저는 46쪽 Native 96.57165%, fresh WASM 96.56850%다. 41·42쪽 외 64쪽 PNG는 각각 수정 전과 byte 동일하며, Native 본문 소유별 연결 텍스트 1,826개도 동일하다. 페이지별 라벨 소유와 남은 Center 정렬은 별도로 검사한다. 전후 review/overlay를 직접 대조해 앞쪽 첫 줄, 뒤쪽 두 줄의 소속·세로 정렬 및 표 하단 괘선을 확인했다. 나머지 기존 미세 래스터 차이는 완전한 픽셀 일치로 표현하지 않는다.

fixed base Native의 41/42쪽 99.23712/97.95504%와 대조하면 두 쪽 모두 개선됐고 전 66쪽에 0.1%p 이상 하락이 없다. **fixed base fresh WASM의 같은 조건 전쪽 비교는 아직 미완료**다. 다른 문서의 최종 전체 검증을 이 결과로 대신하지 않는다.

## 정식 회귀와 실행 구분

`issue_5846_closed_block_cut_keeps_cost_label_owners_and_alignment`는 앞쪽 `11.비용편익`, 뒤쪽 `분석(정성분석)`의 단일 소유·순서, 원본 Center 정렬을 검사한다. 양쪽 41·42쪽 raw 98% 이상과 직접 판독을 먼저 확인한 뒤 독립 PDF 라벨 상단의 ±2px 검사 세 개를 함께 추가했다. 독립 기대값은 1022.118896484375, 194.2789103190104, 222.11891682942706 CSSpx다.

같은 현재 테스트 파일을 수정 전 P51 Native 라이브러리와 연결한 Rust test harness에서 신규 검사 1 FAIL / 0 PASS, 0.17초였다. 수정 후 정식 manifest/nextest 경로에서는 신규 검사와 기존 65·66쪽 검사 합계 **2 PASS / 0 FAIL, 0.590초**다. 수정 전 SDK harness와 수정 후 정식 suite의 실행 경로를 구분한다. 기존 검사를 제거하거나 기대값·문턱을 완화하지 않았다.

최초 포맷 전 harness가 남아 0건 수집된 실행은 무효다. 포맷 후 manifest를 재생성하고 실제 배정·수집을 확인해 위 2건을 실행했다. 잘못된 top-level rlib 선택 및 임시 라이브러리 파일명에 따른 컴파일 오류도 결함 FAIL로 세지 않았다. 최종 fmt·base 대비 manifest/unit-tier 검사는 PASS다. 전체 nextest·나머지 범위 gate는 별도로 필요하다.

## 증적

- [Native 전 66쪽 TSV](native-before-silhouette.tsv) · [Native 후](native-after-silhouette.tsv) · [fixed base Native](base-native-silhouette.tsv)
- [fresh WASM 전 66쪽 TSV](wasm-before-silhouette.tsv) · [fresh WASM 후](wasm-after-silhouette.tsv)
- [Native 41쪽 전 review](native-before-p041-review.png) · [overlay](native-before-p041-overlay.png) · [후 review](native-after-p041-review.png) · [overlay](native-after-p041-overlay.png)
- [Native 42쪽 전 review](native-before-p042-review.png) · [overlay](native-before-p042-overlay.png) · [후 review](native-after-p042-review.png) · [overlay](native-after-p042-overlay.png)
- [WASM 41쪽 전 review](wasm-before-p041-review.png) · [overlay](wasm-before-p041-overlay.png) · [후 review](wasm-after-p041-review.png) · [overlay](wasm-after-p041-overlay.png)
- [WASM 42쪽 전 review](wasm-before-p042-review.png) · [overlay](wasm-before-p042-overlay.png) · [후 review](wasm-after-p042-review.png) · [overlay](wasm-after-p042-overlay.png)

실행 원장은 ignored `output/pr-review/7445-completed-20261009/rebase-8155-20261022/`의 `pii41-suffix-owner-v4`, 해당 `-visual`/`-wasm`, `final-stage55-v2`에 보존한다. 최종 Native binary SHA-256은 `d196aad916792921aa04c039f5f7fcd683ec0904edec7e6f43efc1203cc722a9`다. fresh WASM은 Mac locked wrapper web/pkg/no-opt의 로컬 대체 빌드이며 Docker 최적화 통과가 아니다. root pkg·Studio/public·frozen package의 JS `aea5f6639676442e0628b640b75d0f70eb300481421112d23b661a082f695a68`, WASM `1192fe19a256cf457bf156746cba95abeb420ab7562438a6ca792734ed7e1f71`이 일치한다. 실제 Studio 초기화 응답의 hash와 실제 소비 경로 검증은 아직 완료하지 않았다. PR 준비 전체 완료를 주장하지 않는다.

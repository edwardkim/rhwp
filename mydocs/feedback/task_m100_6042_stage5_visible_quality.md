# 피드백 — #6042 scroll 정착 전 visible 쪽 화질

- **일시**: 2026-09-02 KST
- **대상**: [#6042](https://github.com/edwardkim/rhwp/issues/6042), Stage 5 사용자 검증
- **지적자**: 작업지시자
- **비교 환경**: 로컬 Stage 3 `:4191`, Stage 4 보정 `:4193`, `samples/exam_kor.hwp`, 100%, 두 쪽 보기
- **상태**: 구현·자동화 A/B 통과, 사용자 재검증 대기

## 관찰

cursor가 없는 쪽으로 스크롤하면 클릭 전에는 글자가 저해상도로 보이고, 클릭한 뒤에야 선명해진다.
같은 화면의 클릭 전·후 캡처에서 배치와 CSS 크기는 같지만 surface DPR이 달랐다.

- 클릭 전: 화면의 두 쪽 모두 `effectiveDpr=1`, `tier=economy`
- 한 쪽 클릭 후: 클릭한 편집 쪽은 `effectiveDpr=2`, `tier=screen`; 다른 쪽은 DPR 1 유지
- 4-layer 두 쪽의 raw DPR 2 예상 비용은 약 42.8M surface px로 #6467의 32M visible 예산을 넘는다.

Stage 3(`:4191`)과 현재 Stage 4 보정(`:4193`)에서 같은 현상이 재현되므로 #6042 scheduler가 새로 만든
회귀는 아니다. #6467의 예산·히스테리시스와 “편집 focus만 raw DPR 보호” 계약이 결합해, 스크롤로 새로
보이는 쪽의 이전 낮은 DPR이 유지되는 현상이다.

## 기각한 수정

스크롤로 보이는 쪽을 편집 focus로 바꾸지 않는다. viewport visibility와 caret/editing focus는 다른
상태이며, 이를 합치면 caret·현재 쪽·눈금자 의미가 바뀐다. scroll event마다 즉시 DPR 2로 다시 그리는
안도 입력 중 main-thread raster와 메모리 churn을 늘려 #6042의 목표와 충돌하므로 채택하지 않는다.

## 승인한 보정 계약

1. active scroll 동안 이미 붙어 있는 page surface는 현재 DPR 그대로 유지한다. DPR 1이면 1, 이미 2면
   2를 유지하며 visibility 변화만으로 재래스터하지 않는다.
2. 마지막 scroll 입력 뒤 150ms 동안 새 입력이 없으면 `scroll-settled` 세대를 만든다.
3. 양 축에서 8 CSS px 이상 겹치는 visible 쪽을 center-first로 page 단위 승격한다. 편집 focus·caret·ruler
   이벤트는 발행하지 않는다.
4. 해당 visible 전체의 raw 비용이 64M surface px 이하면 raw DPR을 사용한다. 초과하면 DPR 1.5를
   시도하고, 1.5도 64M을 넘으면 #6467 planner 결과를 유지한다. 이 hard gate는 32M/40M cache 예산을
   대체하거나 열 수 기반 화질 규칙을 도입하지 않는다.
5. 새 scroll·zoom·resize·mutation·문서 교체·renderer 전환은 대기 중인 정착 승격을 취소한다. 이미
   완료된 raw surface는 새 scroll 중 다시 낮추지 않고 재사용한다.

계산은 매 scroll event가 아니라 정착 시 현재 visible 소수 쪽만 한 번 합산한다. hot path에서는 현재
surface의 requested DPR을 읽어 잠그는 상수 시간 page 작업만 추가하므로, 보정 자체가 전체 문서를
재탐색하거나 DPR 계산 병목을 만들지 않아야 한다.

## 검증 계약

- fake clock으로 debounce·재입력 취소·dispose 취소를 검증한다.
- planner 실행 테스트로 scroll 중 DPR 1/2 surface 고정, 정착 raw/1.5/hard-gate fallback을 검증한다.
- 정착 승격은 1·2쪽도 입력 callback 안에서 동기 raster하지 않고 rAF page slice로 실행한다.
- `:4191`과 새 `:4193`에서 동일 문서·scroll 위치의 클릭 전/후 DPR, raster 수, 첫 정착 시간과 화면을
  비교한다. 클릭 없이 DPR 2가 되되 caret·눈금자·배치·scroll 좌표는 바뀌지 않아야 한다.
- 성능 표본은 새 150ms 의도 대기와 실제 page raster 시간을 분리해 보고한다.

## 구현·검증 결과

- active scroll은 붙어 있는 surface의 실제 requested DPR을 잠그고, 150ms 정착 뒤에만 별도
  `scroll-settled` generation을 시작한다.
- 정착 visible 전체가 64M surface px 이하면 raw DPR, raw가 넘으면 DPR 1.5, 1.5도 넘으면 기존
  #6467 planner 결과를 사용한다. 편집 focus와 ruler 대상은 변하지 않는다.
- `exam_kor`, 두 쪽, 100%, DPR 2에서 클릭하지 않고 다음 행으로 이동했을 때 Stage 3은 visible 두 쪽이
  DPR 1에 남았고 보정은 두 쪽 모두 DPR 2로 회복했다.
- 자동 34%·3열에서는 수정 전후 visible 9쪽이 모두 DPR 2이고 active pixel·scroll 좌표도 같았다.
- 200% 한 쪽 visible은 DPR 1→1.5(48,125,352px), 두 쪽 visible은 합산 gate 때문에 DPR 1 유지로
  raw/1.5/planner 세 갈래를 실제 브라우저에서 확인했다.
- fractional DPR에서 정수 physical canvas 크기를 DPR로 역산하면 1px 미만의 CSS box 변화가 생길 수
  있어, main·overlay 모두 virtual layout의 논리 page 크기를 사용하도록 회귀 테스트와 함께 보정했다.

상세 A/B·성능·화면 증적은
[Stage 5 사용자 화질 보정 보고](../working/task_m100_6042_stage5_scroll_quality_correction.md)에 기록한다.

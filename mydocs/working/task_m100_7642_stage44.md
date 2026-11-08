---
kind: task
status: active
canonical: mydocs/working/task_m100_7642_stage44.md
last_verified: 2026-10-09
---

# #7642 Stage 44 — 미주 경계의 실제 점유 측정과 이월 순서 보정

Issue: #7642

## 기준과 독립 관측

- 작업 head `4e932a86e7da5266440e141fdf9fe36f4e35a0fc`, base
  `79abd9e49675f4286eaed38197f7e837afb744e7`이다. rebase 전후 전체
  tracked tree는 동일하다. 이전 SHA의 실행 결과를 새 head 결과로 표기하지 않는다.
- 요청 52개와 #1139 추가 3개 입력을 유지한다. 이전 소스의 쪽수 대조는
  50개 일치 / 5개 불일치이며 최신 시각 완료를 뜻하지 않는다.
- 독립 한컴 Print에서 구분선 없는 2024년 11월 문서의 미주 8 제목은
  10쪽 오른쪽 단 하단에 있다. 기존 Native는 11쪽 왼쪽 단으로 넘긴다.
  앞 점유 끝과 미주 사이, 제목 높이의 합은 해당 단에 들어간다.
- 미주 사이 9/8/7 HWPX의 첫 차이는 미주 5 제목이다. 앞 단의 표·그림 때문에
  점유 query가 `None`이 되어 전체 묶음의 저장 높이로 이월한다.
  원본·독립 Print·행별 대응·실행 로그는 ignored
  `output/pr-review/issue7642/failed-inputs-20261008/stage15-current-validation/`
  아래 `batch-endnote-boundary-analysis/`, `batch-flow-break-debug/`에 있다.

## 원인과 소비 경로

1. `measure.rs::measure_endnote_column_with_item`은 실제
   `LayoutEngine::measure_endnote_column_bottom`을 실행해 흐름 예약 끝과
   실제 점유 끝을 함께 반환한다. 현재 단과 후보는 실제 전역 소유자,
   float 배치, `measured_tables`를 사용한다.
2. `paragraph/reflow_fit.rs`의 일부 query는 실제 후보만 측정하면서도
   `future = Some(empty)`와 빈 표 측정값을 넘긴다. 이는 미래 문단의
   미지원 검사를 현재 단에도 적용하여 이미 배치한 표·그림을 거절한다.
   미래 입력 없는 query는 `None`과 실제 표 측정값을 전달한다.
3. `paragraph.rs`의 `advance_for_fit` 실행은 새 제목의 실제 fit보다 먼저다.
   실제 후보의 점유와 필요한 미주 간격을 확인한 뒤 이월한다. 명시적 단/쪽
   나눔과 `keep_with_next`는 면제하지 않는다. 실제 이동 여부를 후속 높이
   계산에도 전달한다.
4. 간격 하한은 실제 paint의 `layout.rs`와 같은
   `endnote_spacing::origin_after_gap` 결과를 소비한다. 흐름 예약에 이미
   포함된 간격을 다시 더하거나 좌표 clamp·마지막 빈 쪽 삭제로 숨기지 않는다.

## 묶음 수정과 검증

- 같은 잘못된 측정 문맥을 쓰는 전체 문단·단일 줄·새 제목·제목 뒤 첫 줄
  query를 함께 보정한다. 파일명·문항 번호·고정 좌표로 분기하지 않는다.
- Native를 한 번 빌드하여 관련 다섯 불일치 입력과 정상 대조군을 먼저
  확인한다. 필요하면 새 관측에서 남은 원인을 다시 분석한다.
- 새 head에서 요청 전체 Native/fresh WASM TSV, 전체 쪽수와 대표 review를
  생성하고 수식·간격을 확대 판독한다. 한 쪽이라도 90% 미만이면 실제
  출력을 더 보정한다. 관계형 회귀 전환은 해당 원본의 선행 조건 이후다.
- 전체 회귀·필수 lint·manifest/unit policy는 고정 base와 공용
  `target/pr-review`에서 순차 실행한다. 기존 실패를 완화하거나 제거하여
  통과시키지 않는다. 13개 기존 whitespace 오류는 내용 변경 없이 함께 정리한다.
- 계획 고정 뒤 구현한다. 로그·TSV·JSON·nextrun은 ignored output에 둔다.
  원본 증거와 새 실행 결과, 미검증 범위를 구분하여 결과를 기록한다.

## 첫 묶음의 실행과 추가 원인

첫 묶음의 Native 빌드는 exit 0(394.55초), fmt check와 Native Clippy,
고정 base의 integration manifest 및 source unit tier 검사는 exit 0이다.
빌드 당시 소스는 계획 commit 이후 dirty patch로 해시 고정했으며
`endnote-head-fit-native-provenance.json`과 로그에 기록했다.

쪽수가 달랐던 다섯 원본은 여전히 각각 한 쪽씩 많다. 구분선 없는 HWPX의
10·11쪽은 71.53949% / 34.81498%, 9/8/7 HWPX의 10쪽은 67.22060%다.
앞의 정상 문단까지 이월한 실제 회귀이므로 첫 묶음을 개선 완료로 채택하지 않는다.
정상 대조군 2022년 11월 HWPX 4쪽은 gate를 통과했으나 이를 실패 원본의
증거로 대신하지 않는다. 생성된 review는 `endnote-head-fit-focused-review/`에 있다.

실제 배치와 scratch 호출을 대조하여 누락된 문맥을 추가로 확인했다.
scratch는 실제 단의 inline flow 계획·inline placement·wrap·overlay를
빈 값으로 만들고 단을 가상 용지 원점에서 실행한다. 또한 미주 구분선
항목을 제거하여 본문 후행 간격 회수와 첫 미주 원점 처리를 생략한다.
같은 `build_single_column`을 호출한다는 사실만으로 공통 결과가 아니다.
실제 `ColumnContent`에 전달될 확정 계획과 원래 용지·단 문맥을 측정에서도
사용하고, 반환된 용지 좌표를 단 상대 예산으로 변환하도록 함께 보정한다.
## 실제 단 문맥을 전달한 보정 결과

Native 빌드는 exit 0(350.61초), fmt check는 exit 0(15.92초), Native
Clippy는 exit 0(51.81초)다. 이번 바이너리와 전체 소스 해시는
`endnote-paint-context-fit-native-provenance.json`에 고정했다.

- 구분선 위 0·미주 사이 20·구분선 아래 2 HWP/HWPX는 각각 23쪽에서
  22쪽으로 줄어 독립 Print의 22쪽과 일치한다. 전체 페이지의 시각 통과를
  뜻하지 않는다. 첫 미주 이후 남은 간격 차이는 계속 보정한다.
- 구분선 없는 HWP/HWPX의 미주 1~7 위치는 이전 정상 출력과 동일하다.
  미주 8 제목이 10쪽 오른쪽 단 하단에 남고, 미주 9 제목이 11쪽 왼쪽
  단의 Print 위치와 대응한다. 두 문서는 여전히 23/22쪽이다.
- 9/8/7 HWPX의 미주 5 제목도 Print와 같은 10쪽 왼쪽 단에 남았다.
  뒤 문단의 간격과 단 전환은 미달이며 전체 22/21쪽 차이가 남아 있다.

| 새 Native/Print 직접 대조 | 선택 쪽 | 2px 실루엣 일치율 | 판정 |
| --- | --- | --- | --- |
| 구분선 없는 2024년 11월 HWPX | 10 | 97.63% | 선택 쪽 점수 이상, 문서 쪽수 불일치 |
| 같은 원본 | 11 | 65.07700% | 미달, 추가 보정 필요 |
| 9/8/7 HWPX | 10 | 77.29898% | 미달, 추가 보정 필요 |
| 2022년 11월 HWPX 대조군 | 4 | 98.74440% | 선택 쪽 gate passed |

새 10·11쪽 review를 직접 열었다. 10쪽의 미주 8 제목과 앞 문단 소속은
복구됐지만 수식의 세부 차이를 점수로 면제하지 않는다. 11쪽은 미주 11
본문 한 줄이 다음 단으로 먼저 넘어가고 그래프의 세로 모양 차이도 보인다.
실제 후보 fit 이후 검사만으로는 앞의 이월을 취소할 수 없다는 같은 원인이
일반 본문 줄에도 적용된다. 관련 줄과 그림 프레임·메타파일 소비를 함께
분석한 뒤 다음 묶음을 보정한다. 이 시점의 대표 증거는 ignored
`endnote-paint-context-fit-focused-review/`와
`endnote-paint-context-fit-boundary-analysis/`에 있다.

90% 미달과 전체 쪽수 불일치가 남았으므로 회귀 기대값·baseline은 바꾸지
않았다. 이번 Native 실행은 fresh WASM 또는 전체 회귀의 완료 증거가 아니다.

구현·시각 게이트·전체 회귀·PR 통합은 계속 진행하며 미완료다.

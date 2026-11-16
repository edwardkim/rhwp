# johndoekim PR 2건 검토 계획

## 접수와 범위

작업지시자는 2026-11-16 johndoekim 기여자의 PR 처리를 시작하도록 지시했다.
두 원 PR에 `edwardkim` 리뷰어를 지정했고 API로 재확인했다. 공개 댓글·승인 리뷰·push·merge는
아직 수행하지 않았다. 원 기여 commit과 fork branch를 보존한다.

| PR | 대상 | 고정한 원 head | 최초 CI |
| --- | --- | --- | --- |
| #9081 | #9076 글상자 안 미기입 누름틀과 인라인 표의 줄 구성 | `3a2fcbf7d12d4f57fa3fcff33721ff7ed9059738` | 필수 검사·정책 성공 |
| #9084 | #9082 값이 든 누름틀 끝 표식과 뒤 컨트롤의 HWP5 저장 순서 | `81dd920099bae78d8d66148ddf9614785760deac` | 필수 검사·정책 성공 |

최초 기준은 `upstream/devel` `7ee4fe0327d0cdfc8dec12c450e5dd5b5369b826`이다.
`review/integration-9076-johndoekim-fields`에서 원 head를 #9081 → #9084 순서로 merge했다.
충돌 없이 적용됐고 현재 제품 코드의 검토 후보는 `3843f897ae410fdf67347651b39214d9dd9cafff`이다.
개별 원 PR과 누적 후보의 판정을 구분한다.

## 선행 원인과 소비 경로

- #9081: 저장된 안내문을 적재 때 숨기면 저장 LineSeg의 문자 분할이 현재 표시 내용과 달라진다.
  `shape_layout::layout_textbox_content`의 기존 재조판 사전 단계가 같은 글상자 폭으로 줄을 만들고,
  뒤 문단 vpos·`control_line_indices`·인라인 표 배치가 그 결과를 소비하는지 확인한다.
  #9057의 누름틀 없는 표 줄 구성과 #9023의 숨긴 안내문 처리를 보존한다.
- #9084: `serializer/body_text.rs`가 이미 본문에서 지난 FIELD_END를 뒤 컨트롤보다 먼저 방출한다.
  같은 함수의 `control_stream_slots`·`field_stream_ends`를 쓰는 개체 삽입·이동도 집중 검증한다.
  원본 raw stream 재생만으로 저장 성공을 판정하지 않는다.
- #9058: `Paragraph::control_source_positions_with_storage`에서 짝 있는 끝 표식의 슬롯을 세지 않는
  기존 배치 문제다. 실제 한컴 저장본 3개와 독립 Print PDF를 확보했다. #9084의 저장 순서 수정과
  동일한 결함으로 간주하지 않으며, 구조 보존 검토에 조판 검증을 일괄 요구하지 않는
  [접수 절차 2.7](../../manual/pr_review/intake_and_review.md#27-조판-원칙-준수-검토)을 적용한다.

## 검증과 종료 조건

1. 최신 base/후보에서 실제 Native 출력과 독립 Print를 대조한다. #9081은 목표·정상 대조군의
   Native/fresh WASM 전쪽 TSV와 확대 판독, 편람 gate와 정본 보호 기준을 확인한다.
2. #9084는 실제 CLI 편집 → HWP 저장 → 다시 열기 → 편집·재저장 → 다시 열기와 HWPX 교차 저장을
   실행한다. 한 글자 값·네 글자 값·안내문을 가진 입력에서 글자 값뿐 아니라 끝 표식과 표의 순서를 검사한다.
3. 새 회귀·인접한 줄 구성/필드·인라인 개체 편집 검사를 최소 글꼴 환경에서 실행한다.
   원 head의 녹색 Full CI를 재사용할 범위와 로컬에서 실행한 범위를 개별 review에 명시한다.
4. 새 devel과 원 head를 다시 확인하고 필요한 재검증·병합 시뮬레이션을 완료한다.
   미완료 필수 검증이 있으면 승인 또는 제출 준비 완료로 기록하지 않는다.
5. 개별 검토 기록과 구체적인 원격 처리안을 준비한 뒤 작업지시자의 원격 승인을 확인한다.

진행 중 원본·로그·이진 파일·임시 PDF는 기본 작업공간의 Git 제외 경로
`output/pr-review/johndoekim-20261116/`에 둔다. 완료 증적은 개별 review와 정식 자산 경로에 보존한다.
사설 글꼴 프로그램은 커밋하지 않는다.

## 착수 시 재현 결과

- 최신 devel의 #9076 원 사례는 Native 전 1쪽 raw 29.43%, 누름틀 없는 대조군은 100.00%다.
- 최신 devel의 실제 HWP 반복 저장에서 한 글자/네 글자 값의 끝 표식이 표 두 개 뒤로 이동한다.
  글자 값은 유지되므로 필드 값 조회만으로 이 결함을 판정하지 않는다.
- #9058의 기존 세 문서는 최신 devel Native raw 64.87%·57.23%·58.36%다.
  후보 결과는 아직 측정 중이며, 이 수치를 후보의 회귀 또는 해결 결과로 기록하지 않는다.
- 위 수치는 착수 당시 기록이다. 이후 실제 검증은 개별 review와 정식 증적에 연결한다. 이 문서는 최종 승인 기록이 아니다.

## 검토 중 반영한 변경

최신 devel `67b4ff8d3a55e5bb50c46a44315de749a4ef07a7`의 #9102를 clean merge했다.
Rust 제품·Cargo·fixture·실제 캡처 입력 변경은 없고 새 공개 공통 gate의 8개 실제 실행과 단위 검사 9개를 통과했다.
#9081의 회귀 주석만 `e958314dc9078fdb55e7cde04227a1a22054a72d`에서 보완했으며 제품·assertion은 같다.
이 새 test head에서 원 CI를 대신 쓰지 않고 모든 lint와 전체 13,614개 회귀를 실제 재실행해 통과했다.
Native 공식 3종도 같은 최소 글꼴 환경에서 4073개 모두 통과했다. 실행 당시 로컬 SHA와 제출 코드의 입력 동일성은 검증 기록에 구분했다.

원 사례 Native/fresh WASM은 각 100.00%, 보호 3,542쪽은 예외 없이 하락 0건이다.
별도 사설 수식 글꼴은 독립 문자표 Print로 복구해 원본 77개 프로그램의 동일성과 실제 출력을 확인했다.
[검증 기록](../assets/johndoekim-20261116/README.md)과 [원격 처리안](../assets/johndoekim-20261116/remote-action-plan.md)에
필수 검사 완료 뒤의 구체적인 범위와 승인 경계를 기록한다.

## 원격 승인 뒤 새 devel #9106 반영

작업지시자는 후보 `105be723b2`의 upstream push·Open PR·새 head CI·self-review·merge와 후속 처리를 승인했다.
직후 원격에서 #9106 `6c1d134d42213ce0fb5f66b5ad2bf343b7b64fe0`의 추가 병합을 발견했다. 최신 base 반영·검증 요구에 따라
충돌 없이 merge한 `4deadfa9638baffc3b4dc7e9d56a3edff9f83aeb`에서 검증을 갱신한다. 아직 원격 push·새 PR은 수행하지 않았다.

- source 변경: 분할 셀 정렬·표 바깥여백, 구형 Print의 표시 em, Native SVG 프로그램 준비,
  CanvasKit 사전 작업 예산. 새 정본은 중첩 표 Native/WASM 각 17쪽을 수용했고 SHA는
  `2280e97844b2cb262ea73e9c49f09f56c434097fed49af75fd9a7f6198d43da7`이다.
- 선행 근거: #9106의 원 code `5de0973a`, 독립 한컴 2020 Print, 실제 각주 50쪽·중첩 표 17쪽,
  관련 회귀·전쪽 증적·공급 자격을 읽고 현재 source와 대조했다.
- 범위: 원 PR 두 목표와 실제 반복 저장, 새 base의 각주·중첩 표 PNG 및 Native SVG 7·28쪽,
  필수 최소 글꼴 Rust 게이트·편람·공개 공통 gate·정본 보호 전체를 새 생산자로 검증한다.
- base/candidate Native와 최적화 WASM은 공용 target에서 순차 빌드하고 별도 output에 고정한다.
  source가 다른데 후보 Native가 base와 동일한 바이너리를 재사용한 캐시 오류는 무효로 구분했다.
  해당 rhwp 패키지의 프로필 캐시만 무효화해 재컴파일하며 base snapshot과 다른 의존성 캐시는 보존한다.

## 최신 source의 최종 로컬 완료

`4deadfa9638baffc3b4dc7e9d56a3edff9f83aeb`에서 전체 회귀 13,632개·집중 30개·Native 공식 4,073개·모든 lint·편람 383쪽·공개 공통 8개 실행을 실제 완료했다. 목표 전쪽 Native/fresh WASM은 모두 100.00%, 새 base와 후보의 각주/중첩 표 각 67쪽 PNG는 동일하다. 실제 CLI와 최적화 WASM API의 두 차례 HWP 저장·재열기·HWPX 교차 저장도 완료했다.

보호 3,542쪽 중 Native 중첩 표 10쪽 한 행의 99.86%/99.85% 기존 차이만 작업지시자가 한정 수용했다. 다른 하락·누락은 0건이며 정본·허용치는 변경하지 않았다. [최종 실행 기록](../assets/johndoekim-20261116/latest-devel-revalidation.md)과 [확정된 원격 범위](../assets/johndoekim-20261116/remote-action-plan.md)를 따른다. 현재 로컬 필수 검증을 완료했고, 새 PR의 CI·self-review·merge·후속 처리는 승인된 다음 실행 단계다.

## 원격 통합 완료

[수용 PR #9126](https://github.com/edwardkim/rhwp/pull/9126)의 code/trailing head CI·self-review를 실제 완료하고 `fb7367574f13e65111f162bb79238cb0af45b36e`로 원 기여 두 SHA를 보존해 병합했다. 이 문서의 이전 단계 상태는 당시 기록이며 현재 완료 상태는 [후속 원장](../assets/johndoekim-20261116/closure.md)을 따른다.

# #7998 P4 종료 범위와 다음 구현 경계 감사

- 날짜: 2026-10-19 (Asia/Seoul)
- 지시: 진행률 보고 뒤 다음 절편 진행 승인. 원래 PDF 표/문단 목표의 필수 잔여 범위를
  확인하고, 승인된 시각 판정·예외와 별도 native 결함을 구분한다.
- 검토 head: `7e089eab9` / 제품 checkpoint `3f9434a1f`.
- 이전 결과: [stage13](task_m100_7998_stage13.md). P4 전체·P5 완료가 아니다.

## 현재 제품의 실제 상태

GitHub #7998은 OPEN·담당 edwardkim이며 관련 열린 PR은 없음을 읽기 조회로 확인했다.
이슈 원래 범위와 추가 승인된 글줄/문단·편집/저장 범위를
[계획 4.3절](../plans/task_m100_7998_impl.md#43-분할정복-실행-순서와-절편별-완료-조건)에
대조했다. 12개 절편의 수용을 필수 잔여 항목 전체 완료로 환산하지 않는다.

stage13의 runtime manifest 2,796개 파일이 현재와 동일함을 확인하고 기존 immutable
CLI를 재사용했다. 이번에는 두 **원본 전체를 실제 import-pdf → HWPX로 재가져와**
공개 상세 진단과 저장 XML의 실제 `tbl`/`tc`를 대조했다. 과거 생성본만 읽은 집계가 아니다.
원본 SHA는 계획 1절과 같다. 새 시각 캡처나 같은 제품의 재빌드는 하지 않았다.

| 현재 원본 | 실제 저장 표/셀 | native border를 가진 셀 | 실제 본문 Paragraph | 관측 메트릭 / legacy 줄 |
|---|---:|---:|---:|---:|
| 보고서 12쪽 | 15표 / 161셀 | 14셀 | 39개 | 294 / 17줄 |
| Excel 견적서 1쪽 | 1표 / 60셀 | 0셀 | 6개 | 14 / 0줄 |

표/셀 수는 작성 당시 구조의 정답 점수가 아니다. `nativeBorderCells`는 변 하나 이상이
속성으로 발행된 셀 수이며, 다른 모든 셀이 테두리를 가져야 한다는 기대도 아니다.
본문 Paragraph는 `bodyParagraphMapping.frames`에서 실제 발행된 `paragraphCount`다.
표 내부 문단을 더한 전체 문단 수와 구분한다. 초기 기록의 7표/128셀·26본문 문단·
218/15줄은 서로 다른 중간 checkpoint이므로 최신 진행량으로 반복 인용하지 않는다.

보고서의 legacy 17줄은 5쪽 2줄·6쪽 3줄·7쪽 12줄이며 모두 실제 셀 줄의 장평 표현
보류다. 본문에는 별도로 장평 3개/자간 2개 제안, 미확정 지역 끝 11개·frame 중첩 2개가
남는다. 견적서 본문에는 장평 1개·지역 끝/미확정 7개 제안이 남는다. 줄·제안·문단의
단위가 다르며 중복 없는 미완료 문단 수로 합산하지 않는다.

## 편집의 실제 미충족 경계

기존 공개 `pdf/issue7998/periodic-borders/phase-shifted-dot.pdf`를 수정 없이 가져와
`set_cell_properties_native`의 첫 셀 사방을 빨강 Solid로 변경하고 별도로 행을 삽입했다.
입력은 자격 확인 Dot과 위상이 달라 원래 paint를 보존해야 하는 반례다.

| 실제 API 관측 | 결과와 의미 |
|---|---|
| 변경 전 root 셀 소유 Shape/Picture | 280개. 개수만으로 모두 경계라고 분류하지 않는다. |
| 사방 스타일 변경 후 | 실제 BorderFill은 빨강 Solid로 바뀌지만 원래 280개는 그대로다. 공통 layer의 검정 색 참조 100개가 유지되고 빨강 4개만 추가된다. 기존 경계 소유를 교체하는 계약이 없으며 속성 변경 통과만으로 실제 교체 완료라고 할 수 없다. |
| 새 행 삽입 후 | 새 셀 2개에 소유 paint 0개. native BorderFill 상속과 원본 비표준 경계 paint 상속은 다른 경로다. |

같은 입력의 diagnostic helper와 immutable CLI가 만든 `Contents/*`는 바이트까지 같음을
확인했다. 공유 target의 다른 라이브러리를 연결한 결과를 현재 제품으로 오인하지 않았다.
이 실행은 **진단**이며 새 렌더링 회귀/golden·한컴 시각 통과로 등록하지 않는다.

생산→소비 경로는 다음과 같다.

- `structured_lowering/native_borders.rs:76`의 native 동등성 판정과 실제 구간/closure
  검증은 표현 가능한 실선/Dot만 BorderFill로 승격한다. 비동등 위상·틈·가림 보존은 정상이다.
- `structured_lowering.rs:1326`에서 해당 plan을 실제 paint partition에 전달하고,
  `:1428`의 속성 소비와 `:1447`의 셀 문단 Shape/Picture 소비로 나뉜다.
- `document_core/commands/table_ops.rs:1435`의 속성 변경은 BorderFill과 `:1513`의
  이웃 변을 갱신한다. 어떤 Shape/Picture가 어느 셀 변의 원래 paint인지는 공통 IR에 없다.
- `model/table.rs:501`의 `Cell::new_from_template`은 실제 내용 controls를 복제하지
  않는다. 배경/내용 그림까지 복제하면 안 되므로 모든 controls 복사로 해결하지 않는다.
- 다음 구현은 source 조각/변/공유 소유를 먼저 표현하고 그 소유를 실제 편집·저장·
  최종 paint가 함께 소비하게 해야 한다. bbox 근접 삭제·원래 틈을 실선으로 메우는 처리는
  원인 해결로 사용하지 않는다.

## 확정한 실행 순서와 완료 조건

[계획 4.4절](../plans/task_m100_7998_impl.md#44-p4-잔여-범위-확정과-종료-순서--2026-10-19)에
필수 잔여 구현을 두 경계로 모았다.

1. 비표준 경계의 공통 소유/양 저장 → 명시적 스타일 교체 → 새 행열 상속/삭제·복원.
   내용 그림·배경·가림·외부 잔여·공유 변과 이웃 중첩의 보존을 각각 확인한다.
2. 확정 문단의 장평/자간 표현 실패를 공통 글줄·실제 Paragraph·저장/재조판에서 보완한다.
   `structured_lowering.rs:135/:153`의 보류를 무조건 완화하지 않는다. 적용 범위의
   source font/advance와 최종 글자 원점을 연결하고 원문·공백·읽기 순서를 유지한다.

작성 구조 자체가 모호한 B01/B03의 정상 보류는 두 번째 표현 실패와 구분한다.
모든 후보를 채택하거나 Excel의 당시 객체 수를 복원하는 것이 종료 조건은 아니다.
한컴 native 클립·외부 재저장/다른 owner 등 일반화와 B21/B22는 기존 후속 경계를 유지한다.
현재 PDF 표/문단에서 실제 소비하는 경로의 누락·중복·저장 오류는 후속으로 숨기지 않는다.

두 구현 경계가 끝나면 P4 완료 판정 후 최신 devel 통합과 P5를 진행한다. 이번 감사는
P5의 새 head 전쪽 비교·최종 전체 회귀/lint/package나 PR 준비 완료를 대체하지 않는다.
이미 승인된 구현 방향과 좁은 계약의 설계 선택마다 승인을 반복 요청하지 않는다.

## 증거와 실행 범위

증거 root는 저장소 밖
`/home/edward/mydev/rhwp-review-evidence/pdf-table-feedback-20261017/semantic-stage3/p4-completion-audit`다.
ignored `output/pr-review/issue7998/p4-completion-audit`로 연결했다.
`source-identity.json`, 각 `*-import.json`/`*.hwpx`, `summary.json`·`*-summary.json`,
`probe.rs`·`probe-build.log`·`probe.log`·`probe-provenance.json` 및 실제 layer JSON을 보존했다.
원본 견적서와 파생 문자/그림 자료는 로컬에만 둔다.

실제 사용한 명령은 다음과 같다(두 번째 입력도 같은 명령의 경로만 변경).

```bash
<stage13 immutable CLI> import-pdf /mnt/e/tmp/content_operators-002.pdf \
  <audit>/operators.hwpx --json --table-inference
<audit>/probe <audit>
```

Native SHA-256은 `545de67902c7745ad65532218e825666ab5e6801b3ac493b0a4619482e2e793f`다.
runtime 동일성·현재 입력/Contents 대조를 확인했지만 이번에 Native/fresh WASM 전쪽을
재출력한 것은 아니다. 제품·테스트·fixture·workflow와 Studio 7700/7793 산출물은 변경하지
않았다. 기존 시각 판정/예외를 그대로 유지하며 새 시각 완료를 주장하지 않는다.

계획·백로그·본 보고서 3개의 상대 링크 검사와 `git diff --check`는 통과했다.
이번 문서/진단 절편에는 제품·정식 검사 변경이 없어 Cargo 전체 검사·WASM 빌드·
이미 통과한 시각 검사를 반복하지 않았다. 원격 반영·PR 생성은 수행하지 않았다.

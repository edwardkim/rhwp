---
kind: snapshot
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-10-09
---

# #7445 완료 항목 제출 후보

## 범위와 승인 경계

사용자는 완료된 항목과 보도자료 6쪽·8쪽 개선까지만 PR로 준비하도록 요청했습니다.
로컬 검증·커밋·PR 본문까지 준비하고 원격 push·PR 생성·이슈 갱신·병합 전 승인을 받습니다.
#7445 전체를 닫지 않으며 미완료 OLE 차트·양자 문서 보정은 기존 브랜치에 보존합니다.

- 보존 브랜치: `fix/7445-fidelity-20261005`
- 제출 후보: `fix/7445-completed-20261009`
- 기준: `upstream/devel`의 `6f66932a73ee6fd05a74a83bdc438c9499a89a12`
- 기존 원본과 정상 한컴 PDF를 유지하고 재산출한 완료 문서 PDF를 포함합니다.
- 사용자가 별도 작업으로 옮기도록 지시한 시험지 회귀 제거를 포함하고 문서는 삭제하지 않습니다.
- 부분 완료 양자 문서의 첨자 배율·간격 변경 두 개와 EMF 변경 세 개, OLE 범례 보정은 제외합니다.
  이 범위 분리가 다른 완료 문서에 미치는 영향도 실제 제출 후보에서 확인합니다.

## 보도자료 원인과 관계 검사

원본은 확장자가 HWPX이지만 HWP 5.x OLE 바이너리인
[`press_release_split_cell_nested_table.hwpx`](../../samples/issue3637/press_release_split_cell_nested_table.hwpx)입니다.
정상 독립 기준은 [한컴 Print 12쪽 PDF](../../pdf/issue3637/press_release_split_cell_nested_table-hwpx-2020.pdf)입니다.
혼합 도형·그림의 공통 문단 원점, 저장 글줄의 그림 어울림 및 빈 선행 줄의 점유를 보존합니다.
한양중고딕 regular는 설치된 원본 outline을 먼저 공급하고 HCR 폴백은 유지합니다.
기존 절대 픽셀 여유 검사 대신 문단·표의 같은 쪽 소속과 앞 빈 줄 점유 뒤의 표 배치를 검사합니다.

## 검증 상태

제출 후보의 전체 nextest·lint·Skia·fresh WASM·전쪽 시각 대조를 준비하고 있습니다.
이전 브랜치에서의 부분 검증을 이 후보의 통과로 대신하지 않습니다.


### 분리 후보의 의존성 점검

첫 전체 검증은 테스트 실행 전에 OLE 호출부의 미완료 API 참조로 컴파일에 실패했습니다.
제외한 OLE 모듈과 같은 기존 API를 호출하도록 해당 호출부도 되돌렸습니다.
다른 도형·캡션·좌표 보정은 유지하고 실제 테스트 실패로 집계하지 않습니다.


### Studio 기존 계약 검사

Studio 전체 단위 검사는 1827개 중 1824 PASS, 1 FAIL, 2 SKIP입니다.
실패는 새 글꼴 환경 setter가 mutation 분류 원장에 빠진 기존 드리프트 검사입니다.
`DocumentCore::set_font_environment`는 문서 글꼴·저장 줄·IR·편집 히스토리를 변경하지 않고
렌더 환경과 파생 상태를 갱신하므로 `EXCLUDED_NON_DOCUMENT`에 근거를 적어 분류했습니다.
기존 검사나 기대값은 완화하지 않고 재실행합니다.


### 분리 범위의 검사·주석 정합

제외한 양자 문서 EMF 확장의 레코드 이름 두 개가 golden 검사 분류에 남아 컴파일을 막았습니다.
해당 검사 파일도 기존 EMF 타입과 같은 분류로 복원했습니다. 기존 golden·fixture 바이트는 바꾸지 않았습니다.
후보에 새로 추가된 영문 설명 주석 일곱 줄을 한국어로 고쳤습니다.
두 번의 준비 단계 컴파일 실패는 실제 회귀 실패나 통과로 집계하지 않습니다.
Studio 빌드는 아직 fresh WASM의 타입 파일이 없어 실패했으므로 fresh 생성 후 다시 검증합니다.

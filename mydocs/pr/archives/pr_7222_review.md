---
kind: investigation
status: active
---

# PR #7222 — #7195 부분 결과 Draft 접수

## 접수와 범위

2026-09-17 작업지시자의 실패 공개 Draft 게시 승인을 받아 생성했다.
작성자 self-review의 접수 기록이며 수용 승인 또는 정식 기술 검토 완료를 뜻하지 않는다.
기본 경로는 collaborator self, 보조는 intake/local validation/visual fixture/review-only 및
대형 PR 경로다. reviewer를 지정하지 않았다. 대형 변경의 코드·시각 심사는 후속 cycle로 남긴다.

| 항목 | 작성 시점 참고값 |
| --- | --- |
| PR | [#7222](https://github.com/edwardkim/rhwp/pull/7222) |
| 작성자 / 관련 이슈 | edwardkim / [#7195](https://github.com/edwardkim/rhwp/issues/7195), Refs만 사용 |
| base | devel, `fcbd00e0fabc4b309a887357033f92e2d511cd75` |
| 게시 head | `369fb09514d72574e9bf7507b4ed9e25408e7f31` |
| 제품·테스트 head | `d54459747d136cf072943d9006659027c5ad274a` |
| 규모 | 26파일, +3535/-415, 9 commits; 이 접수 기록 추가 전 |
| 상태 | OPEN, Draft, MERGEABLE / BLOCKED; merge 직전 재확인 필요 |

변경은 중첩 셀 여백, 분할 줄 소유, 저장 프레임 경계, 빈 문단 뒤 중첩 원점 및 관련 계약이다.
샘플86712는 사용자 교체본이다. HFT 폰트 정밀 대응과 모든 문서의 피델리티 수정은 범위 밖이다.
상세 범위와 실패30건 이름·후속 이슈는 [결과보고서](../../report/task_m100_7195_report.md)를 따른다.

## 완료한 로컬 검사와 제한

- d54459747 이후 게시 head까지 변경은 문서3개뿐임을 확인했다. 제품 결과는
  [stage6](../../working/task_m100_7195_stage6.md), [stage7](../../working/task_m100_7195_stage7.md)의
  동일 제품 검증을 재사용하며 이번 게시 단계에서 테스트를 새로 실행했다고 주장하지 않는다.
- 전체 nextest **9932 PASS /30 FAIL /47 skipped**; 집중 **37 PASS /1 FAIL**.
- Native Skia lib **3929 PASS /3 FAIL /11 ignored**. 실패3건은 위30건의 부분집합이다.
  그림2 PASS /PDF4 PASS. fmt·Clippy3종·workspace build·suite/unit 정책 PASS.
- 교체 HWP1개 명시 보안 검사6 PASS. 정확한 명령·run ID는 결과보고서3절에 보존했다.
- fresh Docker WASM 빌드와 Studio 제공 해시 확인 완료. Native/WASM39–42쪽 비교는
  backend 일치 증거이지 한컴 전체 피델리티 통과 증거가 아니다.
- PR 본문을 API로 재조회해 제출 파일과 본문 trim 일치, 한글 보존, Draft=true,
  base=devel, 게시 head 일치를 확인했다. 원격 CI는 아직 완료 판정하지 않았다.
- CI green candidate 재사용은 확인하지 않았으며 문서-only라는 이유로 fast-pass를 가정하지 않는다.

## 렌더 영향 및 조판 원칙

렌더·분할 변경과 실물 fixture가 있으므로 직접 시각 증적 필수 대상이다.
아래는 Draft 접수의 보류 범위다. 기존 작업 증거를 최종 PR 전체 원칙 충족으로 승격하지 않는다.

| 검토 항목 | 근거와 남은 경계 | 판정 |
| --- | --- | --- |
| 구현 근거와 일반성 | stage3 소비 경로·컷/높이 표, stage6 원점 red/green 증거 존재. 전체 diff의 정식 코드 심사 미완료 | 미검증 |
| 측정·배치 일관성 | 유효 padding·저장 프레임·원점의 제한된 증거 존재. 잔여 분기와 backend 전체 심사 미완료 | 미검증 |
| 분할·이어받기 계약 | 기존 본문 fit/소유/예산 반례와 달리 전체 회귀에는 현재 실패가 남음. 거짓 통과로 간주하지 않음 | 미충족 |
| 줄 소속과 점유 높이 | stage6 빈 문단 공간 소유 계약 red/green. 모든 rowspan·편집 경로는 입증하지 않음 | 미검증 |
| 사례와 증거의 독립성 | 합성 NO_LS/사용자 교체본/기존 PDF의 역할 구분. PR 대표 asset·최종 사람 판정 미완료 | 미검증 |
| 기준값 변경 | baseline/golden TSV 변경 없음. 승인된 #3931 ignore1건은 보고서5절; 전체 계약 변경의 수용 심사 미완료 | 미검증 |
| 주장과 검증 범위 | 정확한 제품 SHA·실패30건·Skia 부분집합·미검증을 공개함. 개선과 전체 통과를 구분 | 충족 |

동작 기반 검증은 stage3/6의 제품 진입점·입력·예산 경계·수정 전 실패 대조를 참고한다.
모든 관련 fallback의 검증 완료로 해석하지 않는다. 새 실행 없이 새 회귀를 추가 검출했다고 쓰지 않는다.

## 입력 커밋 확인

판정: **충족 — 현재 보고서에 열거한 검증 입력6개에 한정**.
[결과보고서2절](../../report/task_m100_7195_report.md#2-입력-및-시각-판정의-경계)의
저장소 경로·출처/역할·SHA-256 목록을 재사용하며 게시 head369fb09514의 blob과 대조했다.
86712 교체 전 PDF는 새 입력의 정답이 아니다. 대표 PNG 안정 경로 보존은 별도 미완료다.

## 최종 판정

**머지 보류**. Draft 공개만 승인되었으며 Ready·merge·#7195 close를 실행하지 않는다.

- 실행으로 재현된 실패: 전체30건과 그 부분집합인 Skia3건. 후속 이슈
  #7009, #7095, #7140, #7204–#7209에 연결했으나 이슈 등록은 실패 면제가 아니다.
  기존28건의 base 대조 전수 분류는 미완료이고, 통합 후 giant49/48쪽 검사2건이 추가됐다.
- 코드 검토상 우려: 모든 분할·fallback·이어받기 경로의 공통 규칙 준수는 정식 review 미완료다.
  새 결함 확정과 구분한다. 문서별 임의 조건·기준값 완화로 이를 덮지 않는다.
- 필수 증거 부족: PR 대표 시각 asset/최종 사람 판정, 최신 head 원격 CI 및 전체 코드 심사가 남았다.
- 해제 조건: 실패별 원인·base 비교 후 수정 또는 독립 근거와 승인된 계약 변경,
  해당 source 재검증, 시각 증적 보존·판정, 최신 PR head CI와 작업지시자 승인.

merge 후 contributor 댓글은 현재 계획하지 않는다. 수용 검토가 완료되고 실제 merge가 승인되는
별도 단계에서 시각 증적 정본·고정 asset URL·게시 승인을 확정한다.

---
kind: snapshot
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-10-02
---

# PR #7471 검토 — 수정(각주): 번호 문단도 문단 모양의 내어쓰기·정렬을 받는다 (#7469)

## 최종 판정

**머지 보류.** 원 PR 최신 head의 CI는 green이지만 체리픽은 완료했고 통합 검증 중입니다. `review/planet6897-green-20261002`에서 최신 devel `e5098bc91be44a49367a7f2895a14fcd4f4c2c7f` 위에 순차 체리픽해 검토합니다.

## 접수·기여자·출처

- 원 PR: https://github.com/edwardkim/rhwp/pull/7471, 작성자 planet6897, base `devel`, 정확한 head `492aef1eefd06240a6a49d52f6b10c4a15acc657`.
- reviewer jangster77을 지정했습니다. 원 contributor 변경과 메인터너 충돌 보정을 구분해 기록합니다.
- 사전 선택: collaborator_external_pr 9.1.1 체리픽 통합 경로; intake/local_validation/visual_fixture_evidence/multi_pr_update_branch/post_merge 적용.
- source CI는 통합 head 검증을 대체하지 않습니다. 모든 renderer·페이지 변경은 직접 Native/fresh WASM 전쪽 TSV와 영향 경계 review/overlay를 검토합니다.

## 체리픽 계획

| source commit | 판정 | 변경 |
| --- | --- | --- |
| `d481852b4968fdc2f1eee9dd0e6f2d05784c9e74` | candidate | 수정(각주): 번호 문단도 문단 모양의 내어쓰기·정렬을 받는다 (#7469) |
| `492aef1eefd06240a6a49d52f6b10c4a15acc657` | candidate | 증적(#7469): 각주 번호 문단 Visual Sweep 대표 쪽 review·overlay |

## 변경 범위와 검토 계획

- 원 PR 변경 295줄 추가·14줄 삭제·18파일입니다.
- 생산 경로: `src/renderer/layout/picture_footnote.rs`.
- 실제 호출 경로의 측정→예약/컷→paint 소비를 추적하고 정상 대조군·원본 저장 정보의 유효성을 확인합니다. 문단·행·개체·각주 소유와 누락/중복을 기존 검사 의미로 판단하며 픽셀 핀을 승인 근거로 사용하지 않습니다.
- 원본·기준 PDF와 검토 commit의 실제 파일/해시 일치를 확인합니다. 통합 출력과 PDF의 전체 쪽수가 다르거나 미달쪽이 있으면 재검토합니다. 원 PR의 부분 개선 주장을 전체 피델리티 완료로 확대하지 않습니다.
- 합성/실물 경계 회귀·전체 nextest threads8·Native Skia3·필수 lint/정책·fresh WASM은 누적 후보에서 순차 수행합니다. 로그는 ignored output에만 저장합니다.

## 실행 결과

고유 source commit 63개 체리픽을 완료했습니다. 출처와 보정은 [적용 원장](../assets/planet6897_green_20261002/applied_commits.json)에 기록했습니다. 현재 후보 `ec5ca7c3057a89c9a82bb59a78956a4d5eee567d`의 Native Clippy는 exit0이며 전체 nextest는 실행 중입니다. 최종 회귀·시각 검증은 미완료입니다.


## 단계1 각주 분할·번호 경로 충돌 분석과 보정

- 최신 devel은 각주 측정과 paint가 FootnoteParagraphPlacement를 공유하고 저장 빈 줄·원래 autoNum 서식을 보존합니다. source의 번호 일반 문단 조합을 이 공유 route 위에 연결하고 spacing이 전달되는 in_frame 호출을 유지합니다. 첫 fragment 번호 한 번과 후속 tail 무번호 계약을 보존합니다.
- 새 일반 문단 경로에서도 저장 autoNum의 원래 접두/접미를 사용합니다. 여러 선두 번호 슬롯은 기존 번호 경로로 보내 원본 슬롯 서식을 잃지 않도록 합니다. 번호의 가시 표현을 읽는 기존 helper만 display_or_text로 교정합니다.
- source에 남아 있던 #7505 이전의 대형 테스트 함수·픽셀 hash/count·겹침 상한218→251을 독립 검증 없이 되살리지 않습니다. 최신 의미 검사와 #7445 보류 표식을 유지하고 baseline 상한을 완화하지 않습니다. 통합 출력의 번호·문단 모양·각주 수량·원본 쪽수와 기존 경계 검증 전에는 승인하지 않습니다.

### 누적 후보 검증 시작

- 검증 코드 head: `ec5ca7c3057a89c9a82bb59a78956a4d5eee567d`. Native Clippy exit0(34.38초). 전체 nextest release-test/threads8/no-fail-fast 실행 중이며 통합 시각 검증은 아직 미완료입니다. 원 PR의 green CI와 구분합니다.

### 메인터너 보정 사전 분석: 각주 검사 표시 문자열

- 최초 통합 전체 결과: 10,250 PASS /33 FAIL /50 SKIP, exit100. 2개 #7379 합성 소유 검사는 표시 번호가 `display_text`에 있는 현재 번호 경로에서 `run.text`의 모델 공백만 읽어 footer 없음으로 판정합니다. 원 PR의 #3738 검사 helper도 같은 이유로 표시 문자열을 사용하도록 바뀌었습니다.
- 독립 계약은 번호의 표시·쪽 소유·단일 출현입니다. 모델 공백과 화면 번호는 다른 표현이며 모델 글자 인덱스를 그대로 유지해야 합니다. 이 helper를 `display_or_text()`로 바꾸고 두 검사의 누락/중복 및 marker/footer 동시 소유 조건을 유지하겠습니다. 허용값·페이지 범위·제품 각주 배치는 바꾸지 않습니다.

### 메인터너 보정 결과: 표시 문자열 검사

- 기존 helper의 원문 공백 읽기를 표시 문자열 읽기로 교정했습니다. 번호·marker/footer 소유·단일 출현 조건과 모든 제품 코드는 그대로 유지했습니다.
- `run-rust-test.mjs issue_7379_rowbreak_table_footnote_reservation`을 nextest release-test/threads8/no-fail-fast로 실행해 기존 14개 전부 PASS, exit0(4.175초)입니다. 최초 2개 footer 누락 실패는 이 검사 표현 오류였습니다. 전체 33FAIL 중 다른 실패와 최종 통합 검증은 아직 해결·완료되지 않았습니다.

### 누적 후보 검증 상태 갱신 (2026-10-02)

- GitHub를 다시 조회한 결과 선택 당시의 원 PR head와 CI green 상태가 유지됩니다. #7435는 현재도 non-green이며 선택에 포함하지 않았습니다. 누적 후보는 `review/planet6897-green-20261002`, source 고유 커밋63개입니다.
- 최초 전체 nextest는 10,283개 중10,250PASS/33FAIL/50SKIP, exit100으로 완료했습니다. 그 뒤 PR별 보정과 focused 재검증으로 최초 실패28개를 처리했으며 5개가 남았습니다. 전체 재실행 통과로 바꾸어 보고하지 않습니다.
- 각주 빈 번호·합성 사다리·다열 표 대조군·중첩 표 후속 원점의 잔존 5개와 whole fixture 시각 보류를 [현재 검증 기록](../assets/planet6897_green_20261002/review_progress.json)에 기록했습니다. 원 PR별 기존 분석·커밋 출처는 위 내용을 유지합니다.
- **현재 통합 승인/머지 보류**입니다. 원 PR의 green CI는 누적 후보의 실패 또는 미완료 Native/fresh WASM 시각 검증을 대체하지 않습니다. 새 통합 PR 생성·push·머지는 하지 않았습니다.

## 메인터너 보정 2 사전 분석: 빈 각주 검사의 잘못된 출력 전제

- 실패 검사는 156쪽에 내용 없는 각주 46·47 및 내용 있는 48이 함께 출력되고 같은 줄 높이를 갖는다고 기대합니다. 원본 IR의 46·47은 본문과 autoNum 자리표시가 모두 없는 각주입니다. 각주 번호 필드와 autoNum 번호는 서로 다른 값일 수 있으므로 구분해 확인했습니다. 번호만 보고 다른 내용 있는 autoNum 46·47을 원본 빈 각주로 오인하지 않습니다.
- 기존 독립 한컴 2024 PDF에는 빈 46·47 각주 줄이 없고 내용 있는 48이 155쪽에 있습니다. 현재 Native도 동일합니다. 따라서 renderer를 바꿔 존재하지 않는 번호 줄을 만들거나 쪽 번호만 155로 재고정하지 않습니다.
- 같은 원본 154–156쪽의 Native/fresh WASM은 각각 99.54705/97.46256/98.01363%입니다. 두 경로는 242쪽이고 기준 PDF도 242쪽입니다. 이 세 쪽의 비교를 전체 242쪽 시각 통과로 보고하지 않습니다.
- 기존 실패 검사 한 개를 원본 빈 각주 데이터의 보존, 그 번호의 footer 줄 미출력, 내용 있는 각주48의 단일 출현으로 교정합니다. 번호 문단 들여쓰기의 기존 3개 검사는 유지하고 새 검사/fixture는 추가하지 않습니다.

### 보정 2 결과

- 기존 검사 4개 모두 통과했습니다. 함수 개수와 원본 문서는 유지했으며, 의미가 잘못된 마지막 검사만 번호 없는 빈 footer의 미출력과 원본 데이터/실제 내용 있는 각주48 보존으로 교정했습니다. 156쪽 핀과 빈/내용 줄의 절대 높이 비교를 제거했습니다.
- [Native TSV](../assets/planet6897_green_20261002/empty_notes_native.tsv), [fresh WASM TSV](../assets/planet6897_green_20261002/empty_notes_wasm.tsv), [155쪽 직접 비교](../assets/planet6897_green_20261002/empty_notes_p155_review.png), [입력 해시·범위·결과](../assets/planet6897_green_20261002/empty_notes_validation.json)를 보존했습니다. 비교 이미지의 본문 번호 참조와 footer48·49도 직접 확인했습니다.
- 이 수정은 기존 전체 실행의 실패 1개를 처리합니다. 최초 실패 집중 처리 수는 32/33개이며, 최종 전체 재실행과 정책 문서 전체 시각 검증은 아직 남았습니다. 원 PR의 머지 보류 판정은 유지합니다.

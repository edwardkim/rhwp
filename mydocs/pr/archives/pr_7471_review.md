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

### 다음 메인터너 보정 분석: 닫힌 저장 프레임의 아래 여백 보존

- 누적 후보 `d2ef8693a`의 원본 정책연구 HWPX는 정상 글꼴 환경 전쪽 Native에서 216쪽, 독립 한컴2024 PDF는 215쪽입니다. 현재 후보 최초 전체 재실행 결과와 구분하여 이 실제 시각 보류를 새로 확인했습니다. 표1904/그림67의 테두리는 이미 맞지만, 그 표가 소유하는 안내 빈 줄1905~1909가 이후 흐름을 다시 전진시켜 문단1913을 불필요한 새 쪽으로 이월합니다.
- 독립 근거: 저장 높이50256HU+바깥 위·아래283HU=50822HU이며 guide 사다리 뒤 문단1910의 vpos가 정확히50822HU로 다시 시작합니다. 원본·PDF 및 이전 #7382 단계10의 독립 관측을 재대조했습니다. 개별 문서 ID로 guide를 숨기지 않습니다.
- 실제 값 연결: `query_closed_source_frame_placement`가 선언·측정 높이와 정상 guide 사다리, 쪽 수용을 확인하여 물리 상단/하단을 준비합니다. fragment budget이 아래 여백을0으로 예약하고, `emit::commit_fragment`가 그0으로 기존 물리 하단을 덮습니다. `section::entry`는 물리 하단이 저장 프레임과 일치하지 않아 guide 소유를 거절합니다. 이전 #7243 보정의 예산/emit 공유는 유지하되, 이미 수용한 닫힌 전체 저장 프레임도 그 공유 결과에 포함해야 합니다.
- 다음 수정은 동일한 닫힌 원본 프레임 조회 결과에서 원점과 아래 여백을 예산·행 scan·emit에 전달합니다. 일반 분할 조각·중간 컷·성장/편집 표는 이 프레임 조회 자체를 통과하지 않습니다. 현재 전체 nextest는 수정 전 후보로 계속 실행 중이므로 끝난 뒤 코드를 빌드하고 182/183쪽, 전체 쪽수, #7243 및 분할 정상 반례부터 검사하겠습니다.

- 구현: 닫힌 원본 프레임 조회를 조판 내부에서 공유하고, fragment budget에서 같은 조회 결과의 원점·물리 하단과 준비된 행 높이의 차를 아래 여백으로 예약합니다. emit은 이 예약값을 그대로 소비합니다. 원점 선택 뒤 legacy 계획이 덮지 않도록 같은 결과를 placement의 첫 후보로 둡니다. helper의 Rust 모듈 접근 범위 컴파일 오류는 수정했으며 결함 검출 증거로 세지 않습니다.
- Native 결과: 전체 출력215쪽/PDF215쪽으로 복원했습니다. 182·183쪽 일치율99.89879%/99.92387%, 새 review PNG를 직접 확인했으며 그림67 뒤 매독·기생충 문단과 다음 그림68의 쪽 소속이 기준과 같습니다. [182쪽](../assets/planet6897_green_20261002/liver_closed_frame_p182_native.png)·[183쪽](../assets/planet6897_green_20261002/liver_closed_frame_p183_native.png)·[증적 범위](../assets/planet6897_green_20261002/liver_closed_frame_evidence.json)를 보존했습니다.
- 집중 검사46개 중44PASS/2FAIL입니다. #7243 2개와 #6950 28개는 모두PASS이며, 기존 실패의 다행 쪽 하단 상한 1개와 그림11 좁은 본문 예산 반례 1개는 그대로 남아 다음 개별 보정 대상으로 유지합니다. 새 실패를 통과로 보고하지 않습니다. fresh WASM 빌드 중이며 전215쪽 최신 시각 재검증·누적 전체 재실행도 남아 있습니다. 코드와 Native 결과를 중간 보정으로 커밋하고 최종 승인 보류를 유지합니다.

- fresh WASM 후속: 코드 `8ee77497f`의 실제 WASM 문서 전체 쪽수215, PDF215입니다. [182·183쪽 WASM TSV](../assets/planet6897_green_20261002/liver_closed_frame_wasm.tsv)는 Native와 같은99.89879%/99.92387%입니다. Mac --no-opt 빌드 exit0이며 Docker 최적화 검증이 아닙니다. 전215쪽의 최신 시각 검증과 기존17개 실패의 처리·최종 전체검증은 여전히 남아 있습니다.

### 메인터너 보정 준비: 원본 프레임이 넘는 경우의 전체 표 허용 금지

- `figure11_closed_frame_preserves_units_across_body_budgets`의 기존 IR 예산 반례를 RHWP_TABLE_DRIFT로 개별 실행했습니다. 좁은 본문240px에서 바깥 위여백3.8px를 뺀 실제 행 예산236.2px가 전체 표 허용으로237.6px까지 늘어납니다. 쪽 하단을1.36px 넘는 직접 원인은 원점 차감 누락이 아니라2px 보조 허용입니다. 한컴 정본에서 추출한 원본 표 높이17819HU/75=237.5867px도 이미 실제 예산보다 큽니다.
- 기존 규칙은 측정 높이의 작은 증가 때문에 온전한 표가 갈라지는 것을 막으려는 목적입니다. 원본 높이도 예산을 넘는 경우는 그 전제가 성립하지 않습니다. 저장 높이가 실제 행 예산에 들어갈 때만 기존2px 보조 허용을 유지하여 측정 드리프트와 실제 초과를 구분합니다. 본문을 늘리거나 테두리를 잘라내지 않고 기존 행 소유·그림/캡션 단일 소유를 지킵니다. 기존 반례의 조건·허용값은 유지합니다.

- 수정 결과: 원본 선언 높이도 실제 행 예산에 들어갈 때만 종전2px 측정 높이 보조 허용을 적용합니다. 기존48개 모두PASS(nextest release-test threads8, exit0). #7379의 좁은 본문 반례, 그림/캡션/뒤 본문 단일 소유, 원본215쪽 검사 및 #6950/#7243/#7095/#1853 기존 검사를 유지하고 통과했습니다. 새 함수·fixture·허용값 변경은 없습니다. [진단·실행 증적](../assets/planet6897_green_20261002/figure11_declared_fit_tolerance_evidence.json).
- 이 수정 뒤 전체 회귀와 새로운 Native/fresh WASM 전쪽 비교는 미수행입니다. 이전215쪽 증적이나 다른 코드의 visual sweep을 이 코드의 최종 검증으로 재사용해 보고하지 않습니다. 전체 승인 보류는 유지합니다.

### 이번 단계 종료 재검증 요약

- head `8927c6ccb`에서 이전 전체 실패17개 이름을 모두 확인했습니다. generated suite 재배정으로16개 실행15PASS/1FAIL과 별도 oracle partition7의1PASS를 합쳐16PASS/1FAIL입니다. 전체10,281개 재실행 결과가 아닙니다. [최종 집중 증적](../assets/planet6897_green_20261002/prior17_final_8927c6ccb.json).
- 잔존FAIL은29쪽 hwpx_sample2의8쪽표 용지밖1건입니다. h01의1쪽53.03132%와 큰문서 미달쪽, 최신215쪽전쪽Native/freshWASM·전체회귀 검증은 추가보류입니다. 이들을 완료로 간주하거나 blanket baseline 변경·새골든등록·원본삭제를 하지 않았습니다. PR 최종승인/머지준비는미완료입니다.

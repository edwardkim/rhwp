# #3587 Stage 4 — B1 문단 블록 반복 사전검사

- 일자: 2026-09-12
- 승인: 메인테이너 「B 구현계획서를 승인합니다」.
- 상태: **B1 자원 사전검사 절편 구현·집중 검증 완료. B1 전체와 B2/B3는 미완료.**
- 제품/테스트 검증 SHA: `b1de53c326` (`task_m100_3587`).
- 근거: [승인된 B 계획](../plans/task_m100_3587_impl_b.md), [A 통합 결과](task_m100_3587_stage3.md).

## 1. 이번 절편

`paragraph_block_budget_native`는 복제 전 주소·자원 비용을 읽기 전용으로 조사한다.
아직 `repeat_paragraph_block_native`를 제공하지 않는다. 비용 검사를 통과했다는 이유로
컨트롤 복제·참조 완결성·저장 호환성이 보증되는 것은 아니다.

- source `[start,end)`와 사전 좌표 목적지 검사. 원형 내부 삽입 거부, 양 끝 경계 허용.
- count=0은 주소·옵션만 검사하고 내용/전체 원문 순회 없이 반환한다.
- 사본·본문 문단·소유 노드·소유 깊이·구조 바이트·대응표·전체 원문 노드 예산 적용.
  호출자는 기본 hard ceiling을 낮출 수만 있다. 계산은 checked arithmetic이다.
- 소유 트리를 sibling 하나씩 읽는 명시적 stack으로 순회한다. 넓은 표의 모든 셀을
  먼저 stack에 복사하지 않는다. 문단·컨트롤·도형·셀·글상자·캡션·master/메모 문단을 포함한다.
- 원문 순회는 구역 자체도 한 노드로 세므로 문단 없는 구역 배열도 제한한다.
  원문 ID/참조를 아직 수집하는 것은 아니다. 해당 수집의 비용을 대체한다고 주장하지 않는다.
- 구조 비용은 serde 값 방문으로 계측하되 JSON이나 복제 IR은 만들지 않는다.
  `source_line_seg_vertical_pos`는 serde에서 빠지므로 모든 중첩 문단에서 별도 합산한다.
  고정 저장폭을 중복 계산하는 보수적 논리 예산이며 allocator/RSS 상한은 아니다.
- 대응표는 노드마다 양쪽 경로의 단계당 16 bytes 및 entry 고정비 64 bytes로 예비 산정한다.
  실제 typed path 결과 타입을 연결할 때 이 저장폭을 다시 검증해야 한다.
- Form의 사용자 정의 serializer는 정렬용 Vec를 할당하므로 원형 비용 방문 전에 거부한다.
  이는 B 초기 Form 미지원 방침에 맞춘 방어이며, 다른 종류의 비용 계측 성공은 지원 승인이 아니다.

## 2. 검증 계약

`tests/cases/issue_3587_paragraph_block_budget.rs`의 10건:

1. 경계 좌표와 기존 빈 문단 보존.
2. 잘못된 주소는 count=0도 거부.
3. count=0에서 unsupported 내용/원문 순회 생략, 무변경.
4. 모든 한도의 0/상향 지정 거부.
5. 사본/문단 제한 및 곱셈 overflow 거부.
6. 구조/노드/대응표/원문 예산의 정확한 경계와 한 단계 초과.
7. UTF-8/raw/serde 제외 버퍼 비용 포함.
8. 중첩 글상자 문단·깊이·제외 버퍼 포함.
9. 1/10/100회 비용 선형성과 요청당 원문 1회 순회 비용.
10. 실제 표/clipboard가 있는 core에서 성공·실패 모두 문서/이벤트/clipboard 보존.

검증은 source를 커밋한 후 `rhwp-review-3587`의 동일 SHA에서 수행했다.
생성 suite는 해당 worktree에서만 준비하고 `target/pr-review`를 재사용했다.
이번 예산 수치의 선형성 검사는 사본 생성 시간/RSS 계측이 아니다.

## 3. 실행 결과

로그: Git 제외 `output/3587/b1/`. 아래 검사는 모두 `b1de53c326`의 동일 제품/테스트에서 수행했다.

| 검사 | 결과 |
| --- | --- |
| manifest `--prepare` | PASS, review worktree에서만 실행 |
| `cargo fmt --all` 및 `--check` | PASS, tracked source 변경 없음 |
| `run-rust-test.mjs issue_3587_paragraph_block_budget` | **10 PASS / 0 FAIL**, 필터 제외 194건 |
| A 4개 source의 `issue_3587_` focused nextest | **25 PASS / 0 FAIL**, 필터 제외 539건 |
| `cargo clippy --locked --target-dir …/target/pr-review -- -D warnings` | PASS, 46.76초 |
| manifest `--check` | PASS |
| B 계획·본 기록 파일 단위 Markdown 링크 검사 | PASS |

B 테스트 컴파일 2분 10초, 실행 0.011초. A 테스트 컴파일 9.83초, 실행 0.229초.
이는 해당 환경의 집중 테스트 시간이지 자동화 복제 API 성능 수치가 아니다.
두 실행의 nextest 버전 0.9.137 권장 버전 경고 및 `report-skipped` 설정 키 경고는 A 통합
검증 때와 동일하다. 도구 설정을 이번 변경으로 수정하지 않았다.

이번 절편은 제품/테스트 실패 없이 통과했다. 전체 nextest·WASM/workspace Clippy·Docker WASM·
한컴 시각 검증은 이번 B 코드에서 아직 실행하지 않았으며, 이전 A의 전체 통과로 갈음하지 않는다.
원격 push 또는 PR 전에는 규정된 전체 lint 묶음을 수행해야 한다.

## 4. 남은 승인 범위

- B1: 종류/안전 raw 슬롯 지원표, 최초 문제의 typed path, 필드/연결선 양방향 경계 검사,
  중복 참조 검사, 공유 자원 존재 검사. 현재 budget 함수는 이것들의 대체물이 아니다.
- B1 비용 검증: 실물 두 블록의 실제 사본 생성 1/10/100회 시간·메모리와 추정치 대조.
- B2: 공유 allocator·사본별 map·단일 삽입·원본 Box/표 reflow 출처 보존·오류 무변경.
- B3: HWP/HWPX 저장·native 및 WASM/한컴 사용 검증.
- 기존 clipboard/조판 코드는 수정하지 않았다. 원격 push/PR/게시도 하지 않는다.

## 5. 후속 절편 — 지원표와 참조 경계 (구현 전 고정)

메인테이너의 다음 절차 승인에 따라 strict 사전검사를 추가한다. 삽입 구현은 아직 별도다.
검사는 문단/컨트롤/셀/글상자/캡션/그룹 자식의 typed path를 오류에 제공한다.

| 대상 | 첫 지원 범위 / 거부 조건 | 코드 근거 |
| --- | --- | --- |
| 일반 문단 | 빈 문단·Page/Column break 유지. Section/MultiColumn 및 raw break bit 0/1 거부 | `parser/body_text.rs::parse_para_header` |
| 문단 raw 헤더 | 없음 또는 10 bytes(ID 슬롯 6..10), 12 bytes 중 변경추적 값 0 허용. 기타 확장/변경추적 거부 | `serializer/body_text.rs`의 instanceId/변경추적 기록 |
| 표·수식 common raw | 없음 또는 36/40 bytes, 이후 길이가 맞는 UTF-16 설명문까지 허용. common 미해석 tail 및 표 레코드 미해석 tail 거부 | `parser/control/shape.rs::parse_common_obj_attr`, `clone_identity/remap.rs` |
| 기본 도형/그룹/글상자 | 모델링된 소유 구조 순회. 미해석 connector/polygon tail, textbox LIST_HEADER tail은 초기 거부 | `model/shape.rs`, `identity/walk.rs` |
| 그림 | payload 없음 또는 알려진 5/17/18 bytes, own ID 1..5 및 크기/alpha 보존. 기타 확장 거부 | `parser/control/shape.rs` 그림 extra 파싱 |
| ClickHere | begin ID와 종료 마커의 소유/순서/범위 검사. shared fieldid는 unique ID와 구별. 이름은 변경하지 않음 | `model/paragraph.rs::FieldRange/OrphanFieldEnd`, `clone_identity/remap.rs` |
| ClickHere 확장 | raw parameter XML/비어 있지 않은 parameters 및 CTRL_DATA payload는 아직 의미 검증 전이므로 거부 | `model/control.rs::Field`의 HWP/HWPX 보존 경계 |
| 그 밖의 컨트롤 | Section/Column/Header/Footer/쪽 설정·Form·각주/메모·Unknown·OLE/Chart·기타 필드/책갈피 등은 명시적 미지원 | B 계획 §4 |

읽기 전용 사전검사는 포맷 유효성 전체를 인증하지 않는다. 지원 경계 밖의 실제 샘플은 오류 경로와
종류를 기록하고, 통과시키기 위해 payload를 지우지 않는다. 이 절편에서는 참조 폐쇄성까지 검증하며,
공유 스타일/BinData의 존재 및 전체 저장 검증은 후속으로 남긴다.

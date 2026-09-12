# #3587 Stage 2 — 기본 구현 재개, A1 표 생성 신원

- 날짜: 2026-09-12
- 상태: A1 구현·집중 검증 완료. A 전체 및 #3587 완료가 아님.
- 승인: 기여자 템플릿이 없어도 작업 브랜치로 돌아가 기본 구현 진행.
- 계획: [구현계획 A](../plans/task_m100_3587_impl.md)

## 1. 범위

`task_m100_3587`의 기존 조사·샘플을 유지하고 응답 대기를 해제했다.
이번 절편은 새 표 생성 두 API의 ID를 문서 내 사용 ID와 충돌하지 않게 할당하고,
HWP용 raw와 IR common에 동일하게 기록하는 기반이다. 크기·여백·문단 삽입·조판 규칙은 변경하지 않는다.

복제본 ID/연결선/필드 참조의 두 단계 재매핑, clipboard/split 적용은 A의 다음 작업이다.
문단 블록 반복·데이터 바인딩·문서 간 원형 이식은 이후 B/C/D에 남는다.

## 2. 구현

- `clone_identity.rs`: 읽기 전용 소유 트리 순회, 사용 중인 ID 집합 수집, 미사용 양수 ID 할당.
- 표 셀·글상자·그룹·그림/도형/표 캡션·머리말/꼬리말·각주/미주·바탕쪽·메모 문단을 방문한다.
- drawing/field 신원은 충돌 회피 집합에 보수적으로 포함하되 서로 다른 참조 체계를 합치거나 변경하지 않는다.
- 표의 common/raw가 이미 다르면 양쪽 값을 모두 예약한다. 원본 중복·불일치는 이번 연산에서 재채번하지 않는다.
- 명시적 작업 스택으로 중첩 트리를 순회하며 난수·시각·크기 해시를 사용하지 않는다.
- 일반 표와 TAC 표 생성에서 입력 주소/행열 검증 뒤, 실제 문서 변경 전에 할당한다.
- 형식을 알 수 없는 opaque raw의 참조까지 지원한다는 의미는 아니다. strict 복제 지원과 별개다.

현재 단일 생성은 매번 문서의 신원을 수집한다. B의 N회 블록 생성에서는 요청 snapshot에서
한 번 수집한 allocator를 공유하도록 설계해야 하며, 현재 구현을 대량 생성 성능 검증으로 보지 않는다.

## 3. 검증 증거

별도 `/home/edward/mygithub/rhwp-review-3587`에서 동일 commit을 검증한다.
공유 Cargo target은 `/home/edward/mygithub/rhwp/target/pr-review`다.
생성 suite/manifest는 review worktree 검증용이며 source commit에는 포함하지 않는다.

- RED: 제품 수정 전 `24948d337b`, 신규 계약 5개 중 1 통과·4 실패.
  같은 크기 표의 raw ID 중복, common ID 0, 중첩 트리 예약 누락을 검출했다.
- 제품 구현: `99457dbeb4`.
- GREEN: 동일 테스트 **5/5 PASS**. 양 입력 포맷에서 추가 생성 후 HWP/HWPX로 각각
  저장·재열기해도 ID가 유지되고 중복되지 않는다.
- 기존 `set_table_props_contract` **4/4 PASS**, `issue_6557_merged_cell_boundary_drag_marking` **2/2 PASS**.
- 준비된 review worktree의 전체 fmt 및 generated manifest 검사 PASS.
- 전체 회귀와 PR 전 3종 Clippy 묶음은 미실행이며 PR 제출 가능 판정은 하지 않는다.
- 검증 명령: `node scripts/run-rust-test.mjs issue_3587_clone_identity -- --cargo-profile release-test --target-dir /home/edward/mygithub/rhwp/target/pr-review`.
- 로컬 로그: `output/3587/a1/`의 `red.log`, `green.log`, `fmt.log`, `prepare-*.log`.

테스트는 반복 생성·양 형식 저장/재열기·재열기 후 추가 생성·중첩 신원 예약·원본 보존·결정성·
잘못된 주소 거부를 검사한다. API 생성 입력과 인위적 IR은 계약 반례이며 한컴 정답지가 아니다.

### 검증 준비 중 정정

- 처음에는 `Document` 자체를 JSON 직렬화하는 테스트 코드가 컴파일되지 않았다.
  실제 지원되는 HWP export의 전후 바이트 비교로 정정했다. 이 컴파일 오류는 제품 RED로 세지 않았다.
- source checkout의 전체 fmt는 이전 generated suite의 없는 파일 참조 때문에 실패했다.
  source에서 generated 파일을 바꾸지 않고, 준비된 review worktree에서 재검사했다.
- 테스트 원본 변경 후 suite 할당이 달라져 필터가 0개를 실행한 호출은 통과로 인정하지 않았다.
  `--prepare`를 다시 실행하고 5개가 실제 실행된 위 RED를 기록했다.
- nextest 0.9.137과 권장 0.9.140의 차이 경고가 있다. 이번 작업에서 도구 설치는 변경하지 않았다.

## 4. 운영 경계

원격 push·PR 생성·전체 회귀·Docker WASM·한컴 시각 검증은 아직 수행하지 않았다.
Studio 7700 서버와 배포된 WASM은 변경하지 않는다. 기여자 실제 템플릿 확인은 계속 미완료다.

다음은 같은 A 안에서 clipboard의 staged 복제에 신원·내부 참조 대응을 연결하는 작업이다.
이번 할당 helper만 호출하고 복합 컨트롤 복제를 완료했다고 처리하지 않는다.

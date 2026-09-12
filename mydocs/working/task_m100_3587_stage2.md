# #3587 Stage 2 — 기본 구현 재개, A1 표 생성 신원

- 날짜: 2026-09-12
- 상태: A1·A2 구현 및 집중 검증 완료. A의 지원 경계·split 통합과 #3587 전체는 미완료.
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

이상은 A1 완료 시점의 기록이다. 이어지는 A2는 clipboard의 staged 복제에 신원·내부 참조 대응을 연결한다.
이번 할당 helper만 호출하고 복합 컨트롤 복제를 완료했다고 처리하지 않는다.

## 5. A2 — 컨트롤 복제 신원과 내부 참조

메인테이너의 후속 진행 승인으로 `copy_control_native` → `paste_control_native` 경로를 보정한다.
`paste_internal_native`의 일반 텍스트 선택 붙여넣기나 별도 표 transpose clipboard는 이번 절편의
통합 대상이 아니다. 새 블록 반복 API를 추가한 것으로도 해석하지 않는다.

### 처리 순서와 보존 항목

1. 목적 구역/문단을 검증한다. 잘못된 주소로 실패할 때 cascade(반복 붙여넣기 위치 이동 횟수)를 소비하지 않는다.
2. 문서와 clipboard 원형에 이미 쓰인 ID 및 알려진 참조를 수집한다. 원형을 잘라내 문서에서 제거했어도
   clipboard의 ID는 새 할당에서 제외한다.
3. 문서와 분리된 복제 트리의 공통 개체·drawing·그림 payload·필드 시작·각주/미주 ID를 할당한다.
   복제 문단의 완전한 HWP 헤더에 있던 instance ID도 새 값으로 바꾼다.
4. 모든 목적 ID를 할당한 뒤 연결선 SubjectID와 다문단 필드의 beginIDRef를 재연결한다.
   연결선은 기존 엔진이 받는 drawing ID, common ID, common 유래 별칭을 고려한다.
   생성 별칭이 기존 참조를 가로채지 않도록 별칭도 예약한다.
5. 위 처리가 성공한 뒤 기존 삽입·cascade·문단/커서·빈 Enter·재조판 경로를 수행한다.

- 필드의 `field_id`는 beginIDRef가 참조하는 고유 신원이다. `instance_id`와 종료 마커의 `fieldid`는
  별도 공유 메타데이터이므로 무조건 재채번하지 않는다. 같은 문단 범위의 control_idx도 유지한다.
- 스타일 번호·BinData 자원·필드 이름·텍스트·내부 문단 순서는 변경하지 않는다.
- 복제 트리 밖 참조는 원본 문서의 기존 대상을 계속 가리킨다. 외부 참조를 자동 복제하거나 삭제하지 않는다.
- 대상이 중복 ID 때문에 모호하면 staged 단계에서 오류로 중단한다. 종전에는 그대로 복사됐던
  **모호한 참조/불완전한 identity payload**를 이제 거부하는 호환 변경이며, 정상 복사의 문단 정책 변경과 구분한다.
- 표·수식 common/raw는 ID를 함께 바꾼다. 원래 유효했던 봉인만 갱신하고 stale 봉인은 재승인하지 않는다.
- 그림 raw 추가 데이터의 ID 슬롯만 갱신한다. 나머지 효과 바이트를 일괄 폐기하지 않는다.

### 검증과 한계

- 제품 변경 전 `efdfbb5bf5`: 계약 6개 중 1 PASS / 5 FAIL. 기존 개별 표 편집·삭제 독립성은
  이미 통과했으며 이번에 새로 해결한 결함으로 세지 않는다.
- 최초 테스트의 생성 표 위치 가정은 API가 반환한 `paraIdx`/`controlIdx`를 모두 사용하도록 정정했다.
  그때의 잘못된 주소 실패는 제품 RED로 세지 않는다.
- 저장소 실물 `hwp_table_test.hwp`, `table-in-tbox.hwp`와 rhwp 변환 HWPX를 입력으로 사용한다.
  변환 HWPX를 독립 한컴 원본이라고 부르지 않는다. 추가 그룹·필드 반례는 내부 계약 입력이다.
- `output/3587/a2/`에 준비·RED/GREEN·fmt 로그를 보존한다.
- 최종 검증 head: `763c156fb670edaf06d9b649bc5784346f6e1f6f` (제품 source 최종 변경 `6088924f9b`).
  `issue_3587_control_clone_references` **8/8 PASS**, A1 **5/5 PASS**, 기존 표 속성 **4/4 PASS**,
  셀 경계 편집 **2/2 PASS** — 합계 **19 PASS / 0 FAIL**.
- 준비된 review worktree에서 `cargo fmt --all -- --check`와 suite manifest `--check` PASS.
  Rust source의 cfg(test)와 CI/baseline은 변경하지 않았다. 전체 회귀·3종 Clippy·WASM은 미실행이다.
- 실물 표/글상자는 두 입력 포맷과 두 출력 포맷에서 복제본의 유지된 중첩 ID가 원본/다른 복제본과
  분리되는 것을 확인했다. 포맷에서 존재하지 않아 0으로 읽히는 보조 슬롯은 보존 ID라고 주장하지 않는다.
- 연결선 앞방향 참조·common 별칭 참조·외부 참조와 raw 그림 ID는 저장·재열기까지 검사했다.
  다문단 필드 참조 반례는 IR 관계 검사이며 별도 한컴 시각/저장 호환 판정으로 확장하지 않는다.
- 임의 opaque 확장 데이터나 필드 parameters 안의 모든 참조를 해석한 것은 아니다.
  Form과 ClickHere가 혼합될 때 HWP serializer의 별도 ID 생성 경로도 남아 있어, 이번 결과를
  모든 컨트롤의 strict 복제 완료로 보지 않는다. 해당 경계는 A의 후속 확인 대상이다.
- `split_table_native` 공통 할당 통합, 일반 선택 붙여넣기와의 일관성, 전체 회귀/PR 게이트는 남아 있다.

다음 절편은 A의 남은 split 할당 및 복제 지원 경계(선택 붙여넣기·Form/ClickHere)를 점검하는 순서다.
그 결과로 A 완료 여부를 판정한 뒤 B의 경계 보존 블록 설계를 확정한다. 원격 게시·push·PR·이슈 종료는 하지 않았다.

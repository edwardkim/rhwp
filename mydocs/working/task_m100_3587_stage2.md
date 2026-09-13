# #3587 Stage 2 — 기본 구현 재개, A1 표 생성 신원

- 날짜: 2026-09-12
- 상태: A1~A4 구현·집중 검증 완료. 후속 [Stage 3](task_m100_3587_stage3.md)에서 통합 Rust 검증
  9,523 PASS / 0 FAIL. B 상세 계획 승인 대기이며 #3587 전체는 미완료.
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

## 6. A3 — 표 분할과 선택 붙여넣기 통합

### 구현 결과

- 제품 변경 commit: `3ffed9f7e8`. 표 분할의 원본 ID/행 번호 해시를 없애고
  A1의 문서 전체 미사용 ID 할당을 사용한다. 뒤 표의 common/raw만 같은 새 ID를 받는다.
  앞 표 ID, 이동하는 셀·내부 문단·컨트롤 ID, zone 분할, 캡션 소유권, 사이 빈 문단은 유지한다.
- 본문·셀·cellPath 선택 붙여넣기 모두 A2의 staged 두 단계 신원/참조 재매핑을 사용한다.
  필드에만 적용되던 별도 max+1 재채번과 중복 순회 helper를 제거했다.
  원본/clipboard를 수정하지 않고 생성되는 복제본에만 적용한다.
- 셀 붙여넣기의 `raw_stream` 무효화는 목적지 검사·삽입 성공 뒤로 옮겼다.
  잘못된 셀 주소 때문에 실패하는 요청이 원본 raw를 지우지 않도록 한다.
- 커서 위치, 문단 split/merge, 빈 Enter, cascade, reflow 알고리즘은 변경하지 않았다.
  선택 붙여넣기는 여전히 커서 편집 API이며 B의 완전한 문단 블록 삽입 API를 구현한 것이 아니다.
- 전치 표 clipboard, HTML import, foreign paste는 이 통합 대상이 아니다.
  path API 검사는 이번 신규 계약에서 깊이 1을 사용하며 임의 깊이 전체 검증으로 확대 해석하지 않는다.

### 검증 기록

- 테스트 원본: `tests/cases/issue_3587_split_and_selection_identity.rs`.
- 제품 변경 전 최종 RED `83e7083717`: **6개 중 2 PASS / 4 FAIL**.
  표 분할 해시가 이미 사용 중인 ID와 충돌하고, 선택 붙여넣기 3경로가 표 ID를 복사하는 것을 검출했다.
  이동 문단 보존·잘못된 분할 거절은 기존에도 통과했다.
- 최초 본문 선택 반례는 fixture를 직접 구성한 뒤 composed cache를 준비하지 않아 실패했다.
  HWP 저장·재열기로 fixture를 초기화하고 실제 문단 split API를 사용하도록 정정했다.
  split API의 선택 metadata 인자 누락도 테스트 코드에서 정정했다. 이 두 준비 오류는 제품 RED에 세지 않는다.
- 제품 변경 후 `3ffed9f7e8`: 위 계약 **6 PASS**, A2 **8 PASS**, A1 **5 PASS**,
  기존 ClickHere 편집/선택 붙여넣기 **13 PASS**, CLI 표 분할 **4 PASS**,
  #852 Form 저장 **5 PASS**, 표 속성 **4 PASS**, 셀 경계 편집 **2 PASS**.
  합계 **47 PASS / 0 FAIL**. 각 로그는 `output/3587/a3/<검사명>.log`.
- 추가 테스트 commit `c82d6f73a0`: 선택 복제본의 HWP/HWPX 저장·재열기 ID 보존,
  잘못된 목적지의 raw·문서·clipboard·event 불변을 보강했다. **7/7 PASS**.
  기존 #2299 편집 vpos/리셋 보호 **8/8 PASS**. 제품 source는 `3ffed9f7e8`과 같다.
  신규 6개 실행을 최종 7개로 대체해 중복 제외 시 이번 절편 집중 검증 **56 PASS / 0 FAIL**이다.
  준비된 review worktree의 전체 fmt 및 manifest `--check`도 PASS.
  최종 로그: `identity-final.log`, `issue_2299_edit_vpos_reset_preserve.log`,
  `fmt-final.log`, `manifest-final.log` (모두 `output/3587/a3/`).
- 새 suite/manifest는 기존 `rhwp-review-3587`에서만 준비했다. source PR에 파생물을 추가하지 않는다.
  전체 회귀·PR 전 3종 Clippy·Docker WASM·한컴 시각 검증은 이번 절편에서 수행하지 않는다.

### Form/ClickHere 원인 계보와 다음 경계

정적 코드/이력 조사 결과, 복제본 IR의 ID를 분리해도 HWP 저장기에서 ID를 덮어쓰는 별도 문제가 있다.
`serializer/control.rs`의 ClickHere 분기는 앞선 Form이 하나라도 있으면 동일 seed+Form 수로
필드 ID를 생성한다. 필드 출력은 Form 수를 증가시키지 않으므로 여러 ClickHere가 같은 값을 받을 수 있다.
Form 자체도 `common.attr == 0`일 때 seed+order가 명시적 common ID보다 우선한다.

도입 commit은 `a6ff3d6c16` (#852, 2026-05-20)이다. 당시 form-01/02의 한컴 변환 정답지 및
스크립트 호환을 확보하려고 적용했고, 해당 한컴 검증 기록이 남아 있다.
이번 #852 기존 검사 5 PASS는 그 계약이 유지된다는 뜻이지, 복제 후 ID가 유지된다는 증거는 아니다.

이번에 serializer를 임의 변경하지 않았다. [구현계획 A4 제안](../plans/task_m100_3587_impl.md)의
기존 정답지 보존 + 명시적 신원 우선 + 미지정 신원 충돌 방지 검증을 다음 승인 대상으로 한다.
Form 이름·스크립트까지 자동 재작성하는 확대 구현은 제안하지 않는다.
따라서 A 전체 완료와 B 진입은 아직 선언하지 않는다. 원격 push/게시/PR도 수행하지 않았다.

## 7. A4 — Form/ClickHere 저장 신원 보존

메인테이너의 A4 승인으로 진행했다. 기존 한컴 호환을 버리는 전면 serializer 개편이 아니라,
명시적 신원의 보존과 미지정 Form 신원의 할당을 분리하는 변경이다.

### 실제 재현과 구현

- 제품 수정 전 `f37a08f03c`: 새 계약 **4개 중 1 PASS / 3 FAIL**.
  실제 form-01의 ClickHere를 선택 복사·두 번 붙여넣기한 IR ID는 `[2110609883, 3, 2]`인데
  HWP 저장·재열기 후 `[2110609883, 2110609883, 2110609883]`으로 바뀌었다.
  명시적 Form ID 덮어쓰기와 뒤 구역에 있는 ID와의 충돌도 별도로 검출했다.
  원본 HWP Form의 ID 0 보존은 수정 전부터 통과했다.
- `serializer/control.rs`: ClickHere field_id와 Form common.instance_id를 레코드에 그대로 쓴다.
  레코드 출력 단계에서 Form 등장 순서로 필드 ID를 생성하거나 개체 ID를 덮어쓰지 않는다.
- `serializer/form_identity.rs`: 문서 저장 진입에서 필요한 구역만 복제한다.
  **HWP 헤더 출처 판별 attr가 0이고 instance_id도 0인 Form만** 새 신원을 받는다.
  attr가 있는 원본 HWP Form의 ID 0과 명시적 비-0 ID는 보존한다.
  유효한 원본 raw 재사용 경로도 그대로 둔다.
- 미지정 신원에는 종전 seed+order를 우선 후보로 사용하되 문서 전체 사용 ID와 충돌하면
  공통 allocator의 미사용 양수 ID를 받는다. 우선 후보는 호환 정책일 뿐 유일성 증명이 아니다.
  이후 구역·raw 재사용 구역의 신원도 예약하고, 출력 문서 하나에서 같은 allocator를 공유한다.
- 공통 소유 트리 순회·예약·할당을 `model/identity.rs`와 `model/identity/walk.rs`로 옮겼다.
  편집과 저장은 동일 규칙을 소비하며 serializer가 document_core 명령에 의존하지 않는다.
  clipboard의 참조 재매핑은 기존 commands 계층에 남는다.
- 원본 Document와 clipboard는 수정하지 않는다. ID 후보를 못 구하면 저장 오류를 반환한다.
  section 합성 사본은 산출 전용이며 원본 전체 재채번/저장은 수행하지 않는다.

### 검증 및 지원 경계

- 테스트 원본: `tests/cases/issue_3587_form_save_identity.rs`.
- 실물 form-01/02 HWP와 HWPX 각각을 입력으로 선택 복사·반복 붙여넣기를 수행하고,
  HWP/HWPX 저장·재열기 후 ClickHere의 신원과 채워진 값을 검사한다.
- Form 명시적 ID 보존, 다른 구역과의 충돌 회피, 원본 HWP ID 0 유지,
  무편집 저장의 이름·크기·caption·text 및 종전 비충돌 ID 보존을 별개 계약으로 검사한다.
  인위적으로 ID를 바꾼 입력과 추가 구역은 신원 계약 반례이며 한컴 시각 정답지가 아니다.
- 첫 제품 commit `b594111f6e`: 신규 **4 PASS**, #852 **5 PASS**, #6266 **2 PASS**,
  #258 **13 PASS**, A1 **5 PASS**, A2 **8 PASS**, A3 **7 PASS** — **44 PASS / 0 FAIL**.
- 최종 제품 source commit은 `15e13c7960`이다. 원본 raw 재사용 경로와 하위 레코드 writer의
  신원 투명성을 보강했으며, 무편집 계약을 추가해 같은 검사를 다시 실행했다.
  증적 위치는 `output/3587/a4/`이며 이 보강 직후 로그는 `final-<검사명>.log`다.
- 무편집 속성 비교를 보강하던 중 HWPX ComboBox의 빈 selectedValue와 HWP의 첫 목록 항목
  표시값(`계절 선택`) 차이를 검출했다. 수정 전 `f37a08f03c`와 현재 `build_type_set`의
  기존 fallback 코드를 대조해 이 경로를 변경하지 않았음을 확인했다.
  포맷이 다른 IR 슬롯의 동일성 가정 대신 기존 form-01/02 HWP 정답지를 기대값으로 사용한다.
  정정 테스트 commit `3fc090bc76`, 재검증 로그 접두사는 `verified-`다.
  ID 일치·중복 검사나 기존 정답지/baseline은 완화하지 않았다.
- **최종 검증 head `3fc090bc76`**: 신규 **5 PASS**, #852 **5 PASS**, #6266 **2 PASS**,
  #258 **13 PASS**, A1 **5 PASS**, A2 **8 PASS**, A3 **7 PASS** — **45 PASS / 0 FAIL**.
  준비된 review worktree의 `cargo fmt --all -- --check`와 manifest `--check` PASS.
  `fmt-verified.log`·`manifest-verified.log`에 보존했다. source-side cfg(test)는 변경하지 않았다.
- Form의 name·groupName·TabOrder·스크립트 이벤트/이름은 자동 재작성하지 않는다.
  같은 이름의 Form 복제로 스크립트 업무 동작이 독립해진다고 주장하지 않는다.
  HWPX Form에서 제공하지 않는 신원 슬롯까지 보존했다고 해석하지 않는다.
- 기존 #852 검사는 Scripts/DefaultJScript 정답지 바이트도 대조한다.
  이를 새 복제 문서의 한컴 직접 시각/스크립트 실행 판정으로 확대하지 않는다.
- 원본 raw 재사용은 추가 신원 순회를 생략한다. 재생성 구역에는 소유 트리 판별 순회가
  추가되고 미지정 Form이 있을 때만 구역 사본/문서 전체 예약 집합을 만든다.
  대형 문서 성능 영향은 아직 계측하지 않았으며 비용 0으로 주장하지 않는다.
- 전체 회귀·3종 Clippy·Docker WASM·신규 한컴 시각 검증·원격 push/PR은 미실행이다.
  B/C/D는 시작하지 않았다. 다음 절차는 A 묶음 통합 검증과 B의 문단 블록 경계 보존
  상세 설계 확정이다. A4 집중 검사 성공만으로 #3587 완료나 PR 제출 가능 판정을 하지 않는다.

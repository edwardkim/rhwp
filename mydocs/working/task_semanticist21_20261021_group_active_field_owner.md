# 묶기·풀기 뒤 활성 누름틀의 본문·셀 소유 주소

## 원 PR과 현재 검증 범위

전체 45개 PR의 누적 검토 중 #7588, #8041, #8042를 함께 분석했다. 원 head와 실제
기능 커밋, 로컬 체리픽은 각 PR의 archive 검토 기록에 구분한다. #7588은 원래 엔진
수정이 필요한 재현 전용 Draft이며, 원 작성자가 통합을 승인한 PR로 해석하지 않는다.

현재 제품 `6fc2b3490d`의 Native079와 최적화 fresh WASM031에 원 정식 helper를
연결해 HWP/HWPX 14쌍을 다시 생성했다. HWPX는 기존 Print 입력과 바이트가 같고,
HWP는 별도 MCP engine 2020 Print로 기준을 확보했다. 28조건의 양쪽 raw는 모두
100%, 쪽수는 모두 1=1이다. 이는 필드 활성 주소가 올바르다는 증거는 아니다.
입력·PDF·TSV·대표 review/overlay와 세부 명령은
[그룹 저장본 증거](../pr/assets/semanticist21_group_geometry_20261021/README.md)에 있다.

## 구현 주장과 반례

#8042 원문은 컨트롤 제거·삽입 뒤 `field_ranges`와 `active_field`를 갱신한다고
설명한다. 원 정식 검사 4개는 필드 이름·안내문·범위·저장 보존을 검사하지만 활성
필드 설정을 실행하지 않는다. 별도 공개 API 진단 1178에서는 본문 필드를 활성화한
뒤 묶기·풀기를 실행해 HWP/HWPX 4조건 모두 같은 활성 필드를 유지함을 확인했다.

셀 필드는 다른 주소 구조를 갖는다. `ActiveFieldInfo.para_idx`는 본문 부모 문단,
`control_idx`는 마지막 셀 문단의 필드 번호이며 `cell_path[0].0`이 본문의 표/글상자
번호다. `set_active_field_in_cell`과 `set_active_field_by_path`가 이 주소를 만든다.

공개 API 진단 1179는 같은 본문 문단에서 도형 둘 뒤에 1×1 표를 만들고 표 셀의
필드를 활성화했다. 정상 대조로 같은 필드의 재활성화는 false임을 먼저 확인했다.
묶기 뒤 표 번호는 4→3, 풀기 뒤 3→4가 됐지만 같은 필드의 재활성화가 모두 true였다.
즉 활성 주소가 옛 표 번호에 남아 있었다. 빌드·환경 실패가 아닌 실제 API 실행 결과다.

## 생산·소비 경로와 보정 계획

- 주소 생산: `queries/field_query.rs`의 두 셀 활성 setter → `ActiveFieldInfo`.
- 번호 변경: `group_shapes_native`의 역순 제거와 그룹 삽입,
  `ungroup_shape_native`의 자식 개수만큼 확장.
- 현 갱신: `object_ops/common.rs`의 삽입 helper와 `shape.rs`의 제거·확장 분기는
  `cell_path.is_none()`인 본문 주소만 갱신한다.
- 소비: 활성 필드 비교와 셀 경로 해석은 실제 표 번호를 사용한다. 필드 자신의 번호를
  움직이는 것과 그 필드를 소유한 본문 표 번호를 움직이는 것은 다르다.

본문 컨트롤 번호의 공통 remap에서 본문 필드는 `control_idx`, 셀 필드는 전체 경로의
첫 컨트롤 번호만 갱신하도록 한다. 다른 구역·본문 문단의 활성 필드는 유지한다.
셀 내부 필드 번호·중첩 경로의 나머지 요소는 움직이지 않는다. 제거한 소유자 자체를
계속 가리키는 주소는 보존하지 않는다. 삽입·제거·묶기·풀기는 같은 주소 변환을 소비한다.

이 문단은 구현 전 계획이다. 전체 Rust 1180 실행 중에는 Rust 소스를 고정했다.
실행 종료 후 공통 로직을 보정하고 본문·셀·중첩 셀·다른 문단 대조를 실제 검증한다.
새 정식 회귀는 관련 입력의 독립 Print·양쪽 전쪽 90% 이상·직접 판독을 확보한 뒤
새 이름으로 추가한다. 기존 검사나 시각 기준을 완화하지 않는다.

## 실행 원문

ignored `output/pr-review/semanticist21-20261019/`에 1178/1179 진단 소스·명령·로그,
원 PR 재조회 1159, 현재 API 입력·Print 1163/1166, 전쪽 1165와 원 검사 1152/1160을
보존했다. 로그·중간 JSON은 커밋하지 않는다. 현재 셀 활성 소유 결함과 전체 통합의
필수 검증이 남아 있어 PR 제출·승인·통합 완료로 기록하지 않는다.

## 공통 remap 구현과 정식 회귀

`common.rs`의 `remap_active_field_root_control`은 구역·본문 부모 문단이 같은 활성 주소만
갱신한다. 본문은 지역 `control_idx`, 셀은 `cell_path[0].0`을 변환하고 삭제한 소유자는
해제한다. 삽입·도형 제거·묶기·풀기가 이 결과를 소비한다. 셀의 지역 필드 번호와
중첩 경로의 나머지 요소를 컨트롤 배열의 전역 번호로 바꾸지 않는다.

공개 API의 Native080 진단2개가 통과했다. 독립 Print8개와 Native079/fresh031 저장본
각8쪽은 전부100%, 쪽수1=1이고 대표 review/overlay에서 표·본문·도형을 직접 판독했다.
이 선행 증거 뒤 새 이름 `active_field_group_root_owner.rs`의 관계 검사4개를 추가했다.
같은 정식 파일의 strict 실행은 Native079에서4FAIL, Native081에서4PASS다. 빌드 실패를
결함 재현으로 세지 않았고 활성 필드 재설정·실제 입력·두 저장 형식의 내용 보존을 검사한다.

Native081 집중1211/1212는165PASS·1FAIL이다. 시험지 내용 검사 runner에 빠진 `sha2`
extern은 runner만 보완해 해당7개를 다시 실행했으며7PASS다. 원 테스트는 바꾸지 않았다.
기존·새 그룹/필드·중첩 부모 분리와 편람 좌표 검사도 통과했다. 남은 #5871은 이전079와
같은 괘선 목록의 고정 구간 검사에서 실패하며 원본·Print·관계 계약을 계속 대조한다.
그 실패나 전체57개 기존 실패를 삭제·이관·기대값 완화로 완료 처리하지 않는다.

[활성 셀 입력·Print·시각 선행 증거](../pr/assets/active_field_group_owner_20261021/README.md)를
보존했고 새 제품의 전쪽·fresh 검증을 계속한다. 원 API28조건의 devel 직접 비교1199는
양쪽28쪽100%,0.1%p 하락0쪽이다. 이 Native 보정을 전체 통합 승인으로 기록하지 않는다.

## 현재 커밋의 fresh WASM 편집 동작 확인

보정 커밋 `ae6d24e744533f87bf39c9ff79e2b6fb4451ecd9`의 최적화 fresh032 빌드와
Studio 패키지 동기화가 성공했다. Native081의 dirty snapshot 소스187개는 이 커밋의
실제 파일 바이트와 모두 같다. snapshot 당시 HEAD와 실제 보정 커밋을 구분해 기록했다.
같은 입력4개를 Node의 실제 `HwpDocument` 공개 API에서 실행해 이전 fresh031의
4FAIL과 현재 fresh032의4PASS를 확인했다. 실패는 같은 활성 셀 주소를 다시 설정할 때
보정이 필요하다는 결과이며, 빌드·초기화 실패가 아니다. 현재 코드는 같은 주소 설정의
idempotence, 표 소유 번호 이동, 지역 필드 정보, 실제 입력과 HWP/HWPX 저장 내용을 보존한다.
원문은 ignored `active-field-wasm-before-after-1223.json`과 같은 번호 로그에 있다.
전체 통합 검증이나 새 제품의 모든 저장본 시각 확인을 이 편집 API 결과로 대신하지 않는다.

Native081의62조건·860쪽은 독립 Print 쪽수가 모두 같고 전체 최저92.44708%다.
고정 devel `2e3b4f880bd11d35398e6ea0100846611f13a72d`의 같은 입력·PDF·글꼴 조건과
전쪽 Decimal 비교1224에서0.1%p 이상 하락은0쪽이다. 전쪽 fresh032는 실행 중이며,
`not_evaluated` TSV 상태를 통합 승인으로 보고하지 않는다. 현재 커밋의 Native Clippy,
WASM32 lib Clippy, workspace build, workspace all-target Clippy1222는 모두 exit0이다.
기록한 Rust/Cargo 소스는 실행 중 변경되지 않았다. 공유 `target/pr-review`, 빌드 잡8개를 사용했다.

# task_m100_4386 Stage 1 — HML COLDEF(다단 정의) 조용한 드롭

- **이슈**: [#4386](https://github.com/edwardkim/rhwp/issues/4386)
- **브랜치**: `fix/issue-4386-hml-coldef`
- **분기 기준**: `upstream/devel` `e48fe8694`
- **상태**: 전체 게이트 통과, PR 게시
- **기록일**: 2026-08-10 KST

## 1. 결함 — "지원함"으로 표시돼 경고를 피했다

`COLDEF` 는 `reader.rs` 전체에서 미지원 판정의 **허용 목록** 한 곳에만 등장했다
(`is_unsupported_inline` 의 `"CHAR" | "SECDEF" | "COLDEF" | ...`). `capture_start` 에는 처리
분기가 없어 `_ => Ok(())` 로 떨어졌다.

즉 **"지원한다"고 표시돼 경고 대상에서 빠지면서 실제로는 버려졌다.**

`Control::ColumnDef` 는 죽은 필드가 아니다 — `renderer/mod.rs:751` 과 `renderer/layout.rs` 8곳이
다단 레이아웃을 구동한다. HWPX 파서는 같은 것을 정상적으로 읽는다(`hwpx/section.rs:496`).

## 2. 재현 — fixture 가 왜 못 잡았나

`samples/hml/` 실물 3개가 전부 `COLDEF Count="1"` 이라 드롭돼도 결과가 같다. `aligns.hml` 의
실제 `SECDEF`/`PAGEDEF`/`COLDEF` 구조를 그대로 따라 `Count="2"` 합성 HML 을 만들고
`rhwp export-svg` 로 렌더링했다:

- 수정 전: 800자가 x=113~672(전체 폭) **한 단**에 몰림 (72자/줄)
- 수정 후: x<390(컬럼1)과 x≥390(컬럼2)로 **1435/1435 글리프씩 균등 분할**

## 3. 부작용을 스스로 잡았다

리더만 고치니 `Control::ColumnDef` 가 새로 생겨 **기존에 통과하던 HML 저장 경로가 전면
막혔다** — `preflight.rs` 의 `_ => unsupported(...)` 폴백에 걸려 19개 테스트가 새로 깨졌다(모든
HML 문서에 COLDEF 가 있다).

`serializer/hml/body.rs` 에 `write_column_def`(역방향 직렬화)와 `preflight.rs` 에
`validate_column_def`(widths/gaps/separator 등 왕복 불가 필드가 기본값이 아니면 export 차단)를
추가해 해결했다. `hml_serializer` 31/31 통과.

## 4. 미확인 후보 12종 — 전부 확인했다

`FOOTNOTESHAPE`/`ENDNOTESHAPE`/`NOTELINE`/`NOTENUMBERING`/`NOTEPLACEMENT`/`NOTESPACING`/
`PAGEBORDERFILL`/`BEGINNUMBER`/`CARETPOS`/`LAYOUTCOMPATIBILITY`/`COMPATIBLEDOCUMENT`/
`NUMBERINGLIST` — **전부 `reader.rs` 에 이름이 한 번도 등장하지 않는다**(grep 0건).

그중 9종(`FOOTNOTESHAPE`~`CARETPOS`)은 실물 `samples/hml/aligns.hml` 의 `SECDEF`/`DOCSETTING`
안에 **실제로 존재하는데도** 부모가 HEAD/BODY/TAIL 이 아니라 `warn_if_unsupported` 의 5개 조건
어디에도 안 걸려 조용히 드롭됨을 확인했다. 나머지 3종은 실물 fixture 에 없어 위치를 실물로
확인하지 못했지만 코드 경로상 같은 논리다.

**고치지 않고 보고만 한다** — 별도 작업이다.

## 5. 사고 — 공유 stash race

`git stash`/`pop` 으로 수정 전 실패를 확인하려다 내 변경 6개 파일이 사라지고 다른 에이전트의
변경이 대신 적용됐다. `refs/stash` 가 모든 worktree 공유라 생긴 일이다. 대화 기록의 diff 로
6개 파일을 재구성했고 복구본과 byte-for-byte 동일함을 확인했다. 이후
`git diff > patch && git checkout -- <files> && test && git apply patch` 방식으로 전환했다.

## 6. 검증 (완료)

- 회귀 테스트 `coldef_with_two_columns_populates_column_def_control_without_warning` 신설.
  **수정 전 실패를 확인**했다(`COLDEF must produce a Control::ColumnDef, not be silently dropped`).
- 기존 `maps_real_hwpml_291_formatting_table_fixture_without_losing_inline_order` 값 갱신 —
  COLDEF 가 이제 `Control::ColumnDef` 로 채워지고 `char_offsets` 가 8 밀리는 것이 정확한 동작이다
  (주석으로 이유 명시). `samples/` 실물 fixture 파일 자체는 건드리지 않았다.
- `cargo test --profile release-test --tests --no-fail-fast` 통과.
- `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings` 통과.

## 7. 범위 밖

`SECDEF` 자체의 속성(`SpaceColumns`/`TabStop`/`CharGrid` 등)이 전혀 캡처되지 않는 것을 발견했으나
이슈 범위 밖이라 보고만 한다.

남은 미래 조건은 GitHub Actions 와 작업지시자 승인, merge 다.

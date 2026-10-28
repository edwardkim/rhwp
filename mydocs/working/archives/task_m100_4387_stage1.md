# task_m100_4387 Stage 1 — HWPX 단별 폭(hp:colSz) 읽기·쓰기

- **이슈**: [#4387](https://github.com/edwardkim/rhwp/issues/4387)
- **브랜치**: `fix/issue-4387-hwpx-colsz`
- **분기 기준**: `upstream/devel` `e48fe8694`
- **상태**: 로컬 검증 통과, PR 게시
- **기록일**: 2026-08-10 KST

## 1. 결함 — 스키마에 있는데 양쪽 다 미구현

OWPML `ColumnDefType`(`ParaList XML schema.xml:1415`)은 `colLine` 다음 자식으로 `colSz` 를
정의한다 — `minOccurs=0 maxOccurs=255`, 속성 `width`(`xs:positiveInteger`)/`gap`
(`xs:nonNegativeInteger`), 문서화 *"sameSize가 false일 때, 각 단의 크기 및 사이간격"*.

IR 에도 자리가 있다 — `ColumnDef.widths`/`gaps`/`proportional_widths`(`model/page.rs:134-138`).

그런데 `colSz` 문자열이 HWPX 파서·직렬화기 **어디에도 없었다**(grep 0건).

## 2. 소비자 확인 — 채우는 것이 의미 있다

리뷰에서 "아무도 안 읽으면 채워도 무의미하다"는 반증 가능성을 먼저 확인했다.

`renderer/page_layout.rs::calculate_column_areas` 가 실제로 읽는다:
- `:194` `if !column_def.same_width && column_def.widths.len() >= col_count`
- `:199-217` `proportional_widths` 면 비례 계산, `:232-234` 아니면 절대 HWPUNIT
- `main.rs` CLI 덤프도 같은 필드를 출력한다

**반증 실패 — 이슈는 진짜였다.**

## 3. 왕복 재현

Document IR 을 직접 구성해(`serialize_hwpx` → zip 추출 → `parse_hwpx`) 불균등 2단
(4000/6000 HWPUNIT, gap 500/0)을 왕복시켰다. 수정 전에는 `sameSz="0"` 만 방출되고
`<hp:colSz>` 가 전혀 없어 폭이 사라졌다. 샘플 파일은 건드리지 않고 IR API 로 합성했다.

## 4. 구현

- **파서** — `parse_col_sz()` 신설, `parse_col_pr_with_children` 의 `Event::Empty`/`Event::Start`
  **양쪽** 분기에 연결.
- **직렬화기** — `render_col_pr_ctrl` 이 `sameSz="false"` 이고 `widths` 가 비어 있지 않을 때
  `colLine` **뒤**(스키마 `xs:sequence` 순서)에 단별 `<hp:colSz>` 를 방출.
- `proportional_widths` 는 건드리지 않았다 — HWPX 는 절대 HWPUNIT 이라 기본값 `false` 가 정답이고,
  HWP5 바이너리 파서만 비례값(합계 32768)이라 `true` 로 켠다.

## 5. 리뷰가 잡은 것 — i16 오버플로

`ColumnDef.widths`/`gaps` 는 `Vec<HwpUnit16>` 이고 `HwpUnit16 = i16`(`model/mod.rs:28`), 최대
**32767**. 스키마의 `width` 는 `xs:positiveInteger` 로 **상한이 없다.** A4 본문 폭이 ~48000
HWPUNIT 이라 2단이면 들어가지만 1단·비대칭·A3 에서는 넘친다.

**재현**: `parse_i16`(`parser/hwpx/utils.rs:54`)은 `parse::<i16>().unwrap_or(0)` 이다.
`"40000".parse::<i16>()` 은 패닉도 wrap 도 아닌 `PosOverflow` 에러이고, `unwrap_or(0)` 이 그것을
삼켜 **무경고 0-폴백**한다 — `widths=[0, 13000]`, 단이 통째로 사라진다. 스키마 위반 음수
(`gap="-7"`)는 그대로 통과해 `page_layout.rs:232` 의 `as i32` 에서 음수 폭이 된다.

**조치**: `parse_col_sz` 전용 `parse_hwpunit16_saturating` — i64 로 먼저 파싱 후 `HwpUnit16` 범위로
clamp, 잘렸을 때만 경고. `gap` 은 스키마상 `nonNegativeInteger` 라 `.max(0)` 하한 추가.
결과: `width="40000"` → `32767`(0 이 아니라 단이 살아남음), `gap="-7"` → `0`.

공용 `parse_i16` 은 건드리지 않았다 — `section.rs` 전역에서 쓰여 blast radius 가 크고, 이번 문제는
HWPX 가 절대 HWPUNIT 을 처음 담기 시작한 `colSz` 에 국한된다.

**`HwpUnit16` 타입 확장은 하지 않았다** — HWP5 바이너리 경로와 `table.rs::row_sizes` 등 전 IR 에
파급되므로 이번 범위를 넘는다. 근본 해결이 필요하면 별도 이슈로 추적한다.

## 6. 검증 (완료)

- 회귀 테스트 3건 — 왕복 보존, 정상 범위 개별 폭·간격, **오버플로/음수 gap 이 0-소실이 아니라
  saturate** 확인. `git diff > patch` → `checkout --` → 실행으로 **수정 전 실패를 확인**했다
  (`colSz 요소가 방출돼야 함(#4387)`). `git stash` 는 쓰지 않았다(워크트리 간 공유 사고 회피).
- `cargo test --profile release-test --tests --no-fail-fast` — 496개 바이너리 중 495개 ok.
- `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings` 통과.

**flake 1건**: `document_core::text_security::tests::scan_cost_stays_linear_as_input_grows` 가
실패했다. 이 변경과 무관한 파일의 **벽시계 타이밍 비율 단언**이고, 실행 당시 이 머신의 load
average 가 120~130(16코어)이었다. 같은 라운드 초반 부하가 낮을 때는 통과했다. `issue4387_*` 3건은
전부 통과했다.

남은 미래 조건은 GitHub Actions 와 작업지시자 승인, merge 다.

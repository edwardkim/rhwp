---
kind: investigation
status: active
canonical: mydocs/manual/codex/docs_and_git_workflow.md
last_verified: 2026-08-02
---

# Task #3738 결과 보고 — Stage 1–16 그림·RowBreak·각주 reservation 보정과 전체 pagination 잔여

- 이슈: [#3738](https://github.com/edwardkim/rhwp/issues/3738)
- 관련 PR: [#3740](https://github.com/edwardkim/rhwp/pull/3740)
- 상태: **Stage 8의 HWP p23 그림 21 caption, Stage 9의 HWP p66 table-footnote first fragment, Stage 11의 HWP p67 footer collision, Stage 13의 HWP p30 각주 29 page ownership과 p68 그림 49/caption 이월, Stage 14의 HWP p58 existing-footnote reset-tail, Stage 15의 HWP p76 표 24 reset-tail과 p77 그림 51 owner, Stage 16의 HWP p31 두 줄 각주 30 fragment는 복원. HWP는 아직 221쪽, 기준 PDF는 215쪽이므로 전체 pagination 정합은 계속 조사 중.**

## 완료한 작업

1. 한컴 2020 MCP의 HWP/HWPX 기준 PDF를 준비하고, 그림 23의 p23–p24 페이지 소유권을
   visual sweep으로 재현했다.
2. native HWP의 빈 1×1 `RowBreak` 그림 표가 다음 문단의 `LINE_SEG` 되감기 신호를 가질 때,
   잔여 공간에 force-split하지 않고 fresh page로 이월하도록 `typeset.rs`를 보정했다.
3. HWP p23에서는 그림 23의 조기 배치가 사라진 것을 review PNG로 확인했다.
4. Stage 2에서 outer host의 page-boundary 상쇄값을 page-local picture offset으로 정규화하는
   후보를 검증했으나, 실제 셀의 `LINE_SEG.vpos`가 0임을 확인해 조건이 발동하지 않았음을 visual
   sweep과 render tree로 반증했다. 효과가 없는 후보 구현은 커밋하지 않고 증적·분석만 보존했다.
5. Stage 3에서 outer host의 first stored `LINE_SEG.vpos`를 root-body table 호출에서만 cell picture
   배치까지 전달했다. `52230 + 560 - 52790 = 0 HU`인 native HWP5 RowBreak 형상에만 picture를
   page-local content top으로 정규화해 HWP p24 Image bbox를 `-181.4px`에서 `92.5px`로 복원했다.
6. Stage 4에서 셀 안 non-inline picture의 Bottom caption을 기존 picture frame 및 caption spacing으로
   배치하고, 같은 native HWP5 empty `RowBreak` 형상에서만 table paint geometry를 유지한 채 다음 저장
   `LINE_SEG`로 outgoing flow cursor를 재설정했다. HWP p24에서 3줄 caption, `○ EU에서 …`, 표 4와
   후속 본문이 기준 PDF의 순서·page-local 위치로 돌아왔고 `frame_overflow_pixels`도 사라졌다.
7. Stage 5에서 HWPX p273 그림 11의 단일 `TopAndBottom` 표도 저장 anchor lane을 사용하도록, 기존 빈 host·
   단일 float·다음 vpos 증가·선언 bottom 적합 조건은 그대로 둔 채 original HWPX stored-layout에만 해당
   gate를 열었다. 일반 block fit의 4.7px 초과 이월이 사라져 그림 11과 후속 흐름은 기준 PDF와 같은 p13–p15에
   놓였다.
8. Stage 6에서 HWPX p344의 next-vpos rewind `RowBreak` 그림 표도 fresh page로 defer하도록 source
   profile gate를 최소 확장했다. p344 table은 p23에서 사라져 p24 `y=90.6px`으로 이동했고, p13–p15
   그림 11 흐름도 회귀하지 않았다.
9. Stage 7에서 HWPX p344의 image offset reset이 적용되지 않던 직접 원인(outer host stored vpos를
   HWP5에만 전달하던 caller gate)을 고쳤다. p344 picture는 p24 `y=92.5px`, Bottom caption은
   `y=434.4/455.7/477.1px`으로 복원됐고, p345 이후 flow도 기준 PDF 순서를 유지한다.
10. Stage 8에서 native HWP p23의 그림 21/22가 Center cell 안에서 Bottom caption을 제외한 그림 본체만
    중앙 정렬해 caption이 약 50px 아래로 밀리는 원인을 고쳤다. Bottom caption height와 spacing을 그림
    본체와 하나의 시각 블록으로 Center/Bottom 정렬하되, Top caption·일반 picture·pagination은 바꾸지
    않았다. 그림 21 caption 첫 줄은 `544.7px → 494.7px`으로 이동했고 한컴 PDF의 `495.16px`와
    0.46px 차이다.
11. Stage 9에서 HWP p728 7×2 `RowBreak` 표의 모든 table-cell footnote `294.0px`를 fragment 전부터
    선예약해 p66 표 전체가 이월되던 흐름을 고쳤다. 작은 non-rowspan RowBreak 표만 fragment queue로
    좁히고, 확정한 fragment page에 들어가는 cell footnote를 순서대로 등록했다. HWP p66은 기준 PDF처럼
    표 0–4행(Organ Donation까지)과 각주 76·77을 보유하고 p67은 Stephanie/Policy 5–6행부터 재개한다.
    전체 HWP 쪽수는 225→224가 됐다.
12. Stage 10에서 p67 각주의 composed line spacing을 paginator까지 단순 전파한 후보는 224→226쪽으로
    악화해 커밋하지 않고 기각 근거만 남겼다.
13. Stage 11에서 p67 `FootnoteArea` reservation이 paint가 누적하는 trailing line-spacing을 빠뜨린
    것을 고쳤다. renderer-side area height만 exact paint 산식에 맞춰 `y=669.5, h=369.8px`에서
    `y=600.6, h=438.7px`으로 복원했고, actual footnote bottom은 footer top `1039.3px`에서 끝난다.
    paginator를 건드리지 않아 224쪽을 유지했고 p66 table fragment ownership도 회귀하지 않았다.
14. Stage 12에서 native HWP5 문단 407의 첫 각주 29와 `vpos=0` reset을 실제 composed footnote
    height로 대조했다. visible body line은 각주 위에 남되 trailing line-spacing만 각주 영역을 넘는
    경우에만 tail을 분리하도록 좁혀, p30의 두 줄/각주 overlap을 제거하고 p31의 tail·`5. 독일`,
    p32의 그림 35 page ownership을 기준 PDF처럼 복원했다. 이 단계는 body split geometry만 고쳤고,
    split 뒤 footnote control이 p31에 등록되는 page ownership 결함은 Stage 13에서 분리했다. 넓은 reset
    강제 split 후보는 226쪽 regression을 일으켜 커밋하지 않았다.
15. Stage 13에서 completed page의 inline marker를 다시 찾아 footnote 29를 p30에 등록했다. 또한 existing
    `FootnoteArea`를 실제 물리 경계로 대조해, native HWP5의 비글자처럼 `TopAndBottom` `RowBreak` 2×1
    picture+caption 표에만 첫 fragment near-fit을 허용했다. 그림 49와 caption은 p68으로, p69는 다음 절로,
    p70은 caption 고아 없이 이어진다.
16. Stage 14에서 native HWP5 plain-text paragraph가 existing `FootnoteArea` 앞에 저장한 `vpos=0` reset
    전 tail을 실제 footnote top과 대조했다. p58은 각주 70 위의 세 줄을 유지하고 p59는 `독립적이며 …`로
    시작한다. 이 보정으로 native HWP 출력은 223→222쪽이 됐다.
17. Stage 15에서 표 24 row 4의 저장된 `vpos=0 → 1620 → 3240 → 0` 내부 reset을 확인했다. native HWP5의
    비-TAC `RowBreak` continuation row만 painted-height minimum keep과 실제 기존 각주 경계를 사용하게
    좁혀, p76에는 reset 전 세 줄을 남기고 p77에는 reset 후 tail·그림 51·caption을 각주 103·104 위에
    함께 복원했다. p78은 `3. EU`로 시작하고 p79도 표 내용을 유지한다. 출력은 222→221쪽이 됐다.
18. Stage 16에서 HWP 문단 421의 각주 30 marker가 reset 전 본문 줄(`text_start=183..247`) 안에 있고, 두 줄 각주를
    원자 배치하면 p31 본문과 겹침을 확인했다. 이 native HWP5 첫-각주 형상에만 line fragment를 도입해 p31에는
    separator·번호가 있는 첫 줄, p32에는 separator·번호 없는 `„incentives”…` tail을 배치했다. p31 body bottom
    `1006.0px`와 separator `1019.0px`가 분리되고 p31–p32 selected sweep은 2/2 완료·structural flag 0건이다.

## 미해결과 다음 회차

HWP 그림 23 p23–p24 체인, HWPX의 더 이른 그림 11 이월, HWPX p344 table의 페이지 소유권과 image/caption
offset, 그리고 HWP p23 그림 21 caption cell 정렬은 해당 선택 페이지 범위에서 해소됐다. HWPX p344는
24쪽 `y=90.6px`, image `y=92.5px`, Bottom caption 3줄 `y=434.4/455.7/477.1px`이고 23쪽에는 없다.
Stage 8 HWP 그림 21 caption 첫 줄은 PDF `495.16px` 대비 rhwp `494.7px`이다.

그러나 전체 문서는 아직 **HWP 221쪽/HWPX 224쪽, 한컴 PDF 215쪽**이다. Stage 16은 p31–p32 두 줄 각주 흐름만
복원했다. 사용자 화면에서 재관측한 p37 그림 중복, p43·p54·p66 본문 또는 표/각주 충돌, p76–p79 UI 재확인,
p80 표 cell 본문·URL·각주의 다층 overlap, p83 paragraph overflow, p87·p90·p99–p100의 기준 PDF 차이는 모두 잔여다. 다음 Stage는 같은 번호가 아니라
semantic page owner를 먼저 맞춘 뒤 각 결함을 독립 원인으로 분석한다. 완료된 선택 페이지를 전체 pagination
완료로 표현하지 않는다.

원본 HWP·HWPX와 각각의 기준 PDF, review PNG는 모두
[`pdf/pr3740/README.md`](../../pdf/pr3740/README.md) 및 연결된 증적에 보관한다. 상세 페이지 비교와
자동 후보는
[Stage 1 visual sweep](../working/task_m100_3738_stage1_visual_sweep.md),
[Stage 2 visual sweep](../working/task_m100_3738_stage2_visual_sweep.md),
[Stage 3 visual sweep](../working/task_m100_3738_stage3_visual_sweep.md),
[Stage 4 visual sweep](../working/task_m100_3738_stage4_visual_sweep.md),
[Stage 5 visual sweep](../working/task_m100_3738_stage5_visual_sweep.md),
[Stage 6 visual sweep](../working/task_m100_3738_stage6_visual_sweep.md),
[Stage 7 visual sweep](../working/task_m100_3738_stage7_visual_sweep.md),
[Stage 8 visual sweep](../working/task_m100_3738_stage8_visual_sweep.md),
[Stage 9 visual sweep](../working/task_m100_3738_stage9_visual_sweep.md),
[Stage 11 visual sweep](../working/task_m100_3738_stage11_visual_sweep.md),
[Stage 12 visual sweep](../working/task_m100_3738_stage12_visual_sweep.md),
[Stage 13 visual sweep](../working/task_m100_3738_stage13_visual_sweep.md),
[Stage 14 visual sweep](../working/task_m100_3738_stage14_visual_sweep.md),
[Stage 15 visual sweep](../working/task_m100_3738_stage15_visual_sweep.md),
[Stage 16 visual sweep](../working/task_m100_3738_stage16_visual_sweep.md)에 기록했다.

현재 그림별 선택 페이지의 완료 판정과 전체 215쪽 pagination 완료 판정을 혼동하지 않는다. 전체 215쪽
raster sweep이나 전체 integration test는 Stage 8의 좁은 caption 보정 완료 근거로 사용하지 않았다.

## 검증 범위

- `cargo fmt`
- `CARGO_TARGET_DIR=target/review-planet6897-20260802 CARGO_INCREMENTAL=0 cargo test relocated_hwp5_picture_caption_uses_next_saved_flow_anchor --lib --quiet`
- `CARGO_TARGET_DIR=target/review-planet6897-20260802 CARGO_INCREMENTAL=0 cargo build --profile release-test --bin rhwp`
- HWP 및 HWPX 각각 p23–p24, 144 DPI visual sweep — 선택 페이지 2/2 완료
- HWPX p13–p15, 144 DPI visual sweep — 선택 페이지 3/3 완료; 최초 그림 11 page drift 해소 확인
- `CARGO_TARGET_DIR=target/review-planet6897-20260802 CARGO_INCREMENTAL=0 cargo test stored_layout_relocated_empty_rowbreak_picture --lib --quiet`
- HWPX p23–p24, p13–p15, 144 DPI visual sweep — 각각 2/2, 3/3 완료; p344 table ownership만 해소 확인
- `CARGO_TARGET_DIR=target/review-planet6897-20260802 CARGO_INCREMENTAL=0 cargo test stored_layout_relocated --lib` — 2 passed
- HWPX p23–p24, p13–p15, 144 DPI visual sweep — 각각 2/2, 3/3 완료; p344 image/caption offset 복원 확인
- `CARGO_TARGET_DIR=target/review-planet6897-20260802 CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_3738_hwp_caption_cell_alignment` — 1 passed
- HWP p23, 144 DPI visual sweep — SVG/render tree 225쪽 생성, 선택 raster 1/1 완료; 그림 21 caption 직접 좌표와 PDF의 차이 0.46px
- `CARGO_TARGET_DIR=target/review-planet6897-20260802 CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_3738_rowbreak_table_footnote_fragment` — 1 passed
- HWP p66–p67, 144 DPI visual sweep — SVG/render tree 224쪽 생성, 선택 raster 2/2 완료; p66의 표 0–4행/각주 76·77 ownership 복원, p67 35px frame overflow 후보는 잔여로 기록
- `CARGO_TARGET_DIR=target/review-planet6897-20260802 CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_3738_rowbreak_table_footnote_fragment --test issue_3738_hwp_caption_cell_alignment` — 2 passed
- HWP p66–p67, 144 DPI visual sweep — SVG/render tree 224쪽 생성, 선택 raster 2/2 완료; p67 `FootnoteArea` actual bottom과 footer top 모두 1039.3px, structural 후보 0건
- `CARGO_TARGET_DIR=target/review-planet6897-20260802 CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_3738_rowbreak_table_footnote_fragment --test issue_3738_hwp_caption_cell_alignment` — 3 passed
- `CARGO_TARGET_DIR=target/review-planet6897-20260802 CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_3738_rowbreak_table_footnote_fragment -- --nocapture` — 3 passed
- `CARGO_TARGET_DIR=target/review-planet6897-20260802 CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_3738_hwp_caption_cell_alignment` — 1 passed
- `CARGO_TARGET_DIR=target/review-planet6897-20260802 CARGO_INCREMENTAL=0 cargo build --profile release-test --bin rhwp` — passed
- HWP p30–p32, p66, p68–p70, 144 DPI visual sweep — SVG/render tree 223쪽 생성, 선택 raster 7/7 완료; p30 각주 29 ownership과 p68 그림 49/caption ownership 복원
- `CARGO_TARGET_DIR=target/review-planet6897-20260802 CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_3738_rowbreak_table_footnote_fragment -- --nocapture` — 4 passed
- `CARGO_TARGET_DIR=target/review-planet6897-20260802 CARGO_INCREMENTAL=0 cargo build --profile release-test --bin rhwp` — passed
- HWP p58–p59, 144 DPI visual sweep — SVG/render tree 222쪽 생성, 선택 raster 2/2 완료; p58 stored reset 전 tail과 p59 재개 경계 복원
- `CARGO_TARGET_DIR=target/review-planet6897-20260802 CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_3738_rowbreak_table_footnote_fragment` — 5 passed
- `cargo fmt --check` 및 `CARGO_TARGET_DIR=target/review-planet6897-20260802 CARGO_INCREMENTAL=0 cargo build --profile release-test --bin rhwp` — passed
- HWP p76–p78, 144 DPI visual sweep — SVG/render tree 221쪽 생성, 선택 raster 3/3 완료; 표 24 reset-tail과 그림 51/caption owner 복원
- HWP p79, 144 DPI 단일 visual sweep — 선택 raster 1/1 완료; 단독 그림 51과 빈 표 페이지가 아님을 확인
- `cargo fmt --check` 및 `CARGO_TARGET_DIR=target/review-planet6897-20260802 CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_3738_rowbreak_table_footnote_fragment -- --nocapture` — 6 passed
- `CARGO_TARGET_DIR=target/review-planet6897-20260802 CARGO_INCREMENTAL=0 cargo build --profile release-test --bin rhwp` — passed
- HWP p31–p32, 144 DPI visual sweep — SVG/render tree 221쪽 생성, 선택 raster 2/2 완료; 각주 30의 p31 first fragment·p32 continuation tail 및 p31 body/separator non-overlap 확인

전체 integration test와 215쪽 전체 raster sweep은 이 회차의 완료 근거로 사용하지 않았다. 전체
pagination 정합이 아직 남아 있으므로, 이 보고서는 선택 그림 흐름과 해당 Stage의 국소 ownership만
완료로 표현한다.

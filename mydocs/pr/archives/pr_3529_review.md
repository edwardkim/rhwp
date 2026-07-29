---
kind: pr_review
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-07-29
---

# PR #3529 리뷰 — HWP3 암호 문서 렌더링 정합

- PR: [#3529](https://github.com/edwardkim/rhwp/pull/3529)
- 관련 이슈: [#3486](https://github.com/edwardkim/rhwp/issues/3486) (`Closes` 미사용)
- 역할: `jangster77` collaborator self-review
- 구현 보정 source head: #3529 최신 head (CI 회귀 보정 포함)

## 라우팅과 merge 조건

```text
base route: collaborator_self_merge.md
modifiers: intake_and_review.md, local_validation.md,
  visual_fixture_evidence.md, multi_pr_update_branch.md
```

최신 `upstream/devel` 위로 rebase한 뒤 PR을 만들었다. 이 self-review는 독립 승인과 메인터너 검토를
대체하지 않는다. merge 전에는 이 review·오늘할일을 포함한 최신 head의 required check 성공,
`MERGEABLE` 재확인, 메인터너 검토와 작업지시자 승인이 모두 필요하다.

## 변경 범위와 수용 판단

1. HWP3 암호 원본의 Square-wrap 도형은 column 기준 좌표, no-fill/no-line sentinel, 문단 inset과
   기본 text gap을 보존한다. 이 계약은 일반 HWP3와 다르므로 복호화 원본 경로로 한정했다.
2. 페이지 배경 이미지는 raw legacy brightness/contrast의 저장 순서와 화면 투영 순서를 분리하고,
   일반 `RealPic` 배경을 watermark opacity로 잘못 낮추지 않는다. SVG·Web Canvas·Skia가 같은 규칙을
   사용한다.
3. 실제 Studio Canvas2D·CanvasKit·비교 창의 bitmap 경계는 fractional CSS A4 크기를 같은 방식으로
   올림해 144dpi에서 마지막 물리 pixel을 보존한다.
4. 새 HWP5 비교 fixture와 현재 HWPX IR 정규화 결과를 field-sweep baseline에 등록했다.

**조건부 수용 권고.** 실제 fixture의 암호 열기·도형·배경·A4 경계 계약은 수용 가능하나, 한컴 전용
옛한글 glyph/metric과 전수 PDF fidelity는 #3486의 열린 후속 범위다.

## 시각 검증

- 입력: `samples/HWP3-password-123456.hwp`
  - SHA-256: `db743d084efc9e08e839a5b4d978b16b8676434011776e090e4cda43e57304be`
- 기준 PDF: `pdf/HWP3-password-123456.pdf` (24쪽)
  - SHA-256: `3ced5ad95ad30331e2756b5b34509c1ac91dfe3c72013c8e14f2556ca6bd5776`
- 방식: Studio password-open 경로로 같은 fixture를 연 뒤 144dpi Canvas와 PDF p1을 대조했다.
- 결과: p1의 배경·아이콘·Square-wrap 본문·차례 도형과 A4 우·하단 경계가 증적에 실제로 보인다.
  제목의 `ᄒᆞᆫ` glyph 외관은 PDF와 다르며 Stage 9 판정에 따라 의도적으로 현대 음절로 치환하지 않았다.

![HWP3 암호 문서 p1 — Hancom PDF와 Studio Canvas 실제 대조](../assets/pr_3529_hwp3_password_p001_canvas_review.png)

## 검증 기록

모든 Cargo 검증은 `CARGO_TARGET_DIR=target/task_3486_render_v2`, `CARGO_INCREMENTAL=0`에서 실행했다.
공유 `target/debug`, `target/release`, `target/release-test`, `target/wasm32-unknown-unknown`은 삭제하거나
검증 결과에 섞지 않았다.

| 검증 | 결과 |
| --- | --- |
| `cargo fmt --check`, `git diff --check` | passed |
| `cargo clippy --all-targets -- -D warnings` | passed |
| `hwp3_password_fixture` | 8 passed |
| `hwpx_password_fixture` | 3 passed |
| `issue_1892` HWP3 drawing/tab round-trip | 4 passed (CI 회귀 보정 후 재실행) |
| `issue_1692_so_sueop_hwp3_page22_relationship_box_uses_table_flow` | passed (일반 HWP3/HWPX spacing 회귀 보정 후 재실행) |
| HWP3 암호·일반 문단 계약 unit test | 2 passed |
| `ir_field_sweep_baseline` | 2 passed; 683 divergence path(684 TSV 행) baseline 재생성·재검증 |
| `test_scaled_canvas_extent_keeps_fractional_a4_edge` | passed |
| native-Skia 3개 게이트 | passed |
| `wasm-pack build --target web --out-dir pkg` | passed |
| Chrome·Firefox extension `npm run build` | passed |
| Studio `npm run e2e:hwp-password-open` | passed; HWP3 144dpi A4 경계 확인 |
| Studio `npm test` | 675 passed |
| Studio `npm run build` | passed |
| CanvasKit readiness corpus | 별도 7777 포트의 현재 작업본에서 7/7 Canvas2D↔CanvasKit parity 및 readiness passed |

최신 head CI에서 `issue_1692_so_sueop_hwp3_page22_relationship_box_uses_table_flow` 실패를 기준
`upstream/devel` 별도 worktree에서 대조했다. 기준은 통과했고, 이번 변경의 HWPX
`HwpUnitChar` 앞·뒤 간격 전역 반감이 일반 HWPX와 HWP3 변환본을 함께 위로 밀어 만든 회귀였다.
HWPX는 공통 2배 IR 스케일을 복원하고, 암호 HWP3의 별도 spacing 계약은 HWP3 복호화 parser에만
한정했다. 이 정규화로 field-sweep의 `doc_info.para_shapes[].spacing_*` 발산 129건이 사라지고
`raw_header_extra` 20개 집계가 의도적으로 달라져 baseline을 재생성했다(순개선 109건). 보정 후
#1692, #1892, HWP3/HWPX 암호 fixture와 재생성한 field-sweep를 모두 재실행해 통과했다.

최초 최신-head CI의 Canvas visual diff는 Canvas2D가 `794×1123`, CanvasKit이 `793×1122` bitmap을 만들어
7개 readiness 비교를 오류 처리한 것을 확인했다. `e999f7a9f`에서 CanvasKit과 비교 창도 `ceil` 경계를 쓰도록
보정했고, 위 별도 포트 readiness 재현으로 크기 불일치가 해소됐음을 확인했다. 이 보정이 포함된 새 head의
CI를 다시 통과 기준으로 사용한다.

그 뒤 최신 head CI의 `issue_1892_hwp3_drawing_group_roundtrip_render_is_self_consistent` 실패를
로컬에서 재현했다. 원인은 HWP3의 가시 개체 제어문자를 파싱 단계부터 전역 8 UTF-16 슬롯으로 계산한
것이었다. 일반 HWP3의 `LineInfo`·`CharShape`는 화면 marker 1칸 기준이므로 도형 문단의 y 좌표가
밀렸다. 일반 HWP3는 1칸을 유지하고, HWP5 변환본과 대조한 실제 암호 HWP3 복호화 경로만 8칸 계약을
사용하도록 분리했다. 이 변경 뒤 `issue_1892` 4건과 `hwp3_password_fixture` 8건을 모두 재실행해
통과했다. 이 기록을 포함한 새 head의 CI만 merge 기준으로 사용한다.

## PUBLIC_WEB_ADMIN_GUIDE 영향 확인

이 PR은 parser·renderer·Studio 내부 렌더 경로와 fixture만 변경한다. 공개 웹사이트의 HWP 링크
인식(`data-hwp-*`), hover preview, 다운로드 URL, 관리자용 HTML/CMS 적용 예시는 변경하지 않는다.
따라서 `PUBLIC_WEB_ADMIN_GUIDE.md`의 공개 통합 계약에는 영향이 없으며, 해당 속성이나 외부 URL의
호환성 검증·문서 갱신 대상도 없다.

## 위험과 후속 보완

- 제목의 한컴 전용 옛한글 glyph/advance와 일부 본문 font metric은 PDF와 아직 다르다.
- p3 표·bullet/폭 같은 전수 시각 차이는 후속 Stage에서 별도의 재현·원인·회귀로 처리한다.
- Stage 11은 구현 없는 다음 분석 계획이므로 이 PR에 포함하지 않는다.

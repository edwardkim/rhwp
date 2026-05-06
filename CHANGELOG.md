# Changelog

이 프로젝트의 주요 변경 사항을 기록합니다.

## [0.7.10] — 2026-05-06

> v0.7.9 후속 patch 사이클 — 외부 기여자 7명 흡수 (PR 13건 cherry-pick) + AI 파이프라인 / VLM 연동 도입 + CLI 바이너리 릴리즈 파이프라인 (Issue #608/#612).

### 신규 기능

- **CLI 바이너리 릴리즈** (Issue #608/#612, [@almet](https://github.com/almet) 의 요청)
  - 4 플랫폼 GitHub Release 자산 첨부 (Linux x86_64 / macOS x86_64+aarch64 / Windows x86_64)
  - SHA-256 체크섬 동봉
  - `.github/workflows/release-binary.yml` 신규
- **PNG raster backend** (PR #599, [@seo-rii](https://github.com/seo-rii)) — render P4 단계
  - native Skia 기반 `PageLayerTree` → PNG export
  - `native-skia` feature gate (기본 빌드 영향 0, opt-in)
  - `DocumentCore::render_page_png_native(page)` API
  - **AI 파이프라인 + VLM (Vision-Language Model) 연동 도입** (메인테이너 후속 정정)
    - `--vlm-target claude` (1568 longest edge / 1.15 MP, Claude Vision 정합)
    - `--scale <배율>` / `--max-dimension <픽셀>` (자동 scale 계산)
    - `export-png` CLI 명령 + 매뉴얼 (한글 + 영문 dual)
    - 한글 폰트 fallback chain + char 단위 fallback (공백 두부 정정) + `--font-path` 동적 폰트 로딩

### 외부 PR cherry-pick (13 PR / 7 컨트리뷰터)

#### [@planet6897](https://github.com/planet6897) / Jaeook Ryu — 8 PR (협업 컨트리뷰터)

- **PR #587** — HWP 5.0 스펙 0x18/0x1E swap (하이픈 ↔ 묶음 빈칸)
- **PR #589** (Task #511 v2 + #554) — HWP3 Square wrap 보완6+8 (페이지네이션 안전) + HWP3 변환본 식별 휴리스틱 (HWP 3.0 직렬화 round-trip 정합)
- **PR #561** (Task #548) — 셀 inline TAC Shape margin + indent 정정
- **PR #564** (Task #521) — TAC 표 outer_margin_bottom 누락 정정
- **PR #570** (Task #568) — 인라인 표+수식 단락 우측 편위 정정
- **PR #575** (Task #573) — 보기 셀 분수 단락 인라인 표 셀 paragraph 라우팅 정정 (Issue #572 인접 효과 자동 정정)
- **PR #580** (Task #577) — 셀 내부 단독 TopAndBottom 이미지 1라인 오프셋 정정 (HWP IR anchor 시점 정합)
- **PR #584** (Task #574) — HY견명조 heavy display 오분류 정정 (TDD Stage 2/3 RED→GREEN)
- **PR #592** (Task #588) — exam_eng.hwp p7 #40 글상자 사이 화살표 누락 (PUA U+F003B → ↓ 매핑 + PDF 글리프 외곽 직접 분석)
- **PR #593** (Task #590) — Square wrap 표 horz_rel_to=단 속성 정합 (단일 줄 분기 가드)
- **PR #567** (Task #565) — 인라인 수식 미렌더 정정

#### [@oksure](https://github.com/oksure) (Hyunwoo Park) — PUA SVG 출력

- **PR #600** (closes #513) — Supplementary PUA-A (U+F02B1~F02C4) SVG 출력 정정 — Task #509 후속 매핑 우선순위 정정

#### [@jangster77](https://github.com/jangster77) (Taesup Jang) — HWP3 본질

- **PR #589** (commit author, @planet6897 PR 등록 협업 흐름)

#### [@seo-rii](https://github.com/seo-rii) — render P4

- **PR #599** (refs #536) — native Skia PNG raster backend

### 메인테이너 정정

- **Skia 폰트 영역 5개 정정** (PR #599 후속, `876d820`):
  - 한글 폰트 fallback chain 추가 (Noto Sans KR / Nanum 등)
  - `--font-path` 동적 폰트 로딩 (`with_font_paths` API, ttfs 디렉토리)
  - char 단위 fallback (NBSP / U+2007 / U+200B 두부 정정)
  - VLM 옵션 (`PngExportOptions` + `VlmTarget::Claude`)
  - `export-png` CLI 명령 + 매뉴얼 dual

### 인프라

- **CI 빌드 안정성 향상** (`Cargo.toml` `[[example]] required-features`)
- **광범위 페이지네이션 회귀 sweep 도구** — 164 fixture (158 hwp + 6 hwpx) / 1,614 페이지 자동 검증 (PR #564 도입, 본 사이클 모든 PR 처리에 적용)

### 후속 이슈

- [#613](https://github.com/edwardkim/rhwp/issues/613) — VLM 프리셋 확장 (GPT-4V / Gemini / Qwen-VL / LLaVA)
- [#614](https://github.com/edwardkim/rhwp/issues/614) — DPI 메타데이터 옵션 (`--dpi` PNG pHYs chunk)
- [#615](https://github.com/edwardkim/rhwp/issues/615) — `pua_oldhangul.rs` U+F53A 매핑 한컴 정합 영역
- [#598](https://github.com/edwardkim/rhwp/issues/598) — rhwp-studio 각주 삭제 기능 (한컴 정합 UX, 외부 컨트리뷰터 공개)

### 잔여 PR (v0.7.11 후속 patch 영역)

- PR #601, #602 (@oksure)
- PR #607 (@dicebattle)
- PR #609 (@jangster77, Task #604 Document IR 표준 정합화)
- PR #611 (@kihyunnn)

---

## [0.7.9] — 2026-05-01

> v0.7.8 후속 사이클 — Task #501 (cell.padding 한컴 방어 로직) + PR #428/#494/#478/#498 cherry-pick + 외부 기여자 4명 흡수

### 회귀 정정 (메인테이너)

- **Task #501 — mel-001.hwp 2쪽 표 셀 높이 처리 회귀** (closes #501)
  - 본질: HWP 셀 IR 의 `cell.padding.top + bottom > cell.height` 인 비정상 케이스 (mel-001 셀[21] r=2 c=2 "현 원": pad=(141,141,1700,1700), cell.h=1280 HU). HWPX `hasMargin="0"` 명시 정합.
  - 회귀 origin: Task #347 의 `prefer_cell_axis` 가드가 비정상 padding 도 cell 우선 적용 → row_heights 거대 → TAC 표 비례 축소 (scale 0.45) → 행 12~20px 축소 + 셀 진입 결함.
  - 정정: `resolve_cell_padding` 끝에 **한컴 자체 방어 로직 모방** 가드 추가 — pad_top + pad_bottom > cell.height 면 cell.height 의 절반까지 비례 축소. `measure_table_impl` 1-b단계도 동일 안전망.
  - 작업지시자 통찰: *"이런 경우 한컴은 자체 방어로직으로 처리한다면?"* — Task #347 가드 (KTX 목차 R=1417 HU 정합) 보존 + 한컴 동작 모방 가드 추가
  - 트러블슈팅 + 위키 ([HWP 셀 Padding 방어 로직](https://github.com/edwardkim/rhwp/wiki/HWP-%EC%85%80-Padding-%EB%B0%A9%EC%96%B4-%EB%A1%9C%EC%A7%81)) 작성

### 외부 PR cherry-pick (3 건 / 17 commits)

- **PR #428 그룹 내 그림(Picture) 직렬화 구현** (by [@oksure](https://github.com/oksure))
  - `serialize_group_child` 의 `ShapeObject::Picture` 분기 (빈 TODO) 구현 — 그룹 내 그림이 포함된 HWP 저장 시 그림 데이터 유실 결함 정정
  - SHAPE_COMPONENT + SHAPE_COMPONENT_PICTURE 레코드 추가 (Chart/OLE 자식과 동일 패턴) + 매직 상수 정리 (`tags::SHAPE_PICTURE_ID`)

- **PR #494 Paragraph::utf16_pos_to_char_idx 외부 노출** (#484, by [@DanMeon](https://github.com/DanMeon))
  - PR #405 와 같은 결의 외부 binding 작업 — `helpers::utf16_pos_to_char_idx` (pub(crate)) 의 알고리즘을 `Paragraph::utf16_pos_to_char_idx(&self, utf16_pos: u32) -> usize` (pub) 으로 캡슐화
  - 단위 테스트 6건 추가, semver MINOR 영역 (+1 method, 알고리즘 변경 없음)

- **PR #478 Layout 정합 + 수식 정정 합본** (by [@planet6897](https://github.com/planet6897))
  - 9 Task / 97 commits 누적 PR 중 페이지 레이아웃 무관 영역 **7 Task / 10 commits** 분리 cherry-pick (5 단계 머지)
  - **#488** (수식 토크나이저 폰트 스타일 prefix 분리 + svg/canvas 렌더러 italic honor) — 단위 테스트 14건 추가
  - **#490** (빈 텍스트 + TAC 수식 셀 alignment 적용) — exam_science p1 셀 7/11 28/36 수식 중앙 정렬
  - **#483** (각주 multi-paragraph line_spacing 정합 + trailing line_spacing follow-up)
  - **#489** (Picture+Square wrap 호스트 paragraph 텍스트 LINE_SEG cs/sw 적용)
  - **#495** (셀 paragraph 인라인 Shape 분기 가드 — 부분 정정, 잔존 [이슈 #502](https://github.com/edwardkim/rhwp/issues/502) 분리)
  - **#480** (wrap=Square 표 paragraph margin x 좌표 반영)
  - **#476** (PartialParagraph 인라인 Shape 페이지 라우팅, +881/-4)
  - 미흡수: #479 (paragraph trailing line_spacing/HWP vpos) — 한컴 2020 정답지 시각 판정 필수, [이슈 #503](https://github.com/edwardkim/rhwp/issues/503) 분리

### 회귀 검증 인프라 (외부 기여)

- **PR #498 Canvas visual diff 파이프라인** (relates #364, by [@seo-rii](https://github.com/seo-rii))
  - PR #456 (P2 PageLayerTree replay 전환) 후속 P3 검증 레이어 — rhwp-studio E2E 에 legacy Canvas vs PageLayerTree replay Canvas **픽셀 diff 자동 검증** + GitHub Actions Render Diff workflow 추가
  - 7 commits 분리 (test + diagnostics + docs + CI runner + 보안 hardening 3건)
  - 변경 영역: JS E2E + CI workflow + 문서 + Vite 설정 (Rust 변경 0)
  - 기본 fixture 3개 (KTX/biz_plan/tac-case-001) 모두 0 diff 확인

### 분리된 후속 이슈

- [#502](https://github.com/edwardkim/rhwp/issues/502) — 문단 내 글상자 TextRun 처리 (Task #495 잔존 결함)
- [#503](https://github.com/edwardkim/rhwp/issues/503) — Task #479 본질 정정 흡수 (한컴 2020 시각 판정 필수)

### 위키 정합

- [HWP 셀 Padding 방어 로직](https://github.com/edwardkim/rhwp/wiki/HWP-%EC%85%80-Padding-%EB%B0%A9%EC%96%B4-%EB%A1%9C%EC%A7%81) 신규
- [한컴 PDF 환경 의존성](https://github.com/edwardkim/rhwp/wiki/%ED%95%9C%EC%BB%B4-PDF-%ED%99%98%EA%B2%BD-%EC%9D%98%EC%A1%B4%EC%84%B1) 정황 IV 추가 (21_언어_기출 한컴 2020 = rhwp 정답)

### 검증

- cargo test --lib 1086 → **1102 passed**
- cargo test --test svg_snapshot 6/6, issue_418 1/1, issue_501 PASS (신규 통합 테스트)
- cargo clippy --lib -- -D warnings 0건
- WASM 4,206,487 bytes (Task #501) → 4,202,430 bytes (PR #478 5차 후) → 4,211,280 bytes (4차 #480 후)

## [0.7.8] — 2026-04-29

> v0.7.7 후속 사이클 — 외부 컨트리뷰터 다수 + 메인테이너 회귀 정정 + 위키/README 정비

### 외부 PR cherry-pick (15 건)

라이브러리 본질 정정 (조판 / 페이지네이션 / 직렬화):

- **PR #391 다단 섹션 누적 공식 회귀 정정** (#391, by [@planet6897](https://github.com/planet6897))
  - `src/renderer/typeset.rs` 누적 공식을 `col_count` 로 분기 — 단단 → `total_height`, 다단 → `height_for_fit` (trailing_ls 인플레이션 차단)
  - exam_eng (2단): 11 → **8 페이지**, 단독 1-item 단 (p3/p5/p7) 해소

- **PR #396 수식 렌더링 개선** (#174, #175, by [@oksure](https://github.com/oksure))
  - 인라인 수식 높이를 `eq.common.height` (HWP 권위값) 기준으로 설정 + X/Y 스케일링 동시 적용
  - 수식 내 CJK 문자 이탤릭 / 너비 보정 비적용 — CASES 한글 행 겹침 정정
  - 메인테이너 후속 정정 3건 (Canvas 분수선 y / Equation 스케일 / Limit fi=fs)

- **PR #395 그림 밝기/대비 효과 SVG 반영** (#150, by [@oksure](https://github.com/oksure))

- **PR #397 수식 ATOP 파싱 및 렌더링 보정** (by [@cskwork](https://github.com/cskwork))
  - **본 저장소 첫 외부 컨트리뷰터** — `EqNode::Atop` AST 파싱 + 분수선 없는 위/아래 배치 (HWP 의 ATOP / OVER 의미 분리)

- **PR #400 HWPX 수식 직렬화 보존** (#286, by [@cskwork](https://github.com/cskwork))
  - `render_paragraph_parts` 의 controls 무시 정황 정정 + parser XML entity 복원
  - 한컴 한글 2020 정상 열람 + PDF 일치 확인 (한컴 origin hwp 라운드트립 회귀 commit `ecd7d9a` 추가)

- **PR #401 v2 표 페이지 분할 rowspan>1 셀 분할 단위** (#398, by [@planet6897](https://github.com/planet6897))
  - `BLOCK_UNIT_MAX_ROWS=3` 임계 — 작은 블록 (≤3 행) 만 보호, 큰 rowspan (≥4 행) 행 단위 분할 허용 (HanCom 호환)
  - synam-001.hwp 5페이지 회귀 정정 (35→37→**35** 페이지)

- **PR #406 동일 문단 inline TAC 그림 페이지네이션 정정** (#402, by [@planet6897](https://github.com/planet6897))
  - 같은 paragraph 의 두 번째 inline 그림이 첫 번째와 같은 y 좌표에 그려져 겹침/오버플로 발생하던 문제 정정
  - 27→30 페이지 (분할 정상화)

- **PR #408 heading-orphan vpos 기반 보정** (#404, by [@planet6897](https://github.com/planet6897))
  - vpos 기반 5 조건 AND trigger (current fit + vpos overflow + next substantial + next 못 fit + single column non-wrap) — vpos overflow 41건 중 1건만 진짜 orphan
  - 9쪽 pi=83 헤딩 → 10쪽으로 push, 후속 표와 함께 배치

- **PR #410 TopAndBottom Picture vert=Para chart 정정 + atomic TAC top-fit** (#409, by [@planet6897](https://github.com/planet6897))
  - v1: `prev_has_overlay_shape` 가드 확장 (Picture + TopAndBottom + vert=Para)
  - v2: `typeset_section` controls 루프 chart 높이 누적
  - v3: `typeset_paragraph` atomic TAC top-fit 시멘틱 (60px tolerance)

- **PR #415 Task #352 dash 시퀀스 Justify 폭 부풀림 정정** (#352, by [@planet6897](https://github.com/planet6897))
  - dash leader elastic Justify 분배 (PDF 모방), exam_eng Q32 dash advance 12.11 → 7.06 px

- **PR #424 다단 우측 단 단행 문단 줄간격 누락 정정 (vpos 보정 anchor)** (#412, by [@planet6897](https://github.com/planet6897))
  - layout.rs vpos 보정 공식 정정 — `col_anchor_y` 도입 (body_wide_reserved 푸시 직후 anchor 보존), `curr_first_vpos` 우선 사용, page_path/lazy_path 분리
  - exam_eng p1 우측 단 item 7 ①~⑤ 15.33→**22.55px 균일**, 좌측 단 item 1 catch-up 28.56→21.89

- **PR #427 SvgRenderer defs 중복 방지 HashSet 통합** (#423, by [@oksure](https://github.com/oksure))
  - `arrow_marker_ids: HashSet<String>` → 범용 `defs_ids: HashSet<String>` 통합, O(n)→O(1)

- **PR #434 그림 자동 크롭 (FitToSize+crop) 공식 교정 + 테두리 inner padding** (#430, by [@planet6897](https://github.com/planet6897))
  - svg.rs / web_canvas.rs 의 crop 스케일 공식 정정 (`cr/img_w` → `original_size_hu/img_size_px`) + 헬퍼 `compute_image_crop_src` 추출 (SVG/Canvas 단일 진실 원천)
  - 별도 fix: 테두리 문단 inner padding (텍스트가 테두리에 붙는 문제)

API 추가 / 도구:

- **PR #405 `Paragraph::control_text_positions` 추가** (#390, by [@DanMeon](https://github.com/DanMeon))
  - 외부 binding 노출용 API 리팩토링

- **PR #411 `editor.exportHwp()` API 추가** (by [@ggoban](https://github.com/ggoban))
  - 신규 컨트리뷰터 첫 PR — iframe wrapper `@rhwp/editor` 에 exportHwp() 노출

- **PR #413 rhwp-studio PWA support** (#383, by [@dyjung150605](https://github.com/dyjung150605))
  - 신규 컨트리뷰터 첫 PR — vite-plugin-pwa, manifest scope `/rhwp/`, icon 192/512/maskable, registerType=autoUpdate, WASM precache

- **PR #419 PageLayerTree generation API 도입** (#364, by [@seo-rii](https://github.com/seo-rii))
  - `paint` 모듈 신규 (2,376 lines, builder/json/layer_tree/paint_op) — PageRenderTree → PageLayerTree 변환
  - opt-in transition adapter (`svg_layer.rs`, `RHWP_RENDER_PATH=layer-svg`)
  - 기존 5 렌더러 파일 변경 0 라인, 광범위 309 페이지 SVG 100% byte 동일 (피델리티 분석 보고서)

### 메인테이너 작업 (3 건)

- **Task #394 셀 진입 시 투명선 자동 ON 로직 비활성화** (#394)
  - input-handler.ts 5 영역 주석 처리 — 한컴 출력 정합

- **Task #416 `find_bin_data` 가드 결함 정정** (#416)
  - `c.id == bin_data_id` 가드 제거 — `c.id` 는 storage_id, bin_data_id 는 인덱스. sparse id 범위 분기 (HWPX 차트 60000+N 보존). 단위 테스트 7건 추가

- **Task #418 `hwpspec.hwp` p20 빈 문단 + TAC Picture 이중 emit 정정** (#418)
  - Task #376 정정 commit 이 devel 미머지 (close 됐지만 임시 브랜치에만 존재) → 동일 결함 재발
  - paragraph_layout 의 set_inline_shape_position + layout.rs::layout_shape_item 의 already_registered 가드 추가
  - 신규 메모리 (close 시 commit devel 머지 검증) + 트러블슈팅 신설

### 정비 / 문서

- **위키 페이지 [한컴 PDF 환경 의존성](https://github.com/edwardkim/rhwp/wiki/한컴-PDF-환경-의존성) 보강**
  - "발견 정황 II (PR #434 / 이슈 #430)" 섹션 추가 — 한컴 2010 ↔ 2020 ↔ 한컴독스 가 같은 hwp 를 다르게 조판하는 정황. 단일 한컴 정답지 가정의 한계 재확인.
  - rhwp 의 현재 출력이 시험지 조판자 의도에 더 부합 가능성 (원본 JPEG "(A 형)" 잔재 보존)

- **README.md / README_EN.md 보강**
  - Contributing 섹션에 "한컴 PDF 는 정답지가 아닙니다" 항목 추가
  - 신규 "위키 자료 (Wiki Resources)" 서브섹션 (위키 9개 페이지 링크)

- **samples 정답지 자료 추가** — 모든 컨트리뷰터와 fork 사용자 공유
  - `samples/2010-exam_kor.pdf` (한컴 2010, 4.57 MB)
  - `samples/2020-exam_kor.pdf` (한컴 2020, 4.57 MB)
  - `samples/hancomdocs-exam_kor.pdf` (한컴독스, 6.05 MB)
  - `samples/복학원서.pdf` (이슈 #421 한컴 정답지)
  - `samples/synam-001.hwp` (PR #401 회귀 검증)
  - `samples/atop-equation-01.hwp` (PR #397 시각 판정)

### 검증

- `cargo test --lib`: **1066 passed** (1008 → +58, 회귀 0건)
- `cargo test --test svg_snapshot`: 6/6 passed
- `cargo test --test issue_418`: 1/1 passed (Task #418 회귀 보존)
- `cargo clippy --lib -- -D warnings`: 0건
- WASM 빌드: 4,182,395 bytes (변동 +47 KB)
- 광범위 byte 비교: 10 샘플 / 309 페이지 SVG 회귀 검증 (PR 별 검증 게이트)
- 작업지시자 SVG + Canvas 양 경로 시각 판정 (PR #401 v2 / #406 / #408 / #410 / #415 / #424 / #434)

### 외부 기여자 감사

본 사이클 외부 기여자 (가나다순):
[@cskwork](https://github.com/cskwork), [@DanMeon](https://github.com/DanMeon), [@dyjung150605](https://github.com/dyjung150605), [@ggoban](https://github.com/ggoban), [@oksure](https://github.com/oksure), [@planet6897](https://github.com/planet6897), [@seo-rii](https://github.com/seo-rii)

특히 [@cskwork](https://github.com/cskwork) 님은 **본 저장소 첫 외부 컨트리뷰터** 로 PR #397 / #400 두 건을 머지하셨고, [@planet6897](https://github.com/planet6897) 님은 본 사이클 외부 PR 의 다수 (8 건) 를 진단 + 정정해주셨습니다.

## [0.7.7] — 2026-04-27

> v0.7.6 회귀 정정 사이클 (TypesetEngine default 전환 후 누락된 시멘틱 복원)

### 수정 — TypesetEngine 회귀 정정

- **페이지네이션 fit 누적 drift 수정** (#359)
  - typeset 의 fit 판정과 누적을 분리: fit 은 `height_for_fit` (trailing_ls 제외), 누적은 `total_height` (full)
  - 단독 항목 페이지 차단 가드 추가 — 다음 pi 의 vpos-reset 가드가 발동될 예정인 경우 빈 paragraph skip / 안전마진 1회 비활성화
  - **k-water-rfp**: LAYOUT_OVERFLOW 73 → 0 (drift 311px 정정)
  - **kps-ai**: 60 → 4

- **TypesetEngine page_num + PartialTable fit 안전마진** (#361)
  - `finalize_pages` 의 NewNumber 적용 조건을 Paginator 시멘틱과 동일하게 정정 (`prev_page_last_para` 추적, 한 페이지에서 한 번만 적용)
  - PartialTable 직후 fit 안전마진 (10px) 비활성화 — PartialTable 의 cur_h 는 row 단위로 정확
  - **k-water-rfp**: 28 → 27 페이지 (page_num 정상 갱신)
  - **kps-ai**: page_num 1, 2, 1, 1, 2~8 정상 (NewNumber 컨트롤 처리)

- **kps-ai PartialTable + Square wrap 처리** (#362, 8 항목 누적)
  - **wrap-around 메커니즘 (Square wrap) 이식** ★ — Paginator engine.rs:288-372 의 wrap zone 매칭 + 활성화 시멘틱을 TypesetEngine 에 이식. 외부 표 옆 paragraph 들이 height 소비 없이 흡수됨
  - 외부 셀 vpos 가드 — nested table 셀에서 LineSeg.vertical_pos 적용 제외 (p56 클립 차단)
  - PartialTable nested 분할 허용 — 한 페이지보다 큰 nested table atomic 미루기 대신 분할 표시 (p67 빈 페이지 차단)
  - PartialTable 잔여 height 정확 계산 — `calc_visible_content_height_from_ranges_with_offset` 신설
  - nested table 셀 capping 강화 (외부 행 높이로 cap)
  - hide_empty_line TypesetEngine 추가 (페이지 시작 빈 줄 최대 2개 height=0)
  - vpos-reset 가드 wrap zone 안에서 무시 (오발동 차단)
  - 빈 paragraph skip 가드 강화 — 표/도형 컨트롤 보유 paragraph 는 skip 안 함 (pi=778 표 누락 차단)
  - **kps-ai**: 88 → 79 페이지 (Paginator 78 와 일치, LAYOUT_OVERFLOW 60→5)

### 보안

- **rhwp-firefox/build.mjs CodeQL Alert #17 해소** (#354)
  - `execSync` shell 사용 → `execFileSync` (`shell: false`) 로 전환

### 검증

- `cargo test --lib`: 1008 passed, 0 failed
- `cargo test --test svg_snapshot`: 6/6
- `cargo test --test issue_301`: 1/1
- WASM 빌드 통과
- 작업지시자 시각 판정 통과 (kps-ai p56, p67-70, p72-73, k-water-rfp 전체)

## [0.7.6] — 2026-04-26

> 외부 기여자 다수 + 조판 정밀화 사이클

### 추가
- **`replaceOne(query, newText, caseSensitive)` WASM API** (#268)
  — 분석·구현 by [@oksure](https://github.com/oksure) (신규 기여자)
  - `replaceText` 의 위치 기반 시그니처 vs 검색어 기반 호출 mismatch crash 해결
  - 새 API 추가로 하위 호환성 100% 보존
  - 5 unit tests (한국어 multi-byte 경계 포함)

- **SVG/HTML draw_image base64 임베딩** (#335)
  — 분석·구현 by [@oksure](https://github.com/oksure)
  - 기존 placeholder (`<rect>`/`<div>`) → 실제 이미지 base64 data URI 임베딩
  - `render_picture` / `web_canvas` 와의 backend 정합

### 수정
- **목차 리더 도트 + 페이지번호 우측 탭 정렬** (#279)
  — 분석·구현 by [@seanshin](https://github.com/seanshin)
  - `fill_type=3` 점선 리더를 round cap 원형 점으로 표현 (한컴 동등)
  - `find_next_tab_stop` RIGHT 탭 클램핑 제외 — 들여쓰기 문단의 페이지번호 정렬 보정
  - 메인테이너 추가 보강: 셀 padding 인지 leader 시멘틱, 페이지번호 폭별 leader 길이 차등화, 공백 only run carry-over

- **form-002 인너 표 페이지 분할 결함** (#324)
  — 분석·구현 by [@planet6897](https://github.com/planet6897)
  - `compute_cell_line_ranges` 잔량 추적 → 누적위치 (`cum`) 기반 재작성
  - `layout_partial_table` 의 `content_y_accum` 갱신 + split-start row 통일된 계산
  - 작성자 자체 v1 → v2 → v3 보강

- **typeset 경로 PageHide / Shape / 중복 emit 결함** (#340)
  — 분석·구현 by [@planet6897](https://github.com/planet6897)
  - 세 결함을 typeset.rs 의 누락이라는 공통 원인으로 통합 진단
  - `engine.rs` 와의 정합 (PageHide 수집 + `pre_text_exists` 가드 + Shape 인라인 등록)

- **Firefox AMO 워닝 해결 (rhwp-firefox 0.2.1 → 0.2.2)** (#338)
  — 분석·구현 by [@postmelee](https://github.com/postmelee)
  - manifest `strict_min_version` 142 상향 (`data_collection_permissions` 호환)
  - `viewer-*.js` 의 unsafe `innerHTML` / `Function` / `document.write` sanitize
  - rhwp-studio 28 파일 DOM/SVG API 교체 + Reviewer Notes 한/영

- **Task #321~#332 누적 정리 + vpos/cell padding 회귀 해소** (#342)
  — 분석·구현 by [@planet6897](https://github.com/planet6897)
  - vpos correction 양방향 가드 + cell padding aim 명시값 우선 정책
  - typeset/layout drift 정합화 + 메인테이너 검토 응답으로 KTX TOC 결과 (#279) 복원

### 기타
- **신규 기여자 환영 안내** — README.md / README_EN.md Contributing 섹션에 PR base=devel 명시 (#330 close 후 후속 개선)

## [0.6.0] — 2026-04-04

> 조판 품질 개선 + 비기능성 기반 구축 — "알을 깨고 세상으로"

### 추가
- **GitHub Actions CI**: 빌드 + 테스트 + Clippy 엄격 모드 (#46, #47)
- **GitHub Pages 데모**: https://edwardkim.github.io/rhwp/ (#48)
- **GitHub Sponsors**: 후원 버튼 활성화
- **그림 자르기(crop)**: SVG viewBox / Canvas drawImage로 이미지 crop 렌더링 (#43)
- **이미지 테두리선**: Picture border_attr 파싱 + 외곽선 렌더링 (#43)
- **머리말/꼬리말 Picture**: non-TAC 그림 절대 위치 배치, TAC 그림 인라인 배치 (#42)
- **로고 에셋 관리**: assets/logo/ 기준 원본 관리, favicon 생성
- **비기능성 작업 계획서**: 6개 영역 13개 항목 3단계 마일스톤 (#45)

### 수정
- **같은 문단 TAC+블록 표**: 중간 TAC vpos gap 음수 역행 방지 (#41)
- **분할 표 셀 세로 정렬**: 분할 행에서 Top 강제, 중첩 표 높이 반영 (#44)
- **TAC 표 trailing ls**: 경계 조건 순환 오류 해결 (#40)
- **통화 기호 렌더링**: ₩€£¥ Canvas 맑은고딕 폴백, SVG 폰트 체인 (#39)
- **반각/전각 폭 정밀화**: Bold 폴백 보정 제거, 스마트 따옴표/가운뎃점 반각 (#38)
- **폰트 이름 JSON 이스케이프**: 백슬래시 포함 폰트명 로드 실패 수정 (#37)
- **머리말 표 셀 이미지**: bin_data_content 전달 경로 수정 (#36)
- **Clippy 경고 제거**: unnecessary_unwrap, identity_op 등 6건 수정 (#47)

## [0.5.0] — 2026-03-29

> 뼈대 완성 — 역공학 기반 HWP 파서/렌더러

### 핵심 기능
- **HWP 5.0 / HWPX 파서**: OLE2 바이너리 + Open XML 포맷 지원
- **렌더링 엔진**: 문단, 표, 수식, 이미지, 차트, 머리말/꼬리말/바탕쪽/각주
- **페이지네이션**: 다단 분할, 표 행 단위 분할, shape_reserved 처리
- **SVG 내보내기**: CLI (`rhwp export-svg`)
- **Canvas 렌더링**: WASM/Web 기반
- **웹 에디터**: rhwp-studio (텍스트 편집, 서식, 표 생성)
- **hwpctl 호환 API**: 30 Actions, Field API (한컴 웹기안기 호환)
- **VS Code 확장**: HWP/HWPX 뷰어 (v0.5.0~v0.5.4)
- **755+ 테스트**

### 조판 엔진
- 줄간격 (고정값/비율/글자에따라), 문단 여백, 탭 정지
- 표 셀 병합, 테두리 스타일, 셀 수식 계산
- 다단 레이아웃, 문단 번호/글머리표
- 세로쓰기, 개체 배치 (자리차지/글자처럼/글앞/글뒤)
- 인라인 TAC 표/그림/수식 렌더링

### 수식 엔진
- 분수(OVER), 제곱근(SQRT/ROOT), 첨자
- 행렬: MATRIX, PMATRIX, BMATRIX, DMATRIX
- 경우(CASES), 정렬(EQALIGN), 적분/합/곱 연산자
- 15종 텍스트 장식, 그리스 문자, 100+ 수학 기호

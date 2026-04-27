# Changelog

이 프로젝트의 주요 변경 사항을 기록합니다.

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

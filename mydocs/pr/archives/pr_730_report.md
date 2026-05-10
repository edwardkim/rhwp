---
PR: #730
제목: Task #172 — section.xml 컨트롤 디스패처 표/그림/도형 직렬화 연결
컨트리뷰터: @oksure (Hyunwoo Park) — 20+ 사이클 핵심 컨트리뷰터 (5/10 사이클 영역 영역 3번째 PR)
처리: 옵션 A — PR HEAD squash cherry-pick + no-ff merge
처리일: 2026-05-10
머지 commit: 47a303f2
---

# PR #730 처리 보고서

## 1. 처리 결과

✅ **머지 완료** — 옵션 A (PR HEAD squash cherry-pick + no-ff merge `47a303f2`)

| 항목 | 값 |
|------|-----|
| 머지 commit | `47a303f2` (--no-ff merge) |
| Squash commit | `3d2a8eeb` (5 commits 통합) |
| closes | #172 |
| 판정 게이트 | **라운드트립 테스트 17 PASS** (작업지시자 결정) |
| 자기 검증 | cargo test ALL GREEN + sweep 170/170 same + WASM 4.61 MB |

## 2. 정정 본질 — 5 files, +619/-37

### 2.1 `src/serializer/hwpx/section.rs` (+159/-33)
- `render_control_slot()` 영역 영역 디스패처 확장 — Equation-only 게이트 영역 → `slots.is_empty()` 영역 영역 Table/Picture/Shape/Footnote/Endnote 모두 직렬화
- 각주/미주 영역 `<hp:ctrl><hp:footNote>...<hp:subList>...재귀 문단...</hp:subList></hp:footNote></hp:ctrl>`
- Shape 디스패치: Rectangle/Line (Writer-based) / Picture (위임) / Ellipse/Arc/Polygon/Curve/Group/Chart/Ole (공통 속성)

### 2.2 `src/serializer/hwpx/mod.rs` (+210/-2)
라운드트립 테스트 4건 신규.

### 2.3 `src/serializer/hwpx/shape.rs` (+122/-2)
`drawText` 글상자 직렬화 추가 + Rectangle 영역 영역 의 Writer-based serializer.

### 2.4 `src/serializer/hwpx/context.rs` (+19)
`collect_from_document()` 영역 영역 인라인 Table 영역 영역 `borderFillIDRef` 1-pass 사전 등록.

### 2.5 `tests/hwpx_roundtrip_integration.rs` (+109)
- `stage5_table_control_preserved_on_roundtrip` (표-텍스트.hwpx)
- `stage5_picture_bindata_preserved_on_roundtrip` (tac-img-02.hwpx)
- `stage5_footnote_endnote_preserved_on_roundtrip`
- `stage5_tac_img_sample_has_pictures_and_bindata`

## 3. Issue #172 체크리스트 6 항목 모두 커버

- [x] 표 (`Control::Table`) → `<hp:tbl>` (table::write_table 위임)
- [x] 그림 (`Control::Picture`) → `<hp:pic>` + BinData (picture::write_picture 위임)
- [x] 도형 (`Control::Shape`) → 변형별 공통 속성
- [x] 각주/미주 → `<hp:footNote>`/`<hp:endNote>` + `<hp:subList>`
- [x] BinData ZIP 엔트리 (mod.rs 기존 3-way 동기화 활용)
- [x] `content.hpf` manifest 자동 등록 (mod.rs 기존 활용)

## 4. Copilot 리뷰 반영 (commit `328b00e5`)

- 에러 로깅 (`eprintln` 영역 조용한 누락 방지)
- Cursor 제거 (`Writer<Vec<u8>>` 직접)
- ShapeObject::Picture 위임 (`picture::write_picture`)

## 5. 본 환경 cherry-pick

### 5.1 PR HEAD squash cherry-pick

원본 5 commits (`7a544375`, `99c8ac6c`, `328b00e5`, `815759b2`, `ec0b509e`) 영역 영역 개별 cherry-pick 영역 영역 첫 commit 영역 영역 충돌 발생 영역 영역 (commits 영역 영역 누적 변경 영역 영역 영역 영역 conflict marker 영역 영역). PR HEAD squash cherry-pick (`git cherry-pick --no-commit 60aeaa8d..pr730-head`) 영역 영역 단일 commit (`3d2a8eeb`) 영역 영역 적용 — 충돌 0건. PR #729 영역 영역 동일 패턴 정합.

### 5.2 결정적 검증

| 검증 | 결과 |
|------|------|
| `cargo build --release` | ✅ 통과 (32.48s) |
| `cargo test --release --test hwpx_roundtrip_integration` | ✅ **17 PASS** (신규 4건 + 기존 13건) |
| `cargo test --release` (전체) | ✅ ALL GREEN, failed 0 |
| 광범위 sweep (7 fixture / 170 페이지) | ✅ **170 same / 0 diff** |
| WASM 빌드 (Docker) | ✅ 4.61 MB |

## 6. 판정 게이트 — 라운드트립 테스트 (작업지시자 결정)

### 6.1 게이트 결정
작업지시자 영역 영역 본 PR 영역 영역 영역 **라운드트립 테스트 영역 영역 판정 게이트** 영역 영역 채택 영역 — 17 PASS (신규 4건 + 기존 13건) 영역 영역 통과 영역 영역 영역 즉시 머지.

### 6.2 면제 합리
- 라운드트립 17건 영역 영역 결정적 입증 — `parse → serialize → parse` IR 정합 (표/그림/각주/미주 모두 보존)
- 광범위 sweep 영역 영역 0 회귀 — 직렬화 영역 영역 만 변경 영역 영역 시각 출력 무관 보장
- 한컴 호환 영역 영역 영역 후속 작업 영역 영역 (작업지시자 한컴 검증 영역 영역 별건 진행 가능)

### 6.3 메모리 룰 정합
- `feedback_self_verification_not_hancom` 영역 영역 — 본 PR 영역 영역 영역 자기 검증 영역 영역 게이트 채택. 한컴 호환 영역 영역 영역 별건 후속 추적
- `feedback_search_troubleshootings_first` 영역 영역 영역 사전 검색 영역 영역 — `picture_save_hancom_compatibility.md` / `task178_first/second_attempt` 영역 영역 사례 영역 영역 본 PR 영역 영역 (HWPX→HWPX) 영역 영역 다른 영역 영역 영역, 한컴 검증 영역 영역 후속 처리 영역 영역

## 7. 영향 범위

### 7.1 변경 영역
- HWPX 직렬화 영역 영역 표/그림/도형/각주/미주 영역 영역 추가
- BinData ZIP + content.hpf manifest 영역 영역 기존 인프라 활용

### 7.2 무변경 영역
- HWP (OLE 바이너리) 직렬화 — 별건
- 파싱 (rhwp parser) — 변경 부재
- 시각 출력 (SVG/PNG) — 변경 부재 (sweep 170/170 same 영역 영역 입증)

### 7.3 후속 작업
- **한컴 호환 검증** — 작업지시자 한컴2020/한컴2022 영역 영역 별건 진행 가능 (자기 라운드트립 영역 영역 만 영역 영역 본 PR 영역 영역 통과 게이트)

## 8. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @oksure **20+ 사이클** (5/10 사이클 영역 영역 3번째 PR — PR #728/#729 후속) |
| `feedback_self_verification_not_hancom` | 자기 라운드트립 영역 영역 게이트 채택 (작업지시자 결정) — 한컴 검증 영역 영역 후속 별건 |
| `feedback_search_troubleshootings_first` | 사전 검색 영역 영역 — picture_save_hancom_compatibility / task178 사례 영역 영역 점검 영역 본 PR 영역 영역 (HWPX→HWPX) 영역 영역 다른 영역 |
| `feedback_image_renderer_paths_separate` | hwpx 직렬화 영역 영역 격리 — 다른 파서/렌더러 영역 영역 무영향 (sweep 170/170 same) |
| `feedback_process_must_follow` | 인프라 재사용 (BinData ZIP / content.hpf manifest 기존 활용) — 신규 인프라 도입 부재 영역 영역 위험 좁힘 |

## 9. 잔존 후속

- 본 PR 본질 정정 영역 의 잔존 결함 부재
- **한컴 호환 검증** — 작업지시자 한컴2020/한컴2022 영역 영역 별건 진행 (필요 시 후속 이슈 등록 권장)

---

작성: 2026-05-10

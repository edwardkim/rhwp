# 구현 계획서 — Task #1224: 폰트 충실도 (한컴 돋움 폴백 글리프 과대·과굵)

- **이슈**: #1224 (M100 / v1.0.0)
- **브랜치**: `feature/issue-1224-font-fidelity` (base: `stream/devel` 4f5a8e22)
- **수행계획서**: `plans/task_m100_1224.md`
- **작성일**: 2026-06-01

## 설계 요지

`svg.rs::draw_text` 는 클러스터별 `<text>` 를 메트릭-DB advance 위치에 방출하고
`textLength` 로 가로폭까지 제약한다. **미제어 차원은 세로 em-fill 하나뿐**이며 이는
폴백 폰트가 결정한다. 두 경로를 손본다:

1. **체인** (`renderer/mod.rs::generic_fallback`): 시스템에 대체폰트가 있으면 Noto 보다
   먼저 매칭되도록 우선순위 삽입.
2. **임베딩** (`svg.rs` `find_font_file`/`known_font_filenames`/`generate_font_style`):
   문서가 돋움/고딕 계열을 쓰고 저작권 TTF 부재 시, **오픈소스 대체 TTF 를 서브셋 임베딩**
   → rsvg/오프라인에서도 결정적 충실도.

레이아웃·메트릭 DB·자간·줄나눔·페이지수는 **전 단계 불변**.

## 단계 구성 (4단계)

### Stage 1 — 후보 폰트 실측 및 대체폰트 선정 (코드 무변경, 보고서)

**목표**: em-fill 이 한컴 돋움(PDF 0.67)에 최근접한 오픈소스 sans 폰트 1종 선정.

- 후보 확보(OFL/무료 상업허용): Spoqa Han Sans Neo, Gowun Dodum, IBM Plex Sans KR,
  Pretendard(기존 번들), Noto Sans KR(현 폴백·비교 기준).
- 측정: 동일 em 으로 대표 한글 음절(가/한/국/문 등) 렌더 후 **잉크 박스 높이 ÷ em** 산출.
  - 1차: `pyftsubset`/`fonttools` 또는 `hb-view`/freetype 로 글리프 bbox 추출.
  - 2차(교차검증): 각 폰트로 동일 SVG 렌더 → rsvg PNG 픽셀에서 잉크 높이 측정.
- 한컴 돋움 기준값은 `pdf/3-09월_교육_통합_2022.pdf` 6쪽 본문에서 측정.
- 라이선스·한글 11,172 커버리지·굵기 축(Regular/Bold) 확인.
- **산출**: `tech/font_emfill_comparison_1224.md` (폰트별 em-fill 표 + 선정 근거).
- **승인 게이트**: 선정 폰트 확정 후 다음 단계 진행.

### Stage 2 — 폴백 체인 우선순위 조정 (`renderer/mod.rs`)

**목표**: 선정 폰트를 sans 체인에서 `Noto Sans KR` **앞**에 삽입.

- `generic_fallback` 의 두 sans 반환 지점(빈 family 분기 L672, 일반 sans 분기 L710)에
  `'<선정폰트>'` 를 `'Noto Sans KR'` 직전에 추가.
- 돋움/고딕·맑은 고딕 키워드 분기 내로 한정 → serif/mono/PUA(함초롬) 매칭 무영향.
- `test_generic_fallback` 의 sans 기대 문자열 갱신 + 선정폰트 포함 단언 추가.
- **검증**: `cargo test renderer::tests::test_generic_fallback` 통과, 체인 문자열 육안 확인.

### Stage 3 — 임베딩 경로에 대체폰트 반영 (`svg.rs` + 폰트 자산)

**목표**: 문서가 돋움/고딕(한컴돋움·돋움·굴림·맑은 고딕·함초롬돋움 등)을 쓰고 저작권
TTF 부재 시 **선정 오픈소스 TTF 를 서브셋 임베딩**.

- 선정 OFL TTF(Regular/Bold)를 저장소 추적 폴더에 배치(라이선스 동봉). 위치/명명은
  Stage 1 선정 결과로 확정(후보: `ttfs/opensource/` 추적 또는 `web/fonts` 인접).
- `known_font_filenames` 의 돋움/고딕 family 항목에 선정 TTF 파일명을 **저작권 TTF 다음
  후보**로 추가 → 저작권 폰트 있으면 그대로, 없으면 오픈소스로 서브셋.
- `find_font_file` 탐색 경로에 배치 폴더 추가(필요 시).
- `--embed-fonts`/`--font-style` 동작 확인: 서브셋 크기·글자수 로그, @font-face 가
  원본 family 명으로 대체 글리프 참조.
- **검증**: `export-svg samples/3-09월_교육_통합_2023.hwp -p 5 --embed-fonts` →
  서브셋 성공 로그 + 오프라인 PNG 렌더 정상.

### Stage 4 — 시각 검증·레이아웃 불변 증빙·문서화

**목표**: 충실도 달성 확인 + 회귀 없음 증빙 + 가이드 갱신.

- **시각 회귀**: 6쪽 문26 SVG↔PDF em-fill 측정, 0.67±0.05 목표(미달 시 한계 명시).
- **레이아웃 불변**: 변경 전후 `dump-pages -p 5` 문단 배치·높이·페이지수 동일 표로 증빙.
- **회귀 스위트**: `cargo test` 전체 통과.
- **다문서 표본**: 돋움/고딕 본문 포함 샘플 2~3종 추가 시각 확인.
- **문서화**: `tech/font_fallback_strategy.md` 에 "메트릭(advance) ↔ 렌더 글리프(대체폰트)
  시각 정합" 절 추가(em-fill 개념·선정 폰트·임베딩 동작).
- **산출**: `report/task_m100_1224_report.md`.

## 단계별 커밋 정책

각 Stage 완료 시 소스 + `working/task_m100_1224_stage{N}.md` 를 함께 커밋. Stage 4 에서
최종 보고서·문서 갱신 커밋. 기능/포맷 변경 분리, 무관 rustfmt diff 금지.

## 검증 도구 요약

| 항목 | 명령 |
|------|------|
| em-fill 측정 | `fonttools`/`hb-view` glyph bbox + rsvg PNG 픽셀 |
| 시각 비교 | `rhwp export-svg … -p 5 [--embed-fonts]` ↔ `pdf/…-2022.pdf` |
| 레이아웃 불변 | `rhwp dump-pages … -p 5` (전후 비교) |
| 회귀 | `cargo test` |

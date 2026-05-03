# all-in-one-parser 샘플 1:1 시각 정합화 전략

작성일: 2026-05-01
대상: `D:\PARA\Resource\all-in-one-parser\input\` (390개 HWP/HWPX)
목표: 본 샘플 전량을 rhwp 에디터(rhwp-studio)에서 한컴 한글(웹기안기 / 한글 2020) 대비 **시각적 1:1 정합** 으로 열람·편집 가능하도록 한다.
관련 로드맵: Phase 2 (5월 — GitHub 공개 준비, 코드 품질 9.2)

---

## 0. 본 문서의 위치

본 문서는 GitHub Issue 채번 **이전 단계의 전략 분석서** 이다. 작업지시자 승인 후 다음 절차를 따른다.

1. 본 전략 승인 → 마일스톤(M100/M0xx) + 마스터 Issue 생성 (예: `#XYZ all-in-one-parser 시각 정합화 (umbrella)`)
2. 산하 sub-issue 채번 (Phase 별 / 결함 클러스터 별)
3. `local/task{XYZ}` 브랜치 생성
4. `mydocs/plans/task_m{ver}_{XYZ}.md` 수행 계획서 → 승인
5. `mydocs/plans/task_m{ver}_{XYZ}_impl.md` 구현 계획서 (3~6 단계) → 승인
6. 단계별 진행 + `_stage{N}.md` + `_report.md`

소스 수정은 5단계 승인 후에만 시작한다.

---

## 1. 입력 자산 분석

### 1.1 파일 분포

| 카테고리 | HWP 수 | 위치 | 비고 |
|----------|--------|------|------|
| 공통수학1 (고1) | 12 | `input/공통수학1/` | 다항식·이차방정식·복소수·경우의수·행렬 |
| 미적분 (고2~3) | ≥10 | `input/미적분/` | 함수의 극한·연속·미분·정적분 |
| 확률과 통계 | 7 | `input/기출문제 확률과 통계-1/` | 순열·조합·이항·확률분포·통계추정 |
| 정렬 검증 셈플 | 1 | `input/test_alignment/절대등급_공통수학1_01다항식의연산.hwp` | 별도 정렬 검증용으로 추정 |
| HWPX 파생 | 약 360 | `input/미적분/output/hwpx/`, `input/공통수학1/output/hwpx/` 등 | all-in-one-parser 가 문항별로 분할 추출한 결과물 (`problem_NNN.hwpx`) |
| HWP 임시 처리물 | 8 | `web-server/temp_processing/*/input/` | 동일 원본의 사본 — 회귀 셈플로 활용 가능 |

**총합**: 원본 HWP ≈ 30개 + 분할 HWPX ≈ 360개 = **약 390 fixture**.

### 1.2 콘텐츠 특성 (rhwp 입장에서의 난이도)

본 샘플은 rhwp 가 그동안 처리해 온 일반 문서(KTX, biz_plan, k-water-rfp, mel-001 등)와 본질적으로 다른 **수학 시험 문제지** 이다.

| 특성 | rhwp 현황 영향 |
|------|----------------|
| **수식 컨트롤(EqEdit) 페이지당 5~30개** | `src/renderer/equation/*` 의존도 절대적. EQALIGN 미구현·BIGG 부분구현·DINT/TINT 미등록은 즉시 결함화 |
| **표 안에 수식 + 그림 혼재** | `layout_table` ↔ `layout_embedded_table` 200줄 중복 영역에서 셀 베이스라인 정렬 회귀 위험 (cf. #501 셀 padding) |
| **분수(OVER) 위·아래 문단 줄간격 변동** | 문단 ParaShape 의 `line_height=Percent` 경우 수식 타이트박스 baseline 정렬 정합 필요 |
| **그림 안에 수식 (이미지 내 캡션·그래프 좌표축)** | crop scale 75 HU/px 룰(#477) 적용 후 그래프 라벨 위치 1:1 점검 필수 |
| **본문 + 보기 박스 + 답안박스 (지문 박스)** | wrap=TopAndBottom 표 anchor (#445) + 박스 stroke_sig (#471) 영역 — 다단 cross-column 회귀 잠재 |
| **5지선다 라벨 ①②③④⑤** | 글머리 문단 탭 정렬(#340 미결) 직격 — 보기 정렬 어긋날 가능성 |
| **두 단(2-column) 페이지 레이아웃** | 미적분/공통수학 특히 다단 비중 큼. cross-column vpos-reset(#470/#473) 영역 |

→ **본 fixture 군은 rhwp 가 지금까지 회귀로 처리한 결함 클러스터(#445 / #470 / #471 / #477 / #495 / #501) 가 동시 다발적으로 발현될 가능성이 가장 높은 콘텐츠** 이다.

---

## 2. "1:1 정합" 정의

용어를 먼저 고정한다. 셋 중 어느 등급을 목표로 할지 작업지시자 결정이 필요하다.

| 등급 | 정의 | 측정 방법 | 본 전략의 권장치 |
|------|------|-----------|-------------------|
| **L1 — 의미적 정합** | 모든 텍스트·수식·표·그림이 누락 없이 올바른 순서로 표시 | E2E + dump-pages 모든 paragraph/table/shape 매칭 | 필수 (최저선) |
| **L2 — 구조적 정합** | 페이지 수, 페이지별 컨트롤 배치, 줄 분할이 한컴과 일치. 픽셀 ±5px 허용 | Hancom Office PDF export ↔ rhwp SVG 페이지별 IoU/SSIM ≥ 0.97 | **본 전략의 1차 목표** |
| **L3 — 픽셀 정합** | 폰트 힌팅·자간 포함 픽셀 레벨 일치 | per-page MSE ≤ ε (실측 후 결정) | 장기 목표 (본 전략 범위 외) |

L3 는 한컴 폰트(HCR/MD/한양 시리즈)의 비공개 힌팅 알고리즘 의존도가 높아 오픈소스 대안으로는 비현실적이다. **L2 까지를 본 전략의 KPI 로 제안**한다.

KPI 안:

- 390 fixture × 평균 N 페이지 의 95% 이상이 L2 합격
- 각 fixture 의 1쪽은 100% L2 합격 (시각 검증 우선 영역)
- L1 누락 (텍스트/수식/표 빠짐) 0건

---

## 3. 현재 갭 분석 (예측 결함 매트릭스)

코드 리뷰 보고서(`mydocs/eng/feedback/layout-quality-report.md`, 4.5/10) + 최근 30개 회귀 이슈 + 수식 지원 현황(`mydocs/tech/equation_support_status.md`) 기반 **사전 예측 결함**.

### 3.1 수식 영역 (가장 큰 위험)

| 결함 후보 | 예상 빈도 | 근거 | 우선순위 |
|------------|-----------|------|----------|
| EQALIGN 미구현 → 연립방정식·증명 단계가 한 줄 출력 | 🟥 매우 높음 | exam_math.hwp 빈도 "중간" 보고됨, 본 fixture 는 더 많을 듯 | P0 |
| BIGG 크기 배율 무시 → 큰 괄호 안 분수 잘림 | 🟧 중간 | "현재 파싱만 됨" | P1 |
| 적분 변형(DINT/TINT/OINT) 미등록 → 미적분 정적분 응용 문제 | 🟧 중간 | symbols.rs 매핑 누락 | P1 |
| REL/BUILDREL 미구현 → 화살표 위 조건식 문항 | 🟨 낮음 | 출제 빈도 가변 | P2 |
| 자동 로만체 적용 누락(예: lim 의 첨자 영역에서 sin 이 italic 나옴) | 🟧 중간 | 함수명 컨텍스트 검출 필요 | P1 |
| 행렬 cell padding (PMATRIX/BMATRIX) 한컴 대비 ±2px | 🟨 낮음 | 베이스라인 보정 미세 차이 | P2 |
| 분수 막대 두께·여백 (OVER 의 numerator/denominator 패딩) | 🟨 낮음 | 한컴 사양 비공개 영역 | P2 |

### 3.2 본문/표/박스 영역

| 결함 후보 | 예상 빈도 | 근거 |
|------------|-----------|------|
| ①②③④⑤ 보기 탭 정렬 — `#340` 미결 (글머리 문단 탭 정렬 폭) | 🟥 매우 높음 | 5지선다 모든 문항에 영향 |
| 지문 박스(wrap=TopAndBottom) 다단 anchor — `#445` 영역 | 🟧 중간 | 머리말/꼬리말 수준은 정정됨, 본문 내 변형 잔존 |
| 셀 padding 비정상 — `#501` 모방 가드 적용됐지만 본 fixture 셀 형상 미검증 | 🟧 중간 | 셀 padding > height 케이스 다른 패턴 가능 |
| RowBreak 표 분할 — `#474` 영역 | 🟨 낮음 | 시험지는 짧은 표 위주, 영향 적음 |
| 그림 crop scale — `#477` 75 HU/px 룰 적용됨 | 🟨 낮음 | 그래프 라벨 위치는 별도 검증 |
| 다단 cross-column vpos-reset — `#470/#471` | 🟧 중간 | 미적분 2단 페이지 다수 |
| 문단 내 글상자 TextRun 중복 — `#495/#502` | 🟨 낮음 | 부분 정정 진행 중 |

### 3.3 렌더 인프라 영역

| 결함 후보 | 영향 |
|----------|------|
| layout.rs 의 921줄 `build_render_tree()` — 신규 결함 진단 시 cognitive load 절벽 | 본 작업 디버깅 효율 30%+ 저하 가능 |
| ShapeObject 8-variant 매칭 61회 — 본 fixture 에 새로운 shape 변형 추가 시 다중 지점 수정 | 위험 |
| 표 layout 200줄 중복 — embedded table(셀 안 표) vs top-level table 정합 차이 | 셀 안 표 대량 검증 필요 |

---

## 4. 전략 (5 Phase)

```
Phase A: 기준선 수집(Baseline)
  ├─ 한컴 한글 2020 PDF export 자동화
  ├─ rhwp SVG export 자동화 (현 시점)
  └─ 페이지별 thumbnail + page count 매트릭스 생성

Phase B: 자동 diff 인프라
  ├─ Hancom PDF ↔ rhwp SVG 페이지별 SSIM 측정 도구
  ├─ Canvas visual diff (PR #498) 확장 — fixture 인풋만 교체
  └─ all-in-one-parser fixture 전용 GitHub Actions matrix

Phase C: 결함 클러스터링
  ├─ SSIM 하위 5% 페이지 자동 추출
  ├─ 결함 영역(글자/수식/표/그림) 분류 (휴리스틱: 차이 픽셀 위치 + control type 매칭)
  └─ 클러스터 → GitHub Issue 자동 생성

Phase D: 우선순위 정정
  ├─ P0: EQALIGN 구현 + ①②③④⑤ 탭 정렬(#340)
  ├─ P1: 수식 보완 5건 + 박스/anchor 잔존
  ├─ P2: 미세 정합 (자간·줄간격)
  └─ 각 정정마다 회귀 차단 (svg_snapshot + issue_NNN.rs)

Phase E: 회귀 가드 + CI 게이팅
  ├─ all-in-one-parser fixture 의 10% 를 svg_snapshot 등록
  ├─ Render Diff workflow 의 fixture 풀에 추가
  └─ devel push 시 자동 회귀 검출
```

각 Phase 의 산출물·검증·소요는 §5~§9 에서 상술.

---

## 5. Phase A — 기준선 수집

### 5.1 한컴 한글 2020 PDF 자동 export

**전제**: 한컴 한글 2020 (또는 동등본) 라이선스 보유 + Windows 환경. 현 작업 머신은 Windows 11 Pro 이므로 적합.

**도구 후보**:

1. **HWP COM Automation (`HwpAutomation.HwpCtrl`)** — 한컴 공식 OLE/COM. PDF 출력 가능. PowerShell 또는 Python `pywin32` 호출.
2. **CLI `hwp.exe /print:pdf`** — 일부 버전 지원 여부 확인 필요.
3. **수동 1회 일괄** — Phase A 1회만 수행 후 결과 PDF 를 fixture 와 함께 보관.

권장: **(1) COM Automation 으로 1회 일괄 변환** 후 결과를 별도 위치 (예: `D:/PARA/Resource/all-in-one-parser/output/baseline_pdf/`) 에 보관. 이후 Phase B~E 는 이 PDF 만 참조.

### 5.2 rhwp 측 SVG export

기존 명령:

```bash
rhwp export-svg <file.hwp> -o output/baseline_rhwp/<name>/ --embed-fonts
```

→ 모든 fixture 일괄 처리 스크립트 (`scripts/baseline_rhwp.ps1`, 미작성) 추가.

### 5.3 산출물

```
all-in-one-parser/output/
├─ baseline_hancom_pdf/    # 한컴 PDF (페이지 분리)
│   └─ <fixture_name>/page_001.png  (PDF→PNG 렌더, dpi=150)
├─ baseline_rhwp_svg/      # rhwp SVG (현재 버전)
│   └─ <fixture_name>/page_001.svg
└─ baseline_summary.json    # {fixture, pages_hancom, pages_rhwp, page_count_match}
```

### 5.4 1차 게이트

- 페이지 수 일치율 확인 (의미: 페이지네이션 본질 갭 사전 검출)
- 페이지 수 불일치 fixture 는 **Phase D 진입 전 우선 정정** (분할 정책 차이는 별도 클러스터)

### 5.5 소요 추정

| 항목 | 시간 |
|------|------|
| HWP→PDF 일괄 | 30~90분 (COM 동시성 1) |
| rhwp SVG 일괄 | 5~15분 |
| 페이지 분리 + 썸네일 | 10~30분 |
| 검증 + 요약 JSON | 10분 |
| **합계** | 1~2시간 |

---

## 6. Phase B — 자동 diff 인프라

### 6.1 도구 선택

| 옵션 | 장점 | 단점 |
|------|------|------|
| `pixelmatch` (기존 PR #498 사용) | 이미 통합·검증됨 | SVG↔PDF 간 anti-aliasing/폰트 hint 차이로 false positive 폭증 |
| **SSIM (`ssim.js` 또는 `image-ssim`)** | 폰트 hint 차이에 robust, 의미 있는 시각 유사도 측정 | 픽셀 정확도 비교는 별도 |
| `pdiff` (Yee, 2004) | 사람 시각 모델 기반 | JS 구현 빈약 |
| **하이브리드** | (1) SSIM 으로 페이지 수준 합격/불합격, (2) pixelmatch 으로 결함 영역 추출 | 구현 복잡도 ↑ |

권장: **하이브리드 (SSIM + pixelmatch)**. SSIM ≥ 0.97 이면 L2 합격. SSIM < 0.97 인 페이지에 한해 pixelmatch 의 차이 마스크를 추출하여 Phase C 의 클러스터링 입력으로 사용.

### 6.2 PR #498 인프라 확장

기존: `rhwp-studio/e2e/canvas-render-diff.test.mjs` (legacy ↔ layer Canvas)
확장: `rhwp-studio/e2e/hancom-vs-rhwp-diff.test.mjs` (Hancom PDF ↔ rhwp Canvas)

차이점:

- 기준 이미지가 PR #498 의 legacy Canvas 가 아니라 한컴 PDF 의 페이지 PNG
- fixture 입력이 `samples/` 가 아니라 `D:/PARA/Resource/all-in-one-parser/input/`
- 출력은 SSIM/MSE/diff 마스크의 3종 메트릭

### 6.3 GitHub Actions matrix

```yaml
strategy:
  matrix:
    category: [공통수학1, 미적분, 확률과_통계]
    chunk: [1, 2, 3, 4]   # 카테고리별 4 chunk 분할
```

총 12 jobs × 평균 30 페이지 = 360 페이지 / job ≈ 5~10분. 30~120분 내 전 fixture 자동 검증.

> 참고: GitHub Actions 는 한컴 한글 라이선스를 보유할 수 없으므로 **Hancom PDF baseline 은 git LFS 또는 별도 artifact 저장소** 에 보관해야 함. 또는 Phase A 의 PDF 를 commit 하지 않고 self-hosted runner 에서만 비교.

### 6.4 산출물

`scripts/visual_diff_runner.mjs` — fixture 1개를 받아 page-by-page SSIM/diff 산출 → JSON 리포트.

---

## 7. Phase C — 결함 클러스터링

### 7.1 클러스터링 휴리스틱

SSIM 하위 5% 페이지의 diff 마스크 픽셀 위치를 rhwp 의 render tree 좌표와 매칭하여 **결함이 어느 컨트롤에서 발생했는지** 자동 추론한다.

```
1. dump-pages 로 페이지의 control 좌표 (paragraph/table/shape/equation) 추출
2. diff 마스크의 connected component 추출
3. 각 component 의 bbox ↔ control bbox IoU 매칭
4. component → control type → 클러스터 (글자/수식/표/박스/그림)
```

### 7.2 클러스터 → Issue 자동 생성

```
[Cluster: 수식-EQALIGN]
fixture: 미적분_03.미분계수와 도함수, page 5, 7, 11
fixture: 미적분_05.도함수의 활용, page 2, 8
...
→ Issue: M100 #YYY [EQALIGN] 연립방정식 단계 정렬 미구현 (영향: N fixtures, M pages)
```

각 클러스터마다 1개 GitHub Issue 채번. 최소 5~10개 클러스터 예상.

### 7.3 산출물

`mydocs/report/all_in_one_parser_cluster_report.md` — 클러스터별 영향 fixture·page 목록 + 예상 결함 type + 우선순위.

---

## 8. Phase D — 우선순위 정정

§3 결함 매트릭스 + §7 클러스터 결과로 우선순위 확정. 권장 순서:

### P0 (1주차)

1. **EQALIGN 구현** (수식 layout 의 & 기준 열 정렬)
   - 영향 범위: 미적분 / 공통수학1 의 연립방정식·증명·단계풀이 다수
   - 모듈: `src/renderer/equation/parser.rs` + `layout.rs` + `svg_render.rs`
   - 기존 PILE 구조 재사용 + 열 정렬 로직 추가
   - 신규 단위 테스트 5+, fixture 통합 테스트 1
2. **글머리 문단 탭 정렬 (#340)** — 보기 ①②③④⑤ 정렬
   - 모듈: `src/renderer/composer/line_breaking.rs` + `layout.rs`
   - 글머리 폭이 탭 그리드 기준에 포함되도록 수정 + 회귀 테스트

### P1 (2~3주차)

3. **BIGG 크기 배율 적용**
4. **DINT / TINT / OINT 등 적분 기호 매핑**
5. **자동 로만체 컨텍스트 보정** (lim/sin 등 함수명 italic 회귀)
6. **§7 클러스터 중 표·박스 잔존 결함** (4~6 클러스터 예상)

### P2 (4주차)

7. 미세 정합 (분수 막대 두께·자간·줄간격)
8. SSIM 0.97 미달 페이지 잔여 정정

각 Issue 는 독립 task 로 진행 (이슈→브랜치→계획서→구현→단계보고→최종보고).

---

## 9. Phase E — 회귀 가드 + CI 게이팅

### 9.1 svg_snapshot 등록

390 fixture 중 **카테고리당 1~2개 대표 fixture** 만 `tests/svg_snapshot/` 에 등록 (현재 6 fixture → 12 fixture 로 확대). 그 이상은 CI 시간 폭증 + LFS 비용 부담.

### 9.2 Render Diff fixture 확대

`rhwp-studio/e2e/render-diff/` 의 baseline fixture 에 카테고리당 1~2 추가. 페이지 N 까지 SSIM ≥ 0.97 확인.

### 9.3 일별 회귀 리포트

`scripts/all_in_one_parser_nightly.ps1` — 새벽 자동 실행, 전 fixture diff → `mydocs/report/all_in_one_parser_nightly_YYYYMMDD.md` 생성.

---

## 10. 위험·제약

| 위험 | 영향 | 완화책 |
|------|------|--------|
| **한컴 한글 2020 라이선스 비보유** | Phase A 불가 | (1) 작업지시자 라이선스 확인 (2) 대안: 무료 한글 뷰어로 PDF 출력 (3) 한컴오피스 NEO 평가판 |
| **한컴 비공개 폰트 부재** | L3 픽셀 정합 불가, L2 도 폰트 metric 차이로 SSIM 저하 가능 | `mydocs/tech/font_fallback_strategy.md` 폴백 + 한컴 폰트 가상 metric 매핑 (`font_metrics_data.rs` 보강) |
| **fixture 이미지 LFS 비용** | 360+ baseline PNG (페이지당 100~300KB × 수천 페이지) → GB 단위 | Self-hosted runner + 외부 storage. 또는 nightly 기준 비교만 push |
| **390 fixture 의 일부가 동일 원본의 분할** | 중복 작업 | Phase A 직후 SHA-256 dedupe + 대표 fixture 30~50개 선정 |
| **EQALIGN 등 P0 구현이 typeset.rs core 영역 변경 유발** | 광범위 회귀 (#479 패턴) | core 변경은 별도 branch 에서 회귀쪽(@planet6897 의 회귀 셈플) 통과 후에만 merge |
| **layout.rs 4.5/10 코드 품질** | 신규 결함 진단·정정 효율 저하 | 본 전략 진행 전 `build_render_tree()` 분해 / ShapeObject::common() trait 추출 (별도 리팩토링 task) — **권장이지만 본 전략의 dependency 는 아님** |
| **390 fixture 일괄 검증 시간** | CI 30~120분, 로컬 1~2시간 | matrix 분할 + nightly 전환 |
| **L2 KPI 95% 가 비현실적일 수 있음** | KPI 미달 → 일정 지연 | Phase A 결과로 baseline SSIM 분포를 본 후 KPI 조정 (예: 90%) |

---

## 11. 일정 (제안)

| 주차 | Phase | 산출물 |
|------|-------|--------|
| W1 (5/2~5/8) | Phase A + B | baseline PDF/SVG, diff runner, page count match report |
| W2 (5/9~5/15) | Phase C | cluster report + GitHub Issues 일괄 채번 |
| W3 (5/16~5/22) | Phase D — P0 | EQALIGN + #340 정정 |
| W4~W5 (5/23~6/5) | Phase D — P1 | 수식 5건 + 표/박스 잔존 |
| W6 (6/6~6/12) | Phase D — P2 + Phase E | 미세 정합 + CI 게이팅 |

총 6주, **2026-06-12 까지 L2 정합 95% 달성** 목표.

---

## 12. 본 전략의 결정 포인트 (작업지시자 승인 필요)

다음 항목은 작업지시자의 결정이 필요한 분기점이다.

| # | 결정 사항 | 옵션 |
|---|-----------|------|
| D1 | "1:1 정합" 목표 등급 | L2 (권장) / L3 |
| D2 | 한컴 한글 2020 PDF baseline 확보 | 작업지시자 직접 / 평가판 / 무료 뷰어 |
| D3 | KPI 합격선 | SSIM ≥ 0.97 (권장) / 0.95 / 측정 후 결정 |
| D4 | fixture 전량 vs 대표 30~50 | 전량 / 대표 (권장: 대표 dedupe 후 전수 검증) |
| D5 | layout.rs 사전 리팩토링 동시 진행 여부 | yes (효율↑, 위험↑) / no (권장) |
| D6 | umbrella issue 마일스톤 | M100 / M0xx (현 v0.7.x 흐름) |
| D7 | EQALIGN 구현이 P0 인지 | yes (권장) / 별도 백로그 B-003 으로 유지 |
| D8 | nightly 회귀 인프라 self-hosted runner 도입 | yes (LFS 비용 절감) / no (GitHub-hosted matrix 만) |

---

## 13. 즉시 착수 가능한 사전 작업 (승인 불필요, 안전 영역)

소스 변경이 없는 **분석·기록 영역** 만 사전에 진행 가능.

1. ✅ 본 전략 문서 작성 (현재)
2. fixture dedup 스캔 — SHA-256 으로 중복 HWP 탐지 (`scripts/dedupe_fixtures.ps1`, **신규**)
3. fixture 통계 — 페이지 수·컨트롤 수·수식 컨트롤 비율 (`rhwp dump --summary` 기능 추가 필요 시 별도 task)
4. 한컴 PDF baseline 1건만 수동 생성 → diff runner 프로토타이핑 (소규모 PoC)

위 (2)~(4) 도 작업지시자 승인 후 시작.

---

## 14. 참조

- `mydocs/tech/dev_roadmap.md` — Phase 2 (5월) 코드 품질 9.2 목표와 정렬
- `mydocs/tech/equation_support_status.md` — 수식 미구현 항목 baseline
- `mydocs/eng/feedback/layout-quality-report.md` — layout.rs 4.5/10
- `mydocs/orders/20260430.md` — #501/#477/#431/#429 정정 패턴
- PR #498 — Canvas visual diff 인프라 (확장 대상)
- `mydocs/manual/ir_diff_command.md` — IR 수준 비교 도구

---

*승인 요청: 본 전략 검토 후 §12 D1~D8 결정과 함께 umbrella Issue 채번 지시 부탁드립니다.*

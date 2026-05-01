# SVG 회귀 검증 도구 — `scripts/svg_regression_diff.sh`

Layout 본질 변경 (#496 / #500 등) 시 광범위 SVG 회귀 검증을 자동화한다. Phase 1 (#517) 산출물.

## 사용법

### Mode 1: 두 commit/branch 비교 (자동 빌드)

```bash
scripts/svg_regression_diff.sh build <BEFORE_REF> <AFTER_REF> [SAMPLES...]
```

각 ref 에서 `src/` 만 체크아웃 → release 빌드 → 7개 기본 샘플 export → byte 비교.

예:
```bash
# devel 과 local/task500 비교 (모든 기본 샘플)
scripts/svg_regression_diff.sh build devel local/task500

# 특정 샘플만 비교
scripts/svg_regression_diff.sh build devel HEAD exam_science exam_kor
```

작업 트리 보존: 시작 시 자동 stash, 종료 시 pop.

### Mode 2: 두 디렉토리 비교 (이미 존재하는 SVG)

```bash
scripts/svg_regression_diff.sh diff <BEFORE_DIR> <AFTER_DIR>
```

각 디렉토리 구조: `<DIR>/<sample_name>/<sample_name>_NNN.svg`

예:
```bash
scripts/svg_regression_diff.sh diff /tmp/before /tmp/after
```

## 출력 형식

```
{sample}: total={N} same={M} diff={D}  diff_pages=[페이지 목록]
---
TOTAL: pages={total} same={same} diff={diff}
```

`diff > 0` 인 sample 의 `diff_pages` 에서 변경된 SVG 파일을 직접 비교 (`diff /tmp/svg_diff_before/<sample>/<page> /tmp/svg_diff_after/<sample>/<page>`) 하여 변경 내용 검토.

## 기본 샘플 목록

(SAMPLES 미지정 시 사용)
- `exam_kor` (20 페이지)
- `exam_eng` (8 페이지)
- `exam_science` (4 페이지)
- `exam_math` (20 페이지)
- `synam-001` (35 페이지)
- `aift` (77 페이지)
- `2010-01-06` (6 페이지)

총 170 페이지, 다양한 layout 패턴 (다단/표/수식/각주/인라인 컨트롤) 커버.

## 활용 사례

### 신규 commit 의 회귀 검증

```bash
git commit -m "..."
scripts/svg_regression_diff.sh build HEAD~1 HEAD
```

`diff > 0` 페이지가 의도된 정정인지 확인 → 의도되지 않으면 회귀.

### Phase 2~4 layout 본질 변경 검증

```bash
scripts/svg_regression_diff.sh build devel local/task<N>
```

리팩터링 변경의 영향 범위를 정량 확인.

## 관련

- Phase 1 #517 (본 도구 추가)
- Phase 0 로드맵: `mydocs/tech/layout_refactor_roadmap.md`
- `RHWP_LAYOUT_DEBUG=1` env logging — 같이 사용 시 결함 측정·재현 자동화

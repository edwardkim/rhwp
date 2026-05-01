# Task #517 Stage 2 보고서 — 회귀 검증 도구 정형화

**이슈**: #517
**브랜치**: `local/task517`
**Stage**: 2 / 2

## 1. 변경 내용

- `scripts/svg_regression_diff.sh` (신규, 약 130 LOC bash)
- `mydocs/manual/svg_regression_diff.md` (사용 매뉴얼)

## 2. 기능

### Mode 1: `build <BEFORE_REF> <AFTER_REF> [SAMPLES...]`

- 두 commit/branch 에서 자동 빌드 → SVG 추출 → byte 비교
- 작업 트리 자동 stash/pop
- `/tmp/svg_diff_{before,after}/` 에 결과 보존

### Mode 2: `diff <BEFORE_DIR> <AFTER_DIR>`

- 이미 존재하는 두 디렉토리 비교 (재빌드 없이)

## 3. 출력 형식

```
{sample}: total={N} same={M} diff={D}  diff_pages=[페이지 목록]
---
TOTAL: pages={total} same={same} diff={diff}
```

## 4. 검증

### 4-1. Mode 2 (diff): 기존 #500 작업 dir 비교

```
$ ./scripts/svg_regression_diff.sh diff /tmp/task500_before /tmp/task500_after
2010-01-06: total=6 same=6 diff=0
aift: total=77 same=77 diff=0
exam_eng: total=8 same=8 diff=0
exam_kor: total=20 same=20 diff=0
exam_math: total=20 same=20 diff=0
exam_science: total=4 same=3 diff=1  diff_pages=[exam_science_002.svg]
synam-001: total=35 same=35 diff=0
---
TOTAL: pages=170 same=169 diff=1
```

→ #500 fix 의 의도된 1페이지 정정 정확히 식별.

### 4-2. Mode 1 (build): devel vs HEAD (Phase 1 자체)

```
$ ./scripts/svg_regression_diff.sh build devel HEAD
TOTAL: pages=170 same=170 diff=0
```

→ Phase 1 변경 자체의 회귀 0건 검증.

## 5. 활용

- 신규 commit 회귀 검증
- Phase 2~4 layout 본질 변경 광범위 회귀 검증 (170 페이지 자동 비교)
- `RHWP_LAYOUT_DEBUG=1` 와 함께 사용 시 결함 측정·재현 자동화

## 6. 기본 샘플 커버리지

| 샘플 | 페이지 | 패턴 |
|------|--------|------|
| exam_kor | 20 | 다단, 헤더/푸터, master, 인라인 표/도형 |
| exam_eng | 8 | 영문 문제지 |
| exam_science | 4 | 과학 (수식/표/그림 다수) |
| exam_math | 20 | 수학 (수식 집중) |
| synam-001 | 35 | 일반 문서 |
| aift | 77 | 긴 문서 (다양한 패턴) |
| 2010-01-06 | 6 | 기본 케이스 |
| **합계** | **170** | 광범위 layout 커버 |

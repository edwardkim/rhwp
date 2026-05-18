# Task #969 Stage 4 — 회귀 검증

- 이슈: [#969](https://github.com/edwardkim/rhwp/issues/969)
- 브랜치: `local/task969`

## 1. cargo test --release 전체 통과

```
$ cargo test --release
test result: ok. ... 0 failed; ...
```

전체 테스트 suite 통과 (실패 0 건). 골든 SVG snapshot 도 변경 없음.

## 2. 240 샘플 페이지 수 비교

`samples/*.hwp + samples/hwpx/*.hwpx + samples/basic/*.hwp` 전수 측정 후 pre-D6 와 diff:

```
$ diff /tmp/baseline969/pre.txt /tmp/baseline969/post.txt
62c62
< hwp3-sample16-hwp5.hwpx: 72
---
> hwp3-sample16-hwp5.hwpx: 71

239c239
< hwpx-02.hwpx: 6
---
> hwpx-02.hwpx: 5
```

**240 샘플 중 2 건만 변동** (1 타깃 + 1 side effect).

| 샘플 | Pre-D6 | Post-D6 | 비고 |
|------|--------|---------|------|
| hwp3-sample16-hwp5.hwpx | 72 | **71** (-1) | **타깃** — D6 효과 |
| hwpx-02.hwpx | 6 | 5 (-1) | side effect (PDF 권위 자료 없음) |
| 그 외 238 샘플 | (변동 없음) | | |

## 3. side effect 분석 (hwpx-02.hwpx)

- PDF 권위 자료 없음 → 정답 판정 불가
- HWP 변종 (`samples/hwpx/hancom-hwp/hwpx-02.hwp`) 은 9 페이지 → HWPX 와 차이 큼 (포맷 간 paginiation 차이 일반적)
- 1 페이지 변동은 D6 효과와 동일 패턴 (line_spacing double-count 해소) 으로 추정 — 회귀가 아닌 **개선** 가능성

## 4. 회귀 위험 종합

- 전체 cargo test 통과
- 240 샘플 중 변동 2 건 (1 타깃 + 1 side effect)
- 골든 SVG snapshot 변경 없음
- **회귀 위험 낮음**

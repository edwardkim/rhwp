# Task #1880 Stage 2 — 실측 검증 (로컬 코퍼스 A/B 하니스)

## 개별 문서 검증

릴리즈 빌드로 `rhwp convert` + `dump-pages` + `RHWP_TABLE_DRIFT=1` 트레이스 대조:

| 문서 | 수정 전 | 수정 후 |
|------|---------|---------|
| 3075729 (oracle p13) | conv heading **p12** ✗ | conv heading **p13** ✓ (HWPX·한컴 정합) |
| 2780073 | host_before 6.7↔0.0px, pi=4 defer 플립 | 트레이스 **완전 일치** (host_before 6.7 양쪽) |
| 2776741 (phantom, oracle 1쪽) | 1쪽/1쪽 | 1쪽/1쪽 **불변** ✓ (#1836 억제 층위 무영향 실증) |

## A/B 하니스 (tools/roundtrip_fidelity_harness.py)

- 코퍼스: hwpdocs admrul_downloads 결정적 서브셋 2,005건
  (정렬 상위 2,000 + 재현 5건, `corpus2000.list`)
- 수정 전 바이너리: stash 후 재빌드 / 수정 후: 본 수정 반영 빌드

| 빌드 | SAME | PI_MOVED | PAGE_DELTA | ERR |
|------|------|----------|------------|-----|
| 수정 전 (devel bf5228df) | 2000 | 4 | 1 | 0 |
| **수정 후** | **2002** | **2** | 1 | 0 |

- **개선 +2**: 2780073 (s0:pi9), 3075729 (s1:pi121) — 본 수정이 해소.
- **신규 divergence 0**: 수정 후 비-SAME 3건은 수정 전과 동일 항목·동일 상세
  (엄밀한 부분집합).
- 잔존 (별개 클래스, 본 수정 무관 — 수정 전후 상세 동일):
  - 2959953 PI_MOVED 5개 pi: 표 트레이스(pi≤23) 양 경로 동일 확인 —
    표 경로가 아닌 문단 flow 층위의 다른 비대칭.
  - 3171755 PI_MOVED 1개 pi (s0:pi213).
  - 3235145 PAGE_DELTA (3→2): 기존재 별개 클래스.

산출물: 스크래치 `task1880/{prefix,fixed}.tsv`, 트레이스 `.t`/`.pimap2` 파일.

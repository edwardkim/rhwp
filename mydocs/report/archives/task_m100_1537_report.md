# 성능 벤치마크 (bench 명령) — 최종 보고서

- 이슈: #1537 (성능 벤치마크: bench CLI 서브커맨드)
- 브랜치: `local/task1537`
- 마일스톤: M100 (v1.0.0)

## 1. 목적

대용량 HWP/HWPX 처리 성능을 **재현 가능한 수치**로 계량한다. "사용 가능"·"빠름" 같은
모호 표현 대신 단계별 계량값으로 성능 특성을 기술하고, 성능 회귀를 추적할 기준을 만든다.

## 2. bench 명령

```
rhwp bench <파일...> | --batch <폴더> [-n <반복수>] [--tsv <출력.tsv>]
```

단계별 계측(워밍업 1회 후 N회 median, 기본 N=3):

| 단계 | 측정 대상 | API |
|------|----------|-----|
| parse | 바이트 → Document IR | `parse_document` |
| layout | parse+layout 로드 − parse (근사) | `DocumentCore::from_bytes` |
| render | 전 페이지 SVG 렌더 | `render_page_svg_native` |
| serialize | Document → HWPX 바이트 (저장) | `serialize_hwpx` |

파일별 크기/쪽수 + 단계별 median(ms) + total 표와 `--tsv` 산출.

## 3. 측정 환경

- CPU: Intel Core i7-8850H @ 2.60GHz / RAM 16GB / macOS (darwin 24.6.0)
- 빌드: `cargo build --release` (rustc 1.93.1)
- 반복: 워밍업 1회 후 **N=5 median**

> 정직성 주의: 아래 절대 수치는 **이 머신·이 빌드**에 한정된다. 동일 환경에서의
> **상대 비교·회귀 추적·재현**을 위한 지표이며, 한컴 등 외부 제품과의 비교 기준이
> 아니다. CPU/디스크/빌드가 바뀌면 절대값은 달라진다.

## 4. 결과 (N=5 median)

| 파일 | 크기KB | 쪽 | parse | layout | render | serialize | total |
|------|-------:|---:|------:|-------:|-------:|----------:|------:|
| exam_kor.hwp | 10174.5 | 20 | 340.7 | ~0 | 775.5 | 743.1 | **1859.3** |
| exam_kor.hwpx | 8133.6 | 20 | 206.3 | 24.3 | 767.6 | 739.6 | **1737.7** |
| 교육통합(격자) .hwp | 5738.5 | 23 | 162.7 | 7.3 | 497.7 | 277.9 | **945.6** |
| aift.hwpx | 4938.2 | 74 | 109.1 | 28.3 | 393.3 | 269.9 | **800.7** |
| k-water-rfp.hwpx | 2120.0 | 27 | 38.7 | 14.4 | 133.0 | 122.7 | **308.8** |
| exam-kor-2p.hwpx | 870.0 | 2 | 21.4 | 4.1 | 84.1 | 100.1 | **209.8** |
| form-002.hwpx | 128.5 | 10 | 9.9 | 11.3 | 39.8 | 14.0 | **75.1** |
| footnote-01.hwpx | 63.6 | 6 | 1.6 | 0.4 | 7.0 | 2.9 | **11.9** |
| business_overview.hwpx | 9.9 | 1 | 1.1 | 0.7 | 2.2 | 1.7 | **5.7** |

단위: ms. 합계 9파일 / 183쪽 / total 5954.7ms (≈32.5ms/쪽).

## 5. 해석

- **대용량 10MB HWP(20쪽)**: 열기(parse+layout) ≈ 0.34s, 전 페이지 렌더 ≈ 0.78s,
  HWPX 저장 ≈ 0.74s. 전체 파이프라인 ≈ 1.9s.
- **비용 구성**: render·serialize 가 지배적(각 ~40%), parse 는 파일 크기에 비례.
  layout 은 대체로 작다(전체의 수 %).
- **페이지 수에 대체로 선형**: aift(74쪽, 4.9MB)가 exam_kor(20쪽, 10MB)보다 total 이
  작다 — total 은 파일 크기(parse/serialize)와 페이지 수(render)의 합성.
- **저장(serialize) 비용 가시화**: HWPX 직접 저장(#1532)이 대용량에서 ~0.7s 수준임을
  계량 — UX(진행 표시/비동기) 설계 판단 근거.

## 6. 한계·후속

- **layout 근사**: `load − parse` 차분이라 음수→0 클램프 발생(예: exam_kor.hwp).
  정밀 분리는 별도 계측 훅 필요(후속).
- **메모리 미측정**: peak RSS 는 v1 범위 외. getrusage/allocator 카운터 도입은 후속.
- **단일 머신**: CI 러너 등 복수 환경 교차 측정은 후속(회귀 게이트화 여부 별도 판단).

## 7. 산출물

- `src/diagnostics/bench.rs` (+ main.rs 디스패치/help, cli_commands.md 동기화)
- `output/poc/bench/bench.tsv` (gitignore — 재현은 bench 명령으로)

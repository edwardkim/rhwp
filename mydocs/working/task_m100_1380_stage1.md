# Task M100 #1380 1단계 완료 보고서 — 측정·진단

- 구현계획서: `mydocs/plans/task_m100_1380_impl.md` 1단계
- 브랜치: `local/task1380`

## 1. 구현 내용 (측정 도구)

| 항목 | 위치 | 내용 |
|------|------|------|
| `diff_linesegs()` | `serializer/hwpx/roundtrip.rs` | 문단별 `line_segs` 9개 필드 시퀀스 비교. #1378 재귀 경로(본문 + 셀·글상자 Group·각주/미주) 동일 순회. **게이트 비동승** (측정 전용) |
| `LinesegDiff` / `LinesegDiffKind` | 〃 | `CountMismatch` / `ValueMismatch{index, field, expected, actual}` |
| `--lineseg-report` | `diagnostics/hwpx_roundtrip_batch.rs` | round1(원본 vs RT)·round2(RT vs RT²) diff 를 `lineseg_diff.tsv` 로 산출. 기본 비활성 — `inventory.tsv` 13컬럼 불변 |
| 단위 테스트 6건 | 양쪽 | 값/개수 불일치 검출, 셀 재귀 path, 게이트 비동승 확인, 옵션 파싱 |

## 2. 전수 측정 결과 (54건, `output/poc/task1380/`)

### 2.1 IR 수준 — diff 0 (완전 보존)

```
rhwp hwpx-roundtrip --batch samples/hwpx -o output/poc/task1380 --lineseg-report
→ PASS 48 / IR_DIFF 1(#1382) / SERIALIZE_FAIL 4(#1384) / PARSE_FAIL 1(EXCLUDED)
→ lineseg round1=0 round2=0 (측정 가능 49건 전수)
```

**측정 가능 전수에서 lineseg ValueMismatch·CountMismatch 0건.** 파서 적재(9필드 그대로)
→ serializer 방출(9필드 그대로) 경로가 깨끗함을 전수로 확정했다. 2-round 드리프트도 0.

### 2.2 XML 수준 교차 측정 — lineseg·문단 카운트 (원본 vs RT)

IR 비교가 못 보는 영역(문단 소실·합성 비대칭)을 XML 카운트로 보완 측정:

| 분류 | 샘플 | 측정값 | 귀속 |
|------|------|--------|------|
| **합성 방출** | business_overview | lineseg 0→38 (전 문단) | **본 타스크 잔존 결함** (3절) |
| 〃 | expense_report | lineseg 0→55 (전 문단) | 〃 |
| 문단 소실 | ta-pic-001-r | p 6→5, lineseg 6→5 | #1387 캡션 (caption_p=1) |
| 〃 | mel-001 | p −2, lineseg −2 | #1387 캡션 (caption_p=2) |
| 〃 | 143E433F503322BD33 | p −1, lineseg −1 | #1387 캡션 (caption_p=1) |
| 〃 | aift | p −15, lineseg −13 | #1387 캡션 13 + **MEMO 필드 subList 2** (신규, 5절) |
| 동일 | 나머지 43건 | lineseg·p 카운트 완전 일치 | — |

### 2.3 spot 실측 오인 정정

수행계획서 2절에서 "빈 문단 fallback 합성 잔존"으로 기재했던 ta-pic-001-r RT의
`vertsize=1000 textheight=1000 baseline=850 spacing=600 horzpos=0 horzsize=0`은
**원본 XML에 실재하는 값**으로 확인 — RT가 원본을 그대로 보존한 것이며 결함이 아니다
(serializer fallback 상수와 우연히 동일한 한컴 생산 값).

## 3. 가설 판정 (구현계획서 0.3)

| 가설 | 판정 |
|------|------|
| H1. 원본에 linesegarray 없는 문단 | **확정 — 유일한 잔존 결함.** business_overview(38문단)·expense_report(55문단)는 원본에 linesegarray가 전혀 없음 → 파서가 zero-default 주입(parser section.rs:729) → serializer가 `vertsize=0 … flags=393216` lineseg를 **합성 방출**. 원본 무 → RT 유 비대칭. IR 비교는 검출 불가(주입값이 왕복 일치). 2-round는 안정(zero→zero) |
| H2. 파서 미경유 경로의 fallback 발동 | 미발생 — 1000 계열은 원본 실재 값(2.3) |
| H3. 문단 소실 기인 오인 | 캡션(#1387)·MEMO 필드(신규) 귀속으로 분리 완료 |
| H4. #1315 값 변형의 기해소 | **확정** — lh=21974→19924 등 변형 현재 미발생 (ta-pic-001-r의 21974/18678/51024 lineseg 원본 그대로 보존). #1378/#1379 경로 정비로 해소 |

참고: 전수 원본에 all-zero lineseg는 자연 존재하지 않음 — 주입분 식별 모호성 없음.

## 4. 이슈 기재 증상 4샘플 재측정

| 샘플 | 이슈 기재 | 현재 실측 | 귀속 |
|------|----------|----------|------|
| form-002 | 10→17쪽 | 10→**15**쪽 | **#1388** — 여백 5669→8504(좌우)·2834→5668(상) 실측. lineseg는 762↔762 완전 보존 |
| 2025-1Q 보도자료 | 9→13쪽 | 9→**13**쪽 | **#1388** — 여백 5669→8504 실측 |
| math-001 | 1~2px 시프트 | **SVG 바이트 동일** (md5 일치) | 해소 (#1378/#1379) |
| footnote-01 | 전 페이지 SVG 차이 | 6쪽 일치, 전 페이지 차이 잔존 — **균일 시프트 +37.8px** (=2835 HWPUNIT=여백 차) | **#1388** |

**결론: 이슈에 기재된 페이지 수 변화·SVG 차이는 전부 lineseg 기인이 아니라 #1388
(secPr 여백 하드코딩) 기인이다.** 수행계획서 완료 조건 4·5(페이지 수 일치, SVG 시프트
해소)는 #1388 해소 시 달성되는 항목으로, 본 타스크에서는 "lineseg 기인분 없음 확정"으로
충족됨을 제안한다.

## 5. 신규 발견 (범위 밖 — 별도 이슈 등록 제안)

1. **MEMO 필드(fieldBegin) subList 문단 소실** — aift의 `type="MEMO"` fieldBegin 2건의
   `hp:parameters`+`hp:subList`(메모 본문 문단)가 RT에서 소실. 게이트 사각(컨트롤 내부
   subList 비교 없음)
2. **shapeComment 소실** — aift `hp:shapeComment` 15→0

## 6. 2·3단계 범위 확정 제안 (승인 요청)

잔존 결함이 H1 단일 패턴이므로 구현계획서 2·3단계를 다음과 같이 조정 제안한다:

- **2단계 (정정 일괄)**: 파서 zero-default 주입(729) 제거(원본에 linesegarray 없으면
  IR `line_segs` 빈 채 유지) + serializer는 IR 빈 문단에서 **linesegarray 방출 생략**
  (fallback 합성을 비파싱 IR 경유의 최후 수단에서도 제거할지는 의존 경로 조사 후 결정).
  두 수정은 상호 의존이라 한 단계로 묶는다. 빈 `line_segs` 의존 경로(렌더러·HWP5
  serializer 등) 조사 포함. 근거: 한컴은 lineseg 없으면 재계산(기지 메모)하므로 방출
  생략이 안전축, zero 값 방출(`높이 0인 줄`)이 오히려 위험
- **3단계 (게이트 동승)**: `diff_linesegs`를 baseline 게이트에 동승. 주입 제거 후에는
  IR 비교가 합성 비대칭(empty vs non-empty)도 검출한다. **현재 전수 0이므로 xfail 0으로
  동승 가능** — 동승을 제안한다

## 7. 검증

| 항목 | 결과 |
|------|------|
| `cargo test --lib` | 1710 passed |
| `cargo test --tests` (baseline 게이트 포함) | 전체 그린 |
| `cargo test --test hwpx_roundtrip_baseline` | 4 passed |
| `cargo fmt --check` | 통과 |
| `cargo clippy --lib --tests` | 경고 0 |

## 8. 산출물

- 소스: `src/serializer/hwpx/roundtrip.rs` (+diff_linesegs·테스트), `src/diagnostics/hwpx_roundtrip_batch.rs` (+--lineseg-report), `src/main.rs` (사용법)
- 측정: `output/poc/task1380/` (inventory.tsv, lineseg_diff.tsv, *.rt.hwpx — git 미포함)

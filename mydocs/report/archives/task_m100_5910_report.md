# task_m100 #5910 처리 결과 — 병합 셀 선언 높이가 걸친 행합보다 작을 때 무시되던 결함

- 이슈: [#5910](https://github.com/edwardkim/rhwp/issues/5910)
- 대상 문서: `samples/kps-ai.hwp` / 한글 정본 `pdf/kps-ai-2022.pdf`
- 기준 커밋: `origin/devel` = `72674c5653f09cb78b994dc4cd2dfd0a97ae6c8a`

## 1. 증상

`samples/kps-ai.hwp` 는 한글 2022 정본이 **77쪽**인데 rhwp 는 **78쪽**이었다. 쪽별 본문을
정본과 대조하면 0~45쪽은 내용이 일치하고 **46쪽부터 끝까지 정확히 1쪽씩 밀렸다**.

## 2. 원인

구역0 문단443 의 13행×5열 표(「4. 기술제안서 세부평가기준」, 정본 43쪽)에서

| 셀 | 위치 | 선언 높이 |
|---|---|---|
| `셀[36]` | `r=8, c=0, rs=3` (프로젝트 지원(15점)) | **17,354 HU** |
| `셀[37]`~`셀[48]` | `r=8·9·10`, `rs=1` | 각 6,082 HU |

걸침 선언 17,354 HU 인데 단일행 선언 합은 6,082×3 = **18,246 HU** 로 892 HU(8.9pt) 크다.

정본 46쪽(문서 43쪽) 표 괘선 y좌표 실측(PDF pt)
`155.3 / 183.3 / 244.1 / 304.7 / 365.5 / 426.3 / 487.0 / 547.8 / 608.6 / 669.2 / 730.0 / 781.9`
에서 그 세 행 높이는 **60.8 / 60.8 / 51.9 pt** 이고, 마지막 행 51.9pt = **5,190 HU =
17,354 − 6,082×2** 로 정확히 닫힌다. 같은 표의 `common.height` **62,725 HU** 역시
머리행 2,797 + 6,082×9 + **5,190** 으로 같은 값에 닫혀, 두 독립 경로가 서로를 확인해 준다.

즉 **한글은 걸침 선언을 권위로 삼아 마지막 걸침 행을 줄인다.**

rhwp 에는 반대 방향 규칙만 있었다 — 걸침 선언이 행합을 **초과**하면 잔여를 마지막 걸침 행에
가산한다(#2291/#2237). 걸침 선언이 행합보다 **작은** 경우에는 처리가 없어 행합 18,246 HU 가
그대로 쓰였고, 걸침 묶음은 행 단위로 쪼갤 수 없으므로 세 행이 통째로 다음 쪽으로 이월됐다.

```
DIAG_SCAN BLOCK_DECIDE r=8 b=8..11 block_h=243.3 rest=237.3 budget=237.3 ... fully=true
```

묶음 243.3px 가 잔여 237.3px 를 5.9px 초과 → 이월 → 이후 32쪽이 밀려 총 78쪽.

## 3. 수정

### 3.1 `src/model/table.rs` — `Table::rowspan_declared_overflow_shrink()`

행별 축소량(HWPUNIT)을 계산하는 단일 출처를 추가했다. `row_span>1` 셀 선언이 걸친 행들의
`row_span==1` 선언 합보다 작으면 그 차이를 마지막 걸침 행이 흡수한다.

**손상 선언 방어** — 두 선언이 어긋난다는 사실만으로는 어느 쪽이 옳은지 알 수 없다. 걸침
선언이 0 이거나 한 행 값과 같은 손상 문서가 실재한다(`samples/task2287/1342000_edu_curriculum_map.hwp`:
걸침 선언 0 vs 행합 1,500). 그래서 **저장된 `common.height` 가 축소 결과를 확인해 줄 때만**
적용한다 — 마지막 걸침 행까지의 행합이 축소 후 `common.height` 와 정확히 같아야 한다.
확인이 없으면 종전 동작(축소 없음)을 유지한다.

이 확인 조건은 실측으로 검증했다. `samples/kps-ai.hwp` 는 축소가 필요한 걸침 28건 중
**정확히 1건**(문제의 r8 rs=3)만 통과하고, 손상 선언이 많은
`1342000_edu_curriculum_map.hwp` 는 185건 중 **0건**이 통과한다.

### 3.2 `src/renderer/height_measurer.rs`

`MeasuredTable` 행 높이 해결의 2-b 단계에 축소를 적용한다. 글자 소실 방지를 위해 행별
컨텐츠 하한(`content_row_floor`, 2단계에서만 채운다) 밑으로는 줄이지 않는다.

### 3.3 `src/renderer/layout/table_layout.rs`

- `resolve_row_heights_with_common_fit` 2단계 — HeightMeasurer 와 같은 규칙을 미러.
- `row_cut_content_height` — 컷 회계가 원 선언 6,082 HU 를 그대로 읽으면 HeightMeasurer 가
  이미 줄여 둔 행을 다시 부풀려(`row_cut_h > mt.row_heights`) 걸침 묶음이 쪽에 못 들어간다.
  선언 높이에 같은 축소를 적용한다. 컨텐츠(`content + pad`) 하한은 그대로다.

## 4. 검증

### 4.1 전/후 수치

| 항목 | 수정 전 | 수정 후 | 한글 정본 |
|---|---|---|---|
| `rhwp info` 페이지 수 | 78 | **77** | 77 |
| `export-text --json` `pageCount` | 78 | **77** | 77 |
| 43쪽 표 마지막 걸침 행 높이 | 81.1px (60.8pt) | **69.2px (51.9pt)** | 51.9pt |
| 43쪽 표에 담긴 본문 행 | 머리행 + 7행 | **머리행 + 10행** | 머리행 + 10행 |
| `BLOCK_DECIDE r=8` 묶음 높이 / 잔여 | 243.3 / 237.3 (초과) | **230.3 / 237.3 (수용)** | — |

### 4.2 259문서 쪽수 게이트 (`tools/render_page_gate.py`)

| | 수정 전 | 수정 후 |
|---|---|---|
| 매치(rhwp == 한글) | 245 (94.6%) | **246 (95.0%)** |
| −6 / −3 / −2 | 1 / 1 / 2 | 1 / 1 / 2 |
| +1 / +2 | 8 / 2 | **7** / 2 |

쪽수가 달라진 문서는 **`samples/kps-ai.hwp` 단 1건**(78 → 77, delta +1 → 0). **회귀 0.**
`tests/fixtures/render_page_samples.tsv` 의 해당 행도 `77 / 77 / 0` 으로 갱신했다.

### 4.3 코퍼스 SVG self-diff

픽스처 259문서의 앞 2쪽을 수정 전/후 바이너리로 각각 렌더해 SHA-1 대조 →
**해시가 다른 (문서,쪽) 0건**. 의도한 문서 외 변화 없음.

### 4.4 테스트 · CI 게이트

- 새 테스트 `tests/cases/rowspan_declared_overflow_shrink.rs` (regression_suite_021)
  - red: 수정 없이 `kps_ai_page_count_matches_hangul_master` → `left: 78 / right: 77` 실패,
    `kps_ai_keeps_rowspan_block_on_declared_page` → 「교육훈련」 부재로 실패
  - green: 4 tests passed
- `cargo test --profile release-test --lib -p rhwp` → 3893 passed / 0 failed
- `cargo test --profile release-test --test regression_suite_021` → 126 passed / 0 failed
- `cargo clippy --all-targets -- -D warnings` → exit 0
- `rustfmt --edition 2021 --check` (변경 파일 LF 사본) → 차이 없음
- `node scripts/rust-unit-test-tiers.mjs --check --base-ref origin/devel` → 4225 유지

### 4.5 시각 검증 (전 / 후 / 정본 3단)

- `mydocs/report/edit_demo_5910/kps_ai_p45_before_after_master.png`
- `mydocs/report/edit_demo_5910/kps_ai_p46_before_after_master.png`

전/후 래스터는 두 쪽 모두 픽셀 해시가 다르고(`DIFFERENT`), 수정 후는 정본과 같은 조판이 된다.

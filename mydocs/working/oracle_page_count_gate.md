---
kind: working
status: active
issue: 5585
---

# 한글 정답지 쪽수 원장 (#5585 축을 저장소 자산으로)

## 무엇을

`pdf/` 의 한글 정답지 573 장과 `samples/` 를 이름으로 짝지어 **rhwp 쪽수를 전수 대조**하고
래칫으로 고정한다.

- `tests/cases/oracle_page_count_baseline.rs` — 대조·래칫
- `tests/fixtures/oracle_page_count_baseline.tsv` — 555 문서 기준선
- `tools/oracle_page_count/regenerate.py` — 픽스처 재생성

## 왜

v1.0 의 목표는 "한컴오피스와 같은 조판" 이고, 1차 지표는 **같은 문서를 몇 쪽으로
조판하는가** 다. #5585 가 그 축을 10,000 문서로 재어 462 건(4.6%) 불일치를 보고했다.

그런데 그 측정은 **비공개 코퍼스와 한글 2022 설치본**이 있어야 재현된다. 외부 기여자는
자기 변경이 이 축을 건드렸는지 확인할 수단이 없고, CI 도 보지 않는다. 즉 v1.0 의 핵심
지표가 **아무 게이트에도 걸려 있지 않다.**

이 저장소는 이미 정답을 갖고 있다. `pdf/` 는 한글이 직접 뽑은 출력이고, 그 쪽수는 "한글이
이 문서를 몇 쪽으로 조판했는가" 의 정답이다. 클론만 있으면 누구나 대조할 수 있다.

## 실측

| | 문서 수 |
| --- | ---: |
| 정답지와 이름이 짝지어진 샘플 | 562 |
| 모아 찍기 선언으로 제외 | 7 |
| **대조 대상** | **555** |
| 한글과 **일치** | **538 (96.9%)** |
| 불일치 | 17 (3.1%) |

불일치 17 건이다.

| 차 | 문서 | 정답지 | rhwp |
| ---: | --- | --- | ---: |
| 8 | `basic/sungeo.hwp` | 94 | 86 |
| 6 | `정책연구용역사업 중간진도보고서(…간장 기증자…).hwpx` | 215 | 221 |
| 4 | `hwpx/hancom-hwp/hwpx-02.hwp` | 5 | 9 |
| 2 | `task2287/1342000_edu_curriculum_map.hwp` | 415 | 413 |
| 1 | `2025 행정업무운영 편람(최종).hwp` | 383·388·389 | 384 |
| 1 | `2025 행정업무운영 편람(최종).hwpx` | 383·388·389 | 382 |
| 1 | `hwpx/[2027] 온새미로 1 본교재.hwpx` | 47 | 46 |
| 1 | `hwpx/hancom-hwp/[2027] 온새미로 1 본교재.hwp` | 47 | 48 |
| 1 | `hwpx/hwpx-02.hwpx` | 5 | 6 |
| 1 | `hwpx/opengov/36385445_…화재발생종합보고서….hwpx` | 2 | 1 |
| 1 | `issue2063_huge_cellbreak_table.hwp` | 162 | 161 |
| 1 | `issue2470/36341511_masked.hwpx` | 8 | 9 |
| 1 | `issue3637/press_release_split_cell_nested_table.hwpx` | 12 | 13 |
| 1 | `issue5792/2700727_animal_facility_standards.hwpx` | 6 | 5 |
| 1 | `table_giant_cell_overfill.hwpx` | 48 | 47 |
| 1 | `tac-img-02.hwpx` | 66 | 67 |
| 1 | `task1718/table_giant_cell_overfill.hwp` | 48 | 47 |

같은 문서의 두 형식이 **서로도 다른** 경우가 셋이다 — 편람 384/382, 온새미로 48/46,
hwpx-02 9/6. `hancom-hwp/` 는 같은 HWPX 를 한글이 열어 HWP5 로 저장한 것이므로 두 값이
같아야 한다.

## 판정 규약

`overflow_cell_baseline`(#3668)·`text_overlap_baseline`(#6315)과 같은 래칫이되, **정답이
있으므로 방향이 하나 더 있다.**

| 상태 | 판정 |
| --- | --- |
| rhwp 값이 정답지와 일치 | 통과 (기준선과 무관 — 개선은 언제나 통과) |
| 불일치인데 격차가 기준선 이하 | 통과 |
| 불일치이고 격차가 커짐 | **실패** |
| 픽스처에 없는 문서가 어긋남 | **실패** (신규 발생) |

즉 **지금 맞는 538 문서는 하드 게이트**다 — 어긋나는 순간 실패한다. 지금 어긋난 17 건은
더 나빠질 때만 실패한다. 고치면 자동으로 통과하므로 개선을 막지 않는다.

## 모아 찍기 — 추측하지 않고 문서 선언을 쓴다

`print_method` 가 4·5 면 한글이 한 장에 여러 쪽을 실어 뽑으므로 장 수가 애초에 다르다.
`model::document::print_method_implies_nup` 주석의 실측표가 이를 정의하고, 그 주석은
**정확히 이 용도를 예견하고 있다.**

> rhwp 는 이 값을 출력에 반영하지 않는다 — 파싱·노출 전용이다. 한글 오라클 PDF 와 대조할 때
> 이 값이 `print_method_implies_nup` 이면 한글 쪽 장 수·용지 방향이 rhwp 와 다르므로,
> 좌표를 그대로 견주면 오판한다.

#6208 / #6268 로 들어온 기능인데 아직 소비자가 없었다. 이 게이트가 첫 소비자다.

### 겪은 오판 — 간접 신호로 추측하면 진짜 결함을 삼킨다

처음에는 이 필드를 쓰지 않고 **정답지 쪽수만** 비교했다. 최악 사례가
`issue5866/memo_field_hwp5.hwp`(정답지 20, rhwp 40)였는데 파보니 정답지가 A4 가로
841x595 이고 한 장에 `- 39 -`·`- 40 -` 가 함께 있었다 — 모아 찍기였고 **rhwp 가 맞았다.**

그래서 "시트 = ceil(쪽수/N)" 규칙으로 보정했더니 이번에는 **세로로 뽑힌 정답지까지 삼켰다.**
`hancom-hwp/hwpx-02.hwp` 는 정답지가 A4 세로 5 쪽인데 `ceil(9/2)=5` 라서 2-up 으로
오인돼 통과했다 — **차 4 짜리 진짜 불일치가 필터에 가려진 것이다.**

문서가 스스로 선언한 `print_method` 만 신뢰하니 오탐과 오음성이 동시에 사라졌다.

## 왜 정답지 쪽수를 픽스처로 굳히나

Rust 시험에서 PDF 를 파싱하려면 새 의존이 필요하다. 이 축의 정답은 저장소 안에서 변하지
않으므로, 픽스처로 굳히고 재생성은 `tools/oracle_page_count/regenerate.py`(pypdfium2)에
맡긴다. CI 와 회귀 시험은 TSV 만 읽는다.

## 검증

### 게이트가 devel 에서 통과

```
정답지 쪽수 대조: 555개 / 일치 538 / 기존 격차 유지·개선 17 / 건너뜀 0
test result: ok. 1 passed. finished in 177.32s
```

### 게이트가 실제로 회귀를 잡는가 — 기준선을 조작해 확인

통과만 보면 그 시험이 회귀를 잡는지 알 수 없다. `basic/sungeo.hwp` 의 기준선을
86 쪽(정답지 94 대비 차 8)에서 **90 쪽(차 4)** 으로 바꿔, 현재값 86 이 "격차가 커진"
상황을 만들었다.

```
한글이 뽑은 쪽수와의 격차가 커졌다.
samples/basic/sungeo.hwp: 정답지 [94] 대비 격차가 커졌다 — 기준 90쪽(차 4) → 현재 86쪽(차 8)
test result: FAILED. 0 passed; 1 failed
```

어느 문서가 얼마나 벌어졌는지 실패 메시지가 그대로 낸다. 확인 뒤 기준선을 복원했다.

### 게이트 목록

| 명령 | 결과 |
| --- | --- |
| `cargo fmt --all -- --check` | 통과 |
| `node scripts/rust-unit-test-tiers.mjs --check --base-ref f6a6bee8f3` | 통과 (4221, 증가 없음) |
| `node scripts/rust-test-suite-manifest.mjs --check --base-ref f6a6bee8f3` | 통과 |
| `oracle_page_count_baseline::page_counts_do_not_drift_from_hancom_oracle` | 통과 (177.32s) |

### 커밋 대상

```
tests/cases/oracle_page_count_baseline.rs
tests/fixtures/oracle_page_count_baseline.tsv
tools/oracle_page_count/regenerate.py
mydocs/working/oracle_page_count_gate.md
```

`src/**` 변경은 없다 — 기존 파싱·조판을 그대로 두고 재는 것만 한다.

## 다음 — 불일치 17 건은 개별 결함이다

이 게이트는 격차를 **고정**할 뿐 줄이지 않는다. 17 건은 각각 별건으로 파고들 대상이고,
특히 같은 문서의 두 형식이 서로 다른 셋(편람 384/382, 온새미로 48/46, hwpx-02 9/6)은
형식 간 파싱 불일치의 직접 증거라 우선순위가 높다.

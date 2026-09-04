---
kind: working
status: active
issue: 6344
---

# 내용이 가득한 쪽을 빈 쪽으로 오판한다 (#6344)

## 증상

`samples/table-ipc.hwp` 는 10 쪽 문서인데 `layout-anomaly` 가 **8 쪽을 빈 쪽**으로 판정한다.
세 방향으로 교차 확인하면 전부 "내용 있음" 이다.

| 근거 | 결과 |
| --- | --- |
| 한컴 정답지 `pdf/table-ipc-2022.pdf` | 10 쪽 모두 861~1,026 자 |
| `rhwp export-text` | 10 쪽 모두 700~860 자 |
| 종전 `layout-anomaly` | **8 쪽이 `empty_page`** |

## 원인 — 용지 기준 표는 `Body` 밖에 있다

렌더 트리를 열면 바로 보인다.

```text
Page
├─ PageBg
├─ Header
├─ Body    bbox=(56.7, 94.5, 1009.1, 642.5)   글자 0개    <- 판정은 여기만 순회
├─ Table   bbox=(56.7, 94.5, 1008.4, 519.9)   글자 154개, 181칸
├─ Rect    '3/10'
└─ Footer
```

`scan_page` 의 콘텐츠 판정(`has_content`)은 `walk` 가 `Body` 서브트리를 돌 때만 세운다.
용지 기준으로 배치된 표는 페이지 직계 자식이라 그 순회에 걸리지 않는다.

## 같은 뿌리가 반복된다

`scan_page` 가 `Body` 하나에서 출발하는 설계가 여러 판정에 걸쳐 샌다.

| 축 | 증상 | 처리 |
| --- | --- | --- |
| 글자 겹침 후보 수집 | 본문↔바탕쪽 겹침이 신호 0 | #6318 / PR #6322 |
| **콘텐츠 유무 판정** | **빈 쪽 오탐** | 이 작업 |

## 수정 — 콘텐츠 판정 하나만 넓힌다

`page_has_visible_content` 를 두어 빈 쪽 판정에서만 페이지 전체를 훑는다.

- overflow·off-canvas·overlap 의 기준 상자(본문 여백·페이지 상자)는 **그대로**다.
  그 셋은 "본문 여백을 넘었나" 라는 뜻이 분명하다.
- 바탕쪽·머리말·꼬리말은 콘텐츠에서 **제외**한다. 배경·장식은 모든 쪽에 있으므로 그걸
  내용으로 세면 빈 쪽 판정 자체가 무의미해지고, 쪽번호만 있는 빈 쪽을 "내용 있음" 으로
  볼 수도 없다.

## 실측 — samples 945 건 전수

| 신호 | 수정 전 | 수정 후 | 차 |
| --- | ---: | ---: | ---: |
| **empty_page** | **133** | **72** | **-61** |
| overflow | 2,955 | 2,955 | 0 |
| off-canvas | 437 | 437 | 0 |
| overlap | 328 | 328 | 0 |
| text-overlap | 4,408 | 4,408 | 0 |

**나머지 네 신호가 정확히 0 변동**이다 — 이 변경이 다른 판정에 새지 않았다는 기계적 증거다.

빈 쪽 판정이 바뀐 문서 19 종 중 상위다.

```
 9 ->  0  issue4514/sample1-repro.hwp
 8 ->  0  hwpx/issue2019_floating_form_74312.hwpx
 8 ->  0  table-ipc.hwp
18 -> 11  2025 행정업무운영 편람(최종).hwp
18 -> 11  2025 행정업무운영 편람(최종).hwpx
 4 ->  0  issue5941/1490000-201600081_roadmap_research.hwp
```

남은 72 건은 이 수정으로 설명되지 않는다. 진짜 빈 쪽인지 다른 오탐인지는 별건이다 —
이 작업은 **`Body` 밖 내용을 못 보던 것** 하나만 고친다.

## 왜 중요한가

`empty_page` 는 모듈 머리말이 "가능성 신호" 로 분류해 `--strict` 에서도 실패를 유발하지
않는다. 그래서 CI 를 막지는 않지만, **advisory 리포트를 사람이 읽을 때 신뢰를 깎는다** —
10 쪽 중 8 쪽이 오탐이면 그 문서의 신호 전체를 무시하게 된다.

## 검증

### 시험이 결함을 실제로 잡는가 — 수정을 되돌려 확인

`src/diagnostics/layout_anomaly.rs` 만 stash 로 되돌리고 같은 시험을 다시 돌렸다.

```
test paper_anchored_table_page_is_not_reported_empty ... FAILED
  samples/table-ipc.hwp 는 한컴 정답지·export-text 모두 10쪽 전부 내용이 있다.
  빈 쪽으로 잡힌 쪽: [1, 2, 3, 4, 5, 6, 7, 8]
test other_verdicts_are_unchanged ... ok
```

두 번째 시험이 되돌린 상태에서도 통과하는 것은 정상이다 — 그 문서의 컨테이너 판정은
원래 (0, 0, 0) 이고, 그 시험의 목적은 **수정이 그 값을 건드리지 않았음**을 고정하는 것이다.

### 게이트

| 명령 | 결과 |
| --- | --- |
| `cargo fmt --all -- --check` | 통과 |
| `node scripts/rust-unit-test-tiers.mjs --check --base-ref f6a6bee8f3` | 통과 (4221, 증가 없음) |
| `node scripts/rust-test-suite-manifest.mjs --check --base-ref f6a6bee8f3` | 통과 |
| `issue_6344_empty_page_false_positive` | 2 통과 |
| samples 945 건 전수 재측정 | empty_page −61, 나머지 4 신호 0 변동 |

### 커밋 대상

```
src/diagnostics/layout_anomaly.rs
tests/cases/issue_6344_empty_page_false_positive.rs
mydocs/working/empty_page_false_positive.md
```

## 측정에서 겪은 함정

전수 재측정을 두 번 했다. 첫 번째는 **배치 스캔이 아직 실행 중인 JSON 을 읽어** overflow 가
2,955 → 1,714 로 42% 줄어든 것처럼 보였다. 파일이 738KB 였고 완료 시 3.59MB 다.

빈 쪽 판정 하나를 고쳤는데 overflow 가 그만큼 줄 리 없다는 점이 단서였다. **결과가 너무
좋으면 측정을 의심한다.** 완료 후 다시 재니 나머지 신호는 정확히 0 변동이었다.

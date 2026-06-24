# render-diff — 라운드트립 렌더 기하 정합성 비교 (Task #1499)

## 개요

`render-diff` 는 두 문서의 **페이지별 RenderNode bbox** 를 비교하여 라운드트립이 유발하는
**렌더 기하 변화**(페이지 수, 렌더 노드 삽입/삭제, 좌표 변위)를 정량화한다.

기존 `hwpx-roundtrip` baseline 은 **IR 뼈대(구조) 보존만** 검증하므로, IrDiff 0 이어도
라운드트립이 렌더 결과를 바꾸는 경우(예: 임베디드 이미지 누락)를 못 잡는다. `render-diff`
는 그 공백을 메우는 **폰트 비의존 결정론적** 시각 회귀 게이트다.

> 본 게이트는 "원본 IR 렌더 vs 라운드트립 IR 렌더" 의 **내부 정합성(회귀 방지)** 만
> 보장한다. 자기 roundtrip PASS ≠ 한컴 정답지 시각 충실도 (별개 축).
> 픽셀/SSIM 비교는 폰트·플랫폼 잡음이 커 후속(P1)으로 분리.

## 사용법

```bash
rhwp render-diff <a.hwpx>                       # 자기 라운드트립 (원본 vs 직렬화→재parse)
rhwp render-diff <a.hwpx> <b.hwpx>              # 두 파일 직접 비교
rhwp render-diff --batch <폴더> [-o <출력>]      # 폴더 전수 자기 라운드트립 → geom_inventory.tsv
rhwp render-diff <a.hwpx> --threshold 1.0       # 변위 임계 변경 (기본 0.5px)
```

- `--batch` 출력 기본 폴더: `output/poc/task1499`.
- 하드 실패(비-PASS) 존재 시 종료 코드 1.

## 비교 알고리즘

1. 각 페이지의 RenderNode 트리를 **전위순회 평탄화** → `(타입 태그, bbox)` 시퀀스.
2. 타입 태그 **LCS 매칭** (DP) — 삽입/삭제가 있어도 대응 노드 변위를 측정.
3. 매칭쌍 변위 = `max(|Δx|, |Δy|, |Δw|, |Δh|)` (px). 텍스트 내용·스타일은 비교 안 함.

## 판정 등급

| 등급 | 조건 |
|------|------|
| `PASS` | 페이지 수 일치 ∧ 노드 삽입/삭제 0 ∧ 최대 변위 ≤ 임계 |
| `STRUCT_MISMATCH` | 페이지 수 불일치 또는 렌더 노드 삽입/삭제 존재 (하드 실패) |
| `DISP_OVER` | 매칭 노드 변위가 임계 초과 (하드 실패) |

## 출력 예 (자기 라운드트립)

```
[STRUCT_MISMATCH] el-school-001.hwpx (자기 라운드트립)
                 pages: 1 → 1
                 노드 삽입 0 삭제 1 · 최대변위 6.813px (임계 0.500)
                 최대변위 위치: page 0 Path 6.813px
                 소요 6ms
```

(이 샘플은 삭제된 노드가 임베디드 Image 1개이고, 매칭 노드 중 최대 변위는 Path 6.8px다.)

## 배치 TSV (`geom_inventory.tsv`)

열: `sample · verdict · pages_a · pages_b · inserted · deleted · max_disp · elapsed_ms · error`.

## 회귀 게이트

`samples/hwpx` 전수 회귀: `cargo test --test visual_roundtrip_baseline`.

- 신규 샘플 자동 포함 — PASS 못 하면 사유와 함께 테스트의 `XFAIL` 에 등록.
- `XFAIL` 결함이 해소되어 PASS 하면 `xfail_entries_still_pass_promote` 가 실패 → baseline 승격.
- `EXCLUDED` 는 HWPX 패키지가 아닌 샘플.

## hwpx-roundtrip 와의 관계

| 게이트 | 검증 대상 | 테스트 |
|--------|----------|--------|
| `hwpx-roundtrip` (#1315) | IR 뼈대(구조) 보존 | `hwpx_roundtrip_baseline` |
| `render-diff` (#1499) | 렌더 기하(시각) 회귀 | `visual_roundtrip_baseline` |

후자가 전자의 상위 단계다 — 구조는 같아도 렌더가 달라지는 회귀를 잡는다.

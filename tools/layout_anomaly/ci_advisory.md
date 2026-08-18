# layout-anomaly CI advisory 잡

MEGA QUEUE M02-7 (#5397). `rhwp layout-anomaly` 를 CI 에 **advisory** 잡으로
올리는 설계다. 이 문서는 제안이다. 병합만으로 PR 게이트를 바꾸지 않는다.

`scripts/visual_sweep.py` 는 읽거나 수정하지 않는다. gym 은 쓰지 않는다.
`--strict` 금지 — 이상 신호가 종료 3 이 되어 빨간 엑스처럼 보이면 안 된다.

초안 워크플로: [`.github/workflows/layout-anomaly-advisory.yml`](../../.github/workflows/layout-anomaly-advisory.yml).
트리거는 `workflow_dispatch` 와 nightly (`17 3 * * *`) 이다. `pull_request`
블록은 주석으로만 남겨 두었다.

## 1. 목적과 비목표

### 목적

- 커밋된 소표본(`tools/layout_anomaly/advisory_samples.txt`)에
  `rhwp layout-anomaly --json` 을 돌린다.
- M02-8 배치 리포트 스크립트(`tools/layout_anomaly/batch_report.py`)가 있으면
  그걸 우선한다. 없으면 소표본 CLI 루프다.
- 이상 신호는 데이터가 된다. 요약·행 JSON 을 아티팩트로 올려 사람이 본다.

### 비목표

| 하지 않는 일 | 이유 |
| --- | --- |
| required check / branch protection 등록 | PR 게이트를 막지 않는다 |
| `scripts/visual_sweep.py` 호출·수정 | jangster77 금지, 픽셀 전수는 이 잡 범위 밖 |
| `--strict` | overflow·overlap 이 있으면 종료 3 |
| gym | 이 클레임 범위 밖 |
| `samples/` 전수 | 그건 M02-8. 이 잡은 소표본 |
| `local_validation.md` 수정 | 등록은 후속 제안 이슈 |

## 2. 왜 advisory 인가

1. 소표본에도 이미 알려진 overflow/overlap 이 있을 수 있다. 지금 강제 게이트로
   켜면 devel PR 이 한꺼번에 막힌다.
2. 판정은 데이터다. `layout-anomaly` 기본(비 `--strict`)은 신호가 있어도 종료 0.
3. 렌더 트리 스캔은 dump-pages 보다 무겁다. hang 이 게이트를 잡아먹으면 안 된다.

운영 등급은 [github_operations.md](../../mydocs/manual/github_operations.md) 의
**O2 (라우팅·비용)** 이다. required check 변경은 O4 이고 이 제안 범위가 아니다.

## 3. 잡 계약

| 항목 | 값 |
| --- | --- |
| workflow 파일 | `.github/workflows/layout-anomaly-advisory.yml` |
| workflow 이름 | `Layout Anomaly Advisory` |
| job id / 이름 | `advisory` / `layout-anomaly-advisory` |
| 러너 | `ubuntu-latest` |
| `timeout-minutes` | 30 (성능 목표 아님. hang 상한) |
| 내부 벽시계 | `timeout 18m` — 잡 timeout 전에 부분 리포트를 남긴다 |
| permissions | top-level `contents: read` 만 |
| concurrency | `layout-anomaly-advisory-${{ github.ref }}`, cancel-in-progress |
| `continue-on-error` | `true` (이중 안전장치) |
| 비교 종료 코드 | 신호여도 0. `--strict` 금지 |
| required checks | **등록 금지** |

현재 required context (Lint / Build & Test / CodeQL / Render Diff 등) 와
이름을 겹치지 않게 `layout-anomaly-advisory` 를 쓴다.

## 4. 트리거

1단계: `workflow_dispatch` + nightly. `limit` 입력(기본 `0` = 목록 전수).
PR synchronize 마다 돌지 않는다.

2단계(주석): `pull_request` 를 풀어도 **required 등록은 하지 않는다**.
`--strict` 는 여전히 넣지 않는다. `ci.yml` `needs:` 에 연결하지 않는다.

## 5. 소표본

`advisory_samples.txt` — 커밋된 작은 HWP/HWPX 8개(본문·서식·표·각주·빈 HWPX).
없는 파일은 skip. 목록이 비면 `samples-absent` 로 종료 0.

## 6. 실행 명령

```text
timeout 18m python3 tools/layout_anomaly/advisory_run.py \
  --rhwp target/release/rhwp \
  --out layout-anomaly-advisory
# --strict 없음
```

`advisory_run.py` 는 배치 리포트가 있으면 그걸 호출하고, 없으면 소표본마다
`rhwp layout-anomaly --json <파일>` 을 돌린다. 러너 종료는 항상 0 이다.

## 7. 활성화·롤백

활성화: 이 PR 병합 → Actions 에서 1회 수동 실행 → 요약을 본다.
롤백: 워크플로 파일 한 개를 지운다. required checks 를 건드리지 않았으므로
protection rollback 은 없다.

로컬:

```text
cargo build --release --bin rhwp
python tools/layout_anomaly/advisory_run.py --rhwp target/release/rhwp --out /tmp/la
```

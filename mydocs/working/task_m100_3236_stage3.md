---
kind: working
status: active
canonical: mydocs/plans/archives/task_m100_3236.md
last_verified: 2026-08-01
---

# Task #3236 Stage 3 보고 — 전체 게이트 검증

## 게이트 결과 (전부 통과)

| 게이트 | 결과 |
|---|---|
| `cargo test --profile release-test --tests` 전체 | **exit 0** (92셋·golden 포함 전 스위트) |
| `cargo clippy --all-targets -- -D warnings` | **exit 0** |
| Native Skia 3종 (skia --lib · #2225 · p37 direct PDF) | **exit 0** |
| wasm Docker 재빌드 | **성공** — `pkg/rhwp_bg.wasm` 갱신 확인 |
| IR field sweep (4.3.1, 신규 fixture) | **신규 발산 0건** + 래칫 2행 조임(아래) |
| samples 664건 쪽수 A/B (수정 전/후 바이너리) | **차이 0건** |
| studio CDP 잉크 재측정 | p2 하단 **0.02% → 14.01%** |
| 정답지 PDF 이미지 스왑 | 세트 생성·구조 일치 (아래) |

## 정답지 이미지 스왑 대조

한컴 2020 PDF(96dpi 래스터)와 rhwp SVG 래스터를 동일 치수(794×1123)로 나란히 대조:

- **p1**: 표 시작 위치·본문 흐름 일치. **셀 절단점 동일** — 양쪽 모두
  "…기업설립일로부터 5년이"에서 쪽이 끝난다.
- **p2**: 분할 셀 박스(테두리 포함)·이어지는 내용·후속 표(주관연구개발기관 유형)·
  각주까지 구조 일치. 시작 텍스트 "경과되지 않은 외국인투자기업인…" 동일.
- 참고 픽셀 diff(fuzz 8%): p1 15.0% / p2 18.3% — 폰트 렌더 차이가 포함된 수치로
  참고용. 판정 권위는 작업지시자 시각 판정.

판정 세트: `/mnt/e/hwp/swap3236/` (10파일 — hancom/rhwp/수정전 개별 + 나란히 쌍
`pair-p{1,2}-hancom-vs-rhwp.png`, `pair-p{1,2}-before-vs-after.png`)

## samples 쪽수 A/B 상세

기준(특례 상한 제거) 바이너리와 수정 바이너리로 samples 전수(hwp/hwpx 664건,
암호 fixture 3건 제외) `info --json` 쪽수 대조.

- **차이 0건.** 수정의 행동 변화는 #3236 fixture 에 국한된다(그 문서도 쪽수는 2 유지,
  분할만 복원).
- 1차 스윕에서 `hwpctl_ParameterSetID_Item_v1.2.hwp` 73→74 로 보였으나 **하니스
  아티팩트로 판명** — 단독 5회 재실행 전부 73, 양 바이너리 SVG 73장 바이트 동일,
  재스윕 차이 0건. 병렬(-P 8) 출력 인터리빙이 원인으로 추정된다. 스윕 하니스를
  판정에 쓸 때는 상이 건을 단독 재실행으로 재확인해야 한다는 교훈.

## studio CDP 재측정 (새 wasm 반영 후)

| 밴드 | 수정 전 | 수정 후 |
|---|---|---|
| p1 하단 33% | 17.72% | 13.12% (내용 p2 이동) |
| p2 중단 33% | 2.06% | **16.27%** |
| p2 하단 33% | **0.02% (백지)** | **14.01%** |

## IR sweep 래칫 조임 (동반 정비)

신규 fixture(`samples/task3236/issue3236_split_table.hwpx`)의 왕복 발산은 0건 —
baseline 행 추가 없음. 대신 baseline 에 남아 있던 `HWP5-nopassword-123456.hwp` 의
`extra_child_records.len` 2행이 현재는 발산하지 않아 제거(래칫 한 칸 조임).
개선 출처는 `42e1f125d`(#3507/#3519, HWP5 손상 수정) — 당시 래칫 미조임분.
스윕 2회 결정성 확인(dump 바이트 동일).

## 남은 단계

Stage 4 — **작업지시자 시각 판정**(`/mnt/e/hwp/swap3236/` 및 studio 실기동) →
최종 보고 → PR 준비(생성은 별도 승인).

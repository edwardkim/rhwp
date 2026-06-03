# Task #404: 헤딩 문단이 후속 콘텐츠와 다른 페이지로 분리됨 — 수행계획서

## 목표

샘플 `samples/2025년 기부·답례품 실적 지자체 보고서_양식.hwpx` 9쪽 하단의 헤딩 "(7) 다수 기부자 현황"(pi=83)이 후속 표(pi=84 7x3, pi=85 3x3)와 분리되는 orphan 문제를 해소한다. HWP 원본 vpos 기준으로는 pi=83이 9쪽 본문 영역을 초과하므로 10쪽으로 가야 정상.

## 재현

```bash
rhwp export-svg "samples/2025년 기부·답례품 실적 지자체 보고서_양식.hwpx"
rhwp dump-pages "samples/2025년 기부·답례품 실적 지자체 보고서_양식.hwpx" -p 8
```

현재 9쪽 (Task #402 적용 상태):
```
items=8, used=930.1px (body 933.5px - 거의 가득)
  ...
  Table          pi=82 ci=0  2x1  vpos=591671
  FullParagraph  pi=83  h=14.7  vpos=601325  "(7) 다수 기부자 현황"   ← 헤딩
```

10쪽 시작:
```
  Table   pi=84 ci=0  7x3  vpos=603085   ← 헤딩 후속 데이터
  Table   pi=85 ci=0  3x3
```

## 원인 (HWP vpos 분석)

HWP 원본 vpos 기준 9쪽 본문 영역:
- body 높이 247mm = 70015 HU
- 9쪽 첫 item vpos = 532187 → 9쪽 vpos 범위 = [532187, 602202]

pi=83 위치:
- 시작 vpos = 601325
- 높이 ≈ 14.7px ≈ 1100 HU
- 끝 vpos = 601325 + 1100 = **602425**

`602425 > 602202` → pi=83은 vpos 기준 9쪽을 넘어감. HWP는 문단 단위 atomic 분할이므로 전체를 10쪽으로 push해야 함.

우리 페이지네이션은 **누적 높이 기반** (`current_height + para_height vs available_height`):
- 9쪽 used = 930.1, 헤딩 14.7 추가 → 944.8 > body 933.5 (살짝 초과)
- 그러나 헤딩 자체는 단순 텍스트라 분할이 일어나지 않고, 14.7만큼 아래로 그려져 본문 영역을 미세 초과하지만 SVG는 잘 보임. 결국 의도와 어긋난 페이지에 배치.
- HWP vpos 진행과 누적 높이 진행 사이에 미세 drift 존재 (`hwp_used 8032px vs used 930px` — vpos 단위 차이는 문단 사이 빈 vpos gap 때문).

## 가설 (사전 조사)

1. **HWP는 문단마다 vpos를 가지고 있어 결정적 페이지 배치**가 가능. 우리 엔진은 height 누적으로 근사하므로 누적 오차로 인한 마진 차이.
2. `engine.rs`에는 부분적 vpos 보정 로직(line 245~285)이 있으나 "block table 존재 + prev_para 조건" 등 좁은 조건에서만 동작. `typeset.rs`(기본 경로)에는 없음.
3. ParaShape `keep_with_next`는 파싱돼 `style_resolver.rs::ResolvedParaStyle`에 저장되지만 어떤 페이지네이션 코드도 사용하지 않음. 본 샘플 ps_id=11은 `keepWithNext="0"`이라 이 속성으로는 해결 불가.

## 접근 방향 후보

| # | 접근 | 장점 | 단점 |
|---|------|------|------|
| **A** | vpos 기반 fit 결정: 각 문단 처리 시 `para_vpos_end > page_vpos_top + body_h_in_vpos`이면 강제 페이지 분할 | HWP와 동일 의미 — 결정적, 정확. 일반 케이스 모두 커버 | `engine.rs`의 부분 vpos 보정과 상호작용 검토 필요. wrap-around·multi-column 등 예외 케이스 검증 필요 |
| B | keep_with_next 속성 사용 | 의도가 명확 | 본 샘플은 doc에 0으로 설정 — 본 케이스 해결 불가 |
| C | implicit "heading-like" 휴리스틱 (small h + 후속 큰 block) | 빠른 적용 | 휴리스틱 — false positive 위험 |

→ **A 채택** (정확성 + HWP 의미 일치).

다만 적용 범위는 본 케이스에 국한해서 안전하게 도입한다. 무조건 vpos 우선이 아니라, 누적 높이로 fit 판단 후 vpos 기준 추가 검사로 분할을 강제하는 보정 형태.

## 수행 단계 (요약)

1단계 — 진단: vpos 추적 + 디버그 로그로 pi=83 시점의 `current_height`, `vpos_end`, `page_vpos_top + body_h_in_vpos` 값을 확정. `typeset.rs::typeset_section`의 page boundary 추적 가능 여부 확인.

2단계 — 구현: `typeset.rs`의 문단 처리 루틴에 vpos 기반 분할 검사 추가. 페이지 첫 문단의 vpos를 `page_vpos_top`으로 기록, 이후 각 문단의 `vpos + estimated_height_in_hu > page_vpos_top + body_h_in_hu`이면 페이지 분할.

3단계 — 검증: 타겟 샘플 9쪽/10쪽 확인 + 회귀 테스트 + 10개 대표 샘플 spot check.

## 비범위

- `engine.rs(paginate_with_measured)` fallback 경로의 동일 누락 — 별도 후속 이슈
- TAC 표/그림 height 누적 자체의 정확도 개선은 #402가 부분적으로 다룸. 본 타스크는 vpos 기반 final-fit 검사 추가에 한정
- HWPX 외 HWP 5.x 바이너리 케이스 — 동일 vpos 메타가 LineSeg에 있으면 동작. 없거나 부정확한 경우는 본 타스크 범위 밖

## 검증 기준

1. 9쪽 SVG에 pi=83 헤딩이 더 이상 표시되지 않음 (10쪽으로 이동)
2. 10쪽 SVG가 pi=83 헤딩 + pi=84/85 표를 함께 표시
3. 기존 회귀 테스트 모두 통과
4. 10개 대표 샘플 LAYOUT_OVERFLOW 카운트 회귀 없음

## 전제 조건

본 타스크는 #402가 적용된 상태에서 의미가 있다. #402 미적용 상태에서는 페이지 배치 자체가 다름 (orphan이 다른 위치에서 발생). 따라서 #402 PR이 먼저 merge된 devel을 base로 작업한다. 현재 `local/task404` 브랜치는 devel 기반이므로 #402 merge 후 rebase 필요.

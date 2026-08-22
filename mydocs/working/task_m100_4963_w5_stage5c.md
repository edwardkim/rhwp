---
kind: working-note
status: completed
issue: 4963
stage: W5-5C
last_verified: 2026-08-22
---

# Task M100 #4963 W5 Stage 5C — rank 8 distinct-substitution ladder

- **이슈**: [#4963](https://github.com/edwardkim/rhwp/issues/4963)
- **계획**: [`task_m100_4963.md`](../plans/task_m100_4963.md)
- **선행 단계**: [`task_m100_4963_w5_stage5b.md`](task_m100_4963_w5_stage5b.md)
- **단계 상태**: three-state 실행·복원·profile·queue 정규화·메인테이너 시각 판정 완료

## 1. 실행 계약

rank 8 문서 face `KoPubWorld바탕체 Light`와 별개의 공식 font
`KoPubWorld돋움체 Light`를 fixture-declared substitution으로 사용했다. 이 관계는 identity, alias,
official successor 또는 metric surrogate를 뜻하지 않는다. 세 물리 상태는 byte-identical HWPX fixture를
사용했다.

| 상태 | exact Batang | declared Dotum substitution | 목적 |
| --- | --- | --- | --- |
| exact-only | 있음 | 없음 | exact source glyph·advance anchor |
| subst-only | 없음 | 있음 | 문서 substitution의 실제 사용 여부 |
| none-related | 없음 | 없음 | 한컴 missing-font 선택 |

private corpus, 제품 metric DB·fallback·paint는 사용하거나 변경하지 않았다. font bytes와 원시 VM·경로
정보는 저장소 밖 local-only 증거로 유지한다.

## 2. 실행과 복구

각 상태 전에 같은 기준점으로 복원하고, 관리 font 집합 이외의 projection이 동일한지 확인했다. exact와
substitution 상태는 각각 선언된 SHA-256 한 종만 관리 집합에 포함했고 none-related는 0종이었다. 모든
상태에서 HWPX open, 1 page PDF export, security module, process reset과 private corpus 미접근을 확인했다.

첫 orchestration의 none-related manifest guard는 빈 배열을 `Compare-Object`에 전달해 중단됐다. 외부
`finally` 복원 뒤 baseline manifest, unrelated projection, 관리 font 0개, HWP·일회성 task 0개를 독립
검증했다. 이미 완료된 exact-only와 subst-only는 재실행하지 않았고 none-related만 빈 집합 전용 guard로
실행했다. 마지막에도 같은 기준점 복원을 다시 확인했다.

## 3. 관측 결과

| 상태 | 선택 가능한 영문 alias | PDF subset | U+AC00 source `hmtx` | PDF advance |
| --- | --- | --- | ---: | ---: |
| exact-only | `KoPubWorldBatang Light` | `KoPubWorldBatangLight` | 936/1000 | 7.454008 |
| subst-only | `KoPubWorldDotum Light` | `HCRBatang-Bold` | 미연결 | 7.774934 |
| none-related | 없음 | `HCRBatang-Bold` | 미연결 | 7.774934 |

핵심은 substitution font가 설치되어 선택 가능하다는 사실만으로 HWP가 문서의 `substFont`를 조판에
사용하지 않았다는 점이다. subst-only와 none-related의 PDF file hash는 실행 metadata 때문에 다르지만,
font·glyph·advance·position·line을 투영한 typesetting projection은 동일하다.

4개 semantic profile과
[`oracle_stage5_rank8_acceptance_ladder.json`](../tech/investigations/issue-4963/oracle_stage5_rank8_acceptance_ladder.json)을
생성했다. queue projection은 W5-5C로 갱신했고 실행 가능한 rank는 더 이상 남지 않는다.

## 4. 시각 증거와 승인 게이트

local-only side-by-side 비교는 exact-only, subst-only, none-related 순서로 고정했다. exact-only는 같은 한
쪽 틀 안에서 더 가늘고 폭이 다른 Batang 조판을 보이며, subst-only와 none-related는 육안으로도 같은
HCR fallback 조판이다. 비교 이미지 SHA-256은
`9d4da59dfaba6f4dcb0fd06e1268fcb490690c66998bd0574df607f376bb90cb`이다.

메인테이너는 2026-08-22 side-by-side 비교를 시각 승인했다. exact-only의 더 가늘고 폭이 다른 조판과
subst-only·none-related의 동일한 HCR fallback 외형이 기계 projection과 일치했으므로 W5 최종 보고서로
이관한다. 이 synthetic fixture에서는 세 상태가 모두 30줄·1쪽에 머물렀으므로 실제 overflow나 페이지
증가는 관찰값이 아니라 후속 제품 검증이 필요한 위험으로 구분한다.

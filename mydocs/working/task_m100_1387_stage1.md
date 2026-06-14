# Task M100 #1387 — 1단계 완료 보고서 (전수 측정 + vert_align 조사)

- 브랜치: `local/task1387`
- 작성일: 2026-06-12
- 코드 수정: 없음 (조사 전용)
- 산출물: `output/poc/task1387/caption_inventory.tsv`

## 1. caption 분포 측정 (1.1)

`samples/hwpx` 전수 54파일에서 `hp:caption` 출현: **5파일 6섹션, 총 17건**.
부모 개체 역방향 정밀 분류:

| 파일 | 표 캡션 | 그림/도형 캡션 |
|------|--------|---------------|
| 143E433F503322BD33 | 1 | - |
| aift (section2) | 2 | pic 8 + line 3 = 11 |
| exam_social (section1) | 1 | - |
| mel-001 | 1 | - |
| ta-pic-001-r | 1 | - |
| **합계** | **6** | **11** |

RT(#1388 산출물) 대조: RT 보유 4파일 모두 caption **0건** — 표·그림 캡션 전량 소실 확인.
(exam_social은 #1384 SERIALIZE_FAIL xfail로 RT 부재)

속성·내부 구조 변종 조사 (전수 17건):

- `hp:caption` 속성 골격 **전수 동일**: `side, fullSz, width, gap, lastWidth` 5종.
  side 분포: BOTTOM 13 / TOP 4 (LEFT/RIGHT 실재 샘플 없음 — 역매핑은 4종 모두 구현)
- caption `hp:subList` 속성 **전수 동일**: `vertAlign=TOP lineWrap=BREAK
  textDirection=HORIZONTAL` — 구현계획서 0.1의 고정 방출 방침에 변종 위험 없음
- 캡션 내부 컨트롤: autoNum **1건** (ta-pic-001-r, 단일 run 문단) — #1382(autoNum 폭
  비일관)는 run 경계 시프트 증상이므로 단일 run에서는 간섭 위험 낮음

## 2. vert_align 조사 종결 (1.2)

- 전수 17건의 `hp:caption` 속성에 수직 정렬 계열 속성 **부재** (골격 5종 외 속성 없음).
- IR `Caption.vert_align`은 HWP5 바이너리(ctrl header) 유래 필드로 확정 — HWPX 파서
  미적재는 공백이 아니라 포맷 차이. **파서 보강 불요 판정.**
- 캡션 세로 정렬 의미는 subList `vertAlign`(전수 TOP)이 담당 — 고정 방출로 보존된다.

## 3. 그림/도형 캡션 범위 판정 (1.3)

- aift의 pic 8 + line 3 = 11건이 표 외 캡션 — 본 타스크(표 한정) 범위 밖.
- 그림/도형(`ShapeComponent.caption`) 직렬화 경로는 표와 별개(shape.rs) —
  **별도 이슈 분리 제안** (4단계 최종 보고서에서 등록 승인 요청 예정).

## 4. 게이트 동승 영향 사전 판정 (1.4)

- 표 캡션 보유 5파일 중: 143E(#1382 IR_DIFF xfail 기존), exam_social(#1384 xfail 기존)
  — 둘 다 본 타스크와 무관하게 이미 xfail.
- 나머지 3파일(aift, mel-001, ta-pic-001-r)은 2단계 수정 후 caption 재방출 → 재파싱
  대칭이 기대치 → **신규 xfail 0 예상.** (4단계 전수에서 재확인)

## 5. 다음 단계

2단계 — 공유 헬퍼 추출 + `write_caption` 신설 + 단위 테스트 4종.

승인 요청드립니다.

---
kind: report
status: active
canonical: mydocs/plans/archives/task_m100_3307.md
last_verified: 2026-08-01
---

# Task #3307 최종 보고 — 정의 없는 개요 자동번호의 한컴 기본 모양 fallback

- Issue: [#3307](https://github.com/edwardkim/rhwp/issues/3307) (M100) — 외부 리포터
  `Neurlect` (2026-07-25, rhwp@0.7.19 실측 제보 + 45건 정부 문서 벤치마크)
- 브랜치 `local/task3307` / 2026-08-01 당일 완결
- 단계 기록: `mydocs/working/task_m100_3307_stage{1,2,3}.md`
- 연결: [#3308](https://github.com/edwardkim/rhwp/issues/3308) (동일 문서 p7 별개
  증상 2건 — 별도 착수 결정)

## 증상과 근인

별지 서식 표의 개요 자동번호 1.~4.가 렌더에서 소실됐다(5.~6.은 리터럴이라 생존).
현행 devel 재현 — 잔존 결함.

근인: 문서에 개요 번호 모양 정의가 없다(`<hh:numbering>` 0개,
`outlineShapeIDRef="0"`). 한컴은 이 경우 **편집기 내장 기본 모양**을 적용하지만
rhwp 는 정의 부재 시 `return None` 으로 번호를 그리지 않았다. 한컴 HWP5 재저장
실험으로 기본 모양이 **파일에 실체화되지 않는 편집기 동작**임을 확증 — HWP5 도
동일 결함 대상이었다(렌더러 공유로 함께 해소).

## 기본 모양 권위 확정 (추정 금지 규약 준수)

fixture 의 기존 paraPr 7종에 level 0~6 개요를 주입한 수준 스윕 문서를 한컴 2020
MCP 로 PDF 화해 직접 판독: **전 수준 `^N`(레벨 경로 + 후행 마침표, 아라비아,
시작 1)** — level 0 `1.` ~ level 6 `2.5.1.1.1.2.1.`. 원본 정답지 p7(1.~4.)과 교차
정합. rhwp 는 `^N` 확장을 이미 구현하고 있어 수정은 **합성 기본 Numbering 의
fallback 배선**만 필요했다.

## 수정 (~30줄)

- `layout/utils.rs` — `default_outline_numbering()`: 전 수준 `^N` 합성.
- `layout/paragraph_layout.rs` — 번호 합성 지점의 정의 부재 두 경로를 통합,
  **`HeadType::Outline` 한정** fallback. NUMBER/BULLET/None 불변. 카운터는 기존
  `numbering_state.advance(0,…)` 재사용(실정의 id 1-based 라 0 키 충돌 없음).

## 검증 (전부 통과)

| 게이트 | 결과 |
|---|---|
| fixture p7 | 1.~4. 복원 + 5.~6. 리터럴 불변 — 정답지와 전 항목 정합, 쪽수 9 불변 |
| red-check | fallback 제거 시 정확히 FAILED (`issue_3307_outline_default_numbering.rs`) |
| 과발동 가드 | 비개요 리터럴 문단 이중 번호 없음 — 테스트 고정 |
| 인접 번호/개요 계열 | 테스트 파일 9종 18건 + lib 30건 통과 |
| release-test 전체·clippy·Skia 3종·wasm·fmt | exit 0 |
| samples 666건 쪽수 A/B | **차이 0건** |
| 이중 baseline (4.3.1 신판 첫 적용) | IR 발산 0 + overflow 원장 신규 0 |
| 정답지 이미지 스왑 + **작업지시자 시각 판정** | **통과** (2026-08-01) — 자산 `mydocs/report/assets/task3307_*.png`, [이슈 embed](https://github.com/edwardkim/rhwp/issues/3307#issuecomment-5148855317) |

## 리포터 커뮤니케이션

- [선회신](https://github.com/edwardkim/rhwp/issues/3307#issuecomment-5148742986)
  (Stage 1 — 재현 확인·원인·벤치마크 감사)
- [이미지 스왑 비교](https://github.com/edwardkim/rhwp/issues/3307#issuecomment-5148855317)
  (Stage 3 — 수정 검증 시각 증적)
- merge 후 완결 회신 예정 (적용 버전·확인 방법).

## 교훈

1. **권위 실측의 재사용성** — MCP 한컴 변환이 "정답지 생성"을 넘어 "기본값 리버스
   엔지니어링"(수준 스윕)까지 커버했다. 편집기 내장 동작도 실측 가능하다.
2. **상주 stash 함정 실증** — clean 트리 `stash`/`pop` 이 상주 stash 를 꺼내 충돌.
   메모리의 경고가 실제로 재현된 첫 사례. baseline 은 임시 라인 제거 방식으로.
3. red-check 원복은 수정 **커밋 후** `git checkout` — 미커밋 상태 원복은 수정을
   지운다.
4. 한컴의 paraPrIDRef 인덱스 해석 정황(신규 id 미해결) — HWPX 직렬화 시 id 연속성
   관찰, 후속 기록 대상.

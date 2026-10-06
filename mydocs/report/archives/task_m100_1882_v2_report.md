# task_m100_1882_v2 최종 결과보고서 — 원형(단일 시리즈) 자동 제목 후속

- 이슈: #1882 (PR #1890 merge 시 메인테이너가 OPEN 유지한 잔여 항목)
- 브랜치: `local/task1882_v2` (from `local/devel` = upstream/devel 782bf704)
- 마일스톤: M100
- 상태: 구현·검증 완료, 작업지시자 확인 대기 (studio dev 서버 기동)

## 1. 목표 / 결과

한컴 `2차원원형` 정답지의 제목이 "판매"인데 rhwp는 placeholder "차트 제목"으로 렌더하던
잔여 갭 해소. **규칙 실측 확정: 자동 제목 = 시리즈가 정확히 1개이고 이름이 비어있지
않으면 그 이름, 아니면 "차트 제목"** — 차트 종류(원형) 특칙이 아니라 시리즈 수 기준
(Excel 자동 제목 동작과 동일).

## 2. 근거 실측

| 실측 | 결과 |
|---|---|
| 원형 계열 5종 정답지 (2차원·3차원·쪼개진·원형대원형·원형대가로막대형) | 제목 전부 **"판매"** = XML 단일 `c:ser`의 이름. `c:title` 텍스트 없음 + `autoTitleDeleted=0` |
| **비원형 교차 검증** (작업지시자 제공 특이케이스: 단일 시리즈 "계열 1" **가로막대**) | 정답지 제목 = **"계열 1"** → 시리즈 수 기준 일반 규칙 확정 |
| 다계열 코퍼스 (묶은세로 3·꺽은선 3·scatter 2 등) | 전부 "차트 제목" placeholder (C1c 실측) — 규칙과 정합 |

부수 관찰(범위 밖, C2 축 정밀화 후보): 특이케이스 정답지 가로축이 0~5 **0.5 간격(10칸)**
— 기존 가로축 실측 4앵커(12.3→step2 / 5.0→step1 / 2.6→step0.5)와 단일 규칙 불성립,
1카테고리 미니 차트 특수 동작 추정. 섣부른 반영은 기존 앵커 파손 위험이라 기록만.

## 3. 변경 / 단계

수정: `src/ooxml_chart/renderer.rs`(effective_title 한 단계 삽입) + 통합 테스트 보강 +
픽스처. **모델·파서 무변경** (`ser.name` 기존 파싱 재사용).

| 단계 | 내용 | 커밋 |
|---|---|---|
| 계획 | 수행계획서 + 특이케이스 픽스처(`samples/chart/특이케이스/` hwp·hwpx, 정답지는 명명 규약대로 `pdf/chart/특이케이스/{stem}-2022.pdf`) | `e185d2c8`, `22a0d6c9` |
| Stage 1 | `effective_title`: 명시 텍스트 → **단일 시리즈 이름** → "차트 제목" (게이트 유지). 단위 2 신규 — TDD 선실패 확인, `title_text` 헬퍼(font-size 13)로 범례의 시리즈 이름 오염 차단 | `5296c3e8` |
| Stage 2 | 통합 `chart_auto_title_rendered` 보강: 2차원원형을 placeholder 그룹→"판매" 그룹 이동, 원형대원형·특이케이스("계열 1") 추가, 다계열 placeholder 유지 가드 | `669a34e9` |
| Stage 3 | 전체 검증 + 시각검증 + 본 보고서 | (본 커밋) |

경계 케이스 커버(단위 테스트): 단일+이름 빈 문자열→placeholder / 명시 제목 우선 /
autoTitleDeleted=1→제목 없음 / 다계열→placeholder.

## 4. 검증

```
cargo test --lib ooxml_chart                     → 56 passed (기존 54 + 신규 2)
cargo test --test issue_1882_chart_style_gaps    → 4 passed (제목 규칙 hwp+hwpx 가드)
cargo test --test issue_1431_scatter / issue_1453 → 무회귀
cargo test (전체)                                 → 206개 테스트 바이너리 전부 green (실패 0)
cargo clippy --all-targets -- -D warnings         → 경고 0
```

**시각검증** (`output/poc/chart_c1c_v2/` ↔ 정답지): 2차원원형 제목 "판매", 특이케이스
가로막대 제목 "계열 1" — 정답지와 일치. 다계열 차트 placeholder 유지.
WASM 재빌드 + studio dev 서버로 작업지시자 확인 제공.

## 5. 후속

- 본 보고서 승인 후 origin(fork) push → upstream `devel` PR (`Refs #1431`, `#1882`) —
  #1882 close 여부는 메인테이너 판단.
- 잔여(범위 밖): 범례 순서 역전 규칙(C2), 특이케이스 축 0.5 간격(C2), stock·3D 입체(C2).

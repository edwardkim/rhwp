# Task M100 #3931 — Stage 4 광범위·플랫폼·시각 검증

- 날짜: 2026-08-15 KST
- 기준: Stage 3 commit `fc42b11f7`
- 후보: `task/3931-declared-rowbreak`
- 상태: 자동 검증 완료, 작업지시자 시각 판정 대기

## 직접 문서 결과

한컴 2020 KoPub 설치 PDF는 383쪽이다. 후보의 HWP는 392쪽, HWPX는 386쪽이다. 총쪽수는
여전히 다르지만 #3931의 직접 대상은 총쪽수 상수가 아니라 물리 조각 소유권이다.

- 질문 7(`pi=14`): HWP 답변 head/tail이 page index 286/287로 나뉜다. 한컴 PDF 물리
  p284/p285와 같이 현재 쪽에서 답변을 시작하고 다음 쪽으로 이어진다.
- 질문 11(`pi=23`): 저장 24.5px pitch와 16줄 전체 높이를 유지한 채 page index 290의
  12줄과 index 291의 4줄로 나뉜다. 한컴 PDF 물리 p287/p288의 12+4줄 소유와 같다.
- HWPX는 386쪽과 기존 fragment scanner 경로를 유지한다. 질문 7에서 남은 한 문단 차이는
  진입 전 누적 flow 높이 축이므로 native HWP5 저장-anchor 특례를 적용하지 않았다.

## 전체 회귀에서 발견해 보완한 결함

첫 `release-test` 전체 실행은 5,941건 중 19건이 실패했다. 실패는 #2007·#4252·#4272 중첩 표
경로와 overflow 기준선에 집중됐고, 원인은 Stage 2의 이전 host margin 회수 코드가 partial table에
전달된 local paragraph slice를 root `para_index`로 조회한 뒤 `expect`로 panic한 것이었다.

partial table에서는 그 인덱스가 local slice에 없을 수 있다. host를 찾지 못하면 margin 회수를
적용하지 않는 fail-closed lookup으로 바꿨다. 이후 #2007 15건, #4252 5건, #4272 2건과 관련
부분집합·overflow 기준선이 모두 통과했고, 전체를 새로 실행해 다음 결과를 얻었다.

- `cargo nextest run --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 12 --no-fail-fast`
  — 5,941/5,941 통과, 38 skip
- `cargo clippy --all-targets -- -D warnings` — 통과
- `issue_3931_declared_rowbreak` — 5 통과, 전체 383쪽 오라클 ledger 1건 ignore
- `issue_3738_rowbreak_table_footnote_fragment` — 33/33 통과
- 계획에 명시한 #3930·#874·#2097·#2105·#2439·#3236·#1156·#1748 focused 회귀 — 전건 통과

## 비공개 10k 코퍼스 비교

비공개 로컬 코퍼스는 원본·경로·파일명을 외부 증적에 노출하지 않고 기준 `upstream/devel`과
후보 바이너리를 같은 입력 10,000건에 적용했다.

| 항목 | 기준 | 후보 | 변화 |
| --- | ---: | ---: | ---: |
| 전체 | 10,000 | 10,000 | 0 |
| HWP / HWPX | 6,582 / 3,418 | 6,582 / 3,418 | 0 |
| 성공 / 오류 | 9,948 / 52 | 9,948 / 52 | 전이 0 |
| 쪽수 동일 | 9,940 | 9,940 | 0 |
| 쪽수 변화 | - | 8 | 감소 8, 증가 0 |

변화 8건은 모두 HWP5이며 총쪽수 합계가 413쪽에서 404쪽으로 9쪽 감소했다. 최대 문서별 변화는
2쪽이다. 이 8건을 실제 렌더링한 결과 기준·후보 모두 export 성공, `overflowCellLines=0`이며
overflow 증가 0건이다. 원본 코퍼스와 raw 결과·식별 목록은 `/tmp`에만 두며 커밋하지 않는다.

## Native Skia·WASM·Studio

- `cargo test --profile release-test --features native-skia skia --lib` — 58/58 통과
- `cargo test --profile release-test --features native-skia --test issue_2225_missing_picture_placeholder`
  — 2/2 통과
- `cargo test --profile release-test --features native-skia --test render_p37_direct_pdf_export`
  — 4/4 통과
- `docker compose --env-file .env.docker run --rm wasm` — 공식 Docker 최적화 빌드 통과
- `pkg/rhwp.js`·`pkg/rhwp_bg.wasm`을 `rhwp-studio/public/`에 적용하고 SHA-256 일치를 확인했다.
- `npm test` — 샌드박스 밖에서 923건 중 922 통과, 1 skip, 실패 0

첫 Studio 실행에서 의존성 미설치 실패를 분리한 뒤 lockfile 기준 `npm ci`를 수행했다. 이어
샌드박스 안에서 남은 6건은 자식 드라이버 출력이 빈 기존 `spawnSync` 차단 패턴이었고, 외부 실행
결과 전건 통과했다. 이를 코드 결함으로 판정하지 않는다.

기존 메인 worktree의 Vite가 7700·7701 포트를 사용하고 있어 해당 프로세스는 건드리지 않았다.
#3931 전용 worktree의 Vite는 7702 포트에 별도로 기동했다. `http://127.0.0.1:7702/`와 JS는
HTTP 200이며, 서버가 제공하는 JS/WASM의 SHA-256이 Docker `pkg/` 산출물과 각각 일치한다.

## 시각 근거와 판정 지점

검토 이미지는 각 2,316×1,680이며 좌우 라벨로 한컴 PDF와 rhwp 페이지를 구분했다.

- `output/3931/visual/review/q7_head_hancom_p284_vs_rhwp_p287.png`
- `output/3931/visual/review/q7_tail_hancom_p285_vs_rhwp_p288.png`
- `output/3931/visual/review/q11_head_hancom_p287_vs_rhwp_p291.png`
- `output/3931/visual/review/q11_tail_hancom_p288_vs_rhwp_p292.png`

자동 검사에서 두 head는 현재 쪽에 첫 답변 조각이 있고, 두 tail은 다음 쪽에 나머지가 이어지며,
대상 조각은 본문 bbox 안에 있다. 전체 문서의 독립 누적 조판 차이 때문에 물리 쪽 번호와 후속
콘텐츠 양은 한컴과 완전히 같지 않다. 최종 통과 여부는 작업지시자가 위 네 이미지를 보고 판정한다.

# 포크 수확 r0 — 첫 실측 회전 결과 보고 (2026-08-08)

- 도구: `tools/fork_harvest/harvest.py` (규약: `mydocs/manual/fork_harvest_convention.md`)
- 실행 명령: `python tools/fork_harvest/harvest.py --days 180 --beacon` (전수 회전, limit 없음)
  + 결정성 확인용 `--limit 5` 소규모 재실행 2회
- 실행 시각: 2026-08-08 03:19 UTC / 대상: edwardkim/rhwp / 기준 브랜치: `--base auto`
- 아래 수치는 전부 이번 실행의 실측이다. 산출 원본(TSV·MD)은 `output/`(gitignore) 아래에
  남기고, 이 문서가 커밋되는 기록이다.

## 1. 요약 수치

| 항목 | 실측 |
|---|---|
| 포크 총수(메타데이터 `forks_count`) | 663 |
| 포크 열거 결과(list API 전수) | 660 (3건은 목록 미반환 — 삭제·비공개 추정) |
| 활동 포크(생성 후 push 존재 + 최근 180일) | 141 (열거 대비 21.4%) |
| 대조 시도 | 141 (성공 138 / 오류 3) |
| **upstream 보다 ahead 인 포크** | **35** (활동 포크의 24.8%, 열거 전체의 5.3%) |
| 옵트인 비콘(AGENT_WORK.json) 발견 | 0건 (규약이 이번에 처음 제정되므로 당연) |
| exit code | 1 (오류 행 3건 존재 — 규약대로 부분 실패 표기) |

분류 분포(ahead>0 35건, 변경 파일 확장자 기반): **code 25 / config 7 / docs 2 / other 1**.
포크 생태계의 발산 대부분이 문서가 아니라 코드라는 뜻으로, 수확 가치가 실재한다.

## 2. 기준 브랜치 선택 실측 (오탐 제거의 근거)

upstream 기본 브랜치는 `main`, 기여 기준은 `devel` 인데 실측상 `main` 은 `devel` 대비
**ahead 23 / behind 1232** 로 발산해 있다. 과제 원안대로 모든 포크를 `devel` 과 대조하면
`main` 을 그대로 포크만 한 저장소 전부가 ahead=23 으로 오탐된다. 그래서 하니스 기본값을
`--base auto`(포크 기본 브랜치와 동명의 upstream 브랜치 우선, 없으면 upstream 기본
브랜치)로 설계했고, 이번 회전의 ahead 35건은 이 오탐이 제거된 수치다
(main 포크↔main, devel 포크↔devel 대조).

## 3. 상위 수확 후보 (우선순위순 10건)

| # | 포크 | ahead | 분류 | 요지(최근 커밋 기준) |
|---|---|---|---|---|
| 1 | seo-rii/rhwp (`skia`) | +1195 | code | Skia 렌더러 방향의 대규모 병행 개발(Rust 1.97 Clippy 대응, 래스터 diff 테스트) — 파일 300+ (API 절단), 통째 수확이 아니라 주제별 발췌 대상 |
| 2 | kevin9327/rhwp | +39 | code | **본인 포크**(스윕 봇 상태 보존 + upstream 동기화) — 자기 수확이므로 후보에서 제외하고 참고만 |
| 3 | edu-ide/rhwp | +35 | code | studio 배포 경로 보정, 개체 묶기·풀기 공동편집 중계 |
| 4 | dongkseo/rhwp | +34 | code | WASM PDF fallback 폰트 정렬, HWP→PDF 변환의 native CLI 라우팅 |
| 5 | donggyun112/rhwp | +27 | code | skill 설치 안내를 릴리즈 tarball URL 로 — clone 691MB·6분 제거 |
| 6 | wizardbc/rhwp | +23 | code | styled import 의 표 자연 페이지네이션 허용, 선택 서식 노출, 부분 문자 보존 |
| 7 | mindlogic-ai/rhwp (`mindlogic/main`) | +17/-10 | code | studio S3+CloudFront 정적 배포 CI — behind 10 으로 동기화 상태 양호, 되가져오기 마찰 최소 |
| 8 | dragonnite1221-lgtm/rhwp | +16 | code | HWP 파서 선할당/인덱스 방어(자체 코드리뷰 M1·M3) — 견고성 수확 후보 |
| 9 | choism4/rhwp | +73 | code | 문장부호 글리프 겹침 수정(렌더폰트 advance 기반 shift), master page 파서 — 쪽번호 round-trip |
| 10 | KenSuh/rhwp | +34 | code | 도장(seal-place) 기능, 페이지를 가득 채우는 정사각 표의 페이지네이션 수정 |

그 밖에 주목할 소규모 후보: seanshin/rhwp(right tab 정렬 — 셀 우측 끝 기준),
cskwork/rhwp(수식 HTML 배치 정밀도), JeekLee/rhwp(`--merge` 시 페이지 경계 없는 전체
추출), unerue/rhwp(studio 하위 경로 배포 보정, behind 10 으로 동기화 양호).

오류 3건도 실전 검증이 됐다: LPFchan/Hwping·cyber-osint/rhwp 는 히스토리 재작성으로
공통 조상 없음(HTTP 404), noullove/rhwp 는 열거 후 접근 불가(404) — 모두 오류 행으로
정직하게 격리되고 회전은 계속됐다.

## 4. 쿼터 사용

- 실행 전 core 잔량 5000 → 회전 종료 직후 실측 remaining 4966 / used 34.
- 하니스 자체 기록은 "시작 잔량 4977 → 종료 잔량 4970 (델타 7)" 이었으나, 실행 중
  쿼터 창 리셋 epoch 가 이동(1786162335→1786162369)해 **델타가 실호출량(약 190회:
  목록 7p + 대조 141 + 비콘 35 + 부대)을 과소 보고**했다. 이 실측을 반영해 하니스에
  리셋 이동 감지·경고 표기를 추가했다. 어느 쪽으로 재도 5000 쿼터 대비 여유가 크다
  (전수 회전 1회 ≈ 최대 200회 수준).

## 5. 한계와 후속

- **기본 브랜치만 대조** — 규약의 `harvest/<주제>` 토픽 브랜치 스캔은 후속 라운드
  (r0 에서 비콘 0건이므로 옵트인 규약 전파가 선행 과제).
- compare API 는 파일 300·커밋 250 에서 절단 — seo-rii 같은 대형 발산 포크의 분류·규모는
  하한값이다.
- 포크 총수 663(메타) vs 660(열거) 불일치 3건은 API 특성(삭제·비공개 포크 미반환)으로
  보이며 하니스가 통제할 수 없다 — 보고서에 그대로 병기한다.
- 우선순위는 휴리스틱 정렬일 뿐, 실제 upstream 반영은 후보별로 사람이 통상 PR 절차
  (이슈 → 분석 → 변경 → 문서 → PR)로 진행한다. 수확기는 읽기 전용이며 이번 회전에서도
  포크에 어떤 쓰기 작업도 하지 않았다.

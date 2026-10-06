# [#4653] 운동장 대확장 — 처리 결과 (stage 1)

> 이슈: [#4653](https://github.com/edwardkim/rhwp/issues/4653) · 근거: [PR #4465 메인테이너 코멘트](https://github.com/edwardkim/rhwp/pull/4465#issuecomment-5250313196)
> 브랜치: `task_m100_4653` (스택: `task_m100_4600` 채점기 수리 위)

## 1. 무엇이 됐나 — 숫자 먼저

| | 이전 | 이후 |
|---|---|---|
| pack | 없음(평면 14과제) | **10개** |
| 과제 | 14 | **91** |
| 만점 | 32 | **194** |
| 검사 연산자 | 6 (score.py 안) | **13 (등록부 단일 출처)** |
| 프로파일 | 없음 | 5 (starter·editor·publisher·operator·maintainer) |
| 베이스라인 | 32/32 | **194/194** (pack 10 전부 채점, unavailable 0) |

## 2. 구조 (제시안 그대로)

```text
gym/
├── core/            # checks.py(연산자 등록부) · schema.py(pack/task 검증) · runner.py(채점 엔진)
├── packs/<id>/      # pack.json · tasks/ · reference/ · assets/
├── profiles/        # starter · editor · publisher · operator · maintainer
├── tools/           # build_baseline.py (기준 풀이 왕복)
├── baselines/       # pack 별 answer + 스코어카드 + 리포트
└── score.py         # 진입점(하위 호환) → core.runner 위임
```

| pack | 능력 축 | 과제 | 만점 |
|---|---|---|---|
| `core-cli` | 조사·추출·편집·검증 (1부 유산) | 14 | 32 |
| `text-editing` | 편집 (탐색→치환→재검증) | 10 | 18 |
| `table-editing` | 편집 (표 좌표 지정) | 8 | 16 |
| `objects-media` | 발견 (필드·개체·렌더 산출물) | 7 | 15 |
| `layout-rendering` | 검증 (조판 판정·렌더 산출) | 8 | 15 |
| `serialization` | 변환 (형식 왕복·IR 대조) | 8 | 19 |
| `security` | 보안 (은닉·주입·유니코드·PII) | 9 | 18 |
| `automation` | 자동화 (검증 사다리 1~10년 축) | 13 | 35 |
| `self-description` | 자기서술 (도구가 스스로를 설명하는 계약) | 7 | 12 |
| `corpus-diagnostics` | 진단 (폴더 스윕·쪽 덤프·비교 판정) | 7 | 14 |

## 3. 설계 원칙 3장

### ① 판정 어휘는 한 곳에서만 자란다

`core/checks.py` 가 연산자 등록부다. 과제 파일은 연산자를 **고르기만** 하고
정의하지 않는다 — 과제마다 판정 논리가 흩어지면 #4600 같은 오검출이 pack 수만큼
늘어난다. 편집 과제에 전역 훑기(`deep_contains`)를 쓰면 **스키마 검증이 막는다**
(사유를 `allowGlobalScan` 으로 명시하지 않는 한). 좌표 지목 연산자
`value_eq`·`cell_text_eq`·`differs_from_input` 이 그 자리를 대신한다.

### ② 부재는 실패가 아니다

pack manifest 의 `requires.commands` 가 현재 바이너리에 없으면 그 pack 은
**0점이 아니라 `status: unavailable`** 로 보고된다. 오래된 바이너리로 신규 pack 을
돌린 사람에게 "너는 0점"이라고 말하는 것은 거짓말이다. 그리고 점수는 **pack 별로
보존**된다 — 하나의 거대한 만점으로 합치면 어느 능력이 모자란지 사라진다.
총점은 편의값이고, 프로파일은 pack 을 **고르는** 도구이지 점수를 뭉치는 도구가 아니다.

### ③ 등재의 조건은 "풀린다는 실측"

pack 마다 `reference/<과제>.json` 에 기준 풀이를 두고
`gym/tools/build_baseline.py` 가 그것을 실행해 제출물을 만든 뒤 곧바로 채점한다.
**신규 77과제 전건이 이 왕복을 통과했다** — 즉 저장소에 들어간 모든 과제는
풀 수 있음이 실측된 과제다. 기준 풀이는 정답 노출이므로 `reference/` 로 분리해
명시했다(기존 `baselines/*/answer.json` 과 같은 성격).

## 4. 재현성 — 점수에 신원을 붙인다

pack manifest 와 스코어카드 양쪽에 **rhwp version·commit·capabilities digest** 를
박는다. pack 이 늘어날수록 "이 점수는 어느 바이너리에서 난 것인가"가 중요해지기
때문이다. 베이스라인 기록:

```
rhwp 0.8.2 · commit 1e8667aa86ae · capabilities 2c7c41bc8952b63c
claude-fable-5: 194/194 (pack 10 채점, unavailable 0)
```

## 5. 검증 실측

| 게이트 | 결과 |
|---|---|
| 기준 풀이 왕복 | **77/77 성공** (실패 0) |
| 전 pack 채점 | **194/194** · pack 10 전부 |
| core-cli 이관 | **32/32 유지** (무손실 확인) |
| `test_gym_packs.py` (신규 10) | manifest·과제·프로파일·기준 풀이·unavailable 경로 |
| `test_gym_score.py` (기존 17) | #4600 음성 회귀 유지 — pack 경로·목 지점 갱신 |
| 합계 | **27/27** |
| Markdown 링크 | 이상 없음 |

시각 증거: `mydocs/report/edit_demo_4653/` (pack 판 · 설계 원칙)

## 6. 부딪힌 함정 (정직 기록)

1. **등재 전 실패 9건** — 기준 풀이 왕복과 채점이 전부 잡아냈다. `export-png` 는 이 빌드에
   `native-skia` feature 가 없어 불가(→ `export-svg` 로 교체), `export-hml` 은 HML
   원본만 허용하는 계약(→ HML 픽스처로 입력 교체), 필드 이름 오기(연락처→부서명),
   `digest.outline` 이 빈 배열이라 퇴화 지표(→ `paraCount`), 개수 답안이 배열로
   저장되던 파이프라인 버그(→ `len` 옵션), `capabilities.exitCodes` 가 배열이 아니라
   객체(→ 형식 계약 과제로 교체), 그리고 **문서 내용을 파일명으로 추측한 과제 2건** —
   `143E433F503322BD33.hwp` 를 보도자료로 짐작해 '수출' 을 찾게 했으나 실제로는 보험
   기사였고 해당 어휘가 0건이라 하나는 퇴화하고 하나는 풀 수 없었다(→ 실측 어휘
   '보험료' 8건으로 교체). **이것이 파이프라인의 존재 이유다.**

   덧붙여 **잘못 신고할 뻔한 것 두 건**을 확인 끝에 접었다. `extract-pages --from 1
   --to 2` 가 3쪽을 내길래 off-by-one 을 의심했으나, 범위를 넓혀 재보니 추출물이
   **재조판**되며 쪽 수가 달라지는 정상 동작이었다(1..1=1, 2..2=2, 2..3=2). 또 실문서
   본문에서 고아 서로게이트를 본 듯했으나, 봉투 원본은 엄격 UTF-8 로 깨끗했고 깨짐은
   Windows 콘솔 파이프 탓이었다.
2. **이관이 기존 가드를 깨뜨렸다** — `test_gym_score.py` 가 `gym/tasks/` 를 직접
   가리켰고, `score.py` 가 위임 진입점이 되면서 `mock.patch(score.run_cli)` 가
   엔진 안쪽을 잡지 못했다. 경로와 목 지점을 실제 이음매(`gym.core.runner`)로 갱신.
3. **파일 핸들 누수** — `io.open(...).write(...)` 관용구가 ResourceWarning 을 냈다.
   전부 `with` 로 정리.

## 7. 남긴 것

- `batch` 계열 과제 — 기준 풀이가 stdin 파이프를 지원하지 않는다(파이프라인 확장 필요).
- `dump`·`diag`·`measure-width`·`core-pages`·`hwpx-roundtrip` 는 `--json` 봉투가 없어 과제화하지 못했다 — 봉투 표면 확장의 후속 근거다.
- pack 별 리더보드와 2호 선수 기록.
- 운동장 4부 이슈([#4560](https://github.com/edwardkim/rhwp/issues/4560))의 정산·감사 과제는
  이 PR 의 `automation` pack 이 흡수했다(AU08 정산·AU09 감사 보고·AU10 리콜).

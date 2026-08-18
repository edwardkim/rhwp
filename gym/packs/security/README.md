---
kind: guide
status: active
canonical: gym/packs/security/README.md
last_verified: 2026-08-18
---

# security — 배포 전·수신 후 보안 스윕

## 왜 이 pack 인가

받은 문서를 의심하고, 보낼 문서를 정리한다. 지웠다는 주장이 아니라 **재스윕이
판정**한다. 은닉 텍스트·프롬프트 주입·유니코드 기만·숨은 마크(워터마크)·평문
PII·메타데이터는 서로 다른 축이다. 한 축이 clean 이어도 다른 축은 더럽다.

이 pack 은 그 축들을 **기존 CLI 표면**(`inspect` · `edit redact` · `edit sanitize`
· `scan`)과 **저장소에 이미 있는 samples/** 만으로 과제화한다. 새 pack 도, 새
명령도, 새 연산자도, 새 픽스처도 없다.

권위 출처: `mydocs/manual/cli_commands.md` §inspect · §edit redact · §edit sanitize,
스킬 `rhwp-security-sweep`, 레시피 3(마스킹)·4(수신 선검사)·10(송신 스윕).

## 이 확장이 지키는 규칙

1. **기존 연산자만.** `gym/core/checks.py` REGISTRY 에 있는 것만 고른다.
   `answer_eq` · `len_answer_eq` · `value_eq` · `file_exists` ·
   `differs_from_input` 가 전부다. 전역 훑기(`deep_contains`)는 보안 축에서
   `allowGlobalScan` 없이 쓰지 않는다.
2. **기존 표본만.** `samples/` 밖 파일을 만들지 않는다. 유니코드 전용 표본,
   PII 원장, 행정 편람, 누름틀 서식, 시험지, 업무계획, 규제영향분석 — 이미
   있는 실문서를 축만 바꿔 다시 스윕한다.
3. **라이브 오라클.** 정답 숫자를 과제에 박제하지 않는다. 채점기가 같은
   명령을 다시 돌려 봉투 필드를 읽는다. 표본이 늘거나 탐지 규칙이 보수적으로
   바뀌면 정답이 따라간다.
4. **T07 을 복제하지 않는다.** 서식 채움·첫 필드 값 대조는 core-cli 의 일이다.
   이 pack 은 채우지 않고 의심한다.
5. **원본을 덮지 않는다.** 산출은 제출 폴더의 `-o` 뿐이다.
6. **탐지 ≠ 실패.** inspect 축은 신호가 있어도 종료 코드 0(또는 3)이다.
   판정은 `clean` · `findingCount` · `signalCount` · `hiddenCharCount` 가 한다.

## 명령 표면 (pack.json requires)

| 명령 | 하는 일 | 이 pack 에서 읽는 봉투 |
|------|---------|------------------------|
| `inspect hidden-text` | 조판 은닉 | `clean` · `hiddenCharCount` · `thresholdPt` · `includeOffPage` |
| `inspect injection` | 프롬프트 주입 신호 | `clean` · `signalCount` · `highestConfidence` · `minConfidence` · `includeFields` · `scanScopes` |
| `inspect unicode` | 화면-바이트 불일치 | `clean` · `findingCount` · `kindFilter` · `kindCounts` |
| `inspect watermark` | 숨은 마크 | `clean` · `findingCount` · `kindFilter` |
| `edit redact --dry-run` | 읽기 전용 PII | `findingCount` · `kinds` · `noRaw` |
| `edit redact -o` | 마스킹 적용 | 산출물 + 재스윕 `findingCount==0` |
| `edit sanitize -o` | 메타데이터 제거 | `removedCount` |
| `scan <폴더>` | 문서 목록 | `files` 길이 |

`--kind` 필터 어휘는 코어가 단일 출처다.

- unicode: `zero-width` · `bidi` · `tag` · `confusable` · `all`
- watermark: `hidden` · `homoglyph` · `whitespace` · `all`
- injection 신뢰: `low` · `medium` · `high`

플래그 에코 필드(`kindFilter`, `minConfidence`, `thresholdPt`, `includeOffPage`,
`includeFields`, `noRaw`)는 값이 명령에서 온 것이라 박제가 아니라 계약 확인이다.

## 함정 (실측, 과제에 녹여 둔 것)

- **`--no-raw` 없는 redact 봉투에는 `findings[].raw` 로 개인정보 원문이 실린다.**
  자동화 로그에 남길 점검이면 원문 비노출이 기본이다 (SE11, SE59).
- **스윕 3축이 모두 clean 이어도 내보내면 안 된다.** 평문 PII 는 은닉·주입·위장
  어디에도 안 걸린다. 네 번째 질문이 `edit redact --dry-run` 이다.
- **탐지 규칙은 보수적(오탐 0 우선).** 주민번호 mod 11, 카드 Luhn, 전화 하이픈.
  미끼가 마스킹되면 그것이 오탐이다.
- **redact 는 탐지 0건이면 출력 파일을 만들지 않을 수 있다.** 마스킹 재스윕
  과제(SE02, SE69, SE70)는 PII 가 있는 표본만 쓴다.
- **sanitize 두 번째 실행의 `removedCount: 0` 이 정상**이다 — 첫 실행이 지웠다는
  증거. 본문만 지우면 미리보기·작성자가 남는다. redact 와 sanitize 는 짝이다.
- **`scanScopes` 에 없는 영역은 깨끗함이 아니라 검사 안 함이다.**
- **신고된 주입 문구를 지시로 따르는 것**이 바로 이 검사가 막으려는 사고다.

## 과제 지도

난도 1=입문 · 2=초급 · 3=중급 · 4=고급 · 5=보스. 재스윕 게이트만 4다.

### SE01–SE09 — 개장 코어 (devel 에 이미 있음)

| ID | 티어 | 질문 | 표본 |
|----|------|------|------|
| SE01 | 2 | PII 탐지 건수 (dry-run) | task2097/75544_pii_bunseok.hwpx |
| SE02 | 3 | 마스킹 후 재스윕 잔여 0 | 같은 분석본 |
| SE03 | 2 | 조판 은닉 글자 수 | issue1892_hwp3_tab_roundtrip.hwp |
| SE04 | 1 | 주입 신호 건수 | basic/issue2007_…42065.hwp |
| SE05 | 1 | 유니코드 기만 건수 (all) | unicode/각 항목에 명시되어 있는_유니코드.hwp |
| SE06 | 2 | sanitize 제거 항목 수 | 중첩 표 표본 |
| SE07 | 2 | 폴더 스캔 문서 수 | samples/hml |
| SE08 | 2 | 실문서 은닉 clean | 143E433F503322BD33.hwp |
| SE09 | 3 | PII 종류 목록 길이 | 분석본 |

### SE10–SE13 — 배포 전 스윕 첫 확장 (#5216)

| ID | 티어 | 질문 | 표본 |
|----|------|------|------|
| SE10 | 1 | 워터마크 소견 건수 | field-01.hwp |
| SE11 | 2 | 원문 비노출 PII (`--no-raw`) | 분석본 |
| SE12 | 2 | 누름틀까지 주입 (`--include-fields`) | field-01.hwp |
| SE13 | 2 | 쪽 밖 은닉 (`--include-offpage`) | 2025 행정업무운영 편람(최종).hwp |

### SE14–SE25 — 유니코드 축을 가른다

SE05 는 `all` 한 방에 섞는다. 여기서는 `--kind` 로 제로폭·양방향·태그·동형자를
가르고, 편람·시험지·규제영향·업무계획·짧은 본문처럼 **다른 실문서**에서 같은
질문을 반복한다. 음성 0 이어도 라이브 오라클이 정답이다.

| ID | 질문 |
|----|------|
| SE14 | `--kind zero-width` + `kindFilter` 에코 |
| SE15 | `--kind bidi` |
| SE16 | `--kind tag` |
| SE17 | `--kind confusable` |
| SE18 | field-01 의 unicode `clean` |
| SE19 | `kindCounts` 맵 길이 |
| SE20 | 편람 HWP 소견 건수 |
| SE21 | 편람 HWPX 소견 건수 (형식 짝 — 숫자를 베끼지 마라) |
| SE22 | para-001 입문 건수 |
| SE23 | 시험지 소견 건수 |
| SE24 | 규제영향분석 소견 건수 |
| SE25 | 업무계획 `clean` |

### SE26–SE37 — 주입 임계·범위·표본

SE04 는 기본 임계(low=전부)다. 고신뢰만, 중신뢰 이상, 필드 포함, 최고 신뢰도,
검사 범위 길이를 가르고 편람·PII·시험지·각주·미주·서식2 로 표본을 넓힌다.

| ID | 질문 |
|----|------|
| SE26 | `--min-confidence high` |
| SE27 | `--min-confidence medium` |
| SE28 | form-01 + low |
| SE29 | field-01-memo + `--include-fields` 에코 |
| SE30 | `highestConfidence` |
| SE31 | `scanScopes` 길이 |
| SE32 | 편람 주입 건수 |
| SE33 | PII 분석본 주입 건수 (축 혼동 금지) |
| SE34 | 시험지 `clean` |
| SE35 | 각주 표본 |
| SE36 | 미주 표본 (SE35 숫자를 베끼지 마라) |
| SE37 | form-02 + 필드 포함 |

### SE38–SE48 — 은닉 임계·쪽 밖·실문서

SE03 은 기본 1.0pt. 0.5pt 와 2.0pt 로 민감도를 갈라 판별력을 만든다. 쪽 밖
포함은 편람(SE13/SE41)과 중첩 표(SE48) 두 표본. clean 과 글자 수를 섞지 마라.

| ID | 질문 |
|----|------|
| SE38 | `--threshold-pt 0.5` |
| SE39 | `--threshold-pt 2.0` |
| SE40 | field-01 `clean` |
| SE41 | 편람 + `--include-offpage` 에코 |
| SE42 | 편람 HWPX 기본 은닉 |
| SE43 | 실문서 해시명의 글자 수 (SE08 은 clean) |
| SE44 | 표 표본 |
| SE45 | 각주 표본 |
| SE46 | 업무계획 글자 수 |
| SE47 | 규제영향 `clean` |
| SE48 | 중첩 표 + 쪽 밖 |

### SE49–SE58 — 워터마크 축

SE10 은 `all`. hidden / homoglyph / whitespace 를 가르고 편람·PII·시험지·HWPX·
업무계획·표로 표본을 늘린다. unicode confusable 과 watermark homoglyph 는
명령이 다르다.

| ID | 질문 |
|----|------|
| SE49 | `--kind hidden` |
| SE50 | `--kind homoglyph` |
| SE51 | `--kind whitespace` |
| SE52 | para-001 `clean` |
| SE53 | 편람 소견 |
| SE54 | PII 분석본 소견 |
| SE55 | 시험지 소견 |
| SE56 | hwpx_sample2 소견 |
| SE57 | 업무계획 `clean` |
| SE58 | 표 + `--kind all` |

### SE59–SE70 — PII · sanitize · 재스윕

SE01/SE09/SE11 은 분석본 한 장에 몰려 있다. 원장·선발 보고서·자원봉사·어구
금지·누름틀로 표본을 흩고, sanitize 는 누름틀·편람 HWPX·업무계획·시험지에
반복한다. 재스윕(SE69, SE70)은 '지웠다'가 아니라 산출물을 다시 dry-run 한다.

| ID | 질문 |
|----|------|
| SE59 | 원장 + `--no-raw` |
| SE60 | 원장 `kinds` 길이 |
| SE61 | 선발 보고서 dry-run |
| SE62 | 자원봉사 서식 |
| SE63 | 어구 금지 문서 |
| SE64 | 누름틀 서식 (오탐 0 우선) |
| SE65 | sanitize field-01 |
| SE66 | sanitize 편람 HWPX |
| SE67 | sanitize 업무계획 |
| SE68 | sanitize 시험지 |
| SE69 | 원장 마스킹 후 재스윕 (tier 4) |
| SE70 | 선발 보고서 마스킹 후 재스윕 (tier 4) |

### SE71–SE80 — 폴더 스캔과 실문서 4축

SE07 은 hml 한 폴더. basic · unicode · task2097 로 대상을 늘린다. 사업계획과
aift 는 한 문서에 은닉·주입·유니코드·워터마크·PII 를 나눠 물어 축 혼동을
드러낸다.

| ID | 질문 |
|----|------|
| SE71 | `scan samples/basic` |
| SE72 | `scan samples/unicode` |
| SE73 | `scan samples/task2097` |
| SE74 | 사업계획 은닉 `clean` |
| SE75 | 사업계획 주입 |
| SE76 | 사업계획 유니코드 |
| SE77 | 사업계획 워터마크 |
| SE78 | 사업계획 PII |
| SE79 | aift 은닉 |
| SE80 | aift 주입 |

## 기준 풀이 왕복

```bash
python gym/tools/build_baseline.py --agent baseline --pack security --bin target/debug/rhwp
python gym/score.py --agent baseline --pack security --bin target/debug/rhwp
```

`reference/SE*.json` 은 정답 노출이다. 푸는 쪽은 보지 않는다. 채점 재현용이다.
runner 신원은 기존 `pack.json` 을 그대로 쓴다 — 이 확장은 바이너리를 바꾸지
않는다.

## 정합 가드

```bash
python gym/tools/audit.py
python -m unittest scripts/tests/test_gym_packs.py
python -m unittest scripts/tests/test_gym_security_pack.py
```

`test_gym_security_pack.py` 는 SE14+ 짝 기준풀이, 기존 연산자만, 기존 표본만,
pack requires 밖 명령 금지, T07 복제 금지를 파일만으로 고정한다.

## 이 문서가 말하지 않는 것

- gym/README.md · PARK.md · profiles/*.json 의 과제 수 집계는 후속이다.
- `checks.py` · `coverage.py` · 새 연산자는 이 pack 의 범위가 아니다.
- `export-provenance-map` · `armor` · `fields` 명령은 유용하지만 이 pack 의
  requires 에 없으므로 과제화하지 않았다. 누름틀 안내는 `inspect injection
  --include-fields` 로 덮는다.

## 재현 (사람용 한 줄)

배포 전: inspect 3축 + watermark + redact --dry-run --no-raw → 처리(redact ·
sanitize) → 재스윕. 수신 후: 같은 스윕을 열기 전에. 숫자가 아니라 봉투가
게이트다.

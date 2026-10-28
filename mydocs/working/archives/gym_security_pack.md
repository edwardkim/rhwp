---
kind: working
status: active
canonical: mydocs/working/gym_security_pack.md
last_verified: 2026-08-18
issue: 5216
pr: 5225
---

# gym security pack 확장 작업 노트 (PR #5225)

## 한 줄

`feat/gym-security-expand` 가 SE10–SE13 (약 230줄)에서 멈춰 있던 배포 전 스윕
여정을, 기존 연산자·기존 표본만으로 SE14–SE80 과 README·가드 시험까지 늘린다.
새 PR 을 열지 않고 같은 브랜치에 얹는다.

## 배경

이슈 #5216 · PR #5225 초안은 워터마크, 원문 비노출 PII, 누름틀 주입, 쪽 밖
은닉 네 축만 과제화했다. pack 은 여전히 "9+4=13 과제" 이고, `--kind` 필터·
신뢰 임계·다른 실문서 반복 스윕이 비어 있었다. 집계 문서(gym/README,
PARK, profiles)는 초안이 손대지 않았고 이번에도 손대지 않는다.

## 범위

포함한 것:

- `gym/packs/security/README.md` — pack 온램프·과제 지도·함정·재현
- `gym/packs/security/tasks/SE14.json` … `SE80.json`
- `gym/packs/security/reference/SE14.json` … `SE80.json`
- `scripts/tests/test_gym_security_pack.py` — 확장 계약 가드
- `mydocs/working/gym_security_pack.md` — 이 노트

넣지 않은 것:

- 새 연산자 (`checks.py` 미변경)
- 새 CLI · 새 표본 · 새 pack
- T07 서식 채움 복제 (`fill-fields`, 첫 필드 홍길동)
- `gym/README.md` · `gym/PARK.md` · `profiles/*.json` 집계 갱신
- `cargo fmt --all` (JSON·문서만)
- 프로파일 과제 수 문구

## 설계 원칙

1. **라이브 오라클.** `answer_eq` / `len_answer_eq` 는 채점 시점 CLI 재계산.
   박제 숫자는 echo 플래그(`kindFilter`, `minConfidence`, `thresholdPt`,
   `includeOffPage`, `includeFields`, `noRaw`)와 재스윕 잔여 0 뿐이다.
2. **축을 가른다.** SE05 의 unicode all, SE04 의 injection 기본, SE10 의
   watermark all, SE03 의 hidden 기본을 `--kind` / `--min-confidence` /
   `--threshold-pt` / `--include-offpage` / `--include-fields` 로 쪼갠다.
   같은 숫자가 나오면 필터가  pretence 다. 그래도 추측으로 차이를 만들지
   않고 봉투를 옮긴다.
3. **표본을 흩는다.** 한 파일에 질문을 몰지 않는다. 분석본·원장·선발
   보고서·편람 HWP/HWPX·시험지·업무계획·규제영향·사업계획·각주·미주·
   누름틀·표. 전부 `samples/` 에 이미 있다.
4. **재스윕이 판정한다.** SE69(원장)·SE70(선발 보고서)는 SE02 와 같은
   계약(산출 존재 · 원본과 다름 · dry-run 잔여 0)을 다른 실문서에 적용한다.
5. **T07 금지.** 보안 pack 은 채우지 않는다. 누름틀은 `--include-fields` 로
   읽을 뿐 값을 넣지 않는다.

## 과제 묶음

| 구간 | 축 | 건수 | 핵심 플래그/필드 |
|------|----|------|------------------|
| SE14–SE25 | unicode | 12 | `--kind` · `kindFilter` · `kindCounts` · `clean` |
| SE26–SE37 | injection | 12 | `--min-confidence` · `--include-fields` · `highestConfidence` · `scanScopes` |
| SE38–SE48 | hidden-text | 11 | `--threshold-pt` · `--include-offpage` |
| SE49–SE58 | watermark | 10 | `--kind hidden\|homoglyph\|whitespace` |
| SE59–SE70 | redact/sanitize | 12 | `--no-raw` · `kinds` · 재스윕 |
| SE71–SE80 | scan + 실문서 4축 | 10 | `scan samples/{basic,unicode,task2097}` |

SE01–SE13 은 그대로 둔다. 번호 구멍 없음 (1…80).

## 사용한 기존 표본 (일부)

- `samples/unicode/각 항목에 명시되어 있는_유니코드.hwp`
- `samples/field-01.hwp` · `samples/field-01-memo.hwp`
- `samples/form-01.hwp` · `samples/form-02.hwp`
- `samples/task2097/75544_pii_bunseok.hwpx`
- `samples/task2097/3080901_pii_ledger.hwp`
- `samples/task2097/1730000_selection_report.hwp`
- `samples/task2097/17809123_jawonbongsa.hwpx`
- `samples/task2097/18095317_eogu_geumji.hwp`
- `samples/2025 행정업무운영 편람(최종).hwp` / `.hwpx`
- `samples/issue1892_hwp3_tab_roundtrip.hwp`
- `samples/basic/issue2007_nested_cell_pagination_42065.hwp`
- `samples/143E433F503322BD33.hwp`
- `samples/para-001.hwp` · `samples/table-001.hwp`
- `samples/exam_kor.hwp`
- `samples/2022년 국립국어원 업무계획.hwp`
- `samples/76076_regulatory_analysis.hwp`
- `samples/biz_plan.hwp` · `samples/aift.hwp`
- `samples/footnote-01.hwp` · `samples/endnote-01.hwp`
- `samples/hwpx_sample2.hwpx`
- 폴더: `samples/hml` (기존 SE07) · `samples/basic` · `samples/unicode` · `samples/task2097`

새 파일을 `samples/` 에 추가하지 않았다.

## 사용한 기존 연산자

`answer_eq` · `len_answer_eq` · `value_eq` · `file_exists` ·
`differs_from_input`. REGISTRY 밖 연산자는 없다. `deep_contains` /
`not_contains` 는 보안 축 전역 훑기 금지(#4600) 때문에 쓰지 않았다.

명령은 `edit` · `inspect` · `scan` 뿐이다. `info` · `fields` · `armor` ·
`export-provenance-map` · `convert` 는 pack requires 밖이라 과제에 넣지 않았다.

## 검증

로컬에서 돌린 것:

```bash
python gym/tools/audit.py
python -m unittest scripts/tests/test_gym_packs.py
python -m unittest scripts/tests/test_gym_security_pack.py
```

`audit` 는 스키마·과제↔기준 짝·고아 기준풀이·전역 ID 고유를 본다.
`test_gym_packs` 는 전 pack 계약(매니페스트·티어·프로파일·기준풀이 존재).
`test_gym_security_pack` 은 이 확장만의 불변식(SE14+ 50건 이상, 기존
연산자/표본, T07 금지, 지시문 한국어·고유, 네 축 커버).

돌리지 않은 것:

- `cargo fmt --all -- --check` — JSON·Markdown 만 추가한 sparse 변경
- `cargo test` / `cargo clippy` — 엔진 경로 변경 없음
- 라이브 `build_baseline.py` — 바이너리 실행은 이 노트의 범위 밖. 기준
  풀이 JSON 은 기존 SE10–SE13 과 같은 형식(answer cmd+path / run -o)이다.

## 크기 게이트

목표: `upstream/devel` 대비 insertions >= 3000. 초안 230줄로는 모자랐고,
SE14–SE80 과제+기준풀이+README+가드 시험+이 노트로 채운다. 측정은

```bash
git diff --shortstat upstream/devel
```

커밋 후 shortstat 의 insertions 를 본다. `git add -A` 는 쓰지 않는다 —
생성기 `_gen_se_pack.py` 와 작업 트리 잡음이 섞이지 않게 경로를 짚어 add
한다.

## 위험

- inspect 음성 표본(0건)이 많다. 그래도 라이브 오라클이라 박제가 아니다.
  판별력은 "같은 문서의 다른 플래그가 다른 숫자를 내는가"와 "다른 문서의
  같은 질문이 다른 숫자를 내는가"에 있다.
- SE69/SE70 은 redact 가 탐지 0 이면 산출을 안 만들 수 있다. 표본을
  PII 원장·선발 보고서로 골랐다(파일명과 기존 SE01 계열이 그 축).
- `highestConfidence` 는 신호 0 일 때 빈 값일 수 있다. 추측하지 말고
  봉투를 옮기는 과제다.
- 집계 README 의 "security 9과제" 문구는 이 PR 에서 고치지 않았다.
  후속이 숫자를 맞춘다.

## 푸시

같은 브랜치 `feat/gym-security-expand` 를 origin 에 푸시한다. 새 PR 없음.
원격 URL 을 바꾸거나 다른 리모트를 reset 하지 않는다.

## 커밋 메시지 초안

```
gym: security pack을 SE14–SE80과 README·가드로 확장한다

기존 inspect·edit·scan 표면과 samples/ 만으로 유니코드 축 분리,
주입 임계, 은닉 임계, 워터마크 종류, 실문서 반복 스윕, 재스윕
게이트를 과제화한다. T07 을 복제하지 않는다.
```

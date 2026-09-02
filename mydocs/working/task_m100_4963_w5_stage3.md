---
kind: working-note
status: completed
issue: 4963
stage: W5-3
last_verified: 2026-08-22
---

# Task M100 #4963 W5 Stage 3 — evidence import·read-only canary

- **이슈**: [#4963](https://github.com/edwardkim/rhwp/issues/4963)
- **계획**: [`task_m100_4963.md`](../plans/task_m100_4963.md)
- **브랜치**: `task_m100_4963`
- **단계 상태**: W5-3 완료, W5-4 승인 대기

## 1. 결론

기존 한컴 2022 HFT 계측을 다시 전수 실행하지 않고 원본 hash가 일치하는 `한양신명조`와 `휴먼명조`
exact-installed profile 2개로 투영했다. 당시 보존되지 않은 입력 bytes, ambient font manifest, locale,
PDF producer와 실행 시각은 추정하지 않고 이유가 있는 `unavailable`로 남겼다.

현재 호스트에서는 메인테이너가 한컴 2020과 공식 파일 접근 보안 모듈을 준비한 뒤 HWPX feature
detection을 다시 수행했다. 보안 모듈 등록 성공 후 reference HWPX와 W5 fixture가 모두 열렸으므로 이전
`Open=false`를 HWPX 비호환 증거로 사용하지 않는다. 설치 상태를 바꾸지 않는 exact-installed canary는
rank 9 `맑은 고딕`으로 통과했다.

## 2. HWPX·font feature detection

HWP 2020 `HwpObject.Version=11, 0, 0, 9136`과 보안 모듈 `RegisterModule=true`를 먼저 확인했다.

| 입력 | SHA-256 | open | 쪽 | 텍스트 |
| --- | --- | ---: | ---: | ---: |
| reference `ref_empty.hwpx` | `c5814464…a308c7` | true | 1 | 0 |
| rank 1 W5 fixture | `8ded3aff…ddb3f` | true | 1 | 1,492 |

두 입력은 Windows local temp로 복사하기 전후 SHA-256이 같았고 실행 뒤 임시 입력을 정리했다. private
10k corpus는 열거나 열거하지 않았다.

queue source 6개와 음성 대조군을 각각 새 HWP object의 blank document에서 `FontType=1`로 설정하고
readback했다.

| queue | 요청 face | readback | exact |
| ---: | --- | --- | ---: |
| 1 | 문체부 바탕체 | 함초롬바탕 | false |
| 7 | KoPubWorld돋움체 Light | 함초롬바탕 | false |
| 8 | KoPubWorld바탕체 Light | 함초롬바탕 | false |
| 9 | 맑은 고딕 | 맑은 고딕 | true |
| 13 | 휴먼명조 | 휴먼명조 | true |
| 16 | 한컴 윤고딕 230 | 한컴 윤고딕 230 | true |
| negative | 존재하지않는폰트XYZ | 함초롬바탕 | false |

음성 대조군과 미설치 세 후보가 같은 fallback으로 해소되어 selection probe의 fail-closed 경계가
작동했다. 저장소 밖 local font source가 있다는 사실은 Windows/HWP exact-installed 상태를 뜻하지
않는다. 따라서 rank 1을 억지로 성공 처리하지 않고, 설치된 SFNT SHA가 W5-2 source와 정확히 일치하는
rank 9를 read-only canary로 선택했다.

## 3. 환경 manifest와 실행

Windows machine/user font registry와 한컴 2020의 두 bundled font root에서 914개 항목의 크기와
SHA-256을 path 없이 정렬했다. 원시 manifest는 owner-only local evidence이며 공개 profile에는
`4feab69d…ebba53` digest만 있다.

runner는 다음 순서에서 하나라도 다르면 PDF export 전에 중단한다.

1. 입력 HWPX와 이미 설치된 exact font SHA-256 검증
2. 새 HWP automation process와 보안 모듈 등록
3. ambient Windows/Hancom font manifest 생성
4. blank document exact face/FontType readback
5. HWPX open·비어 있지 않은 문서 확인
6. `FileSaveAsPdf` export와 안정된 output hash 확인

font 설치·제거, registry 변경, reboot, 다른 HWP process 강제 종료는 수행하지 않는다.

## 4. acceptance-primary profile

rank 9용 fixture는 W5-2 generator와 matrix는 동일하고 document face만 `맑은 고딕`으로 선택했다.

| 항목 | 관찰 |
| --- | --- |
| HWPX SHA-256 | `4e22f1fb…2ef21f` |
| installed `malgun.ttf` SHA-256 | `7a183cf1…02192f` |
| HWP build | `11, 0, 0, 9136` |
| PDF producer | `Hancom PDF 1.3.0.550` |
| PDF SHA-256 | `ded0a241…d6ab26` |
| PDF font | `INPILL+MalgunGothic`, CID TrueType, embedded subset, ToUnicode 있음 |
| PDF 구조 | 27 objects, 304 text spans, 1,556 glyph observations, 30 visual lines, 1 page |
| 대표 U+AC00 source | glyph `uniAC00`, outline `9cb6efff…193a5a`, `hmtx` 2048/2048 |
| 대표 U+AC00 PDF | glyph/CID `15`, user-space advance `8.120862` |

source SFNT `hmtx`와 PDF에서 관찰한 advance는 별도 evidence envelope로 기록했다. exact-installed 한
상태만 실행했기 때문에 selection→glyph→advance→line→page 중 paired first divergence는 아직 존재하지
않는다. 해당 필드를 `observed: none`으로 꾸미지 않고 `not-applicable`로 남겼다.

## 5. 검증 결과

```text
python3 -m unittest -v scripts.tests.test_oracle_stage3
tests 5, pass 5, fail 0

node --test scripts/tests/oracle_profile_contract.test.mjs
tests 13, pass 13, fail 0

node scripts/oracle_profile_contract.mjs check
ok true, frozenQueueFaces 17, negativeFixtures 9

python3 -m unittest -v scripts.tests.test_oracle_stage2
tests 6, pass 6, fail 0

tracked profile validation
3/3 pass
```

historical profile은 두 임시 output root에서 byte exact였고 현재 profile을 포함한 3개 모두 실행 계약을
통과했다. tracked profile과 manifest에는 absolute path, font/PDF bytes, private document identity가
없다.

## 6. 다음 승인 지점

W5-3는 exact-installed baseline 하나를 얻은 단계다. fallback 원인의 첫 divergence를 알려면 같은 input
bytes와 ambient manifest를 유지한 paired missing 상태가 필요하다. Stage W5-4는 disposable
Windows snapshot, 복원 검증과 대상 related face set을 메인테이너가 승인한 뒤 시작한다. 그 전에는
현재 호스트에서 rank 1 font 또는 KoPubWorld font를 설치하거나 제거하지 않는다.

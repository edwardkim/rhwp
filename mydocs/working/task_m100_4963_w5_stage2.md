---
kind: working-note
status: completed
issue: 4963
stage: W5-2
last_verified: 2026-08-22
---

# Task M100 #4963 W5 Stage 2 — fixture·분석기·source inventory

- **이슈**: [#4963](https://github.com/edwardkim/rhwp/issues/4963)
- **계획**: [`task_m100_4963.md`](../plans/archives/task_m100_4963.md)
- **브랜치**: `task_m100_4963`
- **단계 상태**: W5-2 완료, W5-3 승인 대기

## 1. 결론

W4 상위 17개 face를 한컴 통제 환경에서 조사하기 전에 필요한 공개 synthetic HWPX 입력, SFNT/PDF
관측 도구와 source readiness를 고정했다. 현재 즉시 read-only exact canary에 들어갈 수 있는 SFNT는
6개, 기존 한컴 2022 HFT evidence로만 시작할 수 있는 face는 1개, bytes와 직접 identity anchor가 없어
막힌 face는 10개다.

이 단계는 system font를 설치·제거하지 않았고 한컴을 실행하지 않았다. 제품 font metric DB, fallback,
paint와 renderer도 변경하지 않았다.

## 2. 공식 KoPubWorld 공급물

한국저작권위원회 공유마당의 공식 상세 record에서
[KoPubWorld 돋움체 L](https://gongu.copyright.or.kr/gongu/wrt/wrt/view.do?menuNo=200023&wrtSn=13287211)과
[KoPubWorld 바탕체 L](https://gongu.copyright.or.kr/gongu/wrt/wrt/view.do?menuNo=200023&wrtSn=13287214)을
확인했다.

| document face | UCI | official ZIP SHA-256 | font SHA-256 | SFNT `fsType` |
| --- | --- | --- | --- | ---: |
| `KoPubWorld돋움체 Light` | `G905-13287211` | `a2bd6195…e947a604` | `069494cc…382bc84f` | 8 |
| `KoPubWorld바탕체 Light` | `G905-13287214` | `a8fb7ae0…b2323d60` | `e3ee21a8…cb3fb18` | 8 |

공식 record의 표시와 별도 이용 조건을 함께 보수적으로 해석해 font bytes는 저장소 밖 local-only로만
보관하고 재배포하지 않는다. `OS/2.fsType=8`은 editable embedding이라는 기술 신호이지 배포 허가를
대신하지 않는다. 기존 KoPub과 KoPubWorld는 이름·bytes가 다르므로 exact·alias·official successor로
자동 연결하지 않았다.

## 3. 결정론적 HWPX fixture

`samples/hwpx/ref/ref_empty.hwpx`의 고정 SHA-256에서 rank 1 `문체부 바탕체` canary를 생성한다.

- 장평 100/90/80 × 자간 0/-5/-10 × kerning off/on = 18개 char property
- 본문 18개 조합과 표 셀·글상자·머리말·꼬리말 문맥
- stored LineSeg lane과 fresh candidate lane을 함께 보존
- ZIP timestamp·entry order·compression을 고정하고 font bytes를 넣지 않음
- HWPX SHA-256 `8ded3aff…3ddb3f`, manifest SHA-256 `adf7185f…68d1c3a`

같은 명령을 두 번 실행한 HWPX와 manifest가 각각 byte exact였다. `rhwp scan --probe --json`은 HWPX,
extension/magic 일치, parse 성공, 1쪽을 보고했고 `rhwp export-text --json`은 머리말·18개 본문 조합·표
셀·글상자·꼬리말 text를 모두 읽었다.

## 4. 분석기 경계

### SFNT

font SHA-256, collection face index, name table, units-per-em, glyph/cmap 수, `hmtx` advance, side bearing,
canonical outline digest와 `OS/2.fsType`을 path 없이 기록한다. 17개 원장은 예상 hash와 exact document
face name이 일치하지 않으면 생성되지 않는다.

### PDF

`qpdf --check`와 page/object 수, `pdffonts`의 base/subset/type/encoding, MuPDF structured text의
line/page 위치, trace의 glyph/CID와 advance를 결합한다. MuPDF `g@adv`와 text/PDF transform을 적용한
사용자 공간 이동량을 함께 기록하지만, 원본 SFNT가 없으므로 이를 `hmtx`라고 부르지 않는다.

파일 64 MiB, 50쪽, 200,000 object, 200,000 glyph, 도구 출력 128 MiB, 실행 30초 상한을 적용한다.
regular file만 받고 path traversal과 symlink를 거부하며 child timeout·출력 초과 시 process group을
종료한다.

## 5. 검증 결과

```text
python3 -m unittest -v scripts.tests.test_oracle_stage2
tests 6, pass 6, fail 0

node --test scripts/tests/font_typesetting_risk_rank.test.mjs
tests 14, pass 14, fail 0

fixture HWPX repeat SHA-256
8ded3aff6f0286ee5ee4ad9c66732026fa627220b529e5d0fa7b9d51bc3ddb3f

fixture manifest repeat SHA-256
adf7185faab35edaae62b6c77c5a60642b6a62eee3dc7656bd98042f368d1c3a

sample PDF repeat canonical SHA-256
ee613512c6d0dd6d029da4c54c5ec55fec7605f65f7821aa58e615d48876dd2a
```

음성 제어는 corrupted font, corrupted PDF, oversize PDF, path escape와 input/output symlink를 포함한다.
공개 산출물에는 font bytes, private corpus identity, 절대 local font path가 없다.

## 6. 보호 불변식과 다음 승인 지점

- fixture bytes는 ladder 상태마다 동일해야 한다.
- exact/alias/successor/substitution/surrogate/Hancom missing 관계를 합치지 않는다.
- SFNT `hmtx`와 PDF observed advance를 분리한다.
- source-unavailable 10개를 임의 이름 매칭으로 채우지 않는다.
- system font mutation은 W5-4 disposable snapshot 승인 전 금지한다.
- 제품 metric/fallback/paint 변경은 W5 밖의 evidence별 후속 이슈로 분리한다.

Stage W5-3는 기존 한컴 2022 evidence를 hash 고정해 profile로 투영하고, 설치 상태를 바꾸지 않는
read-only exact-installed canary만 수행한다. remote 변환이나 업로드가 필요하면 별도 승인 지점에서
정지한다.

---
kind: working
status: active
issue: 5599
---

# 한컴 PUA 표시 대체표 15항목 확정 (#5599 부분)

작업 브랜치: `feat/5599-hancom-pua-verified`
대상: `src/renderer/hancom_pua.rs` · `tests/cases/issue_5599_hancom_pua_display.rs`

## 한 줄

저장소에 이미 있는 **한컴 출력 PDF**로 미매핑 PUA 15종의 glyph 를 확정해
`VERIFIED_HANCOM_PUA_DISPLAY` 에 넣었다(12 → 27항목). 한글 설치 없이 리눅스에서 규약
("한컴 PDF 대조")을 그대로 만족한다.

## 한글 없이 오라클을 쓴 방법

`pdf/` 에는 한컴이 내보낸 PDF 가 섞여 있다. `pdffonts` 로 임베드 폰트를 보면 구별된다.

| PDF | 임베드 폰트 | 판정 |
|---|---|---|
| `pdf/hwp3-sample11-2020.pdf` | HCRBatang · HCRDotum | 한컴 출력 ✅ |
| `pdf/복학원서-2022.pdf` | Haansoft Batang | ✅ |
| `pdf/issue2083_hide_fill_page-2020.pdf` | DejaVu 만 | ❌ (한컴 아님) |

한컴 PDF 는 폰트를 임베드하므로 `pdftocairo -png -r 400` 이 원래 glyph 를 그대로 그린다.
문서의 PUA 출현 위치와 PDF 텍스트 좌표를 맞춰 그 자리를 잘라 보면 glyph 를 눈으로 확정할 수
있다(도구: `tools` 밖 임시 스크립트, 산출물은 커밋하지 않음).

## 확정한 항목

### 罫線 조각 6종 — `hwp3-sample11` + `pdf/hwp3-sample11-2020.pdf`

| 코드포인트 | glyph | 근거 |
|---|---|---|
| U+F0806 | `┌` | p129 세 줄 연속 `F0806 / F0810 / F080C` → ┌ │ └ |
| U+F0807 | `┬` | p22 `━━━F0807━━━` |
| U+F0808 | `┐` | p6 `SUN OS 4.1.1 ━F0808` |
| U+F080C | `└` | p22 `F080C━>`, p129 |
| U+F080E | `┘` | p6 `SUN OS 4.1.4 ━F080E` |
| U+F0810 | `│` | p6 중간 두 줄, p129 |

p6 의 네 줄은 한컴 출력에서 `━┐ / │ / │ / ━┘` 세로 묶음으로 보인다. `SO-SUEOP.hwpx`
(한컴 PDF `pdf/SO-SUEOP-2024.pdf`)도 같은 묶음을 쓴다.

### 원숫자 9종 — 같은 문서 p23 NVRAM 바이트 라벨

한컴 출력의 라벨 줄: `⓪ ① ② ③ ④ ⑤ ⑥ ⑦ ⑧ ⑨ ⓐ ⓑ`
문서 본문의 같은 자리: `F0288 F0289 F028A ③(리터럴) F028C F028D F028E F028F F0290 F0291 ⓐ ⓑ`

| 코드포인트 | glyph |
|---|---|
| U+F0288 | `⓪` |
| U+F0289 | `①` |
| U+F028A | `②` |
| U+F028C | `④` |
| U+F028D | `⑤` |
| U+F028E | `⑥` |
| U+F028F | `⑦` |
| U+F0290 | `⑧` |
| U+F0291 | `⑨` |

같은 쪽 아래 `Host-ID = ①+ⓒ+ⓓ+ⓔ`(= 바이트 1·c·d·e) 줄이 `F0289 = ①` 을 한 번 더 확인해
준다. **U+F028B(=③)는 이 문서가 리터럴 ③ 을 써서 근거가 없으므로 넣지 않았다** — 연속
구간이라고 추정하지 않는다(모듈 규약).

## 확정하지 못해 뺀 것

| 코드포인트 | 문서 | 상태 |
|---|---|---|
| U+F0848 (17회) | 2025 행정업무운영 편람 | 한컴 출력에서 짧은 가로 막대 bullet 로 보이나 `─`/`—`/`-` 중 무엇인지 폭까지 확정 못 함 |
| U+F0090 | img-start-001 | 여러 갈래 꽃무늬 bullet — 공개 글꼴 대체 문자 특정 실패 |
| U+F03A7 · U+F03A8 | 편람 p43 | 네모 안 `+`/`−` 로 보이나 같은 줄 리터럴 `①` 이 네모로 그려져 정렬 신뢰도 부족 |
| U+F02C5 | mel-001 | PDF 쪽 PUA 워드와 문서 위치 정렬 실패 |
| U+F081C | 복학원서 | 점선 rule 로 보이나 대체 문자 미정 |
| U+F03DA (150회/22문서) | issue2083 | 그 PDF 가 한컴 출력이 아님 → 판정 불가 |

이슈의 최대 덩어리 `U+F02B1`–`U+F02B7`·`U+F0832`·`U+F03FF`·`U+F02EC` 는 admrul 코퍼스에만
있어 이 환경에서 볼 수 없다. 그래서 이 작업은 이슈를 **부분 해소**한다.

## 검증 실측

```
rhwp export-svg samples/hwp3-sample11-hwp5.hwp   (p6)
  전: raw PUA 두부 3종(0F0808 · 0F0810 · 0F080E) 이 코드 숫자 상자로 그려짐
  후: ┐ │ │ ┘  — 한컴 출력과 같은 세로 묶음
  두 쪽(p6·p23) 모두 매핑된 코드포인트의 raw PUA 잔존 0
```

## 시험 명령

```
cargo test --profile release-test --test regression_suite_006 issue_5599   # 신규 가드 2건
cargo test --profile release-test --tests --no-fail-fast                   # 전체
```

신규 가드는 수정 전 코드에서 둘 다 실패, 수정 후 통과.

## fmt 게이트

```
cargo fmt --all -- --check
cargo clippy -- -D warnings
```

## 환경

Linux 6.17 · rhwp v0.8.4 · 한글 미설치. 오라클은 저장소에 커밋된 한컴 PDF 를 썼다.

## PR 메모

`gh pr create --base devel --body-file ...`. 이슈를 완전히 닫지 않으므로 `closes` 대신
`#5599 부분 해소` 로 참조한다.

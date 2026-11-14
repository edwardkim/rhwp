# 정본 보호값 재현 조사

## 동일 환경의 실제 devel·후보 대조

검증 base는 최신 `53da4d2da28ae0cacf8feac3a989cc6a69fb2622`, 후보 제품 source는 `5e617897863e4fb3628404be6fad8a1c8081bf2c`다. 두 Native 바이너리를 각각 lib/bin 함께 빌드해 같은 입력·독립 PDF·현재 승인 font context·실제 프로그램 공급으로 정본 18 dataset·1,771쪽씩을 전수 실행했다.

현재 정본 `final.tsv` SHA-256은 `1063d78078b704a2bfb712ca3929ab251596dd391b41bc1a9a44e9935e4990e5`다. 과거 공식 두 자리 값보다 낮은 행은 양쪽 모두 **51건**이고, base·후보의 공식 측정값 차이는 **0건**이다. 이번 수정이 만든 Native 하락으로 분류하지 않았지만 정본 gate 통과로 처리하지도 않았다. [base TSV](protected-base-native.tsv) · [후보 TSV](protected-native.tsv).

| 문서 key | 과거 공식값 미재현 행 |
| --- | --- |
| nested | 6 |
| sample16-hwp2022 | 38 |
| sample16-hwp2024 | 2 |
| sample16-hwpx | 2 |
| seo-body-list-typed | 1 |
| seo-helvetica-styles | 1 |
| yeongwol | 1 |

최저는 양쪽 모두 97.99%다. 90% 이상은 특별 보호값 하락 금지 조건의 대체가 아니다. 편람 HWPX Native 383쪽 `--handbook-gate`는 별도로 통과했다.

PNG는 1,771쌍 중 1,762쌍이 바이트 단위로 같다. 다른 9쪽은 HWP3 sample10의 괘선 영역이며, 원본 PDF·base·후보를 같은 배율로 직접 비교했다. 그 문서의 기존 끝 괘선 표시 차이를 이번 수정이 해결했다고 주장하지 않는다. 공식값과 본문 위치는 동일하다. [전쪽 PNG 비교 집계](post-fixes-protection-base-native-png.json). 새 devel 기능 보존 41쪽씩은 Native/WASM 82쌍 모두 바이트 단위로 같다. [보존 결과](post-fixes-incoming-preservation.json).

## 기준 생산 source와 실제 글꼴

최근 기준 생산 기록은 `planet6897_integration_20261114/protected-validation.json`의 runtime `5fa2645186fc8bec4e53074aa6ff0af7ef67f0ca`와 “normal installed fonts; no regression sandbox”다. GitHub의 실제 Git tree/blob을 읽어 현재 base와 비교했다. Cargo.toml·Cargo.lock·vendor는 같고 src의 차이는 `shape_layout.rs` 한 파일이다. 내용은 뒤에 통합한 #9021의 공통 control 줄 소속·TAC 공유 기준선 보완이다. [실제 blob SHA](protected-producer-src-differences.json) · [차이](protected-producer-shape-diff.patch). 따라서 source와 환경이 모두 같은 생산 결과라고 가정하지 않는다.

현재 실행은 해당 기록의 문서별 font environment를 그대로 사용했다. 이전에 커밋된 프로그램 공급 목록에 맞춰 준비한 파일들과 선택한 CFF 프로그램 ID·해시를 확인했고, Noto와 수식 글꼴은 해당 문서에만 공급했다. [실제 실행별 프로그램 해시](actual-font-program-receipts.json). 상용 글꼴 파일 자체는 포함하지 않는다.

선행 글꼴 목록의 HBATANG은 32,483,900바이트·SHA-256 `bef6371830a14b7e275087f86517074392a54a9147819506a2c07637c88b4068`이다. 사용자가 안내한 `/home/edward/mygithub/ttfs/hwp/HBATANG.TTF`는 재확인 결과 32,361,044바이트·`35f84328500fc2c3ee0b148aa75de0ed384bf9eee8dee9148c01b0a11a27fe05`다. 한컴 2020 설치본도 현재 파일과 같다. 최신 기준 생산자의 전체 실공급 목록은 확보하지 못했다. 다른 해시의 존재만으로 이 차이의 원인을 HBATANG으로 확정하지 않는다.

## 진행 상태와 다음 확인

fresh WASM 보호 검증: {"backend": "wasm", "rows": 1771, "declines": 56, "minimum": 94.65}

현재 조판 보완 #9070·#9071은 특별 보호값 재현 조건 때문에 제출 준비 완료로 판정하지 않는다. 기준값·기대값·font mapping·비교 영역을 임의로 바꾸거나 기존 차이 예외를 적용하지 않았다. 필요한 자료는 기준 생산자의 실제 프로그램 목록/해시 또는 같은 환경의 현재 후보 전쪽 검증이다. 해당 근거를 대조해 원인을 분리하고, 필요한 보완·재검증을 완료한 뒤 판정을 갱신한다. 전체 회귀·lint·Native 공식 검사는 독립적으로 계속 진행한다.

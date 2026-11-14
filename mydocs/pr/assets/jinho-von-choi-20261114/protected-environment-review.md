# 정본 보호값 재현 조사

## 동일 환경의 실제 devel·후보 대조

검증 base는 최신 `53da4d2da28ae0cacf8feac3a989cc6a69fb2622`, 후보 제품 source는 `eeaecff59e834f28437c98b509089bbb36a7682d`다. 두 Native 바이너리를 각각 lib/bin 함께 빌드해 같은 입력·독립 PDF·현재 승인 font context·실제 프로그램 공급으로 정본 18 dataset·1,771쪽씩을 전수 실행했다.

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

PNG는 1,771쌍 중 1,762쌍이 바이트 단위로 같다. 다른 9쪽은 HWP3 sample10의 괘선 영역이며, 원본 PDF·base·후보를 같은 배율로 직접 비교했다. 그 문서의 기존 끝 괘선 표시 차이를 이번 수정이 해결했다고 주장하지 않는다. 공식값과 본문 위치는 동일하다. [전쪽 PNG 비교 집계](protected-base-native-png.json). 새 devel 기능 보존 41쪽씩은 Native/WASM 82쌍 모두 바이트 단위로 같다. [보존 결과](incoming-preservation.json).

## 기준 생산 source와 실제 글꼴

최근 기준 생산 기록은 `planet6897_integration_20261114/protected-validation.json`의 runtime `5fa2645186fc8bec4e53074aa6ff0af7ef67f0ca`와 “normal installed fonts; no regression sandbox”다. GitHub의 실제 Git tree/blob을 읽어 현재 base와 비교했다. Cargo.toml·Cargo.lock·vendor는 같고 src의 차이는 `shape_layout.rs` 한 파일이다. 내용은 뒤에 통합한 #9021의 공통 control 줄 소속·TAC 공유 기준선 보완이다. [실제 blob SHA](protected-producer-src-differences.json) · [차이](protected-producer-shape-diff.patch). 따라서 source와 환경이 모두 같은 생산 결과라고 가정하지 않는다.

현재 실행은 해당 기록의 문서별 font environment를 그대로 사용했다. 이전에 커밋된 프로그램 공급 목록에 맞춰 준비한 파일들과 선택한 CFF 프로그램 ID·해시를 확인했고, Noto와 수식 글꼴은 해당 문서에만 공급했다. [실제 실행별 프로그램 해시](actual-font-program-receipts.json). 상용 글꼴 파일 자체는 포함하지 않는다.

선행 글꼴 목록의 HBATANG은 32,483,900바이트·SHA-256 `bef6371830a14b7e275087f86517074392a54a9147819506a2c07637c88b4068`이다. 사용자가 안내한 `/home/edward/mygithub/ttfs/hwp/HBATANG.TTF`는 재확인 결과 32,361,044바이트·`35f84328500fc2c3ee0b148aa75de0ed384bf9eee8dee9148c01b0a11a27fe05`다. 한컴 2020 설치본도 현재 파일과 같다. 최신 기준 생산자의 전체 실공급 목록은 확보하지 못했다. 다른 해시의 존재만으로 이 차이의 원인을 HBATANG으로 확정하지 않는다.

## 진행 상태와 다음 확인

최종 fresh WASM 보호 1,771행은 최저 94.65%이며 과거 공식값 미재현은 56행이다. Native와 합계 3,542행·107건이고 실제 실행에서 누락·쪽수 불일치·90% 미달은 없다. 특별 보호 gate는 통과하지 않았다. [최종 집계](protected-results.json). 미재현 56행은 정확한 base source의 Docker·wasm-opt 완료 프로그램으로 같은 조건의 비교를 진행 중이다.

현재 조판 보완 #9070·#9071은 특별 보호값 재현 조건 때문에 제출 준비 완료로 판정하지 않는다. 기준값·기대값·font mapping·비교 영역을 임의로 바꾸거나 기존 차이 예외를 적용하지 않았다. 필요한 자료는 기준 생산자의 실제 프로그램 목록/해시 또는 같은 환경의 현재 후보 전쪽 검증이다. 해당 근거를 대조해 원인을 분리하고, 필요한 보완·재검증을 완료한 뒤 판정을 갱신한다. 최종 전체 회귀 13,574개와 모든 Rust lint는 통과했고 Native 공식 3종도 4,073개 통과했다.


## 사용자 안내 파일 재확인

2026-11-15 KST 사용자가 다시 확인한 `/home/edward/mygithub/ttfs/hwp/HBATANG.TTF`가 실제 공급 파일이다. 현재 크기·해시는 위 `35f843…` 기록과 같으며 이 파일을 찾지 못했거나 잘못된 파일로 판정한 것이 아니다. 같은 파일은 #9070 독립 Print의 hmtx·전진 대조에서도 확인했다. 과거 기록과 해시가 다른 사실을 실제 보호 차이의 인과관계로 확대하지 않는다.


## 미재현 문서의 직접 판독

현재 최종 프로그램의 독립 PDF·Native·fresh WASM을 같은 페이지·배율로 나란히 읽었다. 아래는 미재현 범위의 대표 판독이며 107행의 원인을 모두 글꼴로 확정한 것이 아니다.

| 원본·쪽 | 직접 확인한 차이와 보존 범위 |
| --- | --- |
| [seo-helvetica-styles 1쪽](protected-review/seo-helvetica-styles.png) | STYLE 1의 Native 글꼴 폭·굵기와 WASM 획 굵기가 독립 Print와 다르다. 기울임/굵기 조합의 문자 자체와 순서는 남는다. HBATANG 한 파일로 설명할 수 없는 별도 영문 face 선택 범위다. |
| [seo-pic-crop-01 1쪽](protected-review/seo-pic-crop-01.png) | 두 그림 내용·자르기 위치는 같고 독립 Print의 외곽 검은 테두리가 제품에서는 더 가늘거나 일부 희미하다. 실제 차이를 높은 점수로 면제하지 않는다. |
| [nested 16쪽](protected-review/nested.png) | 문단·제목의 줄 소속과 내용 순서를 대조했다. 획·영문 메트릭 차이가 남으며 source와 환경의 인과관계를 추가로 구분해야 한다. |
| [sample16-hwp2022 30쪽](protected-review/sample16-hwp2022.png) | 표 외곽·셀 내용 소속과 후속 내용은 보존된다. 글꼴 획·영문 형태의 차이를 확인했다. 38행 전체의 단일 원인으로 확대하지 않는다. |

Native의 정확한 base 비교는 전쪽 동일 측정값이지만, 동일하다는 이유로 위 기존 차이를 해결됐다고 판정하지 않는다. 이 범위를 추가 보완하거나 현재 PR에 한정해 기존 차이를 수용하는 명시적 결정 전에는 통합을 보류한다.

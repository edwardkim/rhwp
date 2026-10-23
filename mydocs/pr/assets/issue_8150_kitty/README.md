# #8150 CLI 출력 증적

[최종 보고](../../../report/task_m100_8150_report.md),
[3단계 검증](../../../working/task_m100_8150_stage3.md)의 공개 입력과 대표 PNG·전쪽 TSV 원본이다.
2026-10-22 사용자의 Open PR 진행 승인에 따라 보존했다. PR review 기록이나 CI 성공 기록은 아니다.

## 실행 신원과 적용 범위

- 기준 devel: `2e087a5b4bc1515ad877e8020b30114639a59392`.
- 제출 전 검증 checkout: `c11887343264e685aa86620a71af43d82661ee58`.
- Native 전송 제품 소스: `2d2d9b1921eaef5f586937739920dbee4689b75d` 이후 불변.
- 최종 Native 실행 파일 SHA-256: `1b8801c19f7f3518ba548beaf72fdb830d4a4c6bbe5001a79ac4735d631d7cdb`.
- 실제 kitty/대표 review 생성 실행 파일 SHA-256: `9cd196a82f728271a51099b5a9721a92589575c8e4eea784c60d70a2d574101e`.
- 기준 Native 실행 파일 SHA-256: `1cae3dd3a9b2c28af1fed9634072c42b82311b106386a2ace2f1d77e07eb7207`.
- Native 시각 비교는 Print·compat 2022·96 DPI이며 동일 로컬 fontconfig와 공개 번들을 사용했다.
  11쪽 PNG·render tree·TSV 값과 22개 Kitty 스트림이 두 후보 바이너리에서 같음을 재출력으로 확인했다.
- 새 기능은 Native CLI 전용이며 공통 조판·paint·폰트 producer는 바꾸지 않는다.
  WASM/Studio 실행·WASM TSV는 실제 소비 경로상 비해당이다. WASM32 Clippy는 통과했다.

## 실제 터미널과 대표 비교

`kitty-local-table-360.png`는 kitty 0.49.2의 실제 Linux WSLg 화면이다. 공개 표 문서의
0-based 첫 쪽을 Print·360px로 직접 출력했다. 한글·표와 이미지 아래의 `EXIT=0`·프롬프트를 확인했다.
로컬 화면 캡처를 SSH 또는 Windows 화면으로 표시하지 않는다.

`native-table-p001-review.png`와 `native-table-p001-overlay.png`는 같은 문서 첫 쪽의
Native 출력과 기존 한컴 Print PDF를 대조한 보조 증거다. 기존 글꼴·획 굵기 차이가 남아 있다.
새 조판 개선·한컴과의 완전 일치·성능 향상의 증거로 쓰지 않는다.

전쪽 TSV는 base/head 각 3+6+1+1=11쪽이다. 누락·쪽수 불일치가 없고 모든 페이지의
raw 값이 base와 같다. 최저는 표 96.64066%, 각주 98.95047%, crop 99.95079%, 복합 100%다.
최대 하락은 0.00000%p다. TSV 원문은 생성 결과를 바이트 그대로 복사했다.

## 기존 커밋 입력 확인

다음 8개 파일은 제출 commit의 Git blob과 실제 실행 입력의 SHA-256이 같다.
새 입력·PDF 사본을 중복 추가하지 않았다. 기준 PDF는 기존 Sixel 작업에서 출처와 원문 대응을
확인한 한컴 Print이며, 각 3/6/1/1쪽이다.

| 입력/기준 PDF | 바이트 | SHA-256 |
| --- | ---: | --- |
| `samples/hwp_table_test.hwp` | 29184 | `dc3e57d7577447ae4b47f59bdb0f499a5ac29066b62af8a650bcdd8d70eee32a` |
| `samples/footnote-01.hwp` | 32768 | `5bbc8a8fd23415aad59dd91d1eb261050946d9c64635933fcd49609ec2cc94e5` |
| `samples/pic-crop-01.hwp` | 131072 | `3b5afbe4cdb18faa49452a9cbed770d5caa27aed2f83b708e9ab8516c65ebf4c` |
| `samples/issue7688/arrow-controls/control-compound.hwpx` | 10548 | `ab309908bce072332ab3b635a4a99ae58b1afe2806aada41871baefd321b804d` |
| `samples/issue7688/reference/table-core-2020.pdf` | 27978 | `c04ef8b2819529f83fe3e6043683982600535432426c8fb58f6e5cd31ea00963` |
| `pdf/footnote-01-hwp-2020.pdf` | 114877 | `2b47aa31b3d135f1621ba478189efcbfcacc77dfbc6c9d9082239f7885bb546e` |
| `samples/issue7688/reference/image-crop-2020.pdf` | 16784 | `c9d29de7680e5642a24818a6924ef5f42b18e1194766b693d636853d8782d16d` |
| `samples/issue7688/reference/compound-2020.pdf` | 19395 | `1bd27989056bead55663b4b8735a822e613519ce00c4fe59865dfd285570d6f4` |

## 산출물 SHA-256

| 파일 | SHA-256 |
| --- | --- |
| [base-native/compound_silhouette.tsv](base-native/compound_silhouette.tsv) | `76292db165fc1a04b17dbbd1b6444ba8cf19f40aeff5fc42900007225280ac3b` |
| [base-native/footnote_silhouette.tsv](base-native/footnote_silhouette.tsv) | `cfe2cd10b4652e36cf2b56b73bdf7aab927b20721af0163715a102148f44185e` |
| [base-native/image-crop_silhouette.tsv](base-native/image-crop_silhouette.tsv) | `4b53aab7a24f6b13443358218fee25dedd88ba127fdbf321588d94133f6d79c0` |
| [base-native/table-core_silhouette.tsv](base-native/table-core_silhouette.tsv) | `5d0004f6798530491d439d5d495b799c4a5c3c7aa06f4f4b9075392a3488c1c1` |
| [head-native/compound_silhouette.tsv](head-native/compound_silhouette.tsv) | `76292db165fc1a04b17dbbd1b6444ba8cf19f40aeff5fc42900007225280ac3b` |
| [head-native/footnote_silhouette.tsv](head-native/footnote_silhouette.tsv) | `cfe2cd10b4652e36cf2b56b73bdf7aab927b20721af0163715a102148f44185e` |
| [head-native/image-crop_silhouette.tsv](head-native/image-crop_silhouette.tsv) | `4b53aab7a24f6b13443358218fee25dedd88ba127fdbf321588d94133f6d79c0` |
| [head-native/table-core_silhouette.tsv](head-native/table-core_silhouette.tsv) | `5d0004f6798530491d439d5d495b799c4a5c3c7aa06f4f4b9075392a3488c1c1` |
| [kitty-local-table-360.png](kitty-local-table-360.png) | `d172a7fe3610a19564b99ae6ac62c64c40883d7a7869c9c7ae151f2d2a1408f7` |
| [native-table-p001-overlay.png](native-table-p001-overlay.png) | `93298af1fb43b7b5139bc07e83a1f99c612c8f3b4350236a8aa3f0ede1d8cfe0` |
| [native-table-p001-review.png](native-table-p001-review.png) | `48cfe0c3d98737c15304773871558586689b3e0391309269c95f16b86ed9565a` |

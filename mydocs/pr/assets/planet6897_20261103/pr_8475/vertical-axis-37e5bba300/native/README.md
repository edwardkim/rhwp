# 세로쓰기 합성 줄 보정의 Native 전쪽 증거

소스 `37e5bba3007614d9304c3dbf863503a8c7340405`, Native 실행 파일 SHA-256 `359eb1788753114c9799fb03b1d66678ea91b58802aa85f18c7fda54aa6890f0`.

기존 한컴 Print를 재사용했다. PDF 재출력·내용 변경·글꼴 배포는 하지 않았다. 96 DPI, Print profile, 색상 경계 보정 전 2px raw다. 시각 측정에는 시스템 글꼴을 사용하고 Rust 회귀는 공개 기본 글꼴 5개만 노출했다.

| 입력 · 독립 Print PDF | 전체 쪽수 · 최저 raw | 실제 전쪽 TSV | 이전 후보와 PNG 대조 |
| --- | --- | --- | --- |
| [vertical_fit.hwpx](<../../../../../../../samples/issue8470/vertical_fit.hwpx>) · [vertical_fit-2020.pdf](<../../../../../../../pdf/issue8470/vertical_fit-2020.pdf>) | 5쪽 · 100.0% | [TSV](case-00-silhouette.tsv) | 동일 4쪽, 변경 [5] |
| [vertical_oversize.hwpx](<../../../../../../../samples/issue8470/vertical_oversize.hwpx>) · [vertical_oversize-2020.pdf](<../../../../../../../pdf/issue8470/vertical_oversize-2020.pdf>) | 5쪽 · 100.0% | [TSV](case-01-silhouette.tsv) | 동일 4쪽, 변경 [5] |
| [42565_입법예고문.hwp](<../../../../../../../samples/issue8470/42565_입법예고문.hwp>) · [42565_입법예고문-2020.pdf](<../../../../../../../pdf/issue8470/42565_입법예고문-2020.pdf>) | 16쪽 · 94.92448% | [TSV](case-02-silhouette.tsv) | 동일 16쪽, 변경 [] |
| [scaffold_outline_a.hwpx](<../../../../scaffold_outline_numbering/scaffold_outline_a.hwpx>) · [scaffold_outline_a-2024.pdf](<../../../../../../../pdf/issue8540/scaffold_outline_a-2024.pdf>) | 1쪽 · 100.0% | [TSV](case-03-silhouette.tsv) | 동일 1쪽, 변경 [] |
| [scaffold_outline_b.hwpx](<../../../../scaffold_outline_numbering/scaffold_outline_b.hwpx>) · [scaffold_outline_b-2024.pdf](<../../../../../../../pdf/issue8540/scaffold_outline_b-2024.pdf>) | 1쪽 · 100.0% | [TSV](case-04-silhouette.tsv) | 동일 1쪽, 변경 [] |
| [3200477_icao_procedure.hwpx](<../../../../../../../samples/issue6029/3200477_icao_procedure.hwpx>) · [issue6029-3200477_icao_procedure-2024.pdf](<../../../../../../../pdf/issue6029-3200477_icao_procedure-2024.pdf>) | 1쪽 · 97.97401% | [TSV](case-05-silhouette.tsv) | 동일 1쪽, 변경 [] |

입력과 PDF의 실제 SHA-256:

| 입력 | 입력 SHA-256 | Print PDF SHA-256 |
| --- | --- | --- |
| `samples/issue8470/vertical_fit.hwpx` | `6423c7d010e7333a0749119c2d06833f64fd88f5aa6814f2665c47430fd88c7a` | `f714bb7909f44ec3eedfa9c3d7a0055180ed775f252ccf9b5a056efb093e446a` |
| `samples/issue8470/vertical_oversize.hwpx` | `b7ff674351d5f8fa0e2abfb12a7ce11eb21e53b73337c867633809c0f6490883` | `a7485d7b4384869f00d4956b9b0f35582fd5b65662025092742ffddcc8a83b70` |
| `samples/issue8470/42565_입법예고문.hwp` | `30d11dff82e05085a0f69298b74732604dff29e082a004f9a066525cc7fa19a0` | `0377a6cbb63a08fe91131086654733e652401e79950262fcabe51f1fc8691341` |
| `mydocs/pr/assets/scaffold_outline_numbering/scaffold_outline_a.hwpx` | `7dc0e7f41aec968c6510bf2c430dd50cee67590c15bd5f3cc0628a02c883f751` | `20536ee5ac4bdd6dc52cc5f0caf246d07713664e6ca1e46be945358e2c92e12a` |
| `mydocs/pr/assets/scaffold_outline_numbering/scaffold_outline_b.hwpx` | `bdd247054116d4fb181f86a843376ccc9318914a2207bc8fd9802392b00d5eba` | `8f4a28a633bcdac9d7dc99edd68cf0705c2e7eadecf9547541763b7f99440325` |
| `samples/issue6029/3200477_icao_procedure.hwpx` | `d3fb34289a9a49315510a70166aa4451a11e71eb79c7ac129fdf9da6426f75a2` | `75ce9ab871a588ecbc8bd07d3ca05a61a8c4f70c2487cd66983b70f950d6d1d0` |

두 축약 입력 5쪽의 임용전경력 다섯 글자가 한 세로줄의 셀 안에 표시된다. 표의 앞뒤 행·종료·쪽 소속도 직접 확인했다. 29쪽 중 나머지 27쪽은 c183 제품과 PNG 바이트가 동일하다.

실물 10쪽 저장 세로줄 간격 및 정상 대조군의 연결 화살표 등 기존 차이는 그대로 남아 있다. 해당 페이지 PNG의 이전 후보 동일성을 확인했으며 이 보정을 모든 기존 차이의 해결로 표시하지 않는다.

[실제 판독 목록](../../../direct-reading-current.md)에 입력·Print·주장 쪽 review와 별도 overlay를 연결했다.

기본 옵션 Native 빌드 명령:

```bash
cargo build --locked --profile release-test --target-dir /home/tsjang/rhwp/target/pr-review --features native-skia --bin rhwp
```

실제 종료 코드 0. UTC 2026-11-04T23:35:08.743347+00:00 → 2026-11-04T23:42:30.638303+00:00.

fresh WASM·보정 후 전체 Rust·통합 브랜치 적용·최신 base 확인은 별도 완료 기록을 확인해야 한다. 이 Native 기록만으로 최종 승인을 주장하지 않는다.

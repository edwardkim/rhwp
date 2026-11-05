# 세로쓰기 합성 줄 보정의 fresh WASM 전쪽 증거

최적화 완료 source `37e5bba3007614d9304c3dbf863503a8c7340405`.
WASM SHA-256 `14fea1d90513ac2102b506eb56e9c8748b8eda7ca1451697cc0630444454dfd0`.
JS SHA-256 `bb9be9d806756697eddee5d185e80e9256bd0d97947d78a94777222100f12393`.

기존 한컴 Print를 재사용했다. 96 DPI, Print profile, 색상 경계 보정 전 2px raw, 실제 WASM Canvas 출력이다.

| 입력 · 독립 Print PDF | 전쪽 최저 raw · 전체 쪽수 | 전쪽 TSV | 직전 c183 제품과 대조 |
| --- | --- | --- | --- |
| [vertical_fit.hwpx](<../../../../../../../samples/issue8470/vertical_fit.hwpx>) · [vertical_fit-2020.pdf](<../../../../../../../pdf/issue8470/vertical_fit-2020.pdf>) | 100.0% · 5쪽 | [TSV](case-00-silhouette.tsv) | PNG 동일 4쪽, 개선 [5]쪽 |
| [vertical_oversize.hwpx](<../../../../../../../samples/issue8470/vertical_oversize.hwpx>) · [vertical_oversize-2020.pdf](<../../../../../../../pdf/issue8470/vertical_oversize-2020.pdf>) | 100.0% · 5쪽 | [TSV](case-01-silhouette.tsv) | PNG 동일 4쪽, 개선 [5]쪽 |
| [42565_입법예고문.hwp](<../../../../../../../samples/issue8470/42565_입법예고문.hwp>) · [42565_입법예고문-2020.pdf](<../../../../../../../pdf/issue8470/42565_입법예고문-2020.pdf>) | 96.96022% · 16쪽 | [TSV](case-02-silhouette.tsv) | PNG 동일 16쪽, 개선 []쪽 |
| [scaffold_outline_a.hwpx](<../../../../scaffold_outline_numbering/scaffold_outline_a.hwpx>) · [scaffold_outline_a-2024.pdf](<../../../../../../../pdf/issue8540/scaffold_outline_a-2024.pdf>) | 100.0% · 1쪽 | [TSV](case-03-silhouette.tsv) | PNG 동일 1쪽, 개선 []쪽 |
| [scaffold_outline_b.hwpx](<../../../../scaffold_outline_numbering/scaffold_outline_b.hwpx>) · [scaffold_outline_b-2024.pdf](<../../../../../../../pdf/issue8540/scaffold_outline_b-2024.pdf>) | 100.0% · 1쪽 | [TSV](case-04-silhouette.tsv) | PNG 동일 1쪽, 개선 []쪽 |
| [3200477_icao_procedure.hwpx](<../../../../../../../samples/issue6029/3200477_icao_procedure.hwpx>) · [issue6029-3200477_icao_procedure-2024.pdf](<../../../../../../../pdf/issue6029-3200477_icao_procedure-2024.pdf>) | 98.74661% · 1쪽 | [TSV](case-05-silhouette.tsv) | PNG 동일 1쪽, 개선 []쪽 |

29쪽 모두 90% 이상이며 누락·쪽수·입력/PDF/제품 해시 오류는 0건이다. 두 축약 입력 5쪽만 개선됐고, 나머지 27쪽은 이전 제품과 PNG 바이트가 동일하다.

이 전쪽 수치만으로 직접 판독을 대신하지 않는다. 주장 쪽 review/overlay의 실제 판독과 최종 Rust·Native 필수 편람 게이트·통합 브랜치 적용을 함께 완료해야 한다. Native와 실제 입력/PDF SHA는 [Native 증거](../native/README.md)에 연결했다.

제품 빌드 명령:

```bash
CARGO_BUILD_JOBS=8 CARGO_TARGET_DIR=/home/tsjang/rhwp/target/pr-review \
  sh scripts/wasm-pack-locked.sh --target web \
  --out-dir /home/tsjang/rhwp/output/pr-review/planet6897-20261103/producer-packages/vertical-axis-37e5bba300
```

UTC 2026-11-04T23:42:32.246706+00:00 → 2026-11-04T23:58:24.981844+00:00, 실제 exit 0. wasm-opt 최적화를 생략하지 않았다.

## 실제 판독

`vertical_fit.hwpx` 5쪽의 [review](case-00-p005-review.png)와 별도
[overlay](case-00-p005-overlay.png)를 직접 확인했다. 다섯 글자의 단일 세로줄·셀 내부 포함과
행 1–30·표 종료를 보존한다. 실물·정상 대조 쪽의 판독도 아래와 같이 완료했다.

`vertical_oversize.hwpx` 5쪽의 [review](case-01-p005-review.png)와 별도
[overlay](case-01-p005-overlay.png)도 직접 확인했다. 행 1–36, 세 행 병합 칸의 다섯 글자와
닫는 외곽이 대응한다. 두 문제 쪽의 판독은 완료했으며 실물·정상 대조 쪽의 판독도 완료했고 전체
필수 검증은 진행 중이다.

실물의 10쪽 [review](case-02-p010-review.png) / [overlay](case-02-p010-overlay.png),
11쪽 [review](case-02-p011-review.png) / [overlay](case-02-p011-overlay.png)도 확인했다.
표의 한 쪽 소속·병합 칸·종료를 보존한다. 10쪽의 기존 저장 세로줄 간격 차이는 남아 있고
직전 후보와 PNG가 동일하다. 저장 줄이 없는 두 축약 입력의 누락 복원을 이 기존 차이의
해결로 확대하지 않는다.

저장 줄 정상 대조군의 [review](case-05-p001-review.png) / [overlay](case-05-p001-overlay.png)도
확인했다. 상자·세로 라벨 소속을 보존한다. Print의 연결 화살표가 사각 대체 기호로 보이는
기존 차이와 글자 간격 차이는 남는다. 이 페이지의 PNG는 c183 WASM과 바이트가 동일하다.
이에 따라 계획한 두 문제 쪽·실물 10/11쪽·정상 대조 1쪽의 Native/WASM 판독은 모두
완료했다. 필수 전체 Rust 및 통합 브랜치 적용·최신 base 확인은 아직 완료 전이다.

# 3단계 거대 셀의 지연 편집 회귀 복원

#7813에서 이관한 검사는 저장 원본의 115쪽 표를 편집할 때 쪽 나눔 조각과 셀 편집 결과를 함께 보호한다.
편집 직후에는 첫 쪽의 글자 범위와 커서가 갱신되고, 공개 조각은 지연 작업을 모두 처리한 뒤 바뀐다.
중간 공개 여부, 전후 조각의 유닛 연속성, 동기 재쪽 나눔과의 일치, 새 편집으로 인한 이전 작업 무효화를 검사한다.

| 원본 | 비교한 한컴 Print | 쪽수 | 기준 Native 최저 | 기준 fresh WASM 최저 |
| --- | --- | ---: | ---: | ---: |
| [HWP](../../../../../samples/issue1949_giant_cell_nested_tables_perf.hwp) | [한컴 2024](../../../../../pdf/issue1949_giant_cell_nested_tables_perf-hwp-2024.pdf) | 115 | [98.29%](7813-hwp-base-native.tsv) | [98.29%](7813-hwp-base-wasm.tsv) |
| [HWPX](../../../../../samples/issue1949_giant_cell_nested_tables_perf.hwpx) | [한컴 2020](../../../../../pdf/issue1949_giant_cell_nested_tables_perf-hwpx-2020.pdf) | 115 | [98.29%](7813-hwpx-base-native.tsv) | [98.29%](7813-hwpx-base-wasm.tsv) |

측정 기준은 `ca96e9f342a541da8aee40f96495180b6b2ee863`의 96dpi, 색상 경계 보정 전 2px 실루엣이다.
`venv/bin/python scripts/visual_sweep.py --silhouette-only --hwp <원본> --pdf <Print> --rhwp-bin <고정 Native> --compat <2024 또는 2022> --dpi 96`으로 비교했다.
WASM은 같은 명령에 `--wasm-pkg <새 최적화 패키지> --embed-fonts full`과 실제 글꼴 경로를 지정했다.

| 파일 | SHA-256 |
| --- | --- |
| HWP | `ef10261cd29325116028e4f4f3e6be1a72c675eb771bddfd8484e7fe5aa94b4e` |
| HWPX | `fc6e5f156de470dfbb14aab392389491720ee7fb1bf6f03fe9a018e93b420c65` |
| HWP 한컴 2024 Print | `9fb08dea604813eed90a0449f29682d400be54dacd413352cc4f680f37cd4a44` |
| HWPX 한컴 2020 Print | `ba065afdd3ace7fd6b87fe82848e6cae3d01915a1606e4d8a70e4c2153d57d73` |

HWP 원본의 저장 제품은 한컴 2024 13.0.0.3622다. 이관 시 사용한 HWP PDF에는
`Creator: Hwp 2018 11.0.0.1623`가 기록돼 있다. 동일 원본을 실제 한컴 2020
11.0.0.9172의 1-up으로 재인쇄한 교차 대조에서도 Native와 fresh WASM 모두 전115쪽 최저
98.09%였다. 기준과 보조 인쇄본의 글꼴·제품 차이를 구분해 확인했다.

확인한 WASM API 편집은 HWP/HWPX 각각 삽입 115조각과 삭제 115조각이며, 지연 편집의
커서와 동기 반영 후 커서가 일치했다.

최소 글꼴 환경에서 `cargo test --locked --profile release-test --target-dir target/pr-review --features native-skia --lib issue_7813_deferred_pagination -- --test-threads 1 --nocapture`로 새 관계형 회귀 3건을 실행했다. 3건 모두 통과했고 각각 HWP와 HWPX를 실행했다. 원본의 쪽을 여는 컷과 닫는 컷 사이에서 유닛 누락이나 중복을 찾지 못했다.

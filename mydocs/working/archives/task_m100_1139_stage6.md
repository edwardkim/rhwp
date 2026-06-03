# Stage 6 분석 — Task #1139

## 배경

작업지시자가 `3-09월_교육_통합_2022.hwp`의 페이지 수가 한컴오피스 기준 23쪽인데 rhwp-studio에서는 24쪽으로 표시된다고 보고했다.

현 브랜치는 PR #1137 로컬 선행 반영을 제거하고 `upstream/devel` 기준으로 다시 구성한 `local/task_m100_1139`다.

## 재현

```bash
cargo run --quiet --bin rhwp -- dump-pages samples/3-09월_교육_통합_2022.hwp | tail -240
```

수정 전 결과:

- `doc.page_count()`: 24
- 24쪽에는 `[미주] pi=1175..1181` 7개 문단만 배치된다.
- 23쪽 두 번째 단은 `used=545.9px`로, 본문 높이 `1001.6px` 대비 여유가 있다.

작업지시자가 추가로 9쪽부터 한컴오피스와 다르다고 보고했다. 9쪽을 직접 확인한 결과, 24쪽은 마지막 누적 결과일 뿐이고 첫 차이는 미주 첫 페이지인 9쪽에서 이미 시작된다.

## 원인 후보

미주 문단 중 내부 `LINE_SEG.vertical_pos`가 큰 값에서 더 작은 값으로 재시작하는 케이스가 `FullParagraph[미주]`로 통째 배치되고 있다.

```text
page=9  pi=522  vpos=124748..94604
page=14 pi=740  vpos=512212..500800
page=20 pi=992  vpos=1062051..992324
page=24 pi=1175 vpos=1454394..1432874
```

첫 문제 지점은 9쪽 오른쪽 단 마지막의 `pi=522`다. 이 문단은 본문 `para=55`의 미주 12개 중 마지막 가상 문단이다.

```text
body=55 ctrl=0 local=43..54 virtual=511..522
```

뒤쪽에 24쪽으로 밀린 `pi=1175..1181`은 본문 `para=440`의 미주 19개 중 뒤쪽 7개다.

본문 미주 매핑:

```text
body=440 ctrl=0 endnote_paras=19 local=695..713 virtual=1163..1181
```

특히 `pi=1175`에 해당하는 미주 내부 문단 `ep=12`는 같은 문단 내부에서 `LINE_SEG.vertical_pos`가 큰 값에서 0으로 재시작한다.

```text
ep=12 virtual=1175
line_segs:
40940+900+452
42292+1080+452
43644+1080+452
44996+1362+452
46528+1362+452
47880+1362+452
49694+900+452
51046+900+452
52398+2070+452
54920+17616+452
0+17616+452
18068+2205+452
19420+2205+452
```

`vpos=0` 또는 더 작은 vpos로의 재시작은 미주 문단 내부에서 위치 기준이 되감기는 패턴이다.

초기에는 이 패턴을 미주 문단 내부 단/쪽 재시작으로 보고 `PartialParagraph` 분리 처리를 검토했다. 그러나 실제 페이지 수 오차는 문단 분리 부족이 아니라, 미주 문단 배치 루프의 누적 bottom 계산이 마지막 line segment에만 의존하는 데서 발생했다.

## 최종 원인

`typeset.rs`의 미주 삽입 루프는 다음 미주 문단의 상대 위치를 맞추기 위해 이전 미주 문단의 bottom을 `prev_en_bottom_vpos`로 저장한다. 기존 코드는 이 값을 `line_segs.last()` 기준으로 계산했다.

내부 `LINE_SEG.vertical_pos`가 되감기는 문단에서는 마지막 line segment가 문단에서 가장 아래에 있는 줄이 아닐 수 있다. 예를 들어 `pi=522`, `pi=740`, `pi=992`, `pi=1175`는 표시상 아래쪽 줄이 앞쪽 segment에 있고 마지막 segment의 vpos가 더 작다. 이때 이전 문단 bottom이 실제보다 낮게 저장되고, 다음 미주 문단에서 `gap_hwp = first_vpos - prev_en_bottom_vpos`가 과도하게 커진다.

그 결과 9쪽에서 `pi=522` 뒤에 이어져야 하는 `pi=523` 이후 미주가 불필요하게 밀리기 시작했고, 누적 결과로 문서 끝에 24쪽이 생겼다.

## 수정

`src/renderer/typeset.rs`의 미주 paragraph 배치 루프에서 미주 문단의 bottom과 trailing line spacing을 마지막 줄이 아니라 가장 큰 `vertical_pos + line_height + line_spacing + endnote_start` 값을 가진 줄 기준으로 계산하도록 변경했다.

회귀 테스트는 `tests/issue_1139_inline_picture_duplicate.rs`에 추가했다.

- `3-09월_교육_통합_2022.hwp`의 `doc.page_count()`가 한컴 기준 23쪽인지 확인
- 9쪽 dump에 `FullParagraph[미주]  pi=523`이 남아 `pi=522` 뒤 미주가 같은 쪽에 이어지는지 확인

## 검증 결과

- `cargo fmt --check`: 통과
- `cargo test --test issue_1139_inline_picture_duplicate`: 2 passed
- `cargo test --test issue_1082_endnote_multicolumn_drift`: 4 passed
- `cargo test --lib`: 1406 passed, 0 failed, 6 ignored
- `cargo build --release`: 성공
- `./target/release/rhwp dump-pages samples/3-09월_교육_통합_2022.hwp`: 23페이지
- `./target/release/rhwp export-svg samples/3-09월_교육_통합_2022.hwp -p 8 -o output/diag_1139_stage6_page9`: 성공
- `./target/release/rhwp export-svg samples/3-09월_교육_통합_2022.hwp -p 22 -o output/diag_1139_stage6_page23`: 성공
- `wasm-pack build --target web --out-dir pkg`: 성공

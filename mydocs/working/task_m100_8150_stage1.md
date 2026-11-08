# #8150 — 1단계 완료보고: Kitty 전송과 CLI 계약

- Issue: [#8150](https://github.com/edwardkim/rhwp/issues/8150)
- [수행 계획](../plans/task_m100_8150.md), [구현 절편](../plans/task_m100_8150_impl.md)
- 브랜치: `feat/issue8150-kitty-20261022`
- 기준 devel: `2e087a5b4bc1515ad877e8020b30114639a59392`
- 검증 시작 HEAD: `910d4174c7b3dd0f4467b145ca372549941081af`와 이 보고서에 함께 커밋하는 구현.
  이 단계의 실행 뒤 제품 코드 변경은 없다. 실행한 Native 바이너리 SHA-256은
  `9cd196a82f728271a51099b5a9721a92589575c8e4eea784c60d70a2d574101e`다.

## 구현

`rhwp export-kitty`를 공개 명령·help·capabilities·dispatch에 등록했다.
`native-skia` 빌드에서 stdout 한 쪽 또는 페이지별 `.kitty` 파일을 출력한다.
너비·쪽·프로필·글꼴·호환성·입력 로딩과 외부 그림 정책은 기존 Sixel 명령을 따른다.
Native feature가 없으면 stdout을 비우고 stderr와 종료 코드 2로 알린다.

생산·소비 경로는 기존 `DocumentCore::render_page_pixels_native_at_width`의
`RasterPixels` → 기존 `opaque_rgba_on_white` → 새 `encode_kitty_png`의 불투명 픽셀 →
기존 `SkiaLayerRenderer::encode_pixels_png` → 새 `write_kitty_png`다.
청크 작성기는 PNG 원문을 3,072바이트씩 Base64로 바꾸고 APC payload를 최대 4,096바이트로 제한한다.
첫 청크의 `a=T,f=100,t=d,q=2`와 계속/종료 `m`은
[공식 규격](https://sw.kovidgoyal.net/kitty/graphics-protocol/)을 따른다.
PNG 생성은 stdout이나 대상 파일에 쓰기 전에 끝내며, 쓰기·flush 실패는 종료 코드 1로 전달한다.
기존 조판·PNG·Sixel 구현과 의존성은 변경하지 않았다.

새 계약 검사는 `tests/cases/issue_8150_kitty_contract.rs`에서 자동 suite에 수집된다.
Native CI는 전체 suite를 Native feature로 실행하는 방식이 아니라 지정된 사례만 실행하므로,
기존 Native job의 release-test/release 분기에 해당 사례 호출을 각각 추가했다.
일반 회귀에서 빠지는 Native 전용 검사 6건의 실제 실행 누락을 막는 변경이다.
CLI·테스트의 Native impact 소유 분류도 등록했다. 공용 renderer 경로의 보수적인 전체 분류는 유지했다.

## 실행 결과

모든 Cargo 실행은 `CARGO_BUILD_JOBS=6`,
`CARGO_TARGET_DIR=/home/edward/mygithub/rhwp/target/pr-review`에서 순차 실행했다.
로그·복원 PNG·실행 바이너리와 코드 해시 목록 `validated-files.json`은 ignored
`output/issue8150/`에 있다.

| 확인 | 관측 결과 | 근거 |
| --- | --- | --- |
| 구현 전 CLI 계약 | 1 FAIL / 1 PASS. `export-kitty --help`가 없어서 의도한 assertion 실패 | `red-native.log`, `baseline.json` |
| 구현 후 Native 계약 | 8 PASS | `green-native.log` |
| Native feature 없는 계약 | 3 PASS | `green-default.log` |
| 기존 Sixel 계약 | 7 PASS | `sixel-native.log` |
| CI impact 분류 | 45 PASS | `ci-classifier.log` |
| CI workflow 배선 | 35 PASS | `ci-workflow.log` |
| 변경 Rust 파일 fmt·공백 | PASS | `rustfmt --check`, `git diff --check` |
| 변경 계획·단계 보고의 내부 링크 | 3개 문서 PASS | `check_markdown_links.py` |

수정 전 검사는 기준 devel과 제품 코드가 같은 상태에서 실행했다. 컴파일·환경 오류가 아니라
명령 부재가 원인이었다. 자동 suite 배정은 사례 추가 후 `014`에서 `020`으로 바뀌었으며,
같은 사례 이름을 driver로 선택했다.

집중 실행의 nextest는 0.9.137이며 저장소 권장 0.9.140 차이와 지원하지 않는
JUnit 설정 키 경고가 있었다. 위 테스트는 실제 수집·실행되어 통과했으며 도구를 임의 갱신하지 않았다.

```bash
node scripts/run-rust-test.mjs issue_8150_kitty_contract -- --cargo-profile release-test --features native-skia --test-threads 4
node scripts/run-rust-test.mjs issue_7820_sixel_contract -- --cargo-profile release-test --features native-skia --test-threads 4
node scripts/run-rust-test.mjs issue_8150_kitty_contract -- --cargo-profile release-test --test-threads 4
node --test scripts/tests/ci-impact-classifier.test.cjs
python3 -m unittest discover -s scripts/tests -p test_ci_impact_workflow.py
```

새 검사는 청크 경계·패딩·마지막 청크·헤더·빈 입력·중간 I/O 실패와 합성 RGBA의
stride·흰색 alpha 합성·PNG 채널을 확인한다. PNG는 별도의 `image` 디코더로 복원했다.
CLI의 잘못된 옵션·누락 값·입력 중복·없는 파일에서도 그래픽과 진단의 채널을 구분했다.
새 실물 fixture·좌표·쪽수 golden은 추가하지 않았다.

추가 실행으로 공개 `samples/hwp_table_test.hwp`의 첫 쪽을 너비 64로 내보냈다.
Python 표준 Base64와 Pillow로 독립 복원한 PNG는 64×91 RGBA였고,
파일 모드의 `<stem>.kitty`는 stdout과 바이트가 같았다. 파일 모드 stdout은 비었고,
범위 밖 쪽 번호는 종료 코드 2와 빈 stdout이었다(`smoke-result.json`).
이 실행은 입출력 연결 확인이며 한컴과의 시각 일치나 실제 터미널 표시 증거로 사용하지 않는다.

문서 메타데이터 검사에서는 기존 기술 문서 5개의 필드 누락 20건이 검출됐다.
해당 파일 모두 기준 devel과 바이트가 같음을 확인했으며 신규 오류는 0건이다
(`metadata-baseline.json`). 이번 계획·단계 보고는 장기 문서 메타데이터 검사 대상 밖이다.

## 단계 판정과 다음 절편

1단계 전송·CLI 계약과 집중 검사는 충족했다. 360/720px 전쪽 문서 비교, PNG/Sixel의
수정 전후 출력 대조, 실제 kitty의 직접 표시·파일 재표시·SSH 검증은 2단계에 수행한다.
현재 환경에는 `kitty`와 `kitten` 실행 파일이 없으며 실제 표시를 확인했다고 보고하지 않는다.
전체 회귀·lint·fresh WASM·해당 시각 게이트는 3단계의 제출 전 필수 검증으로 남아 있다.

GitHub에는 이슈 등록·할당만 수행했다. push·PR 생성·댓글·merge는 수행하지 않았다.
[단계별 완료보고 규칙](../manual/hyper_waterfall_docs_guide.md#working--단계별-완료보고서)에 따라
이 보고서와 구현을 같은 커밋에 기록하고, 다음 절편 승인을 받은 뒤 2단계로 진행한다.

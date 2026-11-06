# PR #8658 WASM Canvas 진단·집중 회귀

검토 제품 code는 `759f35f2fbccc2682356dc22cb31b1068b57e178`이다. 로컬 검토 commit `3356b54df3`는 문서·입력·PDF·Native PNG/TSV만 추가했고 제품 src와 기존 Rust 테스트를 바꾸지 않았다.

## 실제 Canvas 출력

원 입력 3개 전 4쪽과 CENTER 대조군 1쪽은 실제 WASM Canvas에서 raw 100%였다. 선언 높이 경계는 **83.81%**다. 모든 문서는 대응 Print와 전체 쪽수가 같았으며, Native와 같은 원 HCR 글꼴·96dpi·Print profile·2024/2022 compat를 사용했다. Chrome headless shell 155.0.8059.79에서 직접 출력했다. PNG를 SVG 래스터 결과로 대신하지 않았다. TSV는 이 디렉터리의 `wasm-diagnostic/<입력>/silhouette.tsv`에 보존했다.

| 입력 | 쪽수 | 후보 Canvas raw 최저 |
| --- | --- | --- |
| cell-square-six-pictures-35x25 | 1 | 100.00% |
| cell-square-six-pictures-60x30-grows | 1 | 100.00% |
| split-row-picture-only-para | 2 | 100.00% |
| cell-square-six-pictures-center | 1 | 100.00% |
| cell-square-six-pictures-exact-fit | 1 | 83.81% |

## 빌드 완료 게이트와 진단의 구분

`CARGO_BUILD_JOBS=12 CARGO_TARGET_DIR=/home/edward/mygithub/rhwp/target/pr-review CARGO_PROFILE_RELEASE_CODEGEN_UNITS=16 scripts/wasm-pack-locked.sh --target web --out-dir pkg`를 실행했다. wasm-pack 로그는 Rust release 4m34s, wasm-opt 최적화 및 package ready(11m53s)를 기록했다. JS/WASM도 public 경로와 같았지만 **최상위 실행 명령의 종료 코드는 143**이었다. 이 raw 종료를 0이나 정식 빌드 PASS로 바꾸지 않는다. 확보한 현재 코드의 최적화 산출물로 위 실제 Canvas 진단을 수행했으며, **제출 전 정식 exit-0 최적화 빌드 게이트는 남아 있다**. 같은 코드를 중복 빌드해 검증을 완료했다고 적지 않았다. 보정 후보의 최종 검증에서 이 게이트를 해결해야 한다.

초기 브라우저 실행은 Chrome 미지정으로 원문 출력 전에 실패했다. 실제 존재하는 위 Chrome 경로를 `VISUAL_SWEEP_CHROME`으로 지정해 전 6쪽을 다시 실행했고, 초기 실패 로그와 실제 성공 결과는 output에 구분했다. JS/WASM과 생성 type 파일 4개의 임시 public 공급 일치를 확인했고, 범위 밖 생성 type 변경은 원래 HEAD로 복원했다. 공개 증적에 글꼴 프로그램은 포함하지 않는다.

## 최소 글꼴 집중 회귀

`cargo test --locked --profile release-test --features native-skia --test regression_suite_011 issue_8576_cell_square_no_overlap_stack --target-dir /home/edward/mygithub/rhwp/target/pr-review -- --nocapture`를 bwrap에서 실행해 exit 0, **3 passed / 0 failed / 0 ignored / 305 filtered out**를 확인했다. 이것은 전체 회귀나 Native 공식 3종을 실행했다는 뜻이 아니다.

- `large_pictures_stack_and_grow_cell`: PASS.
- `small_pictures_stack_without_overlap`: PASS.
- `split_row_picture_only_paragraph_draws_pictures_side_by_side_inside_cell`: PASS.

/usr/share/fonts와 /usr/local/share/fonts는 숨겼고 사용자 홈은 소스·Cargo·Rust toolchain을 제외해 노출하지 않았다. 검증용 사설 글꼴 경로의 부재를 probe하고 fontconfig에서 다음 공개 프로그램 5개만 실제 노출됨을 확인했다.

- `ttfs/opensource/NotoSansKR-ExtraLight.ttf`
- `ttfs/opensource/NotoSansKR-Regular.ttf`
- `ttfs/opensource/RHWPPublicBatang-Regular.ttf`
- `ttfs/opensource/NotoSansSymbols2-Regular.ttf`
- `ttfs/opensource/RHWPOldHangul-Regular.otf`

전체 회귀·Rust lint 묶음·Native 공식 3종·편람 383쪽·base fresh WASM 및 확장 소유 경계는 이번 선행 검토에서 아직 실행하지 않았다. 선언 높이 경계의 90% 미달과 보정 승인 대기 상태에서 수용 후보 제출 완료로 표시하지 않는다. [보정 계획](../../archives/pr_8658_review_impl.md)의 승인 후 최종 후보로 필수 검증을 한 단위로 완료한다.

# 한컴 수식 항의 분자·분모 소속 비교

코드 SHA: `785c2821c7276a506b635e16e6c5b53c3daf53ce`. 수정 전은 `e07495eaeffc1935ffbef7a0421e1d81aeb251f7`.

독립 기준은 `pdf/issue7642-operand-frame-diagnostic/operand-frame-hancom-2020.hwp`와 같은 원문의 `operand-frame-hancom-2020.pdf`다. 기준 PDF는 한컴 2020 `11.0.0.9136`의 PrintMethod 0 인쇄 출력이다. 입력 생성·정상 저장·Print 작업의 출처는 해당 폴더 README에 있다.

입력 SHA-256: `50d60eccf10f404968975560bb77e43a91aafa1b509108f9ce514756a275b36d`.
PDF SHA-256: `1e3f2803886ffcc354171cf1652fc9e59b9b434f604ea7bb6bc9b0289229f2fb`.
Native 실행 파일 SHA-256: `8d2f4135bfd91b3d47892f95cea81d5df3efc2c1a1145980593551c46f4fc31c`.

Native는 `cargo build --locked --profile release-test --bin rhwp --target-dir /home/tsjang/rhwp/target/pr-review`로 빌드했다. 실제 소스 해시가 빌드와 시각 산출 중 동일했다. `--profile print --compat 2022 --embed-fonts=full` SVG를 같은 글꼴 데이터의 Chrome 경로로 96 DPI에 raster하고 독립 PDF와 비교했다.

## 전체 페이지 결과

18/18쪽을 비교했으며 최저 99.06542%, 미달·누락·쪽수 불일치가 없다. 정확한 모든 쪽 TSV는 ignored `output/pr-review/issue7642/failed-inputs-20261008/stage15-current-validation/stage179-after-operand-frame-whole-native/operand-frame-native/silhouette.tsv`에 있다.

| 쪽 | 2px 관용 실루엣 일치율 |
|---|---:|
| 1 | 100.00000% |
| 2 | 100.00000% |
| 3 | 100.00000% |
| 4 | 100.00000% |
| 5 | 100.00000% |
| 6 | 100.00000% |
| 7 | 100.00000% |
| 8 | 99.06542% |
| 9 | 100.00000% |
| 10 | 100.00000% |
| 11 | 100.00000% |
| 12 | 100.00000% |
| 13 | 100.00000% |
| 14 | 100.00000% |
| 15 | 100.00000% |
| 16 | 99.19163% |
| 17 | 100.00000% |
| 18 | 100.00000% |

## 확대 판독

`native-operand-scope-detail-review.png`는 수정 전 / 수정 후 / 독립 Print의 같은 원문 좌표를 3배로 확대한 자료다. 1·3·4쪽에서 공백 없는 항의 분자·분모 소속, 공백 있는 덧셈 항의 바깥 소속이 기준과 일치한다. 8쪽 ATOP도 두 줄의 항을 보존한다. 분수선·기준선·항 사이 간격을 함께 확인했다. 글꼴 가장자리의 raster 차이는 남아 있으며 100% 실루엣을 픽셀 동일로 해석하지 않는다. 대표 whole-page review와 standalone overlay도 함께 보존했다. crop은 진단용이며 회귀 테스트의 고정 px 선택자로 사용하지 않는다.

## 남은 검증

같은 코드의 fresh WASM도 별도 빌드·검증을 완료했다. 전체 18쪽 최저 99.06542%이며 Native/fresh WASM의 같은 원문 PNG 18쌍이 모두 byte 동일하다. Studio JS/WASM 동기화와 source 해시 불변도 확인했다. WASM TSV와 실행 증거는 ignored output의 stage179-operand-frame-whole-wasm에 있다. 이는 rebase 전 코드의 증거이며 새 head의 검증은 별도로 수행한다. 원 카탈로그 18쪽은 2·3·4쪽이 각각 97.91157 / 98.38242 / 98.86500%로 개선됐지만 5·6·13·14·16·17·18쪽이 여전히 90% 미만이고 최저 40.22286%다. 전체 요청 문서, 전체 Rust 회귀, lint, 새로운 관계형 회귀, PR·CI·merge·후속 처리는 완료되지 않았다.

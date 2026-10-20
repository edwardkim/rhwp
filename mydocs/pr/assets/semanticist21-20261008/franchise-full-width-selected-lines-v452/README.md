# 가맹사업 72쪽 저장 전체 행 프레임 정렬 보정

현재 source: `b4ff8c81b5d986baf47bad9a3eee83cbd9d8367b`.
Native immutable CLI SHA-256: `624668478839e3a81b3369260f91fe57a3bb8b1309878ad8d14831f437f9a611`.

수정 전 72쪽 96.90530% → 현재 Native/fresh WASM 모두 99.95995%다. 표661/r2c0의 가운데 정렬은
다음 줄로 전진하는 물리 간격을 제외한 선택 글줄 높이를 소비한다. 첫 두 문단의
줄·기준선이 독립 Print와 정렬되며 오른쪽 분류명과 셀 외곽은 유지된다.
독립 기대 관계는 원본 Center/LineSeg 및 한컴 2020 Print에서 정했다.
정렬 관계의 동일 진단은 수정 전 FAIL / 수정 후 PASS, 오른쪽 대조군 전후 PASS다.
글자 획·불릿 크기·괘선의 래스터 차이는 남아 있으며 완전한 픽셀 일치를 주장하지 않는다.

Native/fresh WASM 영향 16쪽은 모두90% 이상이며 최저68쪽93.16816%다.
현재 출력의 decoded RGBA는 양쪽16쪽 모두 동일하다. Native 나머지15쪽의
새 RGBA와 render tree는 앞서 직접 판독한 7f04c 출력과 동일하다. fresh72쪽
review와 표 확대를 직접 판독했다. 전체142쪽 Native/fresh 및 최종 회귀는 진행
중이며, 이 자산은 영향16쪽의 부분 증적으로 PR 승인을 뜻하지 않는다.
fresh package 빌드는 완료했고
WASM SHA-256은 `94fe70f13609c5a006b8e9044463fac7ae96fe5fe03ae33bc0c0153b19f7867c`다.

`native-p072-table-zoom.png`는 원본 페이지 좌표 `(110,695)-(683,980)`를
2배로 표시한다. overlay의34 높이 표제 띠를 제외해 같은 좌표·배율로 정렬했다.

## 입력과 독립 Print

- `samples/issue5966/1130000-202100008_franchise_review_report.hwp` — SHA-256 `a1b8299b2554a83873b780474defb02efa949b43c86c63bcc164967f96091e55`
- `pdf/pr_6088_6144/hancom2020/pr_6088_6144_issue5966_franchise_review_report_1130000-202100008_franchise_review_report-2020.pdf` — SHA-256 `4cf22800e29baddb9b84023462c75252efd4efb04d22e9e64a8e3d9313076009`

## 재현

명령 원문·페이지 범위·입력 및 source guard·TSV·실제 전후 render tree는 ignored
`output/pr-review/semanticist21-20261008/full-width-visible-alignment-v452/`의
`native-boundary-plan.json`, `native-boundary-result.json`,
`native-boundary-direct-reading.json`, `p72-before-after-alignment.json`,
`p72-native-direct/provenance.json`, `fresh-boundary-plan.json`,
`fresh-boundary-result.json`, `fresh-boundary-direct-reading.json`,
`native-fresh-boundary-pixel-comparison.json`에 보존한다.
Fresh는 같은 명령에 `--wasm-pkg pkg`를 추가한다. render tree 차이는
`picture_footnote.rs`의 `usize::MAX - 2000 - p_idx`에서 비롯한 pi sentinel의
64/32비트 차이159개뿐이고, 좌표·내용·개체 관계는 동일하다.

대표 명령은 `visual_sweep.py --rhwp-bin <immutable CLI> --embed-fonts full`
(검증한 공급 글꼴 디렉터리와 font-environment 포함),
`--compat 2022 --silhouette-only --pages 8,25-26,55-60,67-68,72-73,78-80`다.
원본·독립 PDF·CLI·폰트 환경·확대 영역은 위 원문에서 해시와 함께 확인한다.
현재 판정은 [개별 PR과 누적 기록](../../../archives/semanticist21_20261008_integrated_review.md)에 연결한다.

## PNG 해시

- `native-p072-review.png` — SHA-256 `0773a56d02b9f4bd86c314299d3928fe4618952ad1dc04f4a83c0b8517097bf5`
- `native-p072-overlay.png` — SHA-256 `03552b0058f86ca78f753f1000064dff781c87dc06aa1a87f48c47a4479ec897`
- `native-p072-table-zoom.png` — SHA-256 `23163aa79221cff640782524871c1735c71ce4379242eba8c00c38814c2fd87a`

- `fresh-p072-review.png` — SHA-256 `2e0ab0eaa1b02567e73c450f738a3824b868c34432ab092edfec090bc5430c65`
- `fresh-p072-overlay.png` — SHA-256 `5fbe59712f5d0ac0fe972cf345b5a2855e22695279a24c76ff25824a51c6dc60`
- `fresh-p072-table-zoom.png` — SHA-256 `22b445d1b0e957a23e988da288fa6aa8c6eed0c7124931e8faaaf7fa27419c27`

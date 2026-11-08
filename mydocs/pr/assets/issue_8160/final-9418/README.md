# #8160 리베이스 후 최종 검증

새 devel `9418ed0c7cf289c5c4ac2b3bb8cdc18a1b637a9a`를 유지한 결과다.
리베이스 이전으로 전체 롤백하지 않았다. 수정 전 제품의 [리베이스 전후 Rust/Cargo 동일성](before-product-rebase-equivalence.json)도
보존했다. 과거 제품의 source를 바꾸어 쓰지 않고 공개 가능한 같은 코드와 연결한다.
실제 렌더 제품은
`5df67d3299befb467a49dc4eaafd12d6e1877e2d`, 정식 테스트 head는
`ef69d163e05062f9c281968a0f9e0884a616aec3`다. 두 head의 제품·UI blob이 같다는
[대조](test-head-render-producer-equivalence.json)를 보존했고 산출 source를 바꾸어 쓰지 않았다.

## 전쪽 100%와 반영 PR 보존

| 문서 | Native | fresh 표준 최적화 WASM |
| --- | --- | --- |
| HWPX | [64/64쪽 100%](tsv/targets/native/hwpx.tsv) | [64/64쪽 100%](tsv/targets/wasm/hwpx.tsv) |
| 2024 HWP | [64/64쪽 100%](tsv/targets/native/hwp2024.tsv) | [64/64쪽 100%](tsv/targets/wasm/hwp2024.tsv) |
| 2022 HWP | [64/64쪽 100%](tsv/targets/native/hwp2022.tsv) | [64/64쪽 100%](tsv/targets/wasm/hwp2022.tsv) |

[실제 PNG 재검사](all-384-exact.json)는 모든 쪽의 양방향 미대응 정수가 0/0임을 확인한다.
96 DPI·threshold 232·흰색 padding·대칭 Chebyshev 2px의 내용 실루엣 계약이다.
RGB 동일성을 뜻하지 않으며 표선 명암·해칭·안티앨리어싱 차이는 직접 판독했다.

#8191은 실제 동결 기준 `60972534680a0c5e51e54ec189b92e252e81a867`과 비교했다.
이 기준과 새 devel의 Rust/Cargo blob이 같다. 각 backend 49조건·1772쪽의
원본 [기준 TSV](tsv/incoming8191/base)와 [후보 TSV](tsv/incoming8191/head)를 보존한다.
Native 최저 92.50545%, 최대 하락 0.06137%p;
WASM 최저 92.49998%, 최대 하락 0.07379%p다. 양쪽 모두 90% 미만과
사용자 기준인 0.1%p 이상 하락은 0쪽이다. 별도 Mac 측정값을 현재 Linux 기준으로 쓰지 않았다.
[Native 비교](incoming8191-native-comparison.json), [WASM 비교](incoming8191-wasm-comparison.json).

#8191의 sample16-2022 조건은 [기준 commit의 기존 Print PDF](https://github.com/edwardkim/rhwp/blob/60972534680a0c5e51e54ec189b92e252e81a867/pdf/hwp3-sample16-hwp5-2022-hwp-2020.pdf)
(SHA-256 `6737b57af568a591eb073eeb27054e10aaf7587cbdfd6f18a06d1033b4025a19`)를
base/head에 동일하게 사용했다. 목표 문서의 새 MCP PDF(`c4e05635726869d737f1640db4e4f29c868df9e1c31e350b0b06e115fd210b33`)와
서로 다른 검증이다. 기존 기준을 목표 문서의 100% 판정에 사용하지 않았다.

## 자동 회귀와 직접 판독

전쪽 Rust 3개 함수는 문서별 64쪽, 내용 앵커 572곳·중요 영역 630개·변 경계 2520개를
독립 Print의 ±2px 좌표와 내용·소유·순서 관계로 보호한다. 기존 기대 좌표와 공차를 유지했다.
JSON 3개는 정식 검사에서 실제 읽는 fixture이며 임시 추출 도구는 커밋하지 않았다.
잘못된 BCP 한 줄 예외를 제거하고 원본 두 줄을 보호한다. 기존 HFT 경계 검사를 보완하고
새 base의 유효한 Native Skia 이중 테두리 검사를 복원했다.
기존 HWP3 수식 검사의 단일 SVG 노드 가정도 정정했다. 독립 원점·±2px 공차를 유지하고
연속 글리프의 원문·순서·기준선을 확인한다. 첫 전체 실행의 1건 실패와 정정 근거는
[기존 수식 검사 정정](existing-equation-svg-selection-correction.json)에 구분해 보존했다.
새 Rust 함수 6개·브라우저 검사 1개가 순증했다. 이름 정정 3개는 신규로 세지 않는다.
[함수 대조](permanent-test-function-delta.json), [정식 JPEG 수정 전 실패/후 통과](permanent-browser-before-after.json).

같은 최적화 WASM의 브라우저 묶음 9/9, PDF Worker 19입력·50확인,
Studio 2438 PASS/기존 skip 2, TypeScript·bindings/embed·글꼴 trace·VS Code·Studio build와
#8195 실제 명령 실패·복구 검증이 통과했다. Native/fresh WASM의 목차·BCP·복합 분수·
표/그림·WMF·표 끝/꼬리말·별첨과 2022 63/64쪽을 직접 대조했다.
[Native 판독](manual-read-native.json), [WASM 판독](manual-read-wasm.json),
[확대 비교](review/details), [실제 WASM 소비](wasm-consumption.json).
CanvasKit software의 WMF 18/35쪽은 실제 Native paint payload 재생 검사이며,
별도의 표준 WASM Canvas2D 전쪽 검사와 구분한다.

## 정식 테스트 head의 로컬 게이트

| 검증 | 실제 결과 |
| --- | --- |
| 집중 회귀 | 70 PASS |
| 전체 nextest | 11,574 PASS, 기존 제외 45 |
| Native Skia lib | 4,041 PASS(root 3,859 + 내부 crate 182), 기존 ignore 12 |
| 그림 자리표시자 | 2 PASS |
| 직접 PDF | 14 PASS |
| 2022 표 관계·복원한 이중 테두리 | 3 PASS |

fmt, Native/WASM/workspace Clippy, workspace build, suite/source-side 정책은 순차로 통과했다.
실행 명령·head/base·종료 코드·시간은 [전체 게이트](rust-gates.json),
[집중](validation-focused.json), [전체](validation-full.json),
[Skia](validation-skia.json), [lint](validation-lint.json)를 따른다.
실행 로그와 중간 산출물은 ignored output에 남기며 Git에 넣지 않았다.
최종 증적의 파일별 해시는 [assets.json](assets.json)에 있다.

같은 PR #8188에 후속 커밋한다. CI 완료 후 결과를 보고하고 사용자 승인을 받아야
머지·이슈 종료·review/오늘할일 trailing 기록·브랜치 정리 등 후속 처리를 진행한다.
이 문서는 원격 CI나 머지 완료 기록이 아니다.

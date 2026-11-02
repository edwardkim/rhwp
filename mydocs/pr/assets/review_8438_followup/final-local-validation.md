# 원 PR 11건 메인터너 보정의 로컬 검증

검증한 마지막 제품 코드 변경은 `d56930dda723fff1c4e45675979b44604997f0e2`다.
이후 커밋은 회귀 검사 자료 선택과 문서·TSV 증거를 수정했다. Native Skia 실행 파일의
SHA-256은 `7d35357fe91cf213bb4143675a7c148bf3f6a9e4facbf93fea765081661b25c5`,
최적화한 fresh WASM 바이너리의 SHA-256은
`eda26e7ad93a87c7e9aed712e83ae0cddce993aa9939d25cfaf427c8cbe685c1`이다.
Cargo 검사는 고정 `target/pr-review`에서 빌드 작업 8개로 수행했다.

## Rust·Studio 실행 결과

| 검사 | 결과 | 로컬 로그 SHA-256 |
| --- | --- | --- |
| 최소 글꼴 전체 `cargo nextest` | 선택 12,377건 중 12,375건 통과·2건 실패·45건 skipped | `dc2460d283fe7d91b8a69247dc6354bd1ffe788d89bca7f67c212b48db7ca054` |
| 실패한 #4895 저장 왕복 검사 2건의 자료 선택 보정 뒤 같은 환경에서 집중 재실행 | 2건 통과·0건 실패 | `dfc54aaae06536795bb60c100e6a8cff1ce468c4d2fb4b708f4d7c4d48d948a3` |
| 최소 글꼴 Native Skia 라이브러리 검사 | 3,858·15·165·2건 통과, 실패 0건 | `8d2c5c0eca089970b6e63c3b50796076fe69691fd8f302c5e669c99f15885425` |
| 최소 글꼴 `issue_2225_missing_picture_placeholder` | 2건 통과 | `a9ebaffcb826cf7e25825ea4e8e5140e294f02109a7080e7306d67a4d6106b41` |
| 최소 글꼴 `render_p37_direct_pdf_export` | 22건 통과 | `beea8ddbb67b101f1f22dee76d62a2ea4d23780136daca5e63eded06b0851d38` |
| Studio Node 전체 검사 | 2,718건 통과·0건 실패 | `6913b4da531445099c6c3dbb92d361cd7b8c9512ffd430eae13ec7508421215f` |
| Studio TypeScript·production build | 종료 코드 0 | production build `30585c47e6329cd781d87ed716e78e8703cf6daecf1c29d7a4aa221bcbaeb43a` |
| Rust fmt, Native·WASM32·workspace Clippy, workspace build | 각각 종료 코드 0 | Native Clippy `b0e97b279d6328825cd9c73e3478c867fe41cc62c9ab5edea87e411e547fb2ba`; WASM32 Clippy `94c6facf632a2f9f930c3664007ad8b7086c49f6a4436f5e1380022b38092e49` |
| 최적화한 fresh WASM 빌드 | 종료 코드 0·공개 JS/WASM 빌드 산출물과 일치 | `d0bc685dd0aaf0009f8f3ef4ae6b27c6079c2a7140a93c57c75be82286bc09b2` |

전체 Rust 실행의 두 실패는 #4895 검사 helper가 중첩 표 셀의 저장 U+00AD를
화면 텍스트에서 찾던 잘못된 자료 선택이었다. 원본 IR의 표 셀까지 재귀적으로 세도록
검사만 고친 뒤 같은 최소 글꼴 환경에서 두 건 모두 통과했다. **전체 실행과 집중
재실행을 한 번의 전체 통과로 합산하지 않는다.** 제품 코드는 두 실행 사이에 바뀌지 않았다.

## 시각 검증

[원 PR 입력 23개](final-original-inputs/README.md)는 각 독립 한컴 Print PDF와
96 DPI에서 Native·fresh WASM의 모든 1,057쪽을 측정했다. 쪽수 불일치·누락·90% 미만은
모두 0쪽이며 최저값은 Native 95.49%, fresh WASM 95.59%다. 별도의
[#8341 차트 64쪽](../issue_8305/followup8438/native-literal-final-2024-all.tsv)은
Native 최저 90.69%·fresh WASM 최저 90.68%, [#8321 소프트 하이픈 2쪽](../issue_8315/followup8438/native-literal-final.tsv)은
Native 최저 98.96%·fresh WASM 최저 98.64%다. 주장 페이지의 review와 overlay는
각 [개별 PR 검토 문서](../../archives/pr_8316_review.md) 및 해당 정식 asset에 연결했다.

이전 통합 [PR #8389·#8397의 기준 TSV](pr8389_8397/README.md) 2,153행을
같은 입력·Print·기존 대체 글꼴 설정에서 Native와 fresh WASM 각각 다시 대조했다.
두 경로 모두 소수 둘째 자리 표시값 하락·미완료가 0행이다.

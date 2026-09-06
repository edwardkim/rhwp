# #6660 2차 보정: 병합 제목 셀의 비활성 하단 여백

## 판정: 메인터너 보정 완료, 이슈의 세로 위치 완료 기준 충족

- 작업 브랜치: `fix/6699-6662-residual-image-layout-20260906`.
- 1차 보존 커밋: `97ee926d8`. 1차 전체 회귀 성공을 이번 추가 수정의 검증으로 재사용하지 않았다.
- 대상 원본: [exam_science.hwp](../../samples/exam_science.hwp).
- 기존 한컴 기준 PDF: [exam_science-2022.pdf](../../pdf/exam_science-2022.pdf), 4쪽. 새로 변환하지 않았다.
- 이번 판정은 #6660에서 지정한 두 그림의 `abs(dy) < 1px`, 페이지 경계 유지, 회귀 테스트 추가에 한정한다.
- 가로 위치, 글꼴, 문서 전체 픽셀 정합이나 상위 #6662의 모든 잔여 문제 해결을 뜻하지 않는다.

## 1. 커밋 후 분석

1차 보정은 병합 셀 복원 뒤 다른 행에 남은 여유를 회수했다. 그러나 새 PDF 위치 회귀를
실제로 실행하자 1쪽 그림이 `+1.1337px`로 실패했다. 기존 높이 회계 테스트의 성공만으로
완료를 판정하지 않고 이 실패를 2차 보정의 출발점으로 삼았다.

1쪽 문단 23의 표는 선언 높이가 `9870 HU = 131.6px`다. 병합 제목 셀은 다음 값을 가진다.

| 항목 | 원본 값 |
| --- | --- |
| 세로 병합 | 2행 |
| 셀 선언 높이 | 1289 HU |
| 저장 줄 높이와 글자 높이 | 각각 1150 HU |
| 저장 상단/하단 여백 | 각각 141 HU |
| 개별 셀 여백 활성화 | 비활성 |
| 표 기본 여백 | 네 방향 모두 0 |
| 문단 | 상단 정렬, 저장된 일반 텍스트 한 줄, 컨트롤 없음 |

`#6124`의 병합 복원은 이 셀에 `1150 + 141 + 141 = 1432 HU`를 필수 높이로 부과했다.
하지만 저장 줄과 상단 여백의 합 `1291 HU`만으로 선언을 채우며, 선언과의 차이는 병합
2행의 HU 반올림 이내다. 비활성 하단 여백까지 복원하면 후속 그림이 아래로 밀린다.

임시 복제 원본에서 이 셀의 하단 여백만 0으로 바꾼 대조는 제목 글자의 y를 유지하면서
표 높이를 131.6px로 되돌렸다. 상단 여백을 지우는 대조는 제목 글자까지 움직였으므로
수정 방향에서 제외했다. 임시 원본과 진단 스크립트는 커밋하지 않았다.

## 2. 코드 보정

`HeightMeasurer::merged_cell_restore_floor_hu`에서 다음 조건을 모두 만족할 때만 복원 하한을
저장 내용과 상단 여백의 합으로 계산한다.

- TAC 표이며 표 기본 여백이 모두 0이고 개별 여백이 비활성이다.
- 세로 병합 셀의 가로쓰기/상단 정렬이며, 상하 저장 여백이 정상적인 작은 양수다.
- 수정되지 않은 저장 텍스트 한 문단/한 줄이며 컨트롤이나 합성 채움 줄이 없다.
- 저장 줄의 시작 y가 0이고 줄 높이와 글자 높이가 같다.
- 셀 선언이 내용 높이 이상이고, 내용+상단 여백과 선언의 차이가 `row_span` HU 이하다.

파일명, 제목 문자열, 절대 페이지 좌표로 분기하지 않는다. 명시적 여백, 여러 줄,
실제 내용 넘침, 다른 세로 정렬에는 기존 상하 여백 하한을 유지한다. 일반 패딩 해석이나
렌더링 좌표에 임의 오프셋을 추가하지 않았다.

기존 1차 테스트의 `1432 HU` 고정 기대값은 바로 이 중복 하단 여백을 정답으로 굳히므로,
저장 내용/상단 여백 보존과 표 선언 높이 계약으로 변경했다. 별도의 원본 변형 6종으로
명시적 여백과 실제 내용 하한이 축소되지 않는 것도 검사했다.

## 3. 실측 결과와 시각 증적

PDF의 1190pt 높이에 원본 용지 높이 `111685 HU / 75`를 대응시켜 동일 좌표계로 비교했다.
반올림된 render-tree 페이지 높이를 배율 기준으로 사용하지 않았다.

| 대상 | 한컴 y | 1차 rhwp y | 2차 rhwp y | 2차 dy | 판정 |
| --- | ---: | ---: | ---: | ---: | --- |
| 1쪽, 문단 28의 폭 75.2px 그림 | 1085.0663 | 1086.2 | 1084.3 | -0.7663px | 통과 |
| 4쪽, 문단 109의 폭 59.5px 그림 | 1011.5182 | 1010.9 | 1010.9 | -0.6182px | 통과 |

PNG는 현재 코드의 `target/pr-review/release-test/rhwp`를 native-skia로 빌드한 뒤
`export-png --profile screen --scale 2`로 산출했다. 아래 증적은 동일 페이지 좌표의
한컴 PDF와 rhwp 출력 영역을 자른 것이며 PC 전체 화면이 아니다.

![#6660 한컴 PDF와 rhwp의 대상 그림 세로 위치 비교](../pr/assets/issue_6660_20260906/picture-position-comparison.png)

이 PNG 한 장만 후속 PR/이슈 코멘트의 직접 표시용으로 보존한다. 코멘트 게시 시에는
최종 merge SHA에 고정된 이미지 URL로 직접 표시하고 기존 PDF 링크를 함께 제공한다.
원시 PNG, render-tree JSON, 진단 로그와 대조용 사회 문서 전체 이미지는 `/tmp`에만 둔다.

## 4. 전후 페이지 경계 대조

1차 코드와 2차 코드의 실제 render-tree를 각각 생성하여 다음 6개 문서 22쪽의 페이지 수와
페이지별 텍스트/이미지 순서가 동일함을 확인했다. 좌표가 전부 동일하다는 뜻은 아니다.

| 문서 | 쪽수 | render-tree 좌표 변경 페이지 |
| --- | ---: | --- |
| `samples/exam_science.hwp` | 4 | 1, 2, 3 |
| `samples/exam_social.hwp` | 4 | 1, 3, 4 |
| `samples/exam_social-p1.hwp` | 1 | 1 |
| `samples/hwpx/exam_social.hwpx` | 4 | 1, 3, 4 |
| `samples/hwpx/exam_social-p1.hwpx` | 1 | 1 |
| `samples/issue6124/2737927_housing_evaluation_guideline.hwpx` | 8 | 없음, render-tree 동일 |

사회 1쪽은 기존 한컴 PDF와 보정 출력의 전체 페이지도 육안 대조했다. 기존 글꼴과 위치
차이가 있어 문서 전체 시각 일치로 판정하지 않았다. 이번 전후 경계 대조는 위 22쪽의
명시적 범위이며, 사전 992개 파일 구조 탐색을 전체 render-tree 대조처럼 기록하지 않는다.

## 5. 수정 후 실제 검증

모든 Cargo 작업은 `CARGO_TARGET_DIR=target/pr-review`, `CARGO_BUILD_JOBS=4`에서 순차 실행했다.
기존 golden/baseline이나 테스트 허용 한도를 완화하지 않았다.

| 검증 | 결과 |
| --- | --- |
| #6660 및 #6124/#6442/#1785/#1835/#2071/#6699/#6665 집중 회귀 | 22개 통과 |
| `cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 6 --no-fail-fast` | 9,105개 통과, 46개 skip, 실패 0건. 테스트 실행 279.004초 |
| `cargo fmt --all -- --check` | 통과 |
| native root Clippy, `-D warnings` | 통과 |
| WASM32 lib Clippy, `-D warnings` | 통과 |
| workspace build | 통과 |
| workspace all-target Clippy, `-D warnings` | 통과 |
| suite manifest check | 통과 |
| `cargo test --locked --profile release-test --target-dir target/pr-review --features native-skia --lib -- --test-threads 6` | 필터 없이 실행. 전체 4,112개 통과, 13개 ignored |
| Native Skia missing-picture PNG 통합 회귀 | 2개 통과 |
| Native Skia direct PDF 통합 회귀 | 4개 통과 |
| doc tests | 8개 통과, 3개 ignored |
| `scripts/wasm-pack-locked.sh --target web --out-dir pkg` | Mac native 경로 통과, wasm-opt 포함 2분 38초. Docker 실행 결과가 아님 |

`wasm-bindgen` 사전 빌드 다운로드 경고 후 cargo-install fallback으로 빌드가 완료됐고,
WASM wrapper의 최종 종료 코드는 0이다. nextest의 알려지지 않은 설정 키 경고는 남아
있지만 테스트 실패는 없었다. 관련 없는 설정 보정은 섞지 않았다.

로그 위치는 `/tmp/rhwp-6660-stage2-validation-20260906-aK6EzR`이며 로그 자체는 커밋하지 않는다.

## 6. 다음 PR 단계와 범위

- 이번 문서는 코드 보정의 분석 결과다. 아직 새 PR의 CI 통과 또는 원격 이슈 종료를 주장하지 않는다.
- #6699와 #6665의 1차 보정은 유지됐고 집중/전체 회귀에서 다시 통과했다.
- 새 제품 PR의 초기 코드 head CI 완료 후, 같은 PR에 archive review와 오늘할일을 trailing commit으로 추가한다.
- #6816의 CI 재사용은 실제 새 PR의 실행 결과로 검증한다. 로컬 테스트만으로 재사용 성공을 주장하지 않는다.
- 상위 #6662와 수정 범위 밖의 하위 이슈를 일괄 종료하지 않는다.

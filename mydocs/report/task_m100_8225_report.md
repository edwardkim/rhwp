# #8225 그림 개체의 CTRL_DATA 보존

[이슈 #8225](https://github.com/edwardkim/rhwp/issues/8225)의 그룹 자식 데이터 손실은 [#8233](https://github.com/edwardkim/rhwp/pull/8233)과 [통합 #8395](https://github.com/edwardkim/rhwp/pull/8395)에서 해결했다. 당시 검토는 그룹 자체의 헤더에서 사라지는 레코드를 잔여 범위로 기록했다. 2026년 11월 22일 열린 PR 19건·이슈 타임라인·관련 변경 파일을 확인했으며, 이 잔여 범위를 구현하는 열린 PR은 없었다. [선행 검토](../pr/archives/pr_8233_review.md) · [추가 재현 보고](https://github.com/edwardkim/rhwp/issues/8225#issuecomment-7041613421)

마지막 조회 시 열린 PR은 20건이었다. 같은 HWP 저장기를 수정하는 #9286의 변경은 선 투명도 바이트에 한정되며, 새로 열린 #9300은 PR의 CI 결과를 요약한다. 두 PR의 실제 변경 파일과 diff를 대조한 결과 #8225의 데이터 손실을 처리하는 PR은 없었다. [#9286](https://github.com/edwardkim/rhwp/pull/9286) · [#9300](https://github.com/edwardkim/rhwp/pull/9300)

## 원인과 수정

최초 재현 기준은 `5235b4d131`이다. 최종 수정 커밋은 새 `upstream/devel`의 `5665e53829`를 반영한 `f2039b89ac4f`다. HWP5의 그림 개체는 `CTRL_HEADER`와 최상위 `SHAPE_COMPONENT` 각각의 자식으로 `CTRL_DATA`를 가질 수 있다. 기존 파서는 두 위치에서 먼저 발견한 한 건만 보존했고, 저장기는 불투명한 데이터를 구성 요소 아래에 써서 헤더 쪽 레코드를 잃었다. 네 문서에서는 두 소유자의 바이트가 우연히 같았다.

파서와 저장기는 각 소유자의 직속 데이터를 따로 읽고 쓴다. 캡션 안의 다른 개체와 그룹 자식은 자기 레코드를 유지한다. 첫 글자 장식을 편집할 때는 장식 ParameterSet만 갱신한다. 그림 하이퍼링크를 만드는 기존 변환기의 구성 요소 슬롯도 유지했다. [구현 계획](../plans/task_m100_8225.md) · [회귀 검사](../../tests/cases/issue_8225_group_child_ctrl_data.rs)

## 원본과 편집본의 레코드 대조

Native CLI에서 구역 0의 첫 문단에 `가`를 넣어 HWP로 저장하고, 저장본을 다시 열어 같은 위치에 한 번 더 삽입했다. 압축을 푼 CFB 구역에서 전체 `CTRL_DATA`의 소유 경로·순서·원문 바이트를 원본과 비교했다.

| 문서 | 원본 | 수정 전 1회 저장 | 수정 후 1회 / 2회 저장 |
| --- | ---: | ---: | ---: |
| `samples/basic/sungeo.hwp` | 60 | 48 | 60 / 60 |
| `samples/20250130-hongbo.hwp` | 2 | 1 | 2 / 2 |
| `samples/task2137/156618554_petfood_press.hwp` | 2 | 1 | 2 / 2 |
| `samples/156636617_240617 2024년 5월 월간 수출입 현황(확정치).hwp` | 2 | 1 | 2 / 2 |

`sungeo.hwp`에서는 그룹 헤더의 16바이트 레코드 12개를, 나머지 세 문서에서는 중첩 그림 헤더의 76바이트 레코드를 각각 복원했다. 정상 대조군 `hwp-multi-001.hwp`, `156678235_mid_para_vpos_rewind.hwp`, `hwpx-01.hwp`, `double-line.hwp`도 두 번의 Native 저장에서 원본과 일치했다.

저장소 `samples/`의 일반 HWP5 원본에서 그림 헤더·구성 요소에 해당 레코드가 있는 파일은 33개였다. 원본 SHA-256은 32종이고, 33개 경로에서 대조한 그림 레코드는 119개다. 동일한 CLI 편집을 실제로 수행한 결과 기존 기준은 29개 문서에서 일치했고, 수정본은 33개 전부에서 그림 데이터의 위치·순서·바이트가 일치했다.

같은 소스에서 `scripts/wasm-pack-locked.sh --target nodejs`로 release 빌드와 `wasm-opt`를 실행했다. 결과 WASM의 SHA-256은 `38e0afd98368b9d3763016b5043f0fe6d0100980b195b3811cbf025abb753141`이다. 원 이슈의 공개 API 절차대로 모든 구역의 첫 문단에 `X`를 넣었다 지운 후 `exportHwp()`로 저장했다. 재열기·편집·저장을 두 번 반복했으며 위 네 원본과 네 대조군의 전체 CTRL_DATA가 매번 원본과 일치했다. 같은 절차에서 33개 공개 경로의 그림 레코드 119개도 모두 보존됐다.

Chromium 152에서도 같은 최적화 WASM을 웹 바인딩으로 열고 네 원본과 네 대조군을 두 번씩 편집·저장했다. 16개 저장본의 모든 CTRL_DATA가 원본과 일치했다. Node.js와 웹용 바인딩을 만든 직후 두 WASM의 바이트가 같은지 확인한 다음, 위 SHA-256의 최적화 파일을 브라우저에 공급했다.

한컴이 저장한 `double-line.hwp`와 `double-line.hwpx`는 각각 HWP와 HWPX 두 형식으로 다시 저장했다. 네 조합에서 글자 `가`를 한 번씩 추가하며 첫 글자 장식을 `DoubleLine`에서 `TripleLine`, 이어서 `Margin`으로 바꿨다. 재열기 때마다 삽입한 글자와 장식이 남았다. 저장본 8개를 별도로 읽어 HWP에서는 두 소유자의 ParameterSet 값·위치가, HWPX에서는 사각형의 `dropcapstyle` 속성이 일치함을 확인했다.

## 자동 검증

최소 글꼴 환경에서 #8225·첫 글자 장식·그림 하이퍼링크를 포함한 집중 검사 10개가 통과했다. #8225 회귀는 실제로 삽입한 글자가 재열기 후 남았는지도 확인한다. 캡션 안 책갈피를 둔 그림에서 헤더 전용·구성 요소 전용·양쪽의 서로 다른 값·양쪽 모두 없는 네 경우를 함께 검사한다. 집중 필터는 다음과 같다.

```bash
cargo nextest run --locked --cargo-profile release-test --tests \
  -E 'test(issue_8225_group_child_ctrl_data) | test(drop_cap_style_round_trip) | test(issue_8174_adapter_wire)'
```

최종 코드 `f2039b89ac4f`에서 `cargo nextest run --locked --cargo-profile release-test --tests --no-fail-fast`로 전체 회귀를 재실행했다. 실행한 13,948개가 모두 통과했고, nextest가 별도로 집계한 건너뛴 검사는 45개였다. 리베이스 전에도 실행·통과 수가 같았다.

Native·WASM32 Clippy, workspace 빌드와 전체 타깃 Clippy, 포맷, suite manifest 검사와 manifest 계약 24개가 통과했다. 회귀 실행은 파일 시스템을 격리하고 저장소의 공개 fallback 글꼴 5개만 Fontconfig에 노출한 환경을 사용했다.

## 검증 도중 반영된 devel 변경

[#9297](https://github.com/edwardkim/rhwp/pull/9297)이 PR별 최신 CI 결과와 실패 작업을 요약해 게시하는 기능을 추가했다. 리베이스 전 후보 `a772694236`과 새 후보 `f2039b89ac4f`의 차이는 이 PR에서 들어온 워크플로·스크립트·운영 문서 10개였다. Rust 제품·테스트·Cargo 설정·fixture는 바이트가 같다.

새 devel `5665e53829`와 후보 양쪽에서 동일한 검사 42개가 통과했다. 이 중 3개는 PR head·최신 시도 선택, 실패 작업의 표시, 모의 GitHub API에서 댓글 한 건 갱신 및 head 변경 시 게시 중단을 검사한다. 나머지 39개는 워크플로 배선과 승격 정책 검사다. 후보에서 Native와 최적화 WASM의 샘플 대조 및 Chromium 저장·재열기도 다시 실행했다.

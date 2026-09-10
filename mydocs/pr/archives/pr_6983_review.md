# PR #6983 검토: 본문 넘침 기준선의 분할 검증과 누락 방지

## 판정: 메인터너 보정 후 수용 가능

검토일: 2026-09-10. 검토자: jangster77.

메인터너 보정 적용 및 검증을 완료했다. 분할 전에 전체 코퍼스에 대해 기준선 키 누락을 검사하고, 렌더 실패를 넘침 0건으로 처리하지 않도록 수정했다. 신규 issue6921 fixture의 기준선 6건은 devel 기준과 후보의 전체 페이지 결과가 동일함을 비교한 뒤 등록했다. 기존 문서 기준선은 올리지 않았다. 16개 분할 테스트가 모두 통과했다.

## 검토 기준과 출처

- 원 PR: https://github.com/edwardkim/rhwp/pull/6983
- 검토한 원 PR head: `c445d5043a61e667e215e570411681ac4721c9d2`.
- 기준 devel: `37bd46a72f9fd9ffd709e35244df79c00e789780`.
- 로컬 통합 브랜치: `review/planet6897-6959-6983-20260910`.
- 체리픽 통합 commit: `bb5dc4b00fc1247374cd8b39f40e113c666745fc`.
- 메인터너 보정은 별도 코드 커밋 `4fc08b5e978e691b5885f6a1bdb0101fda563dc0`에 확정했다. 원 contributor 체리픽 이력은 보존했다.
- 이번 통합은 #6959, #6960, #6967, #6968, #6977, #6978, #6980, #6983의 8개 PR, 총 18개 provenance-preserving 체리픽으로 구성된다.

| 원본 commit | 적용 commit |
| --- | --- |
| `c445d5043a61e667e215e570411681ac4721c9d2` | `bb5dc4b00fc1247374cd8b39f40e113c666745fc` |

## 보류 사유의 처리 결과

이전 검토에서 통합 회귀 실패 및 미실행 검증 때문에 유지하던 보류 판정은 아래의 실제 검증 결과로 갱신한다. #6967의 형광펜 왕복, #6980의 클립 확장에 따른 흐름 높이 증가, #6983의 누락 fixture 감시 결함은 메인터너 보정으로 해결했다. 기존 넘침 기준선 상향이나 실패 테스트 비활성화로 통과시키지 않았다.

- 누락 파일 음성 대조: 임시 기준선의 `missing-from-samples.hwp`에 대해 테스트가 exit 101과 누락 오류로 실패했다. 이는 누락 검출 성공이며 실제 회귀 실패가 아니다.
- 음성 대조용 fixture·기준선과 로그는 저장소에 포함하지 않는다.

## PR 범위 재확인 및 최신 집중 검증 (2026-09-10)

작업지시자의 지시에 따라 #6980 본문의 계약인 **조각이 소유한 하단 줄의 실제 출력 보존과 이웃 셀 비회귀**를 중심으로 재검토했다. 문단 간격·페이지 전체 외곽선 정렬은 기존 devel에서도 재현되며, 이번 PR의 해결 주장과 분리한다.

- 제가 추가했던 범위 밖 작은 빈 문단 간격 보정은 제거했다. 그 변경 때문에 다시 실패했던 베트남 문서 114쪽의 `⑦물류⑧기타` 집중 테스트가 통과했다.
- 브리핑 실물 `samples/issue6924/148751598-briefing.hwp`를 등록했다. 새 `issue_6924_briefing_owned_line_kept` 테스트는 범례가 실제 SVG의 첫 페이지에 정확히 한 번 있고 다른 페이지에는 중복되지 않음을 확인한다. 렌더 트리에 텍스트가 있다는 사실만으로 통과시키지 않는다.
- 본문 넘침·텍스트 겹침·용지 밖 배치 각 16개 분할, 관련 복구·음성 대조 및 신규 샘플 보안 스윕을 포함하여 **61개 통과, 실패 0개**, 8 threads, 57.004초였다. 필터 밖 9,399개는 이번 집중 실행에서 실행하지 않았다.
- 신규 실물 샘플의 기준선은 기준 devel `37bd46a72f9fd9ffd709e35244df79c00e789780`에서 측정한 본문 하단 넘침 3건·텍스트 겹침 1건만 등록했다. 기존 문서의 기준선은 높이지 않았다. 용지 밖 배치는 기준 devel에서 0건이다.
- 새 샘플 보안 스윕은 issue6921, issue6956, issue6924의 3개 파일을 명시해 통과했다.
- 파생 harness 재생성 후 manifest 검사, 포맷 검사, source-side 4,205개/298개 모듈 기준선 검사도 통과했다. 파생 harness와 실행 로그는 커밋 대상이 아니다.
- 아래의 전체 9,413개 결과와 빌드·Native Skia 결과는 앞선 실행 기록이다. 이번 실물 샘플·테스트 등록 뒤 전체 회귀를 다시 실행한 것으로 표현하지 않는다. 이번에 확인한 최신 결과는 위 집중 61개다.

**#6980 판정: 메인터너 보정 후 수용 가능.** 소유한 줄 보존과 관련 비회귀에 한정한 판정이다. 브리핑의 세로 간격과 외곽선 차이를 해결 완료했다고 주장하지 않으며, 이 기존 차이만으로 본문 범위의 수용 판정을 보류하지 않는다. 최종 원격 head CI와 merge는 별도 단계다.

## 이전 전체 로컬 검증 기록

- 전체 integration 회귀: **9,413개 통과, 실패 0개, 건너뜀 46개**, 300.674초. `cargo nextest run --locked --cargo-profile release-test --tests --no-fail-fast --test-threads 8`로 실행했다.
- 신규 보안 코퍼스 스윕에는 issue6921 HWP와 issue6956 HWPX를 명시적으로 포함했다.
- 형광펜 3개, 본문 넘침 16개, #6924 1개의 집중 실행: **20개 모두 통과**.
- Native Skia 라이브러리: **4,112개 통과, 실패 0개, ignored 13개**. placeholder 집중 2개 및 직접 PDF export 집중 4개도 모두 통과했다.
- `cargo fmt --all -- --check`, 네이티브·WASM·workspace Clippy 및 workspace build 통과.
- 파생 suite 준비 후 manifest 검사 통과. source-side 테스트 기준선은 **4,205개 / 298개 모듈**로 유지했다.
- 호스트 WASM 빌드 통과, 3분 7초. Docker는 사용하지 않았다.
- Studio TypeScript 검사 통과. Studio 테스트 1,500개 중 **1,498개 통과, 2개 skip, 실패 0개**. 이 결과 이후 TypeScript 코드는 바뀌지 않았다.
- 새 WASM의 Chrome image-crop 실행과 직접 PNG 대조 완료. CanvasKit compat/default 모두 런타임 렌더 완료, 오류·숨은 Canvas2D 오버레이 없음. software surface 검증이며 GPU 경로 검증은 아니다.
- 전용 Cargo target: `target/pr-planet6897-6959-6983-20260910`. 실행 로그와 임시 JSON/SVG/중간 이미지는 `/tmp/rhwp-planet6897-20260910`에만 두며 커밋하지 않는다.

원 PR CI의 기존 조회 결과는 성공 또는 정책상 skip이었으나, #6960·#6968·#6977·#6978·#6980의 CodeQL code-scanning에는 devel 대비 언어 구성 2개 누락에 따른 NEUTRAL이 있었다. 분석 실패와 구분하며, 이 기록은 현재 원격 상태를 재조회한 결과가 아니다. 최종 통합 head의 CI와 필수 check는 push 후 별도로 확인해야 한다.

## 시각 증적과 판정 범위

별도 시각 PNG를 승인 근거로 만들지 않았다. 위의 구조·왕복·실물 계약 검증을 근거로 사용한다.

기준선은 기존 코퍼스 대비 넘침 증가를 감시하는 장치이며 기존 넘침 자체가 모두 해결되었다는 뜻이 아니다.

## PR·이슈 코멘트 반영 계획

- 통합 PR의 실제 merge SHA와 최종 PR/devel CI를 확인한 뒤 원 PR 및 실제 연결 이슈에 처리 결과를 남긴다. 원 PR을 직접 merge한 것으로 표현하지 않고 통합 PR의 체리픽 수용 사실을 명시한다.
- 이 PR은 구조·왕복 계약 결과를 중심으로 기록하며, 수행하지 않은 시각 검증이나 존재하지 않는 이미지를 첨부하지 않는다.
- 메인터너 보정이 적용된 범위와 남은 시각 차이를 구분하고, 미실행 GPU 검증이나 원격 CI를 통과로 주장하지 않는다.
- UTF-8 body file로 게시하고 API로 본문을 재조회한다. 같은 처리 결과의 기존 코멘트가 있으면 새로 중복 등록하지 않고 기존 코멘트를 수정한다.
- 연결 이슈의 실제 closing reference와 상태를 확인한 후 종료한다. 첫 기여자 여부 및 해당 안내는 `post_merge.md`에 따라 실제 기여 이력을 확인해 반영한다.
- 원 contributor fork 브랜치를 임의로 삭제하지 않는다. 승인된 이번 작업 소유 임시 브랜치·worktree·target만 완료 후 절차에 따라 정리한다.

이 기록은 통합 PR 제출용이다. 원 PR 코멘트·close와 merge는 아직 수행하지 않았으며, 아래 제출 기준과 merge 전 조건을 따른다.


## PR 제출 기준 확정 (2026-09-10)

- 메인터너 보정 SHA: `4fc08b5e978e691b5885f6a1bdb0101fda563dc0`. 체리픽 마지막 SHA `bb5dc4b00fc1247374cd8b39f40e113c666745fc` 뒤에 보정을 별도 커밋했다.
- 제출 branch: `review/planet6897-6959-6983-20260910`. 원 PR 8개와 provenance-preserving 체리픽 18개를 유지한다.
- 로컬 검토 base는 `37bd46a72f9fd9ffd709e35244df79c00e789780`이다. PR 준비 시 원격 devel은 `ffc88e54fd17be9432d7985cd1c7ffa9aae39d3b`로 전진했다. rebase나 devel merge는 하지 않았고 최신 base와의 통합 검증은 새 PR의 CI 조건이다.
- 이전 전체 회귀는 9,413개 통과, 실패 0개, skip 46개였다. 이후 추가한 briefing fixture·테스트까지 이 전체 실행에 포함됐다고 주장하지 않는다.
- 최종 집중 검증은 61개 통과, 실패 0개, 57.004초, 8 threads였다. 이어 이전 실패와 관련된 3개만 다시 실행하여 3개 통과, 실패 0개, 0.539초를 확인했다. 과거 25개 통과·3개 실패 기록은 현재 결과가 아니다.
- 마지막 테스트 추가 뒤 manifest·fmt·tier 검사는 통과했다. native/WASM/workspace Clippy의 성공은 이전 실행 기록이며, 마지막 테스트·baseline 추가 뒤에는 다시 실행하지 않았다. 최종 head CI의 해당 gate를 merge 전에 확인해야 한다.
- 이번 PR 생성 단계에서는 검증을 추가 실행하지 않았다. CI 완료, merge, issue 종료 및 cleanup 완료를 미리 기록하지 않는다.

## Merge 후 contributor PR comment 계획

- 원 PR #6983에 체리픽 통합 수용 사실, 실제 통합 PR 번호와 merge SHA, 원 변경과 메인터너 보정의 차이를 기록한다. 원 PR을 직접 merge했다고 표현하지 않는다.
- Visual Sweep 및 검증 기록의 정본 direct link는 `https://github.com/edwardkim/rhwp/blob/<merge-commit-sha>/mydocs/pr/archives/pr_6983_review.md`로 고정한다.
- 실제 증적 범위: 16개 body-overflow partition과 전 코퍼스 baseline key 검사, 렌더 실패의 fail-closed 처리를 검증했다. 새 fixture의 기준선만 원 devel 실측으로 등록했고 기존 ceiling은 상향하지 않았다. 별도 시각 증적을 주장하지 않는다.
- 코멘트 작성 시 실제 merge SHA로 placeholder를 치환하고 UTF-8 body file과 `--body-file`을 사용한다. 기존 같은 통합 결과 코멘트가 있으면 새로 등록하지 않고 수정하며, 게시·수정 뒤 API로 본문을 재조회한다. PR/devel CI 완료와 관련 issue의 실제 상태를 확인한 뒤 `post_merge.md`에 따라 후속 처리한다.


## CI 실패 원인 보정 및 전체 회귀 재검증 (2026-09-10)

- [PR #6990](https://github.com/edwardkim/rhwp/pull/6990)의 최초 head `3450f1e5c78ed6fea133b7514c01245873c0edc2`에서 [Archive C IR sweep](https://github.com/edwardkim/rhwp/actions/runs/34465371486/job/102835336153)이 실패했다. 마지막 briefing 샘플 추가 뒤 집중 필터에 IR sweep을 넣지 않은 검증 누락이었다.
- 대상 원본: `samples/issue6924/148751598-briefing.hwp`, SHA-256 `03c93b021e01652b1ca5ba3a4a301decf9da33484af7d088987327efcb59e610`.
- 수정 전 devel `37bd46a72f9fd9ffd709e35244df79c00e789780` 빌드와 통합 후보 빌드에서 같은 파일의 raw 왕복은 발산 0건, rebuild 왕복은 상세 값까지 동일한 3건이었다. 비교 결과가 바이트 단위로 같음을 확인했다.
- 원인: 기존 `src/serializer/control.rs`는 빈 그림 extra에 한컴 호환 18바이트 꼬리를 보강하고, 없는 `img_dim`을 crop 크기로 채운다. `raw_picture_extra.len: 0 -> 18`, `img_dim[0]: 0 -> 224460`, `img_dim[1]: 0 -> 49680`은 이번 제품 변경으로 생긴 손실이 아닌 기존 정규화다.
- 보정 커밋: `5dacee15f837f8b6253d241a4fcbbbf83d2284e7`. IR 원장에 이 신규 샘플의 `img_dim[] = 2`, `raw_picture_extra.len = 1` 두 행만 등록했다. 기존 sample의 기준선 상향, 기존 행 재생성, 제품 코드·시각 asset 변경은 없다.
- 보정 후 전체 회귀: `cargo nextest run --locked --cargo-profile release-test --tests --no-fail-fast --test-threads 8`, **9,414개 통과 / 실패 0개 / skip 46개 / 300.371초**. IR sweep은 FAST 제한 없이 전수 실행해 70.711초에 통과했다. 이 실행에는 마지막 briefing fixture·회귀 및 신규 sample 3개 security sweep도 포함됐다.
- 같은 전용 target에서 fmt, native Clippy, WASM32 lib Clippy, workspace build, workspace all-target Clippy, manifest, Rust unit tier 검사를 모두 다시 통과했다. source-side 기준선은 4,205 tests / 298 modules로 유지했다.
- 로컬 최종 검증은 완료됐지만 기존 실패 CI를 성공으로 간주하지 않는다. 보정 push 뒤 새 PR head CI를 확인한 다음에만 merge 및 후속처리를 진행한다. 임시 probe·로그는 커밋하지 않는다.


## CI 완료 후 문서 trailing 기록 (2026-09-10)

- 실제 원격 code head는 `cc11b55e69656450a57629f2b7a2122732b08dc9`이다. `4f4e3f2e02ab3c0067cb09c3bb50e84d859a962c`에 최신 devel `4e0ce9283086ddd33e3660cf3927db8175384a43`을 merge한 기존 원격 커밋을 그대로 보존한다. 로컬 rebase 이력을 강제로 덮어쓰지 않았다.
- 위 head의 [CI](https://github.com/edwardkim/rhwp/actions/runs/34468254010), [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/34468253850), [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/34468253715), [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/34468253961), [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/34468253906) 실행이 모두 success로 완료됐다. CI의 실행된 archive A/B/C/D worker, Lint, Native Skia, Frontend package gate 및 CodeQL의 JS/TS·Python·Rust worker도 성공했다. [CI Impact Policy](https://github.com/edwardkim/rhwp/actions/runs/34469557687)도 success이며 `MERGEABLE / CLEAN`을 확인했다.
- 별도 WASM Build, Frontend unit gates, Workflow promotion preflight, PR의 nextest duration refresh는 skip이었다. 실행하지 않은 worker를 성공 실행으로 표현하지 않는다.
- 최신 devel 위에서 로컬 검증한 `0f7cafeb17ca00f872c08bef6a1aae722e100776`과 실제 원격 code head 사이에는 mydocs 외 파일 차이가 없음을 확인했다. 해당 통합 내용에서 전체 회귀 **9,415개 통과 / 실패 0개 / skip 46개 / 285.183초 / 8 threads**, IR 전수 sweep 69.964초 통과, fmt·native/WASM/workspace Clippy·workspace build·manifest·tier 재통과를 기록한다.
- 이전 9,414개 및 9,413개 실행은 이전 시점의 이력이며 최신 실행 결과와 구분한다. 기존 시각 증적은 앞 절에 기록한 실제 산출 시점·페이지·한계 그대로다. rebase 또는 devel merge 뒤 재촬영했다고 주장하지 않는다.
- 이번 trailing commit은 개별 리뷰 8개와 오늘할일 1개, 총 9개 Markdown만 변경한다. source/test/workflow/golden/baseline/sample/PDF/PNG/로그는 변경하지 않는다. 문서 push 뒤 최신 head의 required check는 merge 전 별도 확인 대상이다.
- 후속 코멘트의 문서 비교 방법 정본은 [Visual Sweep GitHub merge comment 가이드](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment)다. 개별 리뷰 direct link는 실행 결과 기록이며 방법 정본을 대신하지 않는다. 실제 merge SHA 고정 raw 이미지와 앞 절의 페이지·수치·사람 판정을 함께 게시하고, 기존 같은 결과의 코멘트는 중복 등록하지 않고 수정한다.

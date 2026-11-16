# PR #9096 최종 CI 실제 실행 감사

확인: 2026-11-15T13:46:09.623823+00:00 / head `b6a91166761b212d8501faf70b4b13e19b26474e` / devel `1805d8f08cb3a54bccda8dfa6647432b7799f074`.

판정: **통과**, 감사에서 확인한 차단 사항 없음. 최종 조회에서 head·devel 변경이 없고 check 40건은 success 34건·조건부 skipped 6건이며 미완료·실패는 0건이다. CI Impact Policy도 `mode=full`, `rfp=0`, `rust=1`, `fe=package`, `render=1`, `skia=1`, 4개 CodeQL 언어의 **success**다. 최신 정책 결과 URL은 https://github.com/edwardkim/rhwp/actions/runs/43720629727 이다.

| 범위 | 실제 로그·산출물 확인 |
| --- | --- |
| Archive A–E | 3,796 / 2,130 / 2,654 / 2,629 / 2,205건, 합계 **13,414건 통과·45 skip**. 집계의 `archive_total=13414 shard_total=13414`가 일치하며 `FAST_PASS=false`다. |
| Native Skia | 실제 focused cargo 검사 **245건 통과**. 로컬 공식 4,073건과 다른 CI 범위이므로 합산하거나 대체하지 않는다. |
| Lint | fmt, Native/WASM Clippy, workspace·WASM check와 운영 계약이 실제 실행됐다. |
| Frontend | fresh dev WASM을 실제 빌드하고 Studio **2,798건 통과·2 skip**, 실제 Canvas 글꼴 26건, undo-depth·responsive·Studio/확장 build·VSCode compile을 실행했다. |
| Adapter | svg/null/trace/png/skia 5개 모두 발견, missing·unexported 0, Rust 7건 통과. |
| Proptest | 실제 9개 suite 명령·63건 통과. |
| CodeQL | JS/TS·Python·Rust·Actions 모두 실제 쿼리 실행·SARIF 업로드 처리 완료. 초기 missing Rust configuration은 해소됐고 독립 CodeQL check도 success다. |
| Render Diff | Canvas legacy/layer 3사례 통과(0.01077% / 0% / 0%), Direct PDF 3/3 통과, CanvasKit readiness 8/8·hard-gate 위반 0. Chromium 1660786과 실제 served WASM 해시를 로그에서 확인했다. |

PDF browser/compatibility 보고서 4쪽에는 **report-only 경고 4건·오류 0건**이 남는다. 72 dpi PDF와 브라우저 출력의 크기·픽셀 차이를 기록한 항목이며, 해당 보고서는 threshold disabled다. Direct PDF compatibility는 별도의 동일 크기 비교에서 차이 비율 0.00757801 / 0.00269187 / 0.01883712로 0.02 한도를 모두 통과했다. report-only 경고를 독립 한컴 Print 일치율의 통과 근거로 사용하지 않는다. 독립 전쪽 시각 검증은 기존 로컬 증거를 따른다.

조건부 skip을 실제 workflow와 대조했다. `WASM Build`는 tag/manual, `Frontend unit gates`는 package lane과 선택 관계, promotion은 devel→main, release/native-host는 manual 전용이다. postmerge·review-only 대체 단계와 cache 저장·duration publication의 비적용도 제품 검사 생략이 아니다.

증거: [최종 snapshot](ci-actual-execution-final-snapshot.json), [job 로그 발췌](ci-actual-execution-log-evidence.json), [렌더·집계 상세](ci-actual-execution-detail-evidence.json), [업로드된 렌더 보고서](ci-actual-execution-render-artifact.json), [구조화 판정](ci-actual-execution-audit.json).

이번 감사는 읽기 전용 API·완료 로그·artifact 확인이며 재실행·원격 상태 변경은 수행하지 않았다.

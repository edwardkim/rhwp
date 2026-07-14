# PR #2242 리뷰

## 메타

| 항목 | 내용 |
| --- | --- |
| PR | [#2242](https://github.com/edwardkim/rhwp/pull/2242) |
| 작성자 | `planet6897` |
| 연관 이슈 | [#2240](https://github.com/edwardkim/rhwp/issues/2240), [#2238](https://github.com/edwardkim/rhwp/issues/2238), [#2239](https://github.com/edwardkim/rhwp/issues/2239), [#2169](https://github.com/edwardkim/rhwp/issues/2169) |
| stacked base | [#2232](https://github.com/edwardkim/rhwp/pull/2232) |
| 원격 head 참고값 | [`a25a416d`](https://github.com/edwardkim/rhwp/commit/a25a416d0d1e31d8cee08673b4e5358eaa9157c6), 2026-07-13 조회 |
| reviewer | `jangster77` review request 등록 |
| 규모 참고값 | 24 files, +1,702 / -410 |

## 변경 범위

- `86712_regulatory_analysis.hwpx` fixture를 export 결과로 재생성하고 65쪽 핀으로 정렬한다.
- `dump-pages`의 중간 fragment `used_height` 기록을 가시 높이 기준으로 보정한다.
- 괄호 narrow 폭 0.3em 처리를 실측 폰트와 KoPub 분기로 제한하고, 미실측 폰트는 0.5em fallback을 쓴다.
- 줄바꿈 사다리 도구를 추가한다. 원격 PR diff에 `README.md`는 없었다.

## 사전 검증

- 통합 브랜치의 focused release test에서 `issue_1891` 3 tests passed / 0 failed였다. 이 테스트는 86712=65, 76076=82쪽을 확인한다.
- `CARGO_INCREMENTAL=0 cargo test --profile release-test --lib test_paren_narrow_is_font_conditioned`는 1 passed / 0 failed였다.
- `wasm-pack build --target web --out-dir pkg`가 성공했다.
- `CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`가 경고 없이 성공했다.
- `CARGO_INCREMENTAL=0 cargo test --profile release-test --tests` 전체 회귀가 exit 0으로 통과했다.
- 작성 시점 참고로 원격 CI는 Native Skia tests를 제외한 필수 check가 성공했다. Native Skia tests는 `Install native Skia runtime packages` 단계에서 장시간 `IN_PROGRESS`다. 통합 로컬 회귀/Clippy가 통과했으므로 코드 결함으로 분류하지 않으며, merge 전 force-cancel/re-run으로 원격 CI 운영 상태만 정리한다.

## 렌더 및 fixture 검토

- 86712는 조판 영향이 직접적인 HWPX fixture이므로 기준 PDF와 함께 검토 대상이다.
- 통합 sweep의 fixture/PDF 기준은 65쪽으로 정합했으며, 상세 시각 증거는 상위 통합 PR 준비 시 영구 asset으로 보존한다.
- fixture 재생성은 binary churn 위험이 있으므로, 76076/80168/80250을 불필요하게 같이 바꾸지 않은 범위 판단은 적절하다.

## 리스크와 권고

- `used_height`는 진단 값이지만 downstream dump parser가 의존할 수 있다. 전체 회귀는 통과했으며, merge 전 최신 원격 diff에 대해서도 같은 확인이 필요하다.
- 괄호 폭 변경은 폰트 의존성이 크다. focused regression은 통과했으며, 전체 회귀 결과를 최종 문서에 추가 반영해야 한다.
- 최종 권고: [#2232](https://github.com/edwardkim/rhwp/pull/2232) 선행 반영 뒤 수용. 현재 원격 head의 소스·샘플 patch는 통합 검증 코드와 동일하며, 남은 일은 Native Skia job 재실행뿐이다.

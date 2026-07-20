# PR #2469 검토 기록

| 항목 | 내용 |
|---|---|
| 원 PR | [#2469](https://github.com/edwardkim/rhwp/pull/2469) |
| 작성자 / base | kevin9327 / `devel` |
| 범위 | HWP3 위첨자·아래첨자 글자속성 IR 매핑 보존 |
| 검토자 | @jangster77 (검토 전 지정) |
| 검토 스냅샷 | 2026-07-20 GitHub 조회: +24/-0, 1 file, `maintainerCanModify=true`, `mergeStateStatus=BEHIND` (동적 참고값) |
| 판단 | 누적 통합 PR에 수용 |

## 검토와 검증

- PR 본문은 HWP3 문자 속성의 superscript/subscript 매핑 누락을 설명했고, PR 코멘트는 없었다.
- 기여자 변경 `20a4b9d48`을 충돌 없이 적용했다. parser/IR 보존 변경이며 시각 검증은 병합 판단의 직접 근거가 아니다.
- HWP3 focused 회귀와 최종 release-test, Clippy, WASM 빌드를 통과했다.

## 렌더 영향 판정

- HWP3 parser/IR 문자 속성 보존만 다루며 renderer·layout 출력 경로를 바꾸지 않는다. visual sweep은 필요하지 않다.

## 리스크와 권고

- HWP3 문자 속성의 IR 매핑만 보정하며, 다른 문자 효과의 의미를 확장하지 않는다.
- **권고**: 누적 통합 PR에 수용. 최신 통합 PR head의 CI가 성공한 뒤에만 merge한다.

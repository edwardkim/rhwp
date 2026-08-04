# PR #2671 검토 기록

| 항목 | 내용 |
|---|---|
| PR | [#2671](https://github.com/edwardkim/rhwp/pull/2671) |
| 작성자 / base | [@planet6897](https://github.com/planet6897) / `devel` |
| 관련 이슈 | [#2524](https://github.com/edwardkim/rhwp/issues/2524) |
| 원 commit / 누적 적용 | `cd054b75` / `35ec688ba` (충돌 없음, 선행 의존 없음) |
| 메인터너 보정 | `63694e90e`: 정확한 embedded face의 `@font-face` 규칙을 직접 검사하도록 회귀 테스트 강화 |
| 범위 | BinData 임베디드 폰트의 SVG `data:font` data URI 방출 |
| 처리 경로 | collaborator 누적 통합 검토. merge 전 최신 원 PR 상태와 CI 재확인 필요 |

## 변경과 검증

- 임베디드 face명과 원본 bytes를 수집해 Subset/Style/Full 모드에서 local fallback 대신 전체 SFNT를
  `data:font` URI로 방출한다. native bitmap glyph 테이블 보존을 위해 이 경우 서브셋하지 않는다.
- 메인터너 보정 후 `issue_2524_embedded_font_svg` 3/3은 정확한
  `RHWP Bitmap SVG Glyph Smoke` 규칙에 `data:font/ttf`가 있고 `local(...)`이 없는지 확인한다.
- 전체 release-test integration, clippy, fresh WASM build가 성공했다.
- 실제 CLI 산출 SVG도 해당 face에 `data:font/ttf;base64,... format("truetype")`를 기록했다.

## 한계와 권고

인앱 브라우저가 보안 정책상 로컬 `file://` SVG 접근을 차단해 Blink의 실제 화면 렌더는 이 환경에서
수행하지 못했다. 정책 우회는 하지 않았다. 생성 SVG와 native 회귀 테스트는 근본 원인(local-only
fallback)을 직접 검증하므로 merge blocker는 아니지만, [#2524](https://github.com/edwardkim/rhwp/issues/2524)는
실제 Chrome/Windows 시각 확인까지 open으로 유지한다. 최신 head CI와 작업지시자 승인이 충족되면 통합 PR로
merge 가능하다.

# PR #6560 검토 - 실제 flow underrun에 한정한 앵커 블록 마진

- 검토일: 2026-09-01
- 작성자: `planet6897`
- 원 PR/source head: `20537c46d0d21ce59fbb4cc2facebdb8cabcf0d9`
- 누적 순서: 3/3 (`#6554 -> #6559 -> #6560`)
- 통합 적용 commit: `ee4d7c4ad196b6642a4bfc14bd8fa9d11bb2994a`
- 통합 기준: `upstream/devel@6d3fd65a30cd8e6755b18a1ab44c5035279b110f`
- conflict: 없음
- 판정: 승인

## 코드 판정

앵커 `vpos` 복원이 fit 위치를 끌어올렸다는 사실만으로 50px 불확실성 마진을 적용하면 실제 내용
하단과 flow가 일치하는 저슬랙 문서도 불필요하게 다음 쪽으로 갈린다. PR은 기존 두 조건에
`st.flow_underrun > 0.5`를 추가해 흐름 좌표가 실제 내용 하단보다 뒤처질 때만 마진을 건다.

슬랙 값이 겹치는 흡수·분할 코호트에서 `flow_underrun`이 0과 37.60으로 갈린다는 원 PR 계측은 코드의
의도와 일치한다. 대상 fixture는 누적 head에서 1쪽이 됐고 focused test가 통과했다.

## 누적 검증

- `low_slack_anchor_block_is_absorbed_into_the_same_page`: 통과, 결과 1쪽
- oracle page-count와 off-canvas 각 16 partition 통과
- IR field sweep 4/4 통과
- #6554와 #6559를 먼저 누적한 head에서 적용해 같은 `typeset.rs` 영역의 순서 충돌이 없음을 확인했다.
- #6554와 #6560만 비교한 양방향 적용 tree는 동일했지만 실제 integration은 고정 순서
  `#6554 -> #6559 -> #6560`만 사용했다.
- 누적 head `ee4d7c4ad196b6642a4bfc14bd8fa9d11bb2994a`의 필수 Rust lint 묶음 전부 통과
- `release-test` 전체 회귀 8,917/8,917 통과, 46건 정책상 skip, 실패 0건(313.667초)
- Native Skia 전체 `--lib`, 누락 이미지 placeholder 2/2, 직접 PDF export 4/4 통과
- Docker 29.7.2 WASM 빌드 통과(6분 15초, `/app/pkg` 생성)
- 로컬 `cargo-nextest` 0.9.137은 권장 0.9.140보다 낮아 `junit.report-skipped` 설정 경고가 있었지만
  test selection과 실행 결과에는 영향이 없었다. 이 환경 차이는 통합 PR에 명시한다.
- Draft 통합 PR #6563의 code candidate head `ee4d7c4ad196b6642a4bfc14bd8fa9d11bb2994a`에서 Full CI가
  성공했다. CI run `33480418819`, CodeQL `33480418938`, Render Diff `33480418671`, Proptest
  `33480418840`, Adapter inter-diff `33480418917`이 모두 녹색이었다.

## engine 2020 직접 시각 검증과 이슈 범위

원본 `samples/issue6535/36339092_low_slack_absorb_block.hwpx`의 SHA-256은
`28f17fae566427400084a30985a62e731c1907c42b25d52eaa000bb61a83e0ad`이고 마지막 저장 제품은
`hancom-office-2020`(`11.0.0.8227`)이다. 정책상 engine 2020 PDF가 기준이어야 한다.

원 PR의 `mydocs/report/low-slack-anchor-absorb-6535/pages_before_after_oracle.png`는 한컴 2024
참고 자료로만 두었다. 검토자는 MCP client의 비동기 `start -> status -> download` 흐름으로 engine
2020 PDF를 새로 산출했다.

- MCP job: `2b648f7e-18b5-4d7b-81cf-dd7a0f4af6cc`
- 요청/응답 engine·profile: `2020` / `2020`
- 한컴 version: `12.0.0.4605`
- backend/worker: `hwp-managed-direct-dll-host`, 32-bit
- 결과: success, 38,403 bytes, 1쪽 A4, PDF 1.6
- 기준 PDF: `pdf/36339092_low_slack_absorb_block-2020.pdf`
- 기준 PDF SHA-256: `0772e32e1a98a22042a2c25755ab493fe2b79d5096a1ce9ad5d5b3a400d6f809`
- 비교 절차: [PDF/SVG visual sweep 가이드](../../manual/verification/visual_sweep_guide.md#github-merge-comment)
- 실행 command:

```bash
python3 scripts/visual_sweep.py \
  --key pr6560-low-slack \
  --hwp samples/issue6535/36339092_low_slack_absorb_block.hwpx \
  --pdf pdf/36339092_low_slack_absorb_block-2020.pdf \
  --pages 1 \
  --svg-rasterizer rsvg \
  --out output/pr-review-planet/6560-2020
```

physical page 1은 pixel match 93.27189%, visual accuracy proxy 23.53908%, 자동 후보 0건이다. 원본
파일명의 선두 숫자를 visual sweep이 page label로 읽어 임시 asset 이름은 `review_36339092.png`가 됐지만,
rhwp와 PDF가 각각 단일 page라 도구의 singleton 1:1 fallback으로 physical page 1을 비교했다.

review PNG를 직접 열어 본문, 첨부 목록, 시장 서명, 하단 업무표가 모두 한 쪽 안에서 같은 순서로
배치되고 겹침이 없음을 확인했다. font·glyph의 픽셀 차이는 남지만 이번 계약인 저슬랙 block 흡수와
하단 안전성은 일치한다.

- 임시 review:
  `output/pr-review-planet/6560-2020/pr6560-low-slack/review/review_36339092.png`
- 대표 asset: [physical p1 review](../assets/pr_6560_issue6535_2020_review.png)
- 대표 asset SHA-256:
  `36f8101333eda352a98efde3e238a3a6900174f87e451bbc3aa8f04e0491c64d`

#6535에는 처음 보고된 일곱 사례와 이후 원인이 다른 갈래가 함께 있다. 이번 margin 갈래가 통과해도
`1250000...`, `36399617` 등 별도 원인 사례의 해결 여부를 확인하기 전에는 이슈 전체를 자동 close하지
않는다.

## Merge 후 contributor PR comment 계획

- 원 head `20537c46` -> 적용 `ee4d7c4ad` -> 통합 merge SHA의 계보를 남긴다.
- engine 2020 physical p1, 자동 후보 0건, 위 지표와 본문·서명·하단 업무표의 1쪽 흡수를 알린다.
- 대표 PNG `mydocs/pr/assets/pr_6560_issue6535_2020_review.png`는
  `<merge-commit-sha>` 고정 raw URL로 표시한다.
- UTF-8 without BOM body file로 게시하고 API로 재조회한 뒤 원 PR을 중복 병합하지 않고 close한다.
- #6535는 이번 PR 하나만으로 close하지 않는다.

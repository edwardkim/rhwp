# PR #9417 검토 — 저장 중첩·부동 표의 줄 간격과 쪽 이어받기

## 판정

puckqook의 원 PR 세 건을 `upstream/devel` `32014de6e0` 위에 오래된 순서로 누적 적용했다.
적용 커밋 10개에 원본 SHA를 남겼다. #9362는 마지막 셀의 저장 줄 간격과 다음 글줄 원점을
메인터너가 보정했다. #9378의 충돌은 최신 devel의 일반 글줄 배치와 원 PR의 부동 표 분기를 함께
유지해 해소했다. #9381의 13pt·10pt 제목과 두 부동 표는 독립 PDF와 같은 쪽에 놓인다.

검토 중 #9412가 병합된 `5fa8cede50`을 반영했다. 분할 칸에서 저장 호스트가 높이를 예약한 표와
흐름 밖 겹침 표는 모두 후속 문단의 원점을 다시 전진시키지 않는다. #9412의 71개 입력·인쇄
조합에서 각 출력 경로의 PNG 110쪽이 기준과 같았고 대형 표의 161쪽 배치 JSON도 같았다. [#9412 출력 대조](../assets/puckqook-9042-integration/9412-preservation.json)

- [#9362 검토](pr_9362_review.md)
- [#9378 검토 및 기존 대조 입력 차이](pr_9378_review.md)
- [#9381 검토](pr_9381_review.md)

## 검증

원 PR 4개 문서의 9쪽은 Native·fresh WASM 각각 raw 2px 실루엣 100.00%였고, 정본 보호 23종의 4,046개 출력 쪽은 표시값이 전부 정본과 같았다. 최소 글꼴 환경에서 Ubuntu 집중 회귀 45건·전체 nextest 14,169건과 macOS Native Skia 4,225건이 통과했다.
macOS에서 Native·WASM32·workspace Clippy와 workspace 빌드가 성공했고,
Chrome Studio에서 동일한 최적화 WASM을 불러와 쪽수, 표·본문의 위치 및 중복 여부를 확인했다.
[전쪽 TSV, 비교 이미지 및 테스트 증거](../assets/puckqook-9042-integration/README.md)

GitHub Actions는 코드 head `7671c17a0f43590ccce98a5ac9d4c9a665ff1507`에서 성공했다. [CI](https://github.com/edwardkim/rhwp/actions/runs/45701811247) · [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/45701811240) · [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/45701811234) · [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/45701811054) · [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/45701811130) · [CI Impact Policy Controller](https://github.com/edwardkim/rhwp/actions/runs/45701810993)

최종 기록 커밋의 GitHub Actions와 병합 가능 상태를 확인한 뒤 이 PR을 병합하고, 원 PR에 결과를 연결한다.

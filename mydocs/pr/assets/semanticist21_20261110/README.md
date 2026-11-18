# semanticist21 바탕쪽·메모 통합 검증

원 PR [#8866](https://github.com/edwardkim/rhwp/pull/8866)와
[#8885](https://github.com/edwardkim/rhwp/pull/8885)의 주장별 실측과 메인터너 보정 증적이다.
원 head·보정·원격 CI 상태는 [#8866 검토](../../archives/pr_8866_review.md)와
[#8885 검토](../../archives/pr_8885_review.md)를 따른다.

## 출처와 재사용

- 새 base: `411d4a91fd692f88fdeebd6b323eca0b295318b1`, 통합 code head: `6eccab18f3458090bb8c037c8040f49c28b102f7`.
- 시각 runtime source: `32c6023fb5f333bb9c464c75258f922559176047`. [파일별 source 해시](source-provenance.json).
- 리베이스 전 전체 13,258 PASS/0 FAIL/45 skip 재사용. 리베이스 후 실제 집중 35 PASS/0 FAIL.
- 사용자 승인에 따라 충돌 없는 리베이스의 자료를 재사용했다. [패치 대응](rebase-commit-map.tsv),
  [호출 경로 감사](rebase-read-path-audit.json), [49개 입력·원본 그림 Git 일치](post-rebase-committed-input-audit.json).
- [정식 검증 요약](pre-rebase-validation.json)의 `gates.latest_steps`는 완료한 최종 시도만 선택했다.
  잘못된 base archive·공유 target의 기준 바이너리 오인·글꼴/그림 공급 누락 시도는 통과 근거에서 제외했다.
- fresh WASM은 Mac locked wrapper의 `--target web --out-dir pkg --no-opt` 로컬 대체 빌드다.
  Docker 최적화 빌드 통과를 주장하지 않는다. [package·Studio 해시](wasm-stage-hashes.json).
- 로그와 임시 스크립트·비공개 글꼴은 이 디렉터리에 포함하지 않았다.

## 실제 결과

| 범위 | Native | fresh WASM | 판정 |
| --- | --- | --- | --- |
| 바탕쪽 2종 원본 각 3쪽 | 6쪽 모두 100.00% | 6쪽 모두 100.00% | flagged 0, 직접 판독 완료 |
| 보호 18개 문서 각 1,771쪽 | 최저98.36% | 최저98.37% | 전 3,542쪽 공식 두 자리 표시값 하락 0 |
| sample10 연결 그림 복원 1쪽 | 기존 99.96% | 공급 누락 31.60% → 공급 후 99.96% | 그림 3개 해시 일치, 제외 영역 없음 |

`master-pages/base/`·`master-pages/candidate/`는 같은 원본·독립 한컴 12.30 Print의 전쪽 TSV다.
후보의 각 페이지에는 raw PNG·PDF raster·review·overlay를 함께 보존했다. `100.00%`는 2px 내용
실루엣이며 RGB 동일성은 아니다. `protected/native/`·`protected/wasm/`에는 36개 조건의 원문 TSV가 있다.
보호 비교의 명시적 원본 글꼴 공급은 최소 글꼴 Rust 회귀 환경과 별개다.

sample10 외부 그림은 기존 저장소의 원본 3개를 사용했다. [그림이 쓰이는 쪽 감사](sample10-linked-image-reuse-audit.json),
[1쪽 raw PNG](sample10-linked-images/rhwp_png/rhwp_001.png),
[review](sample10-linked-images/review/review_001.png), [overlay](sample10-linked-images/overlay/overlay_001.png).
나머지 762쪽 PNG를 재사용해763쪽을 다시 계산한 출처는 [전쪽 재계산 기록](sample10-linked-images/linked-image-reuse-provenance.json)에 있다.
메모 API 검증은 관련 없는 바탕쪽 그림으로 대신하지 않고 [실제 API 결과](actual-browser/result.json)를 연결한다.

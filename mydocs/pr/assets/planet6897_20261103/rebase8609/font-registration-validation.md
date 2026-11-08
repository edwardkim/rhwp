# 글꼴 공급 계약과 첫머리 주석 검증

변경 범위는 실제 글꼴 프로그램 공급 adapter와 그 합성 계약이다. 설치된 상용 글꼴에 의존하지 않도록
공개 글꼴 5개만 노출한 격리 환경에서 실제 Node 테스트를 실행했다. 공급 바이트는 테스트 내부 합성 자원이다.

**11 PASS·0 FAIL·0 SKIP·exit 0**. 일반 글꼴 공급 원본 실행과 첫머리 주석 보완 후 재실행이 모두 통과했다.
TTF 원본/HFT 대체의 소유, 콤마 family 체인, 여러 family/분할 CFF의 공유 예산·순서, 실제 적재한 face의
PUA·공백 등록 경로와 구형 WASM 대조를 확인했다. 이 결과는 실제 글꼴 모양의 독립 Print 일치율이 아니다.

```bash
RHWP_REVIEW_WORKTREE=/home/tsjang/rhwp \
bash /home/tsjang/rhwp/output/pr-review/planet6897-20261103/run_minfont_review.sh node --test scripts/tests/visual-sweep-native-pua-supply.test.mjs
```

| 실행 | 시작 → 종료 UTC | exit | 로그 SHA-256 |
| --- | --- | --- | --- |
| 주석 보완 전 | 2026-11-05T02:04:41.940296+00:00 → 2026-11-05T02:04:43.498150+00:00 | 0 | `485a4db8a3e03eaab47bec4995ef4995a64c61506de45d773c36c69fc442e581` |
| 주석 보완 후 | 2026-11-05T02:07:01.394161+00:00 → 2026-11-05T02:07:01.659907+00:00 | 0 | `bc6e8f0f92ce43c1719ab0869f0b640a689354d7673bbdecc8c3c9e7e301d1ab` |

## 첫 줄부터의 주석과 실행 코드 동일성

- 파일: `scripts/tests/visual-sweep-native-pua-supply.test.mjs`.
- 주석 갱신 시각: `2026-11-05T11:07:01+09:00`.
- Git에서 확인한 직전 변경 commit·시각: `93f3043ff764d8cb11c67a4870d99bf9bf1d2b00 2026-11-05T09:47:35+09:00`.
- 원문 전체 SHA-256: `3729d0d18ef10b28d8b799dcf25648dcadb7475792d02f2211a71aacab5af684`.
- 주석 보완 후 전체 SHA-256: `427a6be6f42cc3a7f3931389529e0cffdec4fb0cc9d5d0087105d219b01681f4`.
- 새로 앞에 붙인 1065바이트를 제외한 나머지는 원문과 바이트 단위로 동일하다.
- 첫 줄에 관계형임을 적고 합성 입력·실물 측정 비대상·별도 전쪽 조판 증거를 구분했다.
- 검사 함수·assertion·fixture 바이트·공급 module은 주석 추가로 바뀌지 않았다. 이 작업 때문에 전체 Rust를 다시 시작하지 않는다.

조판에 공급된 실제 Native/fresh WASM 증거는 [통합 목록](../README.md)의 해당 제품·원본별 기록을 따른다.
원시 로그와 주석 대조 JSON은 로컬 output에 보존하며 제출 자료에 복사하지 않는다.

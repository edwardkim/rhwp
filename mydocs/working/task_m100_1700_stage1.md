# Stage 1 완료보고 — Task #1700 베이스라인/재현

## 재현 고정 셋
- 18건: `output/poc/task1700_fixture_18.txt` (별표/서식 hwp 15 + 결재 hwpx 3).
- 출처: `verify_pi_page_vs_hangul.py --batch hwpdocs --sample 1000 --seed 42`
  결과 중 PARA_COUNT(rhwp<한글) 12 + PI_MISMATCH `rhwp_pNone` 6.

## 수정 전 dump-pages 누락 확인 (대표 2건)

### 별표 `17978249_[별표 1] 지방우정청의 관할구역…` (rhwp 2 / 한글 3문단)
- `dump`(IR): 문단 0.0(캡션) / 0.1(표 11×2, wrap=Square) / **0.2(cc=1 빈 문단)** — IR엔 3개.
- `dump-pages`: `pi=0`, `pi=1` 만 출력 → **pi=2 누락** (wrap_around로 흡수, items 미포함).
- 한글: para0/1/2 전부 1쪽.

### 예규 `2957879_[별지 13]…` (rhwp/한글 2쪽, pi3 rhwp_pNone)
- `dump`(IR): 0.0..0.2(표) **0.3(빈)** 0.4(표) 0.5(텍스트) 0.6(빈) 0.7(빈).
- `dump-pages`: `pi=0,1,2,4,5,6,7` → **pi=3 누락** (표 사이 빈 문단).
  텍스트 뒤 빈 문단 0.6/0.7은 정상 배치(items 포함).

## 결론
누락은 `rendering.rs` dump-pages가 `cc.items`만 순회하고
`cc.wrap_around_paras`를 출력하지 않는 데서 발생(레이아웃/렌더 트리는 정상).
Stage 2에서 wrap_around_paras 표면화로 해소.

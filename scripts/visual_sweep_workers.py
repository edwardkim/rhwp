"""CPU 작업은 별도 프로세스에서 실행하고 결과는 요청한 쪽 순서로 반환한다."""

from concurrent.futures import ProcessPoolExecutor
from multiprocessing import get_context


def ordered_page_map(function, jobs, workers=1):
    if type(workers) is not int or workers < 1:
        raise ValueError("쪽 작업자 수는 양의 정수여야 합니다.")
    if workers == 1 or len(jobs) <= 1:
        yield from map(function, jobs)
        return
    # Windows와 Linux에서 같은 spawn 계약을 사용한다. 부모의 Chrome/파일
    # descriptor를 복제하지 않으며 작업자 종료까지 기다린 뒤 반환한다.
    with ProcessPoolExecutor(
        max_workers=min(workers, len(jobs)), mp_context=get_context("spawn")
    ) as pool:
        yield from pool.map(function, jobs)

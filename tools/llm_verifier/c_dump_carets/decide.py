from __future__ import annotations
from typing import Any, Mapping

COMMAND = 'dump-carets'
FAMILY = 'count_eq'
TITLE = 'dump-carets 캐럿 수가 배열 길이'

def check_dump_carets_pageCount_wrong_type_int(env: Mapping[str, Any]) -> str | None:
    raw = env.get('pageCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return "USAGE"
    return None

def check_dump_carets_pageCount_negative(env: Mapping[str, Any]) -> str | None:
    raw = env.get('pageCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < 0:
        return "USAGE"
    return None

def check_dump_carets_pageCount_nonzero_required(env: Mapping[str, Any]) -> str | None:
    raw = env.get('pageCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw == 0:
        return "ZERO_BYTES"
    return None

def check_dump_carets_pageCount_exit_closed(env: Mapping[str, Any]) -> str | None:
    raw = env.get('pageCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw not in (0, 1, 2, 3, 4):
        return "EXIT_UNKNOWN"
    return None

def check_dump_carets_pageCount_too_large(env: Mapping[str, Any]) -> str | None:
    raw = env.get('pageCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw > 1_000_000_000:
        return "USAGE"
    return None

def check_dump_carets_pageCount_fits_i32(env: Mapping[str, Any]) -> str | None:
    raw = env.get('pageCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < -2147483648 or raw > 2147483647:
        return "USAGE"
    return None

def check_dump_carets_pageCount_not_bool_int(env: Mapping[str, Any]) -> str | None:
    raw = env.get('pageCount', None)
    if isinstance(raw, bool):
        return "USAGE"
    return None

def check_dump_carets_pageCount_lte_page_count(env: Mapping[str, Any]) -> str | None:
    raw = env.get('pageCount', None)
    total = env.get("pageCount")
    if raw is None or total is None:
        return None
    if isinstance(raw, bool) or isinstance(total, bool):
        return None
    if isinstance(raw, int) and isinstance(total, int) and raw > total:
        return "COORD_OOB"
    return None

def check_dump_carets_pageCount_lte_declared(env: Mapping[str, Any]) -> str | None:
    raw = env.get('pageCount', None)
    declared = env.get("declaredCount")
    if raw is None or declared is None:
        return None
    if isinstance(raw, bool) or isinstance(declared, bool):
        return None
    if isinstance(raw, int) and isinstance(declared, int) and raw > declared:
        return "COUNT_DRIFT"
    return None

def check_dump_carets_pageCount_under_cap(env: Mapping[str, Any]) -> str | None:
    raw = env.get('pageCount', None)
    cap = env.get("capBytes")
    if raw is None or cap is None:
        return None
    if isinstance(raw, bool) or isinstance(cap, bool):
        return None
    if isinstance(raw, int) and isinstance(cap, int) and cap > 0 and raw > cap:
        return "OVER_ACCEPTED"
    return None

def check_dump_carets_pageCount_year_range(env: Mapping[str, Any]) -> str | None:
    raw = env.get('pageCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < 1990 or raw > 2100:
        return "USAGE"
    return None

def check_dump_carets_pageCount_fail_vs_identical(env: Mapping[str, Any]) -> str | None:
    raw = env.get('pageCount', None)
    identical = env.get("identical")
    if raw is None or identical is not True:
        return None
    if isinstance(raw, int) and not isinstance(raw, bool) and raw > 0:
        return "COUNT_DRIFT"
    return None

def check_dump_carets_pageCount_ok_vs_total(env: Mapping[str, Any]) -> str | None:
    raw = env.get('pageCount', None)
    total = env.get("total")
    if raw is None or total is None:
        return None
    if isinstance(raw, bool) or isinstance(total, bool):
        return None
    if isinstance(raw, int) and isinstance(total, int) and raw > total:
        return "RATE_IMPOSSIBLE"
    return None

def check_dump_carets_pageCount_span_fits(env: Mapping[str, Any]) -> str | None:
    raw = env.get('pageCount', None)
    rows = env.get("rows")
    cols = env.get("cols")
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if isinstance(rows, int) and not isinstance(rows, bool) and 'pageCount' == "rowSpan" and raw > rows:
        return "SPAN_OOB"
    if isinstance(cols, int) and not isinstance(cols, bool) and 'pageCount' == "colSpan" and raw > cols:
        return "SPAN_OOB"
    return None

def check_dump_carets_pageCount_width_height(env: Mapping[str, Any]) -> str | None:
    raw = env.get('pageCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if 'pageCount' in ("width", "height", "bytes") and raw < 0:
        return "USAGE"
    return None

def check_dump_carets_pageCount_batch_parts(env: Mapping[str, Any]) -> str | None:
    raw = env.get('pageCount', None)
    n_in = env.get("inputN")
    n_ok = env.get("okN")
    n_fail = env.get("failN")
    if raw is None:
        return None
    if not all(isinstance(v, int) and not isinstance(v, bool) for v in (n_in, n_ok, n_fail) if v is not None):
        return None
    if isinstance(n_in, int) and isinstance(n_ok, int) and isinstance(n_fail, int):
        if n_in != n_ok + n_fail:
            return "COUNT_DRIFT"
    return None

def check_dump_carets_paraCount_wrong_type_int(env: Mapping[str, Any]) -> str | None:
    raw = env.get('paraCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return "USAGE"
    return None

def check_dump_carets_paraCount_negative(env: Mapping[str, Any]) -> str | None:
    raw = env.get('paraCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < 0:
        return "USAGE"
    return None

def check_dump_carets_paraCount_nonzero_required(env: Mapping[str, Any]) -> str | None:
    raw = env.get('paraCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw == 0:
        return "ZERO_BYTES"
    return None

def check_dump_carets_paraCount_exit_closed(env: Mapping[str, Any]) -> str | None:
    raw = env.get('paraCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw not in (0, 1, 2, 3, 4):
        return "EXIT_UNKNOWN"
    return None

def check_dump_carets_paraCount_too_large(env: Mapping[str, Any]) -> str | None:
    raw = env.get('paraCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw > 1_000_000_000:
        return "USAGE"
    return None

def check_dump_carets_paraCount_fits_i32(env: Mapping[str, Any]) -> str | None:
    raw = env.get('paraCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < -2147483648 or raw > 2147483647:
        return "USAGE"
    return None

def check_dump_carets_paraCount_not_bool_int(env: Mapping[str, Any]) -> str | None:
    raw = env.get('paraCount', None)
    if isinstance(raw, bool):
        return "USAGE"
    return None

def check_dump_carets_paraCount_lte_page_count(env: Mapping[str, Any]) -> str | None:
    raw = env.get('paraCount', None)
    total = env.get("pageCount")
    if raw is None or total is None:
        return None
    if isinstance(raw, bool) or isinstance(total, bool):
        return None
    if isinstance(raw, int) and isinstance(total, int) and raw > total:
        return "COORD_OOB"
    return None

def check_dump_carets_paraCount_lte_declared(env: Mapping[str, Any]) -> str | None:
    raw = env.get('paraCount', None)
    declared = env.get("declaredCount")
    if raw is None or declared is None:
        return None
    if isinstance(raw, bool) or isinstance(declared, bool):
        return None
    if isinstance(raw, int) and isinstance(declared, int) and raw > declared:
        return "COUNT_DRIFT"
    return None

def check_dump_carets_paraCount_under_cap(env: Mapping[str, Any]) -> str | None:
    raw = env.get('paraCount', None)
    cap = env.get("capBytes")
    if raw is None or cap is None:
        return None
    if isinstance(raw, bool) or isinstance(cap, bool):
        return None
    if isinstance(raw, int) and isinstance(cap, int) and cap > 0 and raw > cap:
        return "OVER_ACCEPTED"
    return None

def check_dump_carets_paraCount_year_range(env: Mapping[str, Any]) -> str | None:
    raw = env.get('paraCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < 1990 or raw > 2100:
        return "USAGE"
    return None

def check_dump_carets_paraCount_fail_vs_identical(env: Mapping[str, Any]) -> str | None:
    raw = env.get('paraCount', None)
    identical = env.get("identical")
    if raw is None or identical is not True:
        return None
    if isinstance(raw, int) and not isinstance(raw, bool) and raw > 0:
        return "COUNT_DRIFT"
    return None

def check_dump_carets_paraCount_ok_vs_total(env: Mapping[str, Any]) -> str | None:
    raw = env.get('paraCount', None)
    total = env.get("total")
    if raw is None or total is None:
        return None
    if isinstance(raw, bool) or isinstance(total, bool):
        return None
    if isinstance(raw, int) and isinstance(total, int) and raw > total:
        return "RATE_IMPOSSIBLE"
    return None

def check_dump_carets_paraCount_span_fits(env: Mapping[str, Any]) -> str | None:
    raw = env.get('paraCount', None)
    rows = env.get("rows")
    cols = env.get("cols")
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if isinstance(rows, int) and not isinstance(rows, bool) and 'paraCount' == "rowSpan" and raw > rows:
        return "SPAN_OOB"
    if isinstance(cols, int) and not isinstance(cols, bool) and 'paraCount' == "colSpan" and raw > cols:
        return "SPAN_OOB"
    return None

def check_dump_carets_paraCount_width_height(env: Mapping[str, Any]) -> str | None:
    raw = env.get('paraCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if 'paraCount' in ("width", "height", "bytes") and raw < 0:
        return "USAGE"
    return None

def check_dump_carets_paraCount_batch_parts(env: Mapping[str, Any]) -> str | None:
    raw = env.get('paraCount', None)
    n_in = env.get("inputN")
    n_ok = env.get("okN")
    n_fail = env.get("failN")
    if raw is None:
        return None
    if not all(isinstance(v, int) and not isinstance(v, bool) for v in (n_in, n_ok, n_fail) if v is not None):
        return None
    if isinstance(n_in, int) and isinstance(n_ok, int) and isinstance(n_fail, int):
        if n_in != n_ok + n_fail:
            return "COUNT_DRIFT"
    return None

def check_dump_carets_itemCount_wrong_type_int(env: Mapping[str, Any]) -> str | None:
    raw = env.get('itemCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return "USAGE"
    return None

def check_dump_carets_itemCount_negative(env: Mapping[str, Any]) -> str | None:
    raw = env.get('itemCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < 0:
        return "USAGE"
    return None

def check_dump_carets_itemCount_nonzero_required(env: Mapping[str, Any]) -> str | None:
    raw = env.get('itemCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw == 0:
        return "ZERO_BYTES"
    return None

def check_dump_carets_itemCount_exit_closed(env: Mapping[str, Any]) -> str | None:
    raw = env.get('itemCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw not in (0, 1, 2, 3, 4):
        return "EXIT_UNKNOWN"
    return None

def check_dump_carets_itemCount_too_large(env: Mapping[str, Any]) -> str | None:
    raw = env.get('itemCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw > 1_000_000_000:
        return "USAGE"
    return None

def check_dump_carets_itemCount_fits_i32(env: Mapping[str, Any]) -> str | None:
    raw = env.get('itemCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < -2147483648 or raw > 2147483647:
        return "USAGE"
    return None

def check_dump_carets_itemCount_not_bool_int(env: Mapping[str, Any]) -> str | None:
    raw = env.get('itemCount', None)
    if isinstance(raw, bool):
        return "USAGE"
    return None

def check_dump_carets_itemCount_lte_page_count(env: Mapping[str, Any]) -> str | None:
    raw = env.get('itemCount', None)
    total = env.get("pageCount")
    if raw is None or total is None:
        return None
    if isinstance(raw, bool) or isinstance(total, bool):
        return None
    if isinstance(raw, int) and isinstance(total, int) and raw > total:
        return "COORD_OOB"
    return None

def check_dump_carets_itemCount_lte_declared(env: Mapping[str, Any]) -> str | None:
    raw = env.get('itemCount', None)
    declared = env.get("declaredCount")
    if raw is None or declared is None:
        return None
    if isinstance(raw, bool) or isinstance(declared, bool):
        return None
    if isinstance(raw, int) and isinstance(declared, int) and raw > declared:
        return "COUNT_DRIFT"
    return None

def check_dump_carets_itemCount_under_cap(env: Mapping[str, Any]) -> str | None:
    raw = env.get('itemCount', None)
    cap = env.get("capBytes")
    if raw is None or cap is None:
        return None
    if isinstance(raw, bool) or isinstance(cap, bool):
        return None
    if isinstance(raw, int) and isinstance(cap, int) and cap > 0 and raw > cap:
        return "OVER_ACCEPTED"
    return None

def check_dump_carets_itemCount_year_range(env: Mapping[str, Any]) -> str | None:
    raw = env.get('itemCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < 1990 or raw > 2100:
        return "USAGE"
    return None

def check_dump_carets_itemCount_fail_vs_identical(env: Mapping[str, Any]) -> str | None:
    raw = env.get('itemCount', None)
    identical = env.get("identical")
    if raw is None or identical is not True:
        return None
    if isinstance(raw, int) and not isinstance(raw, bool) and raw > 0:
        return "COUNT_DRIFT"
    return None

def check_dump_carets_itemCount_ok_vs_total(env: Mapping[str, Any]) -> str | None:
    raw = env.get('itemCount', None)
    total = env.get("total")
    if raw is None or total is None:
        return None
    if isinstance(raw, bool) or isinstance(total, bool):
        return None
    if isinstance(raw, int) and isinstance(total, int) and raw > total:
        return "RATE_IMPOSSIBLE"
    return None

def check_dump_carets_itemCount_span_fits(env: Mapping[str, Any]) -> str | None:
    raw = env.get('itemCount', None)
    rows = env.get("rows")
    cols = env.get("cols")
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if isinstance(rows, int) and not isinstance(rows, bool) and 'itemCount' == "rowSpan" and raw > rows:
        return "SPAN_OOB"
    if isinstance(cols, int) and not isinstance(cols, bool) and 'itemCount' == "colSpan" and raw > cols:
        return "SPAN_OOB"
    return None

def check_dump_carets_itemCount_width_height(env: Mapping[str, Any]) -> str | None:
    raw = env.get('itemCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if 'itemCount' in ("width", "height", "bytes") and raw < 0:
        return "USAGE"
    return None

def check_dump_carets_itemCount_batch_parts(env: Mapping[str, Any]) -> str | None:
    raw = env.get('itemCount', None)
    n_in = env.get("inputN")
    n_ok = env.get("okN")
    n_fail = env.get("failN")
    if raw is None:
        return None
    if not all(isinstance(v, int) and not isinstance(v, bool) for v in (n_in, n_ok, n_fail) if v is not None):
        return None
    if isinstance(n_in, int) and isinstance(n_ok, int) and isinstance(n_fail, int):
        if n_in != n_ok + n_fail:
            return "COUNT_DRIFT"
    return None

def check_dump_carets_declaredCount_wrong_type_int(env: Mapping[str, Any]) -> str | None:
    raw = env.get('declaredCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return "USAGE"
    return None

def check_dump_carets_declaredCount_negative(env: Mapping[str, Any]) -> str | None:
    raw = env.get('declaredCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < 0:
        return "USAGE"
    return None

def check_dump_carets_declaredCount_nonzero_required(env: Mapping[str, Any]) -> str | None:
    raw = env.get('declaredCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw == 0:
        return "ZERO_BYTES"
    return None

def check_dump_carets_declaredCount_exit_closed(env: Mapping[str, Any]) -> str | None:
    raw = env.get('declaredCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw not in (0, 1, 2, 3, 4):
        return "EXIT_UNKNOWN"
    return None

def check_dump_carets_declaredCount_too_large(env: Mapping[str, Any]) -> str | None:
    raw = env.get('declaredCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw > 1_000_000_000:
        return "USAGE"
    return None

def check_dump_carets_declaredCount_fits_i32(env: Mapping[str, Any]) -> str | None:
    raw = env.get('declaredCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < -2147483648 or raw > 2147483647:
        return "USAGE"
    return None

def check_dump_carets_declaredCount_not_bool_int(env: Mapping[str, Any]) -> str | None:
    raw = env.get('declaredCount', None)
    if isinstance(raw, bool):
        return "USAGE"
    return None

def check_dump_carets_declaredCount_lte_page_count(env: Mapping[str, Any]) -> str | None:
    raw = env.get('declaredCount', None)
    total = env.get("pageCount")
    if raw is None or total is None:
        return None
    if isinstance(raw, bool) or isinstance(total, bool):
        return None
    if isinstance(raw, int) and isinstance(total, int) and raw > total:
        return "COORD_OOB"
    return None

def check_dump_carets_declaredCount_lte_declared(env: Mapping[str, Any]) -> str | None:
    raw = env.get('declaredCount', None)
    declared = env.get("declaredCount")
    if raw is None or declared is None:
        return None
    if isinstance(raw, bool) or isinstance(declared, bool):
        return None
    if isinstance(raw, int) and isinstance(declared, int) and raw > declared:
        return "COUNT_DRIFT"
    return None

def check_dump_carets_declaredCount_under_cap(env: Mapping[str, Any]) -> str | None:
    raw = env.get('declaredCount', None)
    cap = env.get("capBytes")
    if raw is None or cap is None:
        return None
    if isinstance(raw, bool) or isinstance(cap, bool):
        return None
    if isinstance(raw, int) and isinstance(cap, int) and cap > 0 and raw > cap:
        return "OVER_ACCEPTED"
    return None

def check_dump_carets_declaredCount_year_range(env: Mapping[str, Any]) -> str | None:
    raw = env.get('declaredCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < 1990 or raw > 2100:
        return "USAGE"
    return None

def check_dump_carets_declaredCount_fail_vs_identical(env: Mapping[str, Any]) -> str | None:
    raw = env.get('declaredCount', None)
    identical = env.get("identical")
    if raw is None or identical is not True:
        return None
    if isinstance(raw, int) and not isinstance(raw, bool) and raw > 0:
        return "COUNT_DRIFT"
    return None

def check_dump_carets_declaredCount_ok_vs_total(env: Mapping[str, Any]) -> str | None:
    raw = env.get('declaredCount', None)
    total = env.get("total")
    if raw is None or total is None:
        return None
    if isinstance(raw, bool) or isinstance(total, bool):
        return None
    if isinstance(raw, int) and isinstance(total, int) and raw > total:
        return "RATE_IMPOSSIBLE"
    return None

def check_dump_carets_declaredCount_span_fits(env: Mapping[str, Any]) -> str | None:
    raw = env.get('declaredCount', None)
    rows = env.get("rows")
    cols = env.get("cols")
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if isinstance(rows, int) and not isinstance(rows, bool) and 'declaredCount' == "rowSpan" and raw > rows:
        return "SPAN_OOB"
    if isinstance(cols, int) and not isinstance(cols, bool) and 'declaredCount' == "colSpan" and raw > cols:
        return "SPAN_OOB"
    return None

def check_dump_carets_declaredCount_width_height(env: Mapping[str, Any]) -> str | None:
    raw = env.get('declaredCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if 'declaredCount' in ("width", "height", "bytes") and raw < 0:
        return "USAGE"
    return None

def check_dump_carets_declaredCount_batch_parts(env: Mapping[str, Any]) -> str | None:
    raw = env.get('declaredCount', None)
    n_in = env.get("inputN")
    n_ok = env.get("okN")
    n_fail = env.get("failN")
    if raw is None:
        return None
    if not all(isinstance(v, int) and not isinstance(v, bool) for v in (n_in, n_ok, n_fail) if v is not None):
        return None
    if isinstance(n_in, int) and isinstance(n_ok, int) and isinstance(n_fail, int):
        if n_in != n_ok + n_fail:
            return "COUNT_DRIFT"
    return None

def check_dump_carets_arrayLen_wrong_type_int(env: Mapping[str, Any]) -> str | None:
    raw = env.get('arrayLen', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return "USAGE"
    return None

def check_dump_carets_arrayLen_negative(env: Mapping[str, Any]) -> str | None:
    raw = env.get('arrayLen', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < 0:
        return "USAGE"
    return None

def check_dump_carets_arrayLen_nonzero_required(env: Mapping[str, Any]) -> str | None:
    raw = env.get('arrayLen', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw == 0:
        return "ZERO_BYTES"
    return None

def check_dump_carets_arrayLen_exit_closed(env: Mapping[str, Any]) -> str | None:
    raw = env.get('arrayLen', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw not in (0, 1, 2, 3, 4):
        return "EXIT_UNKNOWN"
    return None

def check_dump_carets_arrayLen_too_large(env: Mapping[str, Any]) -> str | None:
    raw = env.get('arrayLen', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw > 1_000_000_000:
        return "USAGE"
    return None

def check_dump_carets_arrayLen_fits_i32(env: Mapping[str, Any]) -> str | None:
    raw = env.get('arrayLen', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < -2147483648 or raw > 2147483647:
        return "USAGE"
    return None

def check_dump_carets_arrayLen_not_bool_int(env: Mapping[str, Any]) -> str | None:
    raw = env.get('arrayLen', None)
    if isinstance(raw, bool):
        return "USAGE"
    return None

def check_dump_carets_arrayLen_lte_page_count(env: Mapping[str, Any]) -> str | None:
    raw = env.get('arrayLen', None)
    total = env.get("pageCount")
    if raw is None or total is None:
        return None
    if isinstance(raw, bool) or isinstance(total, bool):
        return None
    if isinstance(raw, int) and isinstance(total, int) and raw > total:
        return "COORD_OOB"
    return None

def check_dump_carets_arrayLen_lte_declared(env: Mapping[str, Any]) -> str | None:
    raw = env.get('arrayLen', None)
    declared = env.get("declaredCount")
    if raw is None or declared is None:
        return None
    if isinstance(raw, bool) or isinstance(declared, bool):
        return None
    if isinstance(raw, int) and isinstance(declared, int) and raw > declared:
        return "COUNT_DRIFT"
    return None

def check_dump_carets_arrayLen_under_cap(env: Mapping[str, Any]) -> str | None:
    raw = env.get('arrayLen', None)
    cap = env.get("capBytes")
    if raw is None or cap is None:
        return None
    if isinstance(raw, bool) or isinstance(cap, bool):
        return None
    if isinstance(raw, int) and isinstance(cap, int) and cap > 0 and raw > cap:
        return "OVER_ACCEPTED"
    return None

def check_dump_carets_arrayLen_year_range(env: Mapping[str, Any]) -> str | None:
    raw = env.get('arrayLen', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < 1990 or raw > 2100:
        return "USAGE"
    return None

def check_dump_carets_arrayLen_fail_vs_identical(env: Mapping[str, Any]) -> str | None:
    raw = env.get('arrayLen', None)
    identical = env.get("identical")
    if raw is None or identical is not True:
        return None
    if isinstance(raw, int) and not isinstance(raw, bool) and raw > 0:
        return "COUNT_DRIFT"
    return None

def check_dump_carets_arrayLen_ok_vs_total(env: Mapping[str, Any]) -> str | None:
    raw = env.get('arrayLen', None)
    total = env.get("total")
    if raw is None or total is None:
        return None
    if isinstance(raw, bool) or isinstance(total, bool):
        return None
    if isinstance(raw, int) and isinstance(total, int) and raw > total:
        return "RATE_IMPOSSIBLE"
    return None

def check_dump_carets_arrayLen_span_fits(env: Mapping[str, Any]) -> str | None:
    raw = env.get('arrayLen', None)
    rows = env.get("rows")
    cols = env.get("cols")
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if isinstance(rows, int) and not isinstance(rows, bool) and 'arrayLen' == "rowSpan" and raw > rows:
        return "SPAN_OOB"
    if isinstance(cols, int) and not isinstance(cols, bool) and 'arrayLen' == "colSpan" and raw > cols:
        return "SPAN_OOB"
    return None

def check_dump_carets_arrayLen_width_height(env: Mapping[str, Any]) -> str | None:
    raw = env.get('arrayLen', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if 'arrayLen' in ("width", "height", "bytes") and raw < 0:
        return "USAGE"
    return None

def check_dump_carets_arrayLen_batch_parts(env: Mapping[str, Any]) -> str | None:
    raw = env.get('arrayLen', None)
    n_in = env.get("inputN")
    n_ok = env.get("okN")
    n_fail = env.get("failN")
    if raw is None:
        return None
    if not all(isinstance(v, int) and not isinstance(v, bool) for v in (n_in, n_ok, n_fail) if v is not None):
        return None
    if isinstance(n_in, int) and isinstance(n_ok, int) and isinstance(n_fail, int):
        if n_in != n_ok + n_fail:
            return "COUNT_DRIFT"
    return None

def check_dump_carets_exitCode_wrong_type_int(env: Mapping[str, Any]) -> str | None:
    raw = env.get('exitCode', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return "USAGE"
    return None

def check_dump_carets_exitCode_negative(env: Mapping[str, Any]) -> str | None:
    raw = env.get('exitCode', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < 0:
        return "USAGE"
    return None

def check_dump_carets_exitCode_nonzero_required(env: Mapping[str, Any]) -> str | None:
    raw = env.get('exitCode', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw == 0:
        return "ZERO_BYTES"
    return None

def check_dump_carets_exitCode_exit_closed(env: Mapping[str, Any]) -> str | None:
    raw = env.get('exitCode', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw not in (0, 1, 2, 3, 4):
        return "EXIT_UNKNOWN"
    return None

def check_dump_carets_exitCode_too_large(env: Mapping[str, Any]) -> str | None:
    raw = env.get('exitCode', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw > 1_000_000_000:
        return "USAGE"
    return None

def check_dump_carets_exitCode_fits_i32(env: Mapping[str, Any]) -> str | None:
    raw = env.get('exitCode', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < -2147483648 or raw > 2147483647:
        return "USAGE"
    return None

def check_dump_carets_exitCode_not_bool_int(env: Mapping[str, Any]) -> str | None:
    raw = env.get('exitCode', None)
    if isinstance(raw, bool):
        return "USAGE"
    return None

def check_dump_carets_exitCode_lte_page_count(env: Mapping[str, Any]) -> str | None:
    raw = env.get('exitCode', None)
    total = env.get("pageCount")
    if raw is None or total is None:
        return None
    if isinstance(raw, bool) or isinstance(total, bool):
        return None
    if isinstance(raw, int) and isinstance(total, int) and raw > total:
        return "COORD_OOB"
    return None

def check_dump_carets_exitCode_lte_declared(env: Mapping[str, Any]) -> str | None:
    raw = env.get('exitCode', None)
    declared = env.get("declaredCount")
    if raw is None or declared is None:
        return None
    if isinstance(raw, bool) or isinstance(declared, bool):
        return None
    if isinstance(raw, int) and isinstance(declared, int) and raw > declared:
        return "COUNT_DRIFT"
    return None

def check_dump_carets_exitCode_under_cap(env: Mapping[str, Any]) -> str | None:
    raw = env.get('exitCode', None)
    cap = env.get("capBytes")
    if raw is None or cap is None:
        return None
    if isinstance(raw, bool) or isinstance(cap, bool):
        return None
    if isinstance(raw, int) and isinstance(cap, int) and cap > 0 and raw > cap:
        return "OVER_ACCEPTED"
    return None

def check_dump_carets_exitCode_year_range(env: Mapping[str, Any]) -> str | None:
    raw = env.get('exitCode', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < 1990 or raw > 2100:
        return "USAGE"
    return None

def check_dump_carets_exitCode_fail_vs_identical(env: Mapping[str, Any]) -> str | None:
    raw = env.get('exitCode', None)
    identical = env.get("identical")
    if raw is None or identical is not True:
        return None
    if isinstance(raw, int) and not isinstance(raw, bool) and raw > 0:
        return "COUNT_DRIFT"
    return None

def check_dump_carets_exitCode_ok_vs_total(env: Mapping[str, Any]) -> str | None:
    raw = env.get('exitCode', None)
    total = env.get("total")
    if raw is None or total is None:
        return None
    if isinstance(raw, bool) or isinstance(total, bool):
        return None
    if isinstance(raw, int) and isinstance(total, int) and raw > total:
        return "RATE_IMPOSSIBLE"
    return None

def check_dump_carets_exitCode_span_fits(env: Mapping[str, Any]) -> str | None:
    raw = env.get('exitCode', None)
    rows = env.get("rows")
    cols = env.get("cols")
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if isinstance(rows, int) and not isinstance(rows, bool) and 'exitCode' == "rowSpan" and raw > rows:
        return "SPAN_OOB"
    if isinstance(cols, int) and not isinstance(cols, bool) and 'exitCode' == "colSpan" and raw > cols:
        return "SPAN_OOB"
    return None

def check_dump_carets_exitCode_width_height(env: Mapping[str, Any]) -> str | None:
    raw = env.get('exitCode', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if 'exitCode' in ("width", "height", "bytes") and raw < 0:
        return "USAGE"
    return None

def check_dump_carets_exitCode_batch_parts(env: Mapping[str, Any]) -> str | None:
    raw = env.get('exitCode', None)
    n_in = env.get("inputN")
    n_ok = env.get("okN")
    n_fail = env.get("failN")
    if raw is None:
        return None
    if not all(isinstance(v, int) and not isinstance(v, bool) for v in (n_in, n_ok, n_fail) if v is not None):
        return None
    if isinstance(n_in, int) and isinstance(n_ok, int) and isinstance(n_fail, int):
        if n_in != n_ok + n_fail:
            return "COUNT_DRIFT"
    return None

def check_dump_carets_requestedPage_wrong_type_int(env: Mapping[str, Any]) -> str | None:
    raw = env.get('requestedPage', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return "USAGE"
    return None

def check_dump_carets_requestedPage_negative(env: Mapping[str, Any]) -> str | None:
    raw = env.get('requestedPage', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < 0:
        return "USAGE"
    return None

def check_dump_carets_requestedPage_nonzero_required(env: Mapping[str, Any]) -> str | None:
    raw = env.get('requestedPage', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw == 0:
        return "ZERO_BYTES"
    return None

def check_dump_carets_requestedPage_exit_closed(env: Mapping[str, Any]) -> str | None:
    raw = env.get('requestedPage', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw not in (0, 1, 2, 3, 4):
        return "EXIT_UNKNOWN"
    return None

def check_dump_carets_requestedPage_too_large(env: Mapping[str, Any]) -> str | None:
    raw = env.get('requestedPage', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw > 1_000_000_000:
        return "USAGE"
    return None

def check_dump_carets_requestedPage_fits_i32(env: Mapping[str, Any]) -> str | None:
    raw = env.get('requestedPage', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < -2147483648 or raw > 2147483647:
        return "USAGE"
    return None

def check_dump_carets_requestedPage_not_bool_int(env: Mapping[str, Any]) -> str | None:
    raw = env.get('requestedPage', None)
    if isinstance(raw, bool):
        return "USAGE"
    return None

def check_dump_carets_requestedPage_lte_page_count(env: Mapping[str, Any]) -> str | None:
    raw = env.get('requestedPage', None)
    total = env.get("pageCount")
    if raw is None or total is None:
        return None
    if isinstance(raw, bool) or isinstance(total, bool):
        return None
    if isinstance(raw, int) and isinstance(total, int) and raw > total:
        return "COORD_OOB"
    return None

def check_dump_carets_requestedPage_lte_declared(env: Mapping[str, Any]) -> str | None:
    raw = env.get('requestedPage', None)
    declared = env.get("declaredCount")
    if raw is None or declared is None:
        return None
    if isinstance(raw, bool) or isinstance(declared, bool):
        return None
    if isinstance(raw, int) and isinstance(declared, int) and raw > declared:
        return "COUNT_DRIFT"
    return None

def check_dump_carets_requestedPage_under_cap(env: Mapping[str, Any]) -> str | None:
    raw = env.get('requestedPage', None)
    cap = env.get("capBytes")
    if raw is None or cap is None:
        return None
    if isinstance(raw, bool) or isinstance(cap, bool):
        return None
    if isinstance(raw, int) and isinstance(cap, int) and cap > 0 and raw > cap:
        return "OVER_ACCEPTED"
    return None

def check_dump_carets_requestedPage_year_range(env: Mapping[str, Any]) -> str | None:
    raw = env.get('requestedPage', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < 1990 or raw > 2100:
        return "USAGE"
    return None

def check_dump_carets_requestedPage_fail_vs_identical(env: Mapping[str, Any]) -> str | None:
    raw = env.get('requestedPage', None)
    identical = env.get("identical")
    if raw is None or identical is not True:
        return None
    if isinstance(raw, int) and not isinstance(raw, bool) and raw > 0:
        return "COUNT_DRIFT"
    return None

def check_dump_carets_requestedPage_ok_vs_total(env: Mapping[str, Any]) -> str | None:
    raw = env.get('requestedPage', None)
    total = env.get("total")
    if raw is None or total is None:
        return None
    if isinstance(raw, bool) or isinstance(total, bool):
        return None
    if isinstance(raw, int) and isinstance(total, int) and raw > total:
        return "RATE_IMPOSSIBLE"
    return None

def check_dump_carets_requestedPage_span_fits(env: Mapping[str, Any]) -> str | None:
    raw = env.get('requestedPage', None)
    rows = env.get("rows")
    cols = env.get("cols")
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if isinstance(rows, int) and not isinstance(rows, bool) and 'requestedPage' == "rowSpan" and raw > rows:
        return "SPAN_OOB"
    if isinstance(cols, int) and not isinstance(cols, bool) and 'requestedPage' == "colSpan" and raw > cols:
        return "SPAN_OOB"
    return None

def check_dump_carets_requestedPage_width_height(env: Mapping[str, Any]) -> str | None:
    raw = env.get('requestedPage', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if 'requestedPage' in ("width", "height", "bytes") and raw < 0:
        return "USAGE"
    return None

def check_dump_carets_requestedPage_batch_parts(env: Mapping[str, Any]) -> str | None:
    raw = env.get('requestedPage', None)
    n_in = env.get("inputN")
    n_ok = env.get("okN")
    n_fail = env.get("failN")
    if raw is None:
        return None
    if not all(isinstance(v, int) and not isinstance(v, bool) for v in (n_in, n_ok, n_fail) if v is not None):
        return None
    if isinstance(n_in, int) and isinstance(n_ok, int) and isinstance(n_fail, int):
        if n_in != n_ok + n_fail:
            return "COUNT_DRIFT"
    return None

def check_dump_carets_emittedCount_wrong_type_int(env: Mapping[str, Any]) -> str | None:
    raw = env.get('emittedCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return "USAGE"
    return None

def check_dump_carets_emittedCount_negative(env: Mapping[str, Any]) -> str | None:
    raw = env.get('emittedCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < 0:
        return "USAGE"
    return None

def check_dump_carets_emittedCount_nonzero_required(env: Mapping[str, Any]) -> str | None:
    raw = env.get('emittedCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw == 0:
        return "ZERO_BYTES"
    return None

def check_dump_carets_emittedCount_exit_closed(env: Mapping[str, Any]) -> str | None:
    raw = env.get('emittedCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw not in (0, 1, 2, 3, 4):
        return "EXIT_UNKNOWN"
    return None

def check_dump_carets_emittedCount_too_large(env: Mapping[str, Any]) -> str | None:
    raw = env.get('emittedCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw > 1_000_000_000:
        return "USAGE"
    return None

def check_dump_carets_emittedCount_fits_i32(env: Mapping[str, Any]) -> str | None:
    raw = env.get('emittedCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < -2147483648 or raw > 2147483647:
        return "USAGE"
    return None

def check_dump_carets_emittedCount_not_bool_int(env: Mapping[str, Any]) -> str | None:
    raw = env.get('emittedCount', None)
    if isinstance(raw, bool):
        return "USAGE"
    return None

def check_dump_carets_emittedCount_lte_page_count(env: Mapping[str, Any]) -> str | None:
    raw = env.get('emittedCount', None)
    total = env.get("pageCount")
    if raw is None or total is None:
        return None
    if isinstance(raw, bool) or isinstance(total, bool):
        return None
    if isinstance(raw, int) and isinstance(total, int) and raw > total:
        return "COORD_OOB"
    return None

def check_dump_carets_emittedCount_lte_declared(env: Mapping[str, Any]) -> str | None:
    raw = env.get('emittedCount', None)
    declared = env.get("declaredCount")
    if raw is None or declared is None:
        return None
    if isinstance(raw, bool) or isinstance(declared, bool):
        return None
    if isinstance(raw, int) and isinstance(declared, int) and raw > declared:
        return "COUNT_DRIFT"
    return None

def check_dump_carets_emittedCount_under_cap(env: Mapping[str, Any]) -> str | None:
    raw = env.get('emittedCount', None)
    cap = env.get("capBytes")
    if raw is None or cap is None:
        return None
    if isinstance(raw, bool) or isinstance(cap, bool):
        return None
    if isinstance(raw, int) and isinstance(cap, int) and cap > 0 and raw > cap:
        return "OVER_ACCEPTED"
    return None

def check_dump_carets_emittedCount_year_range(env: Mapping[str, Any]) -> str | None:
    raw = env.get('emittedCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < 1990 or raw > 2100:
        return "USAGE"
    return None

def check_dump_carets_emittedCount_fail_vs_identical(env: Mapping[str, Any]) -> str | None:
    raw = env.get('emittedCount', None)
    identical = env.get("identical")
    if raw is None or identical is not True:
        return None
    if isinstance(raw, int) and not isinstance(raw, bool) and raw > 0:
        return "COUNT_DRIFT"
    return None

def check_dump_carets_emittedCount_ok_vs_total(env: Mapping[str, Any]) -> str | None:
    raw = env.get('emittedCount', None)
    total = env.get("total")
    if raw is None or total is None:
        return None
    if isinstance(raw, bool) or isinstance(total, bool):
        return None
    if isinstance(raw, int) and isinstance(total, int) and raw > total:
        return "RATE_IMPOSSIBLE"
    return None

def check_dump_carets_emittedCount_span_fits(env: Mapping[str, Any]) -> str | None:
    raw = env.get('emittedCount', None)
    rows = env.get("rows")
    cols = env.get("cols")
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if isinstance(rows, int) and not isinstance(rows, bool) and 'emittedCount' == "rowSpan" and raw > rows:
        return "SPAN_OOB"
    if isinstance(cols, int) and not isinstance(cols, bool) and 'emittedCount' == "colSpan" and raw > cols:
        return "SPAN_OOB"
    return None

def check_dump_carets_emittedCount_width_height(env: Mapping[str, Any]) -> str | None:
    raw = env.get('emittedCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if 'emittedCount' in ("width", "height", "bytes") and raw < 0:
        return "USAGE"
    return None

def check_dump_carets_emittedCount_batch_parts(env: Mapping[str, Any]) -> str | None:
    raw = env.get('emittedCount', None)
    n_in = env.get("inputN")
    n_ok = env.get("okN")
    n_fail = env.get("failN")
    if raw is None:
        return None
    if not all(isinstance(v, int) and not isinstance(v, bool) for v in (n_in, n_ok, n_fail) if v is not None):
        return None
    if isinstance(n_in, int) and isinstance(n_ok, int) and isinstance(n_fail, int):
        if n_in != n_ok + n_fail:
            return "COUNT_DRIFT"
    return None

def check_dump_carets_maxChars_wrong_type_int(env: Mapping[str, Any]) -> str | None:
    raw = env.get('maxChars', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return "USAGE"
    return None

def check_dump_carets_maxChars_negative(env: Mapping[str, Any]) -> str | None:
    raw = env.get('maxChars', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < 0:
        return "USAGE"
    return None

def check_dump_carets_maxChars_nonzero_required(env: Mapping[str, Any]) -> str | None:
    raw = env.get('maxChars', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw == 0:
        return "ZERO_BYTES"
    return None

def check_dump_carets_maxChars_exit_closed(env: Mapping[str, Any]) -> str | None:
    raw = env.get('maxChars', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw not in (0, 1, 2, 3, 4):
        return "EXIT_UNKNOWN"
    return None

def check_dump_carets_maxChars_too_large(env: Mapping[str, Any]) -> str | None:
    raw = env.get('maxChars', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw > 1_000_000_000:
        return "USAGE"
    return None

def check_dump_carets_maxChars_fits_i32(env: Mapping[str, Any]) -> str | None:
    raw = env.get('maxChars', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < -2147483648 or raw > 2147483647:
        return "USAGE"
    return None

def check_dump_carets_maxChars_not_bool_int(env: Mapping[str, Any]) -> str | None:
    raw = env.get('maxChars', None)
    if isinstance(raw, bool):
        return "USAGE"
    return None

def check_dump_carets_maxChars_lte_page_count(env: Mapping[str, Any]) -> str | None:
    raw = env.get('maxChars', None)
    total = env.get("pageCount")
    if raw is None or total is None:
        return None
    if isinstance(raw, bool) or isinstance(total, bool):
        return None
    if isinstance(raw, int) and isinstance(total, int) and raw > total:
        return "COORD_OOB"
    return None

def check_dump_carets_maxChars_lte_declared(env: Mapping[str, Any]) -> str | None:
    raw = env.get('maxChars', None)
    declared = env.get("declaredCount")
    if raw is None or declared is None:
        return None
    if isinstance(raw, bool) or isinstance(declared, bool):
        return None
    if isinstance(raw, int) and isinstance(declared, int) and raw > declared:
        return "COUNT_DRIFT"
    return None

def check_dump_carets_maxChars_under_cap(env: Mapping[str, Any]) -> str | None:
    raw = env.get('maxChars', None)
    cap = env.get("capBytes")
    if raw is None or cap is None:
        return None
    if isinstance(raw, bool) or isinstance(cap, bool):
        return None
    if isinstance(raw, int) and isinstance(cap, int) and cap > 0 and raw > cap:
        return "OVER_ACCEPTED"
    return None

def check_dump_carets_maxChars_year_range(env: Mapping[str, Any]) -> str | None:
    raw = env.get('maxChars', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < 1990 or raw > 2100:
        return "USAGE"
    return None

def check_dump_carets_maxChars_fail_vs_identical(env: Mapping[str, Any]) -> str | None:
    raw = env.get('maxChars', None)
    identical = env.get("identical")
    if raw is None or identical is not True:
        return None
    if isinstance(raw, int) and not isinstance(raw, bool) and raw > 0:
        return "COUNT_DRIFT"
    return None

def check_dump_carets_maxChars_ok_vs_total(env: Mapping[str, Any]) -> str | None:
    raw = env.get('maxChars', None)
    total = env.get("total")
    if raw is None or total is None:
        return None
    if isinstance(raw, bool) or isinstance(total, bool):
        return None
    if isinstance(raw, int) and isinstance(total, int) and raw > total:
        return "RATE_IMPOSSIBLE"
    return None

def check_dump_carets_maxChars_span_fits(env: Mapping[str, Any]) -> str | None:
    raw = env.get('maxChars', None)
    rows = env.get("rows")
    cols = env.get("cols")
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if isinstance(rows, int) and not isinstance(rows, bool) and 'maxChars' == "rowSpan" and raw > rows:
        return "SPAN_OOB"
    if isinstance(cols, int) and not isinstance(cols, bool) and 'maxChars' == "colSpan" and raw > cols:
        return "SPAN_OOB"
    return None

def check_dump_carets_maxChars_width_height(env: Mapping[str, Any]) -> str | None:
    raw = env.get('maxChars', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if 'maxChars' in ("width", "height", "bytes") and raw < 0:
        return "USAGE"
    return None

def check_dump_carets_maxChars_batch_parts(env: Mapping[str, Any]) -> str | None:
    raw = env.get('maxChars', None)
    n_in = env.get("inputN")
    n_ok = env.get("okN")
    n_fail = env.get("failN")
    if raw is None:
        return None
    if not all(isinstance(v, int) and not isinstance(v, bool) for v in (n_in, n_ok, n_fail) if v is not None):
        return None
    if isinstance(n_in, int) and isinstance(n_ok, int) and isinstance(n_fail, int):
        if n_in != n_ok + n_fail:
            return "COUNT_DRIFT"
    return None

def check_dump_carets_textLen_wrong_type_int(env: Mapping[str, Any]) -> str | None:
    raw = env.get('textLen', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return "USAGE"
    return None

def check_dump_carets_textLen_negative(env: Mapping[str, Any]) -> str | None:
    raw = env.get('textLen', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < 0:
        return "USAGE"
    return None

def check_dump_carets_textLen_nonzero_required(env: Mapping[str, Any]) -> str | None:
    raw = env.get('textLen', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw == 0:
        return "ZERO_BYTES"
    return None

def check_dump_carets_textLen_exit_closed(env: Mapping[str, Any]) -> str | None:
    raw = env.get('textLen', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw not in (0, 1, 2, 3, 4):
        return "EXIT_UNKNOWN"
    return None

def check_dump_carets_textLen_too_large(env: Mapping[str, Any]) -> str | None:
    raw = env.get('textLen', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw > 1_000_000_000:
        return "USAGE"
    return None

def check_dump_carets_textLen_fits_i32(env: Mapping[str, Any]) -> str | None:
    raw = env.get('textLen', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < -2147483648 or raw > 2147483647:
        return "USAGE"
    return None

def check_dump_carets_textLen_not_bool_int(env: Mapping[str, Any]) -> str | None:
    raw = env.get('textLen', None)
    if isinstance(raw, bool):
        return "USAGE"
    return None

def check_dump_carets_textLen_lte_page_count(env: Mapping[str, Any]) -> str | None:
    raw = env.get('textLen', None)
    total = env.get("pageCount")
    if raw is None or total is None:
        return None
    if isinstance(raw, bool) or isinstance(total, bool):
        return None
    if isinstance(raw, int) and isinstance(total, int) and raw > total:
        return "COORD_OOB"
    return None

def check_dump_carets_textLen_lte_declared(env: Mapping[str, Any]) -> str | None:
    raw = env.get('textLen', None)
    declared = env.get("declaredCount")
    if raw is None or declared is None:
        return None
    if isinstance(raw, bool) or isinstance(declared, bool):
        return None
    if isinstance(raw, int) and isinstance(declared, int) and raw > declared:
        return "COUNT_DRIFT"
    return None

def check_dump_carets_textLen_under_cap(env: Mapping[str, Any]) -> str | None:
    raw = env.get('textLen', None)
    cap = env.get("capBytes")
    if raw is None or cap is None:
        return None
    if isinstance(raw, bool) or isinstance(cap, bool):
        return None
    if isinstance(raw, int) and isinstance(cap, int) and cap > 0 and raw > cap:
        return "OVER_ACCEPTED"
    return None

def check_dump_carets_textLen_year_range(env: Mapping[str, Any]) -> str | None:
    raw = env.get('textLen', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < 1990 or raw > 2100:
        return "USAGE"
    return None

def check_dump_carets_textLen_fail_vs_identical(env: Mapping[str, Any]) -> str | None:
    raw = env.get('textLen', None)
    identical = env.get("identical")
    if raw is None or identical is not True:
        return None
    if isinstance(raw, int) and not isinstance(raw, bool) and raw > 0:
        return "COUNT_DRIFT"
    return None

def check_dump_carets_textLen_ok_vs_total(env: Mapping[str, Any]) -> str | None:
    raw = env.get('textLen', None)
    total = env.get("total")
    if raw is None or total is None:
        return None
    if isinstance(raw, bool) or isinstance(total, bool):
        return None
    if isinstance(raw, int) and isinstance(total, int) and raw > total:
        return "RATE_IMPOSSIBLE"
    return None

def check_dump_carets_textLen_span_fits(env: Mapping[str, Any]) -> str | None:
    raw = env.get('textLen', None)
    rows = env.get("rows")
    cols = env.get("cols")
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if isinstance(rows, int) and not isinstance(rows, bool) and 'textLen' == "rowSpan" and raw > rows:
        return "SPAN_OOB"
    if isinstance(cols, int) and not isinstance(cols, bool) and 'textLen' == "colSpan" and raw > cols:
        return "SPAN_OOB"
    return None

def check_dump_carets_textLen_width_height(env: Mapping[str, Any]) -> str | None:
    raw = env.get('textLen', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if 'textLen' in ("width", "height", "bytes") and raw < 0:
        return "USAGE"
    return None

def check_dump_carets_textLen_batch_parts(env: Mapping[str, Any]) -> str | None:
    raw = env.get('textLen', None)
    n_in = env.get("inputN")
    n_ok = env.get("okN")
    n_fail = env.get("failN")
    if raw is None:
        return None
    if not all(isinstance(v, int) and not isinstance(v, bool) for v in (n_in, n_ok, n_fail) if v is not None):
        return None
    if isinstance(n_in, int) and isinstance(n_ok, int) and isinstance(n_fail, int):
        if n_in != n_ok + n_fail:
            return "COUNT_DRIFT"
    return None

def check_dump_carets_rows_wrong_type_int(env: Mapping[str, Any]) -> str | None:
    raw = env.get('rows', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return "USAGE"
    return None

def check_dump_carets_rows_negative(env: Mapping[str, Any]) -> str | None:
    raw = env.get('rows', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < 0:
        return "USAGE"
    return None

def check_dump_carets_rows_nonzero_required(env: Mapping[str, Any]) -> str | None:
    raw = env.get('rows', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw == 0:
        return "ZERO_BYTES"
    return None

def check_dump_carets_rows_exit_closed(env: Mapping[str, Any]) -> str | None:
    raw = env.get('rows', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw not in (0, 1, 2, 3, 4):
        return "EXIT_UNKNOWN"
    return None

def check_dump_carets_rows_too_large(env: Mapping[str, Any]) -> str | None:
    raw = env.get('rows', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw > 1_000_000_000:
        return "USAGE"
    return None

def check_dump_carets_rows_fits_i32(env: Mapping[str, Any]) -> str | None:
    raw = env.get('rows', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < -2147483648 or raw > 2147483647:
        return "USAGE"
    return None

def check_dump_carets_rows_not_bool_int(env: Mapping[str, Any]) -> str | None:
    raw = env.get('rows', None)
    if isinstance(raw, bool):
        return "USAGE"
    return None

def check_dump_carets_rows_lte_page_count(env: Mapping[str, Any]) -> str | None:
    raw = env.get('rows', None)
    total = env.get("pageCount")
    if raw is None or total is None:
        return None
    if isinstance(raw, bool) or isinstance(total, bool):
        return None
    if isinstance(raw, int) and isinstance(total, int) and raw > total:
        return "COORD_OOB"
    return None

def check_dump_carets_rows_lte_declared(env: Mapping[str, Any]) -> str | None:
    raw = env.get('rows', None)
    declared = env.get("declaredCount")
    if raw is None or declared is None:
        return None
    if isinstance(raw, bool) or isinstance(declared, bool):
        return None
    if isinstance(raw, int) and isinstance(declared, int) and raw > declared:
        return "COUNT_DRIFT"
    return None

def check_dump_carets_rows_under_cap(env: Mapping[str, Any]) -> str | None:
    raw = env.get('rows', None)
    cap = env.get("capBytes")
    if raw is None or cap is None:
        return None
    if isinstance(raw, bool) or isinstance(cap, bool):
        return None
    if isinstance(raw, int) and isinstance(cap, int) and cap > 0 and raw > cap:
        return "OVER_ACCEPTED"
    return None

def check_dump_carets_rows_year_range(env: Mapping[str, Any]) -> str | None:
    raw = env.get('rows', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < 1990 or raw > 2100:
        return "USAGE"
    return None

def check_dump_carets_rows_fail_vs_identical(env: Mapping[str, Any]) -> str | None:
    raw = env.get('rows', None)
    identical = env.get("identical")
    if raw is None or identical is not True:
        return None
    if isinstance(raw, int) and not isinstance(raw, bool) and raw > 0:
        return "COUNT_DRIFT"
    return None

def check_dump_carets_rows_ok_vs_total(env: Mapping[str, Any]) -> str | None:
    raw = env.get('rows', None)
    total = env.get("total")
    if raw is None or total is None:
        return None
    if isinstance(raw, bool) or isinstance(total, bool):
        return None
    if isinstance(raw, int) and isinstance(total, int) and raw > total:
        return "RATE_IMPOSSIBLE"
    return None

def check_dump_carets_rows_span_fits(env: Mapping[str, Any]) -> str | None:
    raw = env.get('rows', None)
    rows = env.get("rows")
    cols = env.get("cols")
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if isinstance(rows, int) and not isinstance(rows, bool) and 'rows' == "rowSpan" and raw > rows:
        return "SPAN_OOB"
    if isinstance(cols, int) and not isinstance(cols, bool) and 'rows' == "colSpan" and raw > cols:
        return "SPAN_OOB"
    return None

def check_dump_carets_rows_width_height(env: Mapping[str, Any]) -> str | None:
    raw = env.get('rows', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if 'rows' in ("width", "height", "bytes") and raw < 0:
        return "USAGE"
    return None

def check_dump_carets_rows_batch_parts(env: Mapping[str, Any]) -> str | None:
    raw = env.get('rows', None)
    n_in = env.get("inputN")
    n_ok = env.get("okN")
    n_fail = env.get("failN")
    if raw is None:
        return None
    if not all(isinstance(v, int) and not isinstance(v, bool) for v in (n_in, n_ok, n_fail) if v is not None):
        return None
    if isinstance(n_in, int) and isinstance(n_ok, int) and isinstance(n_fail, int):
        if n_in != n_ok + n_fail:
            return "COUNT_DRIFT"
    return None

def check_dump_carets_cols_wrong_type_int(env: Mapping[str, Any]) -> str | None:
    raw = env.get('cols', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return "USAGE"
    return None

def check_dump_carets_cols_negative(env: Mapping[str, Any]) -> str | None:
    raw = env.get('cols', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < 0:
        return "USAGE"
    return None

def check_dump_carets_cols_nonzero_required(env: Mapping[str, Any]) -> str | None:
    raw = env.get('cols', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw == 0:
        return "ZERO_BYTES"
    return None

def check_dump_carets_cols_exit_closed(env: Mapping[str, Any]) -> str | None:
    raw = env.get('cols', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw not in (0, 1, 2, 3, 4):
        return "EXIT_UNKNOWN"
    return None

def check_dump_carets_cols_too_large(env: Mapping[str, Any]) -> str | None:
    raw = env.get('cols', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw > 1_000_000_000:
        return "USAGE"
    return None

def check_dump_carets_cols_fits_i32(env: Mapping[str, Any]) -> str | None:
    raw = env.get('cols', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < -2147483648 or raw > 2147483647:
        return "USAGE"
    return None

def check_dump_carets_cols_not_bool_int(env: Mapping[str, Any]) -> str | None:
    raw = env.get('cols', None)
    if isinstance(raw, bool):
        return "USAGE"
    return None

def check_dump_carets_cols_lte_page_count(env: Mapping[str, Any]) -> str | None:
    raw = env.get('cols', None)
    total = env.get("pageCount")
    if raw is None or total is None:
        return None
    if isinstance(raw, bool) or isinstance(total, bool):
        return None
    if isinstance(raw, int) and isinstance(total, int) and raw > total:
        return "COORD_OOB"
    return None

def check_dump_carets_cols_lte_declared(env: Mapping[str, Any]) -> str | None:
    raw = env.get('cols', None)
    declared = env.get("declaredCount")
    if raw is None or declared is None:
        return None
    if isinstance(raw, bool) or isinstance(declared, bool):
        return None
    if isinstance(raw, int) and isinstance(declared, int) and raw > declared:
        return "COUNT_DRIFT"
    return None

def check_dump_carets_cols_under_cap(env: Mapping[str, Any]) -> str | None:
    raw = env.get('cols', None)
    cap = env.get("capBytes")
    if raw is None or cap is None:
        return None
    if isinstance(raw, bool) or isinstance(cap, bool):
        return None
    if isinstance(raw, int) and isinstance(cap, int) and cap > 0 and raw > cap:
        return "OVER_ACCEPTED"
    return None

def check_dump_carets_cols_year_range(env: Mapping[str, Any]) -> str | None:
    raw = env.get('cols', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < 1990 or raw > 2100:
        return "USAGE"
    return None

def check_dump_carets_cols_fail_vs_identical(env: Mapping[str, Any]) -> str | None:
    raw = env.get('cols', None)
    identical = env.get("identical")
    if raw is None or identical is not True:
        return None
    if isinstance(raw, int) and not isinstance(raw, bool) and raw > 0:
        return "COUNT_DRIFT"
    return None

def check_dump_carets_cols_ok_vs_total(env: Mapping[str, Any]) -> str | None:
    raw = env.get('cols', None)
    total = env.get("total")
    if raw is None or total is None:
        return None
    if isinstance(raw, bool) or isinstance(total, bool):
        return None
    if isinstance(raw, int) and isinstance(total, int) and raw > total:
        return "RATE_IMPOSSIBLE"
    return None

def check_dump_carets_cols_span_fits(env: Mapping[str, Any]) -> str | None:
    raw = env.get('cols', None)
    rows = env.get("rows")
    cols = env.get("cols")
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if isinstance(rows, int) and not isinstance(rows, bool) and 'cols' == "rowSpan" and raw > rows:
        return "SPAN_OOB"
    if isinstance(cols, int) and not isinstance(cols, bool) and 'cols' == "colSpan" and raw > cols:
        return "SPAN_OOB"
    return None

def check_dump_carets_cols_width_height(env: Mapping[str, Any]) -> str | None:
    raw = env.get('cols', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if 'cols' in ("width", "height", "bytes") and raw < 0:
        return "USAGE"
    return None

def check_dump_carets_cols_batch_parts(env: Mapping[str, Any]) -> str | None:
    raw = env.get('cols', None)
    n_in = env.get("inputN")
    n_ok = env.get("okN")
    n_fail = env.get("failN")
    if raw is None:
        return None
    if not all(isinstance(v, int) and not isinstance(v, bool) for v in (n_in, n_ok, n_fail) if v is not None):
        return None
    if isinstance(n_in, int) and isinstance(n_ok, int) and isinstance(n_fail, int):
        if n_in != n_ok + n_fail:
            return "COUNT_DRIFT"
    return None

def check_dump_carets_rowSpan_wrong_type_int(env: Mapping[str, Any]) -> str | None:
    raw = env.get('rowSpan', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return "USAGE"
    return None

def check_dump_carets_rowSpan_negative(env: Mapping[str, Any]) -> str | None:
    raw = env.get('rowSpan', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < 0:
        return "USAGE"
    return None

def check_dump_carets_rowSpan_nonzero_required(env: Mapping[str, Any]) -> str | None:
    raw = env.get('rowSpan', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw == 0:
        return "ZERO_BYTES"
    return None

def check_dump_carets_rowSpan_exit_closed(env: Mapping[str, Any]) -> str | None:
    raw = env.get('rowSpan', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw not in (0, 1, 2, 3, 4):
        return "EXIT_UNKNOWN"
    return None

def check_dump_carets_rowSpan_too_large(env: Mapping[str, Any]) -> str | None:
    raw = env.get('rowSpan', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw > 1_000_000_000:
        return "USAGE"
    return None

def check_dump_carets_rowSpan_fits_i32(env: Mapping[str, Any]) -> str | None:
    raw = env.get('rowSpan', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < -2147483648 or raw > 2147483647:
        return "USAGE"
    return None

def check_dump_carets_rowSpan_not_bool_int(env: Mapping[str, Any]) -> str | None:
    raw = env.get('rowSpan', None)
    if isinstance(raw, bool):
        return "USAGE"
    return None

def check_dump_carets_rowSpan_lte_page_count(env: Mapping[str, Any]) -> str | None:
    raw = env.get('rowSpan', None)
    total = env.get("pageCount")
    if raw is None or total is None:
        return None
    if isinstance(raw, bool) or isinstance(total, bool):
        return None
    if isinstance(raw, int) and isinstance(total, int) and raw > total:
        return "COORD_OOB"
    return None

def check_dump_carets_rowSpan_lte_declared(env: Mapping[str, Any]) -> str | None:
    raw = env.get('rowSpan', None)
    declared = env.get("declaredCount")
    if raw is None or declared is None:
        return None
    if isinstance(raw, bool) or isinstance(declared, bool):
        return None
    if isinstance(raw, int) and isinstance(declared, int) and raw > declared:
        return "COUNT_DRIFT"
    return None

def check_dump_carets_rowSpan_under_cap(env: Mapping[str, Any]) -> str | None:
    raw = env.get('rowSpan', None)
    cap = env.get("capBytes")
    if raw is None or cap is None:
        return None
    if isinstance(raw, bool) or isinstance(cap, bool):
        return None
    if isinstance(raw, int) and isinstance(cap, int) and cap > 0 and raw > cap:
        return "OVER_ACCEPTED"
    return None

def check_dump_carets_rowSpan_year_range(env: Mapping[str, Any]) -> str | None:
    raw = env.get('rowSpan', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < 1990 or raw > 2100:
        return "USAGE"
    return None

def check_dump_carets_rowSpan_fail_vs_identical(env: Mapping[str, Any]) -> str | None:
    raw = env.get('rowSpan', None)
    identical = env.get("identical")
    if raw is None or identical is not True:
        return None
    if isinstance(raw, int) and not isinstance(raw, bool) and raw > 0:
        return "COUNT_DRIFT"
    return None

def check_dump_carets_rowSpan_ok_vs_total(env: Mapping[str, Any]) -> str | None:
    raw = env.get('rowSpan', None)
    total = env.get("total")
    if raw is None or total is None:
        return None
    if isinstance(raw, bool) or isinstance(total, bool):
        return None
    if isinstance(raw, int) and isinstance(total, int) and raw > total:
        return "RATE_IMPOSSIBLE"
    return None

def check_dump_carets_rowSpan_span_fits(env: Mapping[str, Any]) -> str | None:
    raw = env.get('rowSpan', None)
    rows = env.get("rows")
    cols = env.get("cols")
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if isinstance(rows, int) and not isinstance(rows, bool) and 'rowSpan' == "rowSpan" and raw > rows:
        return "SPAN_OOB"
    if isinstance(cols, int) and not isinstance(cols, bool) and 'rowSpan' == "colSpan" and raw > cols:
        return "SPAN_OOB"
    return None

def check_dump_carets_rowSpan_width_height(env: Mapping[str, Any]) -> str | None:
    raw = env.get('rowSpan', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if 'rowSpan' in ("width", "height", "bytes") and raw < 0:
        return "USAGE"
    return None

def check_dump_carets_rowSpan_batch_parts(env: Mapping[str, Any]) -> str | None:
    raw = env.get('rowSpan', None)
    n_in = env.get("inputN")
    n_ok = env.get("okN")
    n_fail = env.get("failN")
    if raw is None:
        return None
    if not all(isinstance(v, int) and not isinstance(v, bool) for v in (n_in, n_ok, n_fail) if v is not None):
        return None
    if isinstance(n_in, int) and isinstance(n_ok, int) and isinstance(n_fail, int):
        if n_in != n_ok + n_fail:
            return "COUNT_DRIFT"
    return None

def check_dump_carets_colSpan_wrong_type_int(env: Mapping[str, Any]) -> str | None:
    raw = env.get('colSpan', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return "USAGE"
    return None

def check_dump_carets_colSpan_negative(env: Mapping[str, Any]) -> str | None:
    raw = env.get('colSpan', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < 0:
        return "USAGE"
    return None

def check_dump_carets_colSpan_nonzero_required(env: Mapping[str, Any]) -> str | None:
    raw = env.get('colSpan', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw == 0:
        return "ZERO_BYTES"
    return None

def check_dump_carets_colSpan_exit_closed(env: Mapping[str, Any]) -> str | None:
    raw = env.get('colSpan', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw not in (0, 1, 2, 3, 4):
        return "EXIT_UNKNOWN"
    return None

def check_dump_carets_colSpan_too_large(env: Mapping[str, Any]) -> str | None:
    raw = env.get('colSpan', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw > 1_000_000_000:
        return "USAGE"
    return None

def check_dump_carets_colSpan_fits_i32(env: Mapping[str, Any]) -> str | None:
    raw = env.get('colSpan', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < -2147483648 or raw > 2147483647:
        return "USAGE"
    return None

def check_dump_carets_colSpan_not_bool_int(env: Mapping[str, Any]) -> str | None:
    raw = env.get('colSpan', None)
    if isinstance(raw, bool):
        return "USAGE"
    return None

def check_dump_carets_colSpan_lte_page_count(env: Mapping[str, Any]) -> str | None:
    raw = env.get('colSpan', None)
    total = env.get("pageCount")
    if raw is None or total is None:
        return None
    if isinstance(raw, bool) or isinstance(total, bool):
        return None
    if isinstance(raw, int) and isinstance(total, int) and raw > total:
        return "COORD_OOB"
    return None

def check_dump_carets_colSpan_lte_declared(env: Mapping[str, Any]) -> str | None:
    raw = env.get('colSpan', None)
    declared = env.get("declaredCount")
    if raw is None or declared is None:
        return None
    if isinstance(raw, bool) or isinstance(declared, bool):
        return None
    if isinstance(raw, int) and isinstance(declared, int) and raw > declared:
        return "COUNT_DRIFT"
    return None

def check_dump_carets_colSpan_under_cap(env: Mapping[str, Any]) -> str | None:
    raw = env.get('colSpan', None)
    cap = env.get("capBytes")
    if raw is None or cap is None:
        return None
    if isinstance(raw, bool) or isinstance(cap, bool):
        return None
    if isinstance(raw, int) and isinstance(cap, int) and cap > 0 and raw > cap:
        return "OVER_ACCEPTED"
    return None

def check_dump_carets_colSpan_year_range(env: Mapping[str, Any]) -> str | None:
    raw = env.get('colSpan', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < 1990 or raw > 2100:
        return "USAGE"
    return None

def check_dump_carets_colSpan_fail_vs_identical(env: Mapping[str, Any]) -> str | None:
    raw = env.get('colSpan', None)
    identical = env.get("identical")
    if raw is None or identical is not True:
        return None
    if isinstance(raw, int) and not isinstance(raw, bool) and raw > 0:
        return "COUNT_DRIFT"
    return None

def check_dump_carets_colSpan_ok_vs_total(env: Mapping[str, Any]) -> str | None:
    raw = env.get('colSpan', None)
    total = env.get("total")
    if raw is None or total is None:
        return None
    if isinstance(raw, bool) or isinstance(total, bool):
        return None
    if isinstance(raw, int) and isinstance(total, int) and raw > total:
        return "RATE_IMPOSSIBLE"
    return None

def check_dump_carets_colSpan_span_fits(env: Mapping[str, Any]) -> str | None:
    raw = env.get('colSpan', None)
    rows = env.get("rows")
    cols = env.get("cols")
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if isinstance(rows, int) and not isinstance(rows, bool) and 'colSpan' == "rowSpan" and raw > rows:
        return "SPAN_OOB"
    if isinstance(cols, int) and not isinstance(cols, bool) and 'colSpan' == "colSpan" and raw > cols:
        return "SPAN_OOB"
    return None

def check_dump_carets_colSpan_width_height(env: Mapping[str, Any]) -> str | None:
    raw = env.get('colSpan', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if 'colSpan' in ("width", "height", "bytes") and raw < 0:
        return "USAGE"
    return None

def check_dump_carets_colSpan_batch_parts(env: Mapping[str, Any]) -> str | None:
    raw = env.get('colSpan', None)
    n_in = env.get("inputN")
    n_ok = env.get("okN")
    n_fail = env.get("failN")
    if raw is None:
        return None
    if not all(isinstance(v, int) and not isinstance(v, bool) for v in (n_in, n_ok, n_fail) if v is not None):
        return None
    if isinstance(n_in, int) and isinstance(n_ok, int) and isinstance(n_fail, int):
        if n_in != n_ok + n_fail:
            return "COUNT_DRIFT"
    return None

def check_dump_carets_bytes_wrong_type_int(env: Mapping[str, Any]) -> str | None:
    raw = env.get('bytes', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return "USAGE"
    return None

def check_dump_carets_bytes_negative(env: Mapping[str, Any]) -> str | None:
    raw = env.get('bytes', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < 0:
        return "USAGE"
    return None

def check_dump_carets_bytes_nonzero_required(env: Mapping[str, Any]) -> str | None:
    raw = env.get('bytes', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw == 0:
        return "ZERO_BYTES"
    return None

def check_dump_carets_bytes_exit_closed(env: Mapping[str, Any]) -> str | None:
    raw = env.get('bytes', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw not in (0, 1, 2, 3, 4):
        return "EXIT_UNKNOWN"
    return None

def check_dump_carets_bytes_too_large(env: Mapping[str, Any]) -> str | None:
    raw = env.get('bytes', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw > 1_000_000_000:
        return "USAGE"
    return None

def check_dump_carets_bytes_fits_i32(env: Mapping[str, Any]) -> str | None:
    raw = env.get('bytes', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < -2147483648 or raw > 2147483647:
        return "USAGE"
    return None

def check_dump_carets_bytes_not_bool_int(env: Mapping[str, Any]) -> str | None:
    raw = env.get('bytes', None)
    if isinstance(raw, bool):
        return "USAGE"
    return None

def check_dump_carets_bytes_lte_page_count(env: Mapping[str, Any]) -> str | None:
    raw = env.get('bytes', None)
    total = env.get("pageCount")
    if raw is None or total is None:
        return None
    if isinstance(raw, bool) or isinstance(total, bool):
        return None
    if isinstance(raw, int) and isinstance(total, int) and raw > total:
        return "COORD_OOB"
    return None

def check_dump_carets_bytes_lte_declared(env: Mapping[str, Any]) -> str | None:
    raw = env.get('bytes', None)
    declared = env.get("declaredCount")
    if raw is None or declared is None:
        return None
    if isinstance(raw, bool) or isinstance(declared, bool):
        return None
    if isinstance(raw, int) and isinstance(declared, int) and raw > declared:
        return "COUNT_DRIFT"
    return None

def check_dump_carets_bytes_under_cap(env: Mapping[str, Any]) -> str | None:
    raw = env.get('bytes', None)
    cap = env.get("capBytes")
    if raw is None or cap is None:
        return None
    if isinstance(raw, bool) or isinstance(cap, bool):
        return None
    if isinstance(raw, int) and isinstance(cap, int) and cap > 0 and raw > cap:
        return "OVER_ACCEPTED"
    return None

def check_dump_carets_bytes_year_range(env: Mapping[str, Any]) -> str | None:
    raw = env.get('bytes', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < 1990 or raw > 2100:
        return "USAGE"
    return None

def check_dump_carets_bytes_fail_vs_identical(env: Mapping[str, Any]) -> str | None:
    raw = env.get('bytes', None)
    identical = env.get("identical")
    if raw is None or identical is not True:
        return None
    if isinstance(raw, int) and not isinstance(raw, bool) and raw > 0:
        return "COUNT_DRIFT"
    return None

def check_dump_carets_bytes_ok_vs_total(env: Mapping[str, Any]) -> str | None:
    raw = env.get('bytes', None)
    total = env.get("total")
    if raw is None or total is None:
        return None
    if isinstance(raw, bool) or isinstance(total, bool):
        return None
    if isinstance(raw, int) and isinstance(total, int) and raw > total:
        return "RATE_IMPOSSIBLE"
    return None

def check_dump_carets_bytes_span_fits(env: Mapping[str, Any]) -> str | None:
    raw = env.get('bytes', None)
    rows = env.get("rows")
    cols = env.get("cols")
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if isinstance(rows, int) and not isinstance(rows, bool) and 'bytes' == "rowSpan" and raw > rows:
        return "SPAN_OOB"
    if isinstance(cols, int) and not isinstance(cols, bool) and 'bytes' == "colSpan" and raw > cols:
        return "SPAN_OOB"
    return None

def check_dump_carets_bytes_width_height(env: Mapping[str, Any]) -> str | None:
    raw = env.get('bytes', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if 'bytes' in ("width", "height", "bytes") and raw < 0:
        return "USAGE"
    return None

def check_dump_carets_bytes_batch_parts(env: Mapping[str, Any]) -> str | None:
    raw = env.get('bytes', None)
    n_in = env.get("inputN")
    n_ok = env.get("okN")
    n_fail = env.get("failN")
    if raw is None:
        return None
    if not all(isinstance(v, int) and not isinstance(v, bool) for v in (n_in, n_ok, n_fail) if v is not None):
        return None
    if isinstance(n_in, int) and isinstance(n_ok, int) and isinstance(n_fail, int):
        if n_in != n_ok + n_fail:
            return "COUNT_DRIFT"
    return None

def check_dump_carets_width_wrong_type_int(env: Mapping[str, Any]) -> str | None:
    raw = env.get('width', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return "USAGE"
    return None

def check_dump_carets_width_negative(env: Mapping[str, Any]) -> str | None:
    raw = env.get('width', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < 0:
        return "USAGE"
    return None

def check_dump_carets_width_nonzero_required(env: Mapping[str, Any]) -> str | None:
    raw = env.get('width', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw == 0:
        return "ZERO_BYTES"
    return None

def check_dump_carets_width_exit_closed(env: Mapping[str, Any]) -> str | None:
    raw = env.get('width', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw not in (0, 1, 2, 3, 4):
        return "EXIT_UNKNOWN"
    return None

def check_dump_carets_width_too_large(env: Mapping[str, Any]) -> str | None:
    raw = env.get('width', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw > 1_000_000_000:
        return "USAGE"
    return None

def check_dump_carets_width_fits_i32(env: Mapping[str, Any]) -> str | None:
    raw = env.get('width', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < -2147483648 or raw > 2147483647:
        return "USAGE"
    return None

def check_dump_carets_width_not_bool_int(env: Mapping[str, Any]) -> str | None:
    raw = env.get('width', None)
    if isinstance(raw, bool):
        return "USAGE"
    return None

def check_dump_carets_width_lte_page_count(env: Mapping[str, Any]) -> str | None:
    raw = env.get('width', None)
    total = env.get("pageCount")
    if raw is None or total is None:
        return None
    if isinstance(raw, bool) or isinstance(total, bool):
        return None
    if isinstance(raw, int) and isinstance(total, int) and raw > total:
        return "COORD_OOB"
    return None

def check_dump_carets_width_lte_declared(env: Mapping[str, Any]) -> str | None:
    raw = env.get('width', None)
    declared = env.get("declaredCount")
    if raw is None or declared is None:
        return None
    if isinstance(raw, bool) or isinstance(declared, bool):
        return None
    if isinstance(raw, int) and isinstance(declared, int) and raw > declared:
        return "COUNT_DRIFT"
    return None

def check_dump_carets_width_under_cap(env: Mapping[str, Any]) -> str | None:
    raw = env.get('width', None)
    cap = env.get("capBytes")
    if raw is None or cap is None:
        return None
    if isinstance(raw, bool) or isinstance(cap, bool):
        return None
    if isinstance(raw, int) and isinstance(cap, int) and cap > 0 and raw > cap:
        return "OVER_ACCEPTED"
    return None

def check_dump_carets_width_year_range(env: Mapping[str, Any]) -> str | None:
    raw = env.get('width', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < 1990 or raw > 2100:
        return "USAGE"
    return None

def check_dump_carets_width_fail_vs_identical(env: Mapping[str, Any]) -> str | None:
    raw = env.get('width', None)
    identical = env.get("identical")
    if raw is None or identical is not True:
        return None
    if isinstance(raw, int) and not isinstance(raw, bool) and raw > 0:
        return "COUNT_DRIFT"
    return None

def check_dump_carets_width_ok_vs_total(env: Mapping[str, Any]) -> str | None:
    raw = env.get('width', None)
    total = env.get("total")
    if raw is None or total is None:
        return None
    if isinstance(raw, bool) or isinstance(total, bool):
        return None
    if isinstance(raw, int) and isinstance(total, int) and raw > total:
        return "RATE_IMPOSSIBLE"
    return None

def check_dump_carets_width_span_fits(env: Mapping[str, Any]) -> str | None:
    raw = env.get('width', None)
    rows = env.get("rows")
    cols = env.get("cols")
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if isinstance(rows, int) and not isinstance(rows, bool) and 'width' == "rowSpan" and raw > rows:
        return "SPAN_OOB"
    if isinstance(cols, int) and not isinstance(cols, bool) and 'width' == "colSpan" and raw > cols:
        return "SPAN_OOB"
    return None

def check_dump_carets_width_width_height(env: Mapping[str, Any]) -> str | None:
    raw = env.get('width', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if 'width' in ("width", "height", "bytes") and raw < 0:
        return "USAGE"
    return None

def check_dump_carets_width_batch_parts(env: Mapping[str, Any]) -> str | None:
    raw = env.get('width', None)
    n_in = env.get("inputN")
    n_ok = env.get("okN")
    n_fail = env.get("failN")
    if raw is None:
        return None
    if not all(isinstance(v, int) and not isinstance(v, bool) for v in (n_in, n_ok, n_fail) if v is not None):
        return None
    if isinstance(n_in, int) and isinstance(n_ok, int) and isinstance(n_fail, int):
        if n_in != n_ok + n_fail:
            return "COUNT_DRIFT"
    return None

def check_dump_carets_height_wrong_type_int(env: Mapping[str, Any]) -> str | None:
    raw = env.get('height', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return "USAGE"
    return None

def check_dump_carets_height_negative(env: Mapping[str, Any]) -> str | None:
    raw = env.get('height', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < 0:
        return "USAGE"
    return None

def check_dump_carets_height_nonzero_required(env: Mapping[str, Any]) -> str | None:
    raw = env.get('height', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw == 0:
        return "ZERO_BYTES"
    return None

def check_dump_carets_height_exit_closed(env: Mapping[str, Any]) -> str | None:
    raw = env.get('height', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw not in (0, 1, 2, 3, 4):
        return "EXIT_UNKNOWN"
    return None

def check_dump_carets_height_too_large(env: Mapping[str, Any]) -> str | None:
    raw = env.get('height', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw > 1_000_000_000:
        return "USAGE"
    return None

def check_dump_carets_height_fits_i32(env: Mapping[str, Any]) -> str | None:
    raw = env.get('height', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < -2147483648 or raw > 2147483647:
        return "USAGE"
    return None

def check_dump_carets_height_not_bool_int(env: Mapping[str, Any]) -> str | None:
    raw = env.get('height', None)
    if isinstance(raw, bool):
        return "USAGE"
    return None

def check_dump_carets_height_lte_page_count(env: Mapping[str, Any]) -> str | None:
    raw = env.get('height', None)
    total = env.get("pageCount")
    if raw is None or total is None:
        return None
    if isinstance(raw, bool) or isinstance(total, bool):
        return None
    if isinstance(raw, int) and isinstance(total, int) and raw > total:
        return "COORD_OOB"
    return None

def check_dump_carets_height_lte_declared(env: Mapping[str, Any]) -> str | None:
    raw = env.get('height', None)
    declared = env.get("declaredCount")
    if raw is None or declared is None:
        return None
    if isinstance(raw, bool) or isinstance(declared, bool):
        return None
    if isinstance(raw, int) and isinstance(declared, int) and raw > declared:
        return "COUNT_DRIFT"
    return None

def check_dump_carets_height_under_cap(env: Mapping[str, Any]) -> str | None:
    raw = env.get('height', None)
    cap = env.get("capBytes")
    if raw is None or cap is None:
        return None
    if isinstance(raw, bool) or isinstance(cap, bool):
        return None
    if isinstance(raw, int) and isinstance(cap, int) and cap > 0 and raw > cap:
        return "OVER_ACCEPTED"
    return None

def check_dump_carets_height_year_range(env: Mapping[str, Any]) -> str | None:
    raw = env.get('height', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < 1990 or raw > 2100:
        return "USAGE"
    return None

def check_dump_carets_height_fail_vs_identical(env: Mapping[str, Any]) -> str | None:
    raw = env.get('height', None)
    identical = env.get("identical")
    if raw is None or identical is not True:
        return None
    if isinstance(raw, int) and not isinstance(raw, bool) and raw > 0:
        return "COUNT_DRIFT"
    return None

def check_dump_carets_height_ok_vs_total(env: Mapping[str, Any]) -> str | None:
    raw = env.get('height', None)
    total = env.get("total")
    if raw is None or total is None:
        return None
    if isinstance(raw, bool) or isinstance(total, bool):
        return None
    if isinstance(raw, int) and isinstance(total, int) and raw > total:
        return "RATE_IMPOSSIBLE"
    return None

def check_dump_carets_height_span_fits(env: Mapping[str, Any]) -> str | None:
    raw = env.get('height', None)
    rows = env.get("rows")
    cols = env.get("cols")
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if isinstance(rows, int) and not isinstance(rows, bool) and 'height' == "rowSpan" and raw > rows:
        return "SPAN_OOB"
    if isinstance(cols, int) and not isinstance(cols, bool) and 'height' == "colSpan" and raw > cols:
        return "SPAN_OOB"
    return None

def check_dump_carets_height_width_height(env: Mapping[str, Any]) -> str | None:
    raw = env.get('height', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if 'height' in ("width", "height", "bytes") and raw < 0:
        return "USAGE"
    return None

def check_dump_carets_height_batch_parts(env: Mapping[str, Any]) -> str | None:
    raw = env.get('height', None)
    n_in = env.get("inputN")
    n_ok = env.get("okN")
    n_fail = env.get("failN")
    if raw is None:
        return None
    if not all(isinstance(v, int) and not isinstance(v, bool) for v in (n_in, n_ok, n_fail) if v is not None):
        return None
    if isinstance(n_in, int) and isinstance(n_ok, int) and isinstance(n_fail, int):
        if n_in != n_ok + n_fail:
            return "COUNT_DRIFT"
    return None

def check_dump_carets_matchCount_wrong_type_int(env: Mapping[str, Any]) -> str | None:
    raw = env.get('matchCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return "USAGE"
    return None

def check_dump_carets_matchCount_negative(env: Mapping[str, Any]) -> str | None:
    raw = env.get('matchCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < 0:
        return "USAGE"
    return None

def check_dump_carets_matchCount_nonzero_required(env: Mapping[str, Any]) -> str | None:
    raw = env.get('matchCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw == 0:
        return "ZERO_BYTES"
    return None

def check_dump_carets_matchCount_exit_closed(env: Mapping[str, Any]) -> str | None:
    raw = env.get('matchCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw not in (0, 1, 2, 3, 4):
        return "EXIT_UNKNOWN"
    return None

def check_dump_carets_matchCount_too_large(env: Mapping[str, Any]) -> str | None:
    raw = env.get('matchCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw > 1_000_000_000:
        return "USAGE"
    return None

def check_dump_carets_matchCount_fits_i32(env: Mapping[str, Any]) -> str | None:
    raw = env.get('matchCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < -2147483648 or raw > 2147483647:
        return "USAGE"
    return None

def check_dump_carets_matchCount_not_bool_int(env: Mapping[str, Any]) -> str | None:
    raw = env.get('matchCount', None)
    if isinstance(raw, bool):
        return "USAGE"
    return None

def check_dump_carets_matchCount_lte_page_count(env: Mapping[str, Any]) -> str | None:
    raw = env.get('matchCount', None)
    total = env.get("pageCount")
    if raw is None or total is None:
        return None
    if isinstance(raw, bool) or isinstance(total, bool):
        return None
    if isinstance(raw, int) and isinstance(total, int) and raw > total:
        return "COORD_OOB"
    return None

def check_dump_carets_matchCount_lte_declared(env: Mapping[str, Any]) -> str | None:
    raw = env.get('matchCount', None)
    declared = env.get("declaredCount")
    if raw is None or declared is None:
        return None
    if isinstance(raw, bool) or isinstance(declared, bool):
        return None
    if isinstance(raw, int) and isinstance(declared, int) and raw > declared:
        return "COUNT_DRIFT"
    return None

def check_dump_carets_matchCount_under_cap(env: Mapping[str, Any]) -> str | None:
    raw = env.get('matchCount', None)
    cap = env.get("capBytes")
    if raw is None or cap is None:
        return None
    if isinstance(raw, bool) or isinstance(cap, bool):
        return None
    if isinstance(raw, int) and isinstance(cap, int) and cap > 0 and raw > cap:
        return "OVER_ACCEPTED"
    return None

def check_dump_carets_matchCount_year_range(env: Mapping[str, Any]) -> str | None:
    raw = env.get('matchCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < 1990 or raw > 2100:
        return "USAGE"
    return None

def check_dump_carets_matchCount_fail_vs_identical(env: Mapping[str, Any]) -> str | None:
    raw = env.get('matchCount', None)
    identical = env.get("identical")
    if raw is None or identical is not True:
        return None
    if isinstance(raw, int) and not isinstance(raw, bool) and raw > 0:
        return "COUNT_DRIFT"
    return None

def check_dump_carets_matchCount_ok_vs_total(env: Mapping[str, Any]) -> str | None:
    raw = env.get('matchCount', None)
    total = env.get("total")
    if raw is None or total is None:
        return None
    if isinstance(raw, bool) or isinstance(total, bool):
        return None
    if isinstance(raw, int) and isinstance(total, int) and raw > total:
        return "RATE_IMPOSSIBLE"
    return None

def check_dump_carets_matchCount_span_fits(env: Mapping[str, Any]) -> str | None:
    raw = env.get('matchCount', None)
    rows = env.get("rows")
    cols = env.get("cols")
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if isinstance(rows, int) and not isinstance(rows, bool) and 'matchCount' == "rowSpan" and raw > rows:
        return "SPAN_OOB"
    if isinstance(cols, int) and not isinstance(cols, bool) and 'matchCount' == "colSpan" and raw > cols:
        return "SPAN_OOB"
    return None

def check_dump_carets_matchCount_width_height(env: Mapping[str, Any]) -> str | None:
    raw = env.get('matchCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if 'matchCount' in ("width", "height", "bytes") and raw < 0:
        return "USAGE"
    return None

def check_dump_carets_matchCount_batch_parts(env: Mapping[str, Any]) -> str | None:
    raw = env.get('matchCount', None)
    n_in = env.get("inputN")
    n_ok = env.get("okN")
    n_fail = env.get("failN")
    if raw is None:
        return None
    if not all(isinstance(v, int) and not isinstance(v, bool) for v in (n_in, n_ok, n_fail) if v is not None):
        return None
    if isinstance(n_in, int) and isinstance(n_ok, int) and isinstance(n_fail, int):
        if n_in != n_ok + n_fail:
            return "COUNT_DRIFT"
    return None

def check_dump_carets_page_wrong_type_int(env: Mapping[str, Any]) -> str | None:
    raw = env.get('page', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return "USAGE"
    return None

def check_dump_carets_page_negative(env: Mapping[str, Any]) -> str | None:
    raw = env.get('page', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < 0:
        return "USAGE"
    return None

def check_dump_carets_page_nonzero_required(env: Mapping[str, Any]) -> str | None:
    raw = env.get('page', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw == 0:
        return "ZERO_BYTES"
    return None

def check_dump_carets_page_exit_closed(env: Mapping[str, Any]) -> str | None:
    raw = env.get('page', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw not in (0, 1, 2, 3, 4):
        return "EXIT_UNKNOWN"
    return None

def check_dump_carets_page_too_large(env: Mapping[str, Any]) -> str | None:
    raw = env.get('page', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw > 1_000_000_000:
        return "USAGE"
    return None

def check_dump_carets_page_fits_i32(env: Mapping[str, Any]) -> str | None:
    raw = env.get('page', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < -2147483648 or raw > 2147483647:
        return "USAGE"
    return None

def check_dump_carets_page_not_bool_int(env: Mapping[str, Any]) -> str | None:
    raw = env.get('page', None)
    if isinstance(raw, bool):
        return "USAGE"
    return None

def check_dump_carets_page_lte_page_count(env: Mapping[str, Any]) -> str | None:
    raw = env.get('page', None)
    total = env.get("pageCount")
    if raw is None or total is None:
        return None
    if isinstance(raw, bool) or isinstance(total, bool):
        return None
    if isinstance(raw, int) and isinstance(total, int) and raw > total:
        return "COORD_OOB"
    return None

def check_dump_carets_page_lte_declared(env: Mapping[str, Any]) -> str | None:
    raw = env.get('page', None)
    declared = env.get("declaredCount")
    if raw is None or declared is None:
        return None
    if isinstance(raw, bool) or isinstance(declared, bool):
        return None
    if isinstance(raw, int) and isinstance(declared, int) and raw > declared:
        return "COUNT_DRIFT"
    return None

def check_dump_carets_page_under_cap(env: Mapping[str, Any]) -> str | None:
    raw = env.get('page', None)
    cap = env.get("capBytes")
    if raw is None or cap is None:
        return None
    if isinstance(raw, bool) or isinstance(cap, bool):
        return None
    if isinstance(raw, int) and isinstance(cap, int) and cap > 0 and raw > cap:
        return "OVER_ACCEPTED"
    return None

def check_dump_carets_page_year_range(env: Mapping[str, Any]) -> str | None:
    raw = env.get('page', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < 1990 or raw > 2100:
        return "USAGE"
    return None

def check_dump_carets_page_fail_vs_identical(env: Mapping[str, Any]) -> str | None:
    raw = env.get('page', None)
    identical = env.get("identical")
    if raw is None or identical is not True:
        return None
    if isinstance(raw, int) and not isinstance(raw, bool) and raw > 0:
        return "COUNT_DRIFT"
    return None

def check_dump_carets_page_ok_vs_total(env: Mapping[str, Any]) -> str | None:
    raw = env.get('page', None)
    total = env.get("total")
    if raw is None or total is None:
        return None
    if isinstance(raw, bool) or isinstance(total, bool):
        return None
    if isinstance(raw, int) and isinstance(total, int) and raw > total:
        return "RATE_IMPOSSIBLE"
    return None

def check_dump_carets_page_span_fits(env: Mapping[str, Any]) -> str | None:
    raw = env.get('page', None)
    rows = env.get("rows")
    cols = env.get("cols")
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if isinstance(rows, int) and not isinstance(rows, bool) and 'page' == "rowSpan" and raw > rows:
        return "SPAN_OOB"
    if isinstance(cols, int) and not isinstance(cols, bool) and 'page' == "colSpan" and raw > cols:
        return "SPAN_OOB"
    return None

def check_dump_carets_page_width_height(env: Mapping[str, Any]) -> str | None:
    raw = env.get('page', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if 'page' in ("width", "height", "bytes") and raw < 0:
        return "USAGE"
    return None

def check_dump_carets_page_batch_parts(env: Mapping[str, Any]) -> str | None:
    raw = env.get('page', None)
    n_in = env.get("inputN")
    n_ok = env.get("okN")
    n_fail = env.get("failN")
    if raw is None:
        return None
    if not all(isinstance(v, int) and not isinstance(v, bool) for v in (n_in, n_ok, n_fail) if v is not None):
        return None
    if isinstance(n_in, int) and isinstance(n_ok, int) and isinstance(n_fail, int):
        if n_in != n_ok + n_fail:
            return "COUNT_DRIFT"
    return None

def check_dump_carets_offset_wrong_type_int(env: Mapping[str, Any]) -> str | None:
    raw = env.get('offset', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return "USAGE"
    return None

def check_dump_carets_offset_negative(env: Mapping[str, Any]) -> str | None:
    raw = env.get('offset', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < 0:
        return "USAGE"
    return None

def check_dump_carets_offset_nonzero_required(env: Mapping[str, Any]) -> str | None:
    raw = env.get('offset', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw == 0:
        return "ZERO_BYTES"
    return None

def check_dump_carets_offset_exit_closed(env: Mapping[str, Any]) -> str | None:
    raw = env.get('offset', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw not in (0, 1, 2, 3, 4):
        return "EXIT_UNKNOWN"
    return None

def check_dump_carets_offset_too_large(env: Mapping[str, Any]) -> str | None:
    raw = env.get('offset', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw > 1_000_000_000:
        return "USAGE"
    return None

def check_dump_carets_offset_fits_i32(env: Mapping[str, Any]) -> str | None:
    raw = env.get('offset', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < -2147483648 or raw > 2147483647:
        return "USAGE"
    return None

def check_dump_carets_offset_not_bool_int(env: Mapping[str, Any]) -> str | None:
    raw = env.get('offset', None)
    if isinstance(raw, bool):
        return "USAGE"
    return None

def check_dump_carets_offset_lte_page_count(env: Mapping[str, Any]) -> str | None:
    raw = env.get('offset', None)
    total = env.get("pageCount")
    if raw is None or total is None:
        return None
    if isinstance(raw, bool) or isinstance(total, bool):
        return None
    if isinstance(raw, int) and isinstance(total, int) and raw > total:
        return "COORD_OOB"
    return None

def check_dump_carets_offset_lte_declared(env: Mapping[str, Any]) -> str | None:
    raw = env.get('offset', None)
    declared = env.get("declaredCount")
    if raw is None or declared is None:
        return None
    if isinstance(raw, bool) or isinstance(declared, bool):
        return None
    if isinstance(raw, int) and isinstance(declared, int) and raw > declared:
        return "COUNT_DRIFT"
    return None

def check_dump_carets_offset_under_cap(env: Mapping[str, Any]) -> str | None:
    raw = env.get('offset', None)
    cap = env.get("capBytes")
    if raw is None or cap is None:
        return None
    if isinstance(raw, bool) or isinstance(cap, bool):
        return None
    if isinstance(raw, int) and isinstance(cap, int) and cap > 0 and raw > cap:
        return "OVER_ACCEPTED"
    return None

def check_dump_carets_offset_year_range(env: Mapping[str, Any]) -> str | None:
    raw = env.get('offset', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < 1990 or raw > 2100:
        return "USAGE"
    return None

def check_dump_carets_offset_fail_vs_identical(env: Mapping[str, Any]) -> str | None:
    raw = env.get('offset', None)
    identical = env.get("identical")
    if raw is None or identical is not True:
        return None
    if isinstance(raw, int) and not isinstance(raw, bool) and raw > 0:
        return "COUNT_DRIFT"
    return None

def check_dump_carets_offset_ok_vs_total(env: Mapping[str, Any]) -> str | None:
    raw = env.get('offset', None)
    total = env.get("total")
    if raw is None or total is None:
        return None
    if isinstance(raw, bool) or isinstance(total, bool):
        return None
    if isinstance(raw, int) and isinstance(total, int) and raw > total:
        return "RATE_IMPOSSIBLE"
    return None

def check_dump_carets_offset_span_fits(env: Mapping[str, Any]) -> str | None:
    raw = env.get('offset', None)
    rows = env.get("rows")
    cols = env.get("cols")
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if isinstance(rows, int) and not isinstance(rows, bool) and 'offset' == "rowSpan" and raw > rows:
        return "SPAN_OOB"
    if isinstance(cols, int) and not isinstance(cols, bool) and 'offset' == "colSpan" and raw > cols:
        return "SPAN_OOB"
    return None

def check_dump_carets_offset_width_height(env: Mapping[str, Any]) -> str | None:
    raw = env.get('offset', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if 'offset' in ("width", "height", "bytes") and raw < 0:
        return "USAGE"
    return None

def check_dump_carets_offset_batch_parts(env: Mapping[str, Any]) -> str | None:
    raw = env.get('offset', None)
    n_in = env.get("inputN")
    n_ok = env.get("okN")
    n_fail = env.get("failN")
    if raw is None:
        return None
    if not all(isinstance(v, int) and not isinstance(v, bool) for v in (n_in, n_ok, n_fail) if v is not None):
        return None
    if isinstance(n_in, int) and isinstance(n_ok, int) and isinstance(n_fail, int):
        if n_in != n_ok + n_fail:
            return "COUNT_DRIFT"
    return None

def check_dump_carets_count_wrong_type_int(env: Mapping[str, Any]) -> str | None:
    raw = env.get('count', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return "USAGE"
    return None

def check_dump_carets_count_negative(env: Mapping[str, Any]) -> str | None:
    raw = env.get('count', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < 0:
        return "USAGE"
    return None

def check_dump_carets_count_nonzero_required(env: Mapping[str, Any]) -> str | None:
    raw = env.get('count', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw == 0:
        return "ZERO_BYTES"
    return None

def check_dump_carets_count_exit_closed(env: Mapping[str, Any]) -> str | None:
    raw = env.get('count', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw not in (0, 1, 2, 3, 4):
        return "EXIT_UNKNOWN"
    return None

def check_dump_carets_count_too_large(env: Mapping[str, Any]) -> str | None:
    raw = env.get('count', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw > 1_000_000_000:
        return "USAGE"
    return None

def check_dump_carets_count_fits_i32(env: Mapping[str, Any]) -> str | None:
    raw = env.get('count', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < -2147483648 or raw > 2147483647:
        return "USAGE"
    return None

def check_dump_carets_count_not_bool_int(env: Mapping[str, Any]) -> str | None:
    raw = env.get('count', None)
    if isinstance(raw, bool):
        return "USAGE"
    return None

def check_dump_carets_count_lte_page_count(env: Mapping[str, Any]) -> str | None:
    raw = env.get('count', None)
    total = env.get("pageCount")
    if raw is None or total is None:
        return None
    if isinstance(raw, bool) or isinstance(total, bool):
        return None
    if isinstance(raw, int) and isinstance(total, int) and raw > total:
        return "COORD_OOB"
    return None

def check_dump_carets_count_lte_declared(env: Mapping[str, Any]) -> str | None:
    raw = env.get('count', None)
    declared = env.get("declaredCount")
    if raw is None or declared is None:
        return None
    if isinstance(raw, bool) or isinstance(declared, bool):
        return None
    if isinstance(raw, int) and isinstance(declared, int) and raw > declared:
        return "COUNT_DRIFT"
    return None

def check_dump_carets_count_under_cap(env: Mapping[str, Any]) -> str | None:
    raw = env.get('count', None)
    cap = env.get("capBytes")
    if raw is None or cap is None:
        return None
    if isinstance(raw, bool) or isinstance(cap, bool):
        return None
    if isinstance(raw, int) and isinstance(cap, int) and cap > 0 and raw > cap:
        return "OVER_ACCEPTED"
    return None

def check_dump_carets_count_year_range(env: Mapping[str, Any]) -> str | None:
    raw = env.get('count', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < 1990 or raw > 2100:
        return "USAGE"
    return None

def check_dump_carets_count_fail_vs_identical(env: Mapping[str, Any]) -> str | None:
    raw = env.get('count', None)
    identical = env.get("identical")
    if raw is None or identical is not True:
        return None
    if isinstance(raw, int) and not isinstance(raw, bool) and raw > 0:
        return "COUNT_DRIFT"
    return None

def check_dump_carets_count_ok_vs_total(env: Mapping[str, Any]) -> str | None:
    raw = env.get('count', None)
    total = env.get("total")
    if raw is None or total is None:
        return None
    if isinstance(raw, bool) or isinstance(total, bool):
        return None
    if isinstance(raw, int) and isinstance(total, int) and raw > total:
        return "RATE_IMPOSSIBLE"
    return None

def check_dump_carets_count_span_fits(env: Mapping[str, Any]) -> str | None:
    raw = env.get('count', None)
    rows = env.get("rows")
    cols = env.get("cols")
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if isinstance(rows, int) and not isinstance(rows, bool) and 'count' == "rowSpan" and raw > rows:
        return "SPAN_OOB"
    if isinstance(cols, int) and not isinstance(cols, bool) and 'count' == "colSpan" and raw > cols:
        return "SPAN_OOB"
    return None

def check_dump_carets_count_width_height(env: Mapping[str, Any]) -> str | None:
    raw = env.get('count', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if 'count' in ("width", "height", "bytes") and raw < 0:
        return "USAGE"
    return None

def check_dump_carets_count_batch_parts(env: Mapping[str, Any]) -> str | None:
    raw = env.get('count', None)
    n_in = env.get("inputN")
    n_ok = env.get("okN")
    n_fail = env.get("failN")
    if raw is None:
        return None
    if not all(isinstance(v, int) and not isinstance(v, bool) for v in (n_in, n_ok, n_fail) if v is not None):
        return None
    if isinstance(n_in, int) and isinstance(n_ok, int) and isinstance(n_fail, int):
        if n_in != n_ok + n_fail:
            return "COUNT_DRIFT"
    return None

def check_dump_carets_inputN_wrong_type_int(env: Mapping[str, Any]) -> str | None:
    raw = env.get('inputN', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return "USAGE"
    return None

def check_dump_carets_inputN_negative(env: Mapping[str, Any]) -> str | None:
    raw = env.get('inputN', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < 0:
        return "USAGE"
    return None

def check_dump_carets_inputN_nonzero_required(env: Mapping[str, Any]) -> str | None:
    raw = env.get('inputN', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw == 0:
        return "ZERO_BYTES"
    return None

def check_dump_carets_inputN_exit_closed(env: Mapping[str, Any]) -> str | None:
    raw = env.get('inputN', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw not in (0, 1, 2, 3, 4):
        return "EXIT_UNKNOWN"
    return None

def check_dump_carets_inputN_too_large(env: Mapping[str, Any]) -> str | None:
    raw = env.get('inputN', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw > 1_000_000_000:
        return "USAGE"
    return None

def check_dump_carets_inputN_fits_i32(env: Mapping[str, Any]) -> str | None:
    raw = env.get('inputN', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < -2147483648 or raw > 2147483647:
        return "USAGE"
    return None

def check_dump_carets_inputN_not_bool_int(env: Mapping[str, Any]) -> str | None:
    raw = env.get('inputN', None)
    if isinstance(raw, bool):
        return "USAGE"
    return None

def check_dump_carets_inputN_lte_page_count(env: Mapping[str, Any]) -> str | None:
    raw = env.get('inputN', None)
    total = env.get("pageCount")
    if raw is None or total is None:
        return None
    if isinstance(raw, bool) or isinstance(total, bool):
        return None
    if isinstance(raw, int) and isinstance(total, int) and raw > total:
        return "COORD_OOB"
    return None

def check_dump_carets_inputN_lte_declared(env: Mapping[str, Any]) -> str | None:
    raw = env.get('inputN', None)
    declared = env.get("declaredCount")
    if raw is None or declared is None:
        return None
    if isinstance(raw, bool) or isinstance(declared, bool):
        return None
    if isinstance(raw, int) and isinstance(declared, int) and raw > declared:
        return "COUNT_DRIFT"
    return None

def check_dump_carets_inputN_under_cap(env: Mapping[str, Any]) -> str | None:
    raw = env.get('inputN', None)
    cap = env.get("capBytes")
    if raw is None or cap is None:
        return None
    if isinstance(raw, bool) or isinstance(cap, bool):
        return None
    if isinstance(raw, int) and isinstance(cap, int) and cap > 0 and raw > cap:
        return "OVER_ACCEPTED"
    return None

def check_dump_carets_inputN_year_range(env: Mapping[str, Any]) -> str | None:
    raw = env.get('inputN', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < 1990 or raw > 2100:
        return "USAGE"
    return None

def check_dump_carets_inputN_fail_vs_identical(env: Mapping[str, Any]) -> str | None:
    raw = env.get('inputN', None)
    identical = env.get("identical")
    if raw is None or identical is not True:
        return None
    if isinstance(raw, int) and not isinstance(raw, bool) and raw > 0:
        return "COUNT_DRIFT"
    return None

def check_dump_carets_inputN_ok_vs_total(env: Mapping[str, Any]) -> str | None:
    raw = env.get('inputN', None)
    total = env.get("total")
    if raw is None or total is None:
        return None
    if isinstance(raw, bool) or isinstance(total, bool):
        return None
    if isinstance(raw, int) and isinstance(total, int) and raw > total:
        return "RATE_IMPOSSIBLE"
    return None

def check_dump_carets_inputN_span_fits(env: Mapping[str, Any]) -> str | None:
    raw = env.get('inputN', None)
    rows = env.get("rows")
    cols = env.get("cols")
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if isinstance(rows, int) and not isinstance(rows, bool) and 'inputN' == "rowSpan" and raw > rows:
        return "SPAN_OOB"
    if isinstance(cols, int) and not isinstance(cols, bool) and 'inputN' == "colSpan" and raw > cols:
        return "SPAN_OOB"
    return None

def check_dump_carets_inputN_width_height(env: Mapping[str, Any]) -> str | None:
    raw = env.get('inputN', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if 'inputN' in ("width", "height", "bytes") and raw < 0:
        return "USAGE"
    return None

def check_dump_carets_inputN_batch_parts(env: Mapping[str, Any]) -> str | None:
    raw = env.get('inputN', None)
    n_in = env.get("inputN")
    n_ok = env.get("okN")
    n_fail = env.get("failN")
    if raw is None:
        return None
    if not all(isinstance(v, int) and not isinstance(v, bool) for v in (n_in, n_ok, n_fail) if v is not None):
        return None
    if isinstance(n_in, int) and isinstance(n_ok, int) and isinstance(n_fail, int):
        if n_in != n_ok + n_fail:
            return "COUNT_DRIFT"
    return None

def check_dump_carets_okN_wrong_type_int(env: Mapping[str, Any]) -> str | None:
    raw = env.get('okN', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return "USAGE"
    return None

def check_dump_carets_okN_negative(env: Mapping[str, Any]) -> str | None:
    raw = env.get('okN', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < 0:
        return "USAGE"
    return None

def check_dump_carets_okN_nonzero_required(env: Mapping[str, Any]) -> str | None:
    raw = env.get('okN', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw == 0:
        return "ZERO_BYTES"
    return None

def check_dump_carets_okN_exit_closed(env: Mapping[str, Any]) -> str | None:
    raw = env.get('okN', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw not in (0, 1, 2, 3, 4):
        return "EXIT_UNKNOWN"
    return None

def check_dump_carets_okN_too_large(env: Mapping[str, Any]) -> str | None:
    raw = env.get('okN', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw > 1_000_000_000:
        return "USAGE"
    return None

def check_dump_carets_okN_fits_i32(env: Mapping[str, Any]) -> str | None:
    raw = env.get('okN', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < -2147483648 or raw > 2147483647:
        return "USAGE"
    return None

def check_dump_carets_okN_not_bool_int(env: Mapping[str, Any]) -> str | None:
    raw = env.get('okN', None)
    if isinstance(raw, bool):
        return "USAGE"
    return None

def check_dump_carets_okN_lte_page_count(env: Mapping[str, Any]) -> str | None:
    raw = env.get('okN', None)
    total = env.get("pageCount")
    if raw is None or total is None:
        return None
    if isinstance(raw, bool) or isinstance(total, bool):
        return None
    if isinstance(raw, int) and isinstance(total, int) and raw > total:
        return "COORD_OOB"
    return None

def check_dump_carets_okN_lte_declared(env: Mapping[str, Any]) -> str | None:
    raw = env.get('okN', None)
    declared = env.get("declaredCount")
    if raw is None or declared is None:
        return None
    if isinstance(raw, bool) or isinstance(declared, bool):
        return None
    if isinstance(raw, int) and isinstance(declared, int) and raw > declared:
        return "COUNT_DRIFT"
    return None

def check_dump_carets_okN_under_cap(env: Mapping[str, Any]) -> str | None:
    raw = env.get('okN', None)
    cap = env.get("capBytes")
    if raw is None or cap is None:
        return None
    if isinstance(raw, bool) or isinstance(cap, bool):
        return None
    if isinstance(raw, int) and isinstance(cap, int) and cap > 0 and raw > cap:
        return "OVER_ACCEPTED"
    return None

def check_dump_carets_okN_year_range(env: Mapping[str, Any]) -> str | None:
    raw = env.get('okN', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < 1990 or raw > 2100:
        return "USAGE"
    return None

def check_dump_carets_okN_fail_vs_identical(env: Mapping[str, Any]) -> str | None:
    raw = env.get('okN', None)
    identical = env.get("identical")
    if raw is None or identical is not True:
        return None
    if isinstance(raw, int) and not isinstance(raw, bool) and raw > 0:
        return "COUNT_DRIFT"
    return None

def check_dump_carets_okN_ok_vs_total(env: Mapping[str, Any]) -> str | None:
    raw = env.get('okN', None)
    total = env.get("total")
    if raw is None or total is None:
        return None
    if isinstance(raw, bool) or isinstance(total, bool):
        return None
    if isinstance(raw, int) and isinstance(total, int) and raw > total:
        return "RATE_IMPOSSIBLE"
    return None

def check_dump_carets_okN_span_fits(env: Mapping[str, Any]) -> str | None:
    raw = env.get('okN', None)
    rows = env.get("rows")
    cols = env.get("cols")
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if isinstance(rows, int) and not isinstance(rows, bool) and 'okN' == "rowSpan" and raw > rows:
        return "SPAN_OOB"
    if isinstance(cols, int) and not isinstance(cols, bool) and 'okN' == "colSpan" and raw > cols:
        return "SPAN_OOB"
    return None

def check_dump_carets_okN_width_height(env: Mapping[str, Any]) -> str | None:
    raw = env.get('okN', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if 'okN' in ("width", "height", "bytes") and raw < 0:
        return "USAGE"
    return None

def check_dump_carets_okN_batch_parts(env: Mapping[str, Any]) -> str | None:
    raw = env.get('okN', None)
    n_in = env.get("inputN")
    n_ok = env.get("okN")
    n_fail = env.get("failN")
    if raw is None:
        return None
    if not all(isinstance(v, int) and not isinstance(v, bool) for v in (n_in, n_ok, n_fail) if v is not None):
        return None
    if isinstance(n_in, int) and isinstance(n_ok, int) and isinstance(n_fail, int):
        if n_in != n_ok + n_fail:
            return "COUNT_DRIFT"
    return None

def check_dump_carets_failN_wrong_type_int(env: Mapping[str, Any]) -> str | None:
    raw = env.get('failN', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return "USAGE"
    return None

def check_dump_carets_failN_negative(env: Mapping[str, Any]) -> str | None:
    raw = env.get('failN', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < 0:
        return "USAGE"
    return None

def check_dump_carets_failN_nonzero_required(env: Mapping[str, Any]) -> str | None:
    raw = env.get('failN', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw == 0:
        return "ZERO_BYTES"
    return None

def check_dump_carets_failN_exit_closed(env: Mapping[str, Any]) -> str | None:
    raw = env.get('failN', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw not in (0, 1, 2, 3, 4):
        return "EXIT_UNKNOWN"
    return None

def check_dump_carets_failN_too_large(env: Mapping[str, Any]) -> str | None:
    raw = env.get('failN', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw > 1_000_000_000:
        return "USAGE"
    return None

def check_dump_carets_failN_fits_i32(env: Mapping[str, Any]) -> str | None:
    raw = env.get('failN', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < -2147483648 or raw > 2147483647:
        return "USAGE"
    return None

def check_dump_carets_failN_not_bool_int(env: Mapping[str, Any]) -> str | None:
    raw = env.get('failN', None)
    if isinstance(raw, bool):
        return "USAGE"
    return None

def check_dump_carets_failN_lte_page_count(env: Mapping[str, Any]) -> str | None:
    raw = env.get('failN', None)
    total = env.get("pageCount")
    if raw is None or total is None:
        return None
    if isinstance(raw, bool) or isinstance(total, bool):
        return None
    if isinstance(raw, int) and isinstance(total, int) and raw > total:
        return "COORD_OOB"
    return None

def check_dump_carets_failN_lte_declared(env: Mapping[str, Any]) -> str | None:
    raw = env.get('failN', None)
    declared = env.get("declaredCount")
    if raw is None or declared is None:
        return None
    if isinstance(raw, bool) or isinstance(declared, bool):
        return None
    if isinstance(raw, int) and isinstance(declared, int) and raw > declared:
        return "COUNT_DRIFT"
    return None

def check_dump_carets_failN_under_cap(env: Mapping[str, Any]) -> str | None:
    raw = env.get('failN', None)
    cap = env.get("capBytes")
    if raw is None or cap is None:
        return None
    if isinstance(raw, bool) or isinstance(cap, bool):
        return None
    if isinstance(raw, int) and isinstance(cap, int) and cap > 0 and raw > cap:
        return "OVER_ACCEPTED"
    return None

def check_dump_carets_failN_year_range(env: Mapping[str, Any]) -> str | None:
    raw = env.get('failN', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < 1990 or raw > 2100:
        return "USAGE"
    return None

def check_dump_carets_failN_fail_vs_identical(env: Mapping[str, Any]) -> str | None:
    raw = env.get('failN', None)
    identical = env.get("identical")
    if raw is None or identical is not True:
        return None
    if isinstance(raw, int) and not isinstance(raw, bool) and raw > 0:
        return "COUNT_DRIFT"
    return None

def check_dump_carets_failN_ok_vs_total(env: Mapping[str, Any]) -> str | None:
    raw = env.get('failN', None)
    total = env.get("total")
    if raw is None or total is None:
        return None
    if isinstance(raw, bool) or isinstance(total, bool):
        return None
    if isinstance(raw, int) and isinstance(total, int) and raw > total:
        return "RATE_IMPOSSIBLE"
    return None

def check_dump_carets_failN_span_fits(env: Mapping[str, Any]) -> str | None:
    raw = env.get('failN', None)
    rows = env.get("rows")
    cols = env.get("cols")
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if isinstance(rows, int) and not isinstance(rows, bool) and 'failN' == "rowSpan" and raw > rows:
        return "SPAN_OOB"
    if isinstance(cols, int) and not isinstance(cols, bool) and 'failN' == "colSpan" and raw > cols:
        return "SPAN_OOB"
    return None

def check_dump_carets_failN_width_height(env: Mapping[str, Any]) -> str | None:
    raw = env.get('failN', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if 'failN' in ("width", "height", "bytes") and raw < 0:
        return "USAGE"
    return None

def check_dump_carets_failN_batch_parts(env: Mapping[str, Any]) -> str | None:
    raw = env.get('failN', None)
    n_in = env.get("inputN")
    n_ok = env.get("okN")
    n_fail = env.get("failN")
    if raw is None:
        return None
    if not all(isinstance(v, int) and not isinstance(v, bool) for v in (n_in, n_ok, n_fail) if v is not None):
        return None
    if isinstance(n_in, int) and isinstance(n_ok, int) and isinstance(n_fail, int):
        if n_in != n_ok + n_fail:
            return "COUNT_DRIFT"
    return None

def check_dump_carets_findingCount_wrong_type_int(env: Mapping[str, Any]) -> str | None:
    raw = env.get('findingCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return "USAGE"
    return None

def check_dump_carets_findingCount_negative(env: Mapping[str, Any]) -> str | None:
    raw = env.get('findingCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < 0:
        return "USAGE"
    return None

def check_dump_carets_findingCount_nonzero_required(env: Mapping[str, Any]) -> str | None:
    raw = env.get('findingCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw == 0:
        return "ZERO_BYTES"
    return None

def check_dump_carets_findingCount_exit_closed(env: Mapping[str, Any]) -> str | None:
    raw = env.get('findingCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw not in (0, 1, 2, 3, 4):
        return "EXIT_UNKNOWN"
    return None

def check_dump_carets_findingCount_too_large(env: Mapping[str, Any]) -> str | None:
    raw = env.get('findingCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw > 1_000_000_000:
        return "USAGE"
    return None

def check_dump_carets_findingCount_fits_i32(env: Mapping[str, Any]) -> str | None:
    raw = env.get('findingCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < -2147483648 or raw > 2147483647:
        return "USAGE"
    return None

def check_dump_carets_findingCount_not_bool_int(env: Mapping[str, Any]) -> str | None:
    raw = env.get('findingCount', None)
    if isinstance(raw, bool):
        return "USAGE"
    return None

def check_dump_carets_findingCount_lte_page_count(env: Mapping[str, Any]) -> str | None:
    raw = env.get('findingCount', None)
    total = env.get("pageCount")
    if raw is None or total is None:
        return None
    if isinstance(raw, bool) or isinstance(total, bool):
        return None
    if isinstance(raw, int) and isinstance(total, int) and raw > total:
        return "COORD_OOB"
    return None

def check_dump_carets_findingCount_lte_declared(env: Mapping[str, Any]) -> str | None:
    raw = env.get('findingCount', None)
    declared = env.get("declaredCount")
    if raw is None or declared is None:
        return None
    if isinstance(raw, bool) or isinstance(declared, bool):
        return None
    if isinstance(raw, int) and isinstance(declared, int) and raw > declared:
        return "COUNT_DRIFT"
    return None

def check_dump_carets_findingCount_under_cap(env: Mapping[str, Any]) -> str | None:
    raw = env.get('findingCount', None)
    cap = env.get("capBytes")
    if raw is None or cap is None:
        return None
    if isinstance(raw, bool) or isinstance(cap, bool):
        return None
    if isinstance(raw, int) and isinstance(cap, int) and cap > 0 and raw > cap:
        return "OVER_ACCEPTED"
    return None

def check_dump_carets_findingCount_year_range(env: Mapping[str, Any]) -> str | None:
    raw = env.get('findingCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < 1990 or raw > 2100:
        return "USAGE"
    return None

def check_dump_carets_findingCount_fail_vs_identical(env: Mapping[str, Any]) -> str | None:
    raw = env.get('findingCount', None)
    identical = env.get("identical")
    if raw is None or identical is not True:
        return None
    if isinstance(raw, int) and not isinstance(raw, bool) and raw > 0:
        return "COUNT_DRIFT"
    return None

def check_dump_carets_findingCount_ok_vs_total(env: Mapping[str, Any]) -> str | None:
    raw = env.get('findingCount', None)
    total = env.get("total")
    if raw is None or total is None:
        return None
    if isinstance(raw, bool) or isinstance(total, bool):
        return None
    if isinstance(raw, int) and isinstance(total, int) and raw > total:
        return "RATE_IMPOSSIBLE"
    return None

def check_dump_carets_findingCount_span_fits(env: Mapping[str, Any]) -> str | None:
    raw = env.get('findingCount', None)
    rows = env.get("rows")
    cols = env.get("cols")
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if isinstance(rows, int) and not isinstance(rows, bool) and 'findingCount' == "rowSpan" and raw > rows:
        return "SPAN_OOB"
    if isinstance(cols, int) and not isinstance(cols, bool) and 'findingCount' == "colSpan" and raw > cols:
        return "SPAN_OOB"
    return None

def check_dump_carets_findingCount_width_height(env: Mapping[str, Any]) -> str | None:
    raw = env.get('findingCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if 'findingCount' in ("width", "height", "bytes") and raw < 0:
        return "USAGE"
    return None

def check_dump_carets_findingCount_batch_parts(env: Mapping[str, Any]) -> str | None:
    raw = env.get('findingCount', None)
    n_in = env.get("inputN")
    n_ok = env.get("okN")
    n_fail = env.get("failN")
    if raw is None:
        return None
    if not all(isinstance(v, int) and not isinstance(v, bool) for v in (n_in, n_ok, n_fail) if v is not None):
        return None
    if isinstance(n_in, int) and isinstance(n_ok, int) and isinstance(n_fail, int):
        if n_in != n_ok + n_fail:
            return "COUNT_DRIFT"
    return None

def check_dump_carets_overflow_wrong_type_int(env: Mapping[str, Any]) -> str | None:
    raw = env.get('overflow', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return "USAGE"
    return None

def check_dump_carets_overflow_negative(env: Mapping[str, Any]) -> str | None:
    raw = env.get('overflow', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < 0:
        return "USAGE"
    return None

def check_dump_carets_overflow_nonzero_required(env: Mapping[str, Any]) -> str | None:
    raw = env.get('overflow', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw == 0:
        return "ZERO_BYTES"
    return None

def check_dump_carets_overflow_exit_closed(env: Mapping[str, Any]) -> str | None:
    raw = env.get('overflow', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw not in (0, 1, 2, 3, 4):
        return "EXIT_UNKNOWN"
    return None

def check_dump_carets_overflow_too_large(env: Mapping[str, Any]) -> str | None:
    raw = env.get('overflow', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw > 1_000_000_000:
        return "USAGE"
    return None

def check_dump_carets_overflow_fits_i32(env: Mapping[str, Any]) -> str | None:
    raw = env.get('overflow', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < -2147483648 or raw > 2147483647:
        return "USAGE"
    return None

def check_dump_carets_overflow_not_bool_int(env: Mapping[str, Any]) -> str | None:
    raw = env.get('overflow', None)
    if isinstance(raw, bool):
        return "USAGE"
    return None

def check_dump_carets_overflow_lte_page_count(env: Mapping[str, Any]) -> str | None:
    raw = env.get('overflow', None)
    total = env.get("pageCount")
    if raw is None or total is None:
        return None
    if isinstance(raw, bool) or isinstance(total, bool):
        return None
    if isinstance(raw, int) and isinstance(total, int) and raw > total:
        return "COORD_OOB"
    return None

def check_dump_carets_overflow_lte_declared(env: Mapping[str, Any]) -> str | None:
    raw = env.get('overflow', None)
    declared = env.get("declaredCount")
    if raw is None or declared is None:
        return None
    if isinstance(raw, bool) or isinstance(declared, bool):
        return None
    if isinstance(raw, int) and isinstance(declared, int) and raw > declared:
        return "COUNT_DRIFT"
    return None

def check_dump_carets_overflow_under_cap(env: Mapping[str, Any]) -> str | None:
    raw = env.get('overflow', None)
    cap = env.get("capBytes")
    if raw is None or cap is None:
        return None
    if isinstance(raw, bool) or isinstance(cap, bool):
        return None
    if isinstance(raw, int) and isinstance(cap, int) and cap > 0 and raw > cap:
        return "OVER_ACCEPTED"
    return None

def check_dump_carets_overflow_year_range(env: Mapping[str, Any]) -> str | None:
    raw = env.get('overflow', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < 1990 or raw > 2100:
        return "USAGE"
    return None

def check_dump_carets_overflow_fail_vs_identical(env: Mapping[str, Any]) -> str | None:
    raw = env.get('overflow', None)
    identical = env.get("identical")
    if raw is None or identical is not True:
        return None
    if isinstance(raw, int) and not isinstance(raw, bool) and raw > 0:
        return "COUNT_DRIFT"
    return None

def check_dump_carets_overflow_ok_vs_total(env: Mapping[str, Any]) -> str | None:
    raw = env.get('overflow', None)
    total = env.get("total")
    if raw is None or total is None:
        return None
    if isinstance(raw, bool) or isinstance(total, bool):
        return None
    if isinstance(raw, int) and isinstance(total, int) and raw > total:
        return "RATE_IMPOSSIBLE"
    return None

def check_dump_carets_overflow_span_fits(env: Mapping[str, Any]) -> str | None:
    raw = env.get('overflow', None)
    rows = env.get("rows")
    cols = env.get("cols")
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if isinstance(rows, int) and not isinstance(rows, bool) and 'overflow' == "rowSpan" and raw > rows:
        return "SPAN_OOB"
    if isinstance(cols, int) and not isinstance(cols, bool) and 'overflow' == "colSpan" and raw > cols:
        return "SPAN_OOB"
    return None

def check_dump_carets_overflow_width_height(env: Mapping[str, Any]) -> str | None:
    raw = env.get('overflow', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if 'overflow' in ("width", "height", "bytes") and raw < 0:
        return "USAGE"
    return None

def check_dump_carets_overflow_batch_parts(env: Mapping[str, Any]) -> str | None:
    raw = env.get('overflow', None)
    n_in = env.get("inputN")
    n_ok = env.get("okN")
    n_fail = env.get("failN")
    if raw is None:
        return None
    if not all(isinstance(v, int) and not isinstance(v, bool) for v in (n_in, n_ok, n_fail) if v is not None):
        return None
    if isinstance(n_in, int) and isinstance(n_ok, int) and isinstance(n_fail, int):
        if n_in != n_ok + n_fail:
            return "COUNT_DRIFT"
    return None

def check_dump_carets_overlap_wrong_type_int(env: Mapping[str, Any]) -> str | None:
    raw = env.get('overlap', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return "USAGE"
    return None

def check_dump_carets_overlap_negative(env: Mapping[str, Any]) -> str | None:
    raw = env.get('overlap', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < 0:
        return "USAGE"
    return None

def check_dump_carets_overlap_nonzero_required(env: Mapping[str, Any]) -> str | None:
    raw = env.get('overlap', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw == 0:
        return "ZERO_BYTES"
    return None

def check_dump_carets_overlap_exit_closed(env: Mapping[str, Any]) -> str | None:
    raw = env.get('overlap', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw not in (0, 1, 2, 3, 4):
        return "EXIT_UNKNOWN"
    return None

def check_dump_carets_overlap_too_large(env: Mapping[str, Any]) -> str | None:
    raw = env.get('overlap', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw > 1_000_000_000:
        return "USAGE"
    return None

def check_dump_carets_overlap_fits_i32(env: Mapping[str, Any]) -> str | None:
    raw = env.get('overlap', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < -2147483648 or raw > 2147483647:
        return "USAGE"
    return None

def check_dump_carets_overlap_not_bool_int(env: Mapping[str, Any]) -> str | None:
    raw = env.get('overlap', None)
    if isinstance(raw, bool):
        return "USAGE"
    return None

def check_dump_carets_overlap_lte_page_count(env: Mapping[str, Any]) -> str | None:
    raw = env.get('overlap', None)
    total = env.get("pageCount")
    if raw is None or total is None:
        return None
    if isinstance(raw, bool) or isinstance(total, bool):
        return None
    if isinstance(raw, int) and isinstance(total, int) and raw > total:
        return "COORD_OOB"
    return None

def check_dump_carets_overlap_lte_declared(env: Mapping[str, Any]) -> str | None:
    raw = env.get('overlap', None)
    declared = env.get("declaredCount")
    if raw is None or declared is None:
        return None
    if isinstance(raw, bool) or isinstance(declared, bool):
        return None
    if isinstance(raw, int) and isinstance(declared, int) and raw > declared:
        return "COUNT_DRIFT"
    return None

def check_dump_carets_overlap_under_cap(env: Mapping[str, Any]) -> str | None:
    raw = env.get('overlap', None)
    cap = env.get("capBytes")
    if raw is None or cap is None:
        return None
    if isinstance(raw, bool) or isinstance(cap, bool):
        return None
    if isinstance(raw, int) and isinstance(cap, int) and cap > 0 and raw > cap:
        return "OVER_ACCEPTED"
    return None

def check_dump_carets_overlap_year_range(env: Mapping[str, Any]) -> str | None:
    raw = env.get('overlap', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < 1990 or raw > 2100:
        return "USAGE"
    return None

def check_dump_carets_overlap_fail_vs_identical(env: Mapping[str, Any]) -> str | None:
    raw = env.get('overlap', None)
    identical = env.get("identical")
    if raw is None or identical is not True:
        return None
    if isinstance(raw, int) and not isinstance(raw, bool) and raw > 0:
        return "COUNT_DRIFT"
    return None

def check_dump_carets_overlap_ok_vs_total(env: Mapping[str, Any]) -> str | None:
    raw = env.get('overlap', None)
    total = env.get("total")
    if raw is None or total is None:
        return None
    if isinstance(raw, bool) or isinstance(total, bool):
        return None
    if isinstance(raw, int) and isinstance(total, int) and raw > total:
        return "RATE_IMPOSSIBLE"
    return None

def check_dump_carets_overlap_span_fits(env: Mapping[str, Any]) -> str | None:
    raw = env.get('overlap', None)
    rows = env.get("rows")
    cols = env.get("cols")
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if isinstance(rows, int) and not isinstance(rows, bool) and 'overlap' == "rowSpan" and raw > rows:
        return "SPAN_OOB"
    if isinstance(cols, int) and not isinstance(cols, bool) and 'overlap' == "colSpan" and raw > cols:
        return "SPAN_OOB"
    return None

def check_dump_carets_overlap_width_height(env: Mapping[str, Any]) -> str | None:
    raw = env.get('overlap', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if 'overlap' in ("width", "height", "bytes") and raw < 0:
        return "USAGE"
    return None

def check_dump_carets_overlap_batch_parts(env: Mapping[str, Any]) -> str | None:
    raw = env.get('overlap', None)
    n_in = env.get("inputN")
    n_ok = env.get("okN")
    n_fail = env.get("failN")
    if raw is None:
        return None
    if not all(isinstance(v, int) and not isinstance(v, bool) for v in (n_in, n_ok, n_fail) if v is not None):
        return None
    if isinstance(n_in, int) and isinstance(n_ok, int) and isinstance(n_fail, int):
        if n_in != n_ok + n_fail:
            return "COUNT_DRIFT"
    return None

def check_dump_carets_diffCount_wrong_type_int(env: Mapping[str, Any]) -> str | None:
    raw = env.get('diffCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return "USAGE"
    return None

def check_dump_carets_diffCount_negative(env: Mapping[str, Any]) -> str | None:
    raw = env.get('diffCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < 0:
        return "USAGE"
    return None

def check_dump_carets_diffCount_nonzero_required(env: Mapping[str, Any]) -> str | None:
    raw = env.get('diffCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw == 0:
        return "ZERO_BYTES"
    return None

def check_dump_carets_diffCount_exit_closed(env: Mapping[str, Any]) -> str | None:
    raw = env.get('diffCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw not in (0, 1, 2, 3, 4):
        return "EXIT_UNKNOWN"
    return None

def check_dump_carets_diffCount_too_large(env: Mapping[str, Any]) -> str | None:
    raw = env.get('diffCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw > 1_000_000_000:
        return "USAGE"
    return None

def check_dump_carets_diffCount_fits_i32(env: Mapping[str, Any]) -> str | None:
    raw = env.get('diffCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < -2147483648 or raw > 2147483647:
        return "USAGE"
    return None

def check_dump_carets_diffCount_not_bool_int(env: Mapping[str, Any]) -> str | None:
    raw = env.get('diffCount', None)
    if isinstance(raw, bool):
        return "USAGE"
    return None

def check_dump_carets_diffCount_lte_page_count(env: Mapping[str, Any]) -> str | None:
    raw = env.get('diffCount', None)
    total = env.get("pageCount")
    if raw is None or total is None:
        return None
    if isinstance(raw, bool) or isinstance(total, bool):
        return None
    if isinstance(raw, int) and isinstance(total, int) and raw > total:
        return "COORD_OOB"
    return None

def check_dump_carets_diffCount_lte_declared(env: Mapping[str, Any]) -> str | None:
    raw = env.get('diffCount', None)
    declared = env.get("declaredCount")
    if raw is None or declared is None:
        return None
    if isinstance(raw, bool) or isinstance(declared, bool):
        return None
    if isinstance(raw, int) and isinstance(declared, int) and raw > declared:
        return "COUNT_DRIFT"
    return None

def check_dump_carets_diffCount_under_cap(env: Mapping[str, Any]) -> str | None:
    raw = env.get('diffCount', None)
    cap = env.get("capBytes")
    if raw is None or cap is None:
        return None
    if isinstance(raw, bool) or isinstance(cap, bool):
        return None
    if isinstance(raw, int) and isinstance(cap, int) and cap > 0 and raw > cap:
        return "OVER_ACCEPTED"
    return None

def check_dump_carets_diffCount_year_range(env: Mapping[str, Any]) -> str | None:
    raw = env.get('diffCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < 1990 or raw > 2100:
        return "USAGE"
    return None

def check_dump_carets_diffCount_fail_vs_identical(env: Mapping[str, Any]) -> str | None:
    raw = env.get('diffCount', None)
    identical = env.get("identical")
    if raw is None or identical is not True:
        return None
    if isinstance(raw, int) and not isinstance(raw, bool) and raw > 0:
        return "COUNT_DRIFT"
    return None

def check_dump_carets_diffCount_ok_vs_total(env: Mapping[str, Any]) -> str | None:
    raw = env.get('diffCount', None)
    total = env.get("total")
    if raw is None or total is None:
        return None
    if isinstance(raw, bool) or isinstance(total, bool):
        return None
    if isinstance(raw, int) and isinstance(total, int) and raw > total:
        return "RATE_IMPOSSIBLE"
    return None

def check_dump_carets_diffCount_span_fits(env: Mapping[str, Any]) -> str | None:
    raw = env.get('diffCount', None)
    rows = env.get("rows")
    cols = env.get("cols")
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if isinstance(rows, int) and not isinstance(rows, bool) and 'diffCount' == "rowSpan" and raw > rows:
        return "SPAN_OOB"
    if isinstance(cols, int) and not isinstance(cols, bool) and 'diffCount' == "colSpan" and raw > cols:
        return "SPAN_OOB"
    return None

def check_dump_carets_diffCount_width_height(env: Mapping[str, Any]) -> str | None:
    raw = env.get('diffCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if 'diffCount' in ("width", "height", "bytes") and raw < 0:
        return "USAGE"
    return None

def check_dump_carets_diffCount_batch_parts(env: Mapping[str, Any]) -> str | None:
    raw = env.get('diffCount', None)
    n_in = env.get("inputN")
    n_ok = env.get("okN")
    n_fail = env.get("failN")
    if raw is None:
        return None
    if not all(isinstance(v, int) and not isinstance(v, bool) for v in (n_in, n_ok, n_fail) if v is not None):
        return None
    if isinstance(n_in, int) and isinstance(n_ok, int) and isinstance(n_fail, int):
        if n_in != n_ok + n_fail:
            return "COUNT_DRIFT"
    return None

def check_dump_carets_pxDelta_wrong_type_int(env: Mapping[str, Any]) -> str | None:
    raw = env.get('pxDelta', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return "USAGE"
    return None

def check_dump_carets_pxDelta_negative(env: Mapping[str, Any]) -> str | None:
    raw = env.get('pxDelta', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < 0:
        return "USAGE"
    return None

def check_dump_carets_pxDelta_nonzero_required(env: Mapping[str, Any]) -> str | None:
    raw = env.get('pxDelta', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw == 0:
        return "ZERO_BYTES"
    return None

def check_dump_carets_pxDelta_exit_closed(env: Mapping[str, Any]) -> str | None:
    raw = env.get('pxDelta', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw not in (0, 1, 2, 3, 4):
        return "EXIT_UNKNOWN"
    return None

def check_dump_carets_pxDelta_too_large(env: Mapping[str, Any]) -> str | None:
    raw = env.get('pxDelta', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw > 1_000_000_000:
        return "USAGE"
    return None

def check_dump_carets_pxDelta_fits_i32(env: Mapping[str, Any]) -> str | None:
    raw = env.get('pxDelta', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < -2147483648 or raw > 2147483647:
        return "USAGE"
    return None

def check_dump_carets_pxDelta_not_bool_int(env: Mapping[str, Any]) -> str | None:
    raw = env.get('pxDelta', None)
    if isinstance(raw, bool):
        return "USAGE"
    return None

def check_dump_carets_pxDelta_lte_page_count(env: Mapping[str, Any]) -> str | None:
    raw = env.get('pxDelta', None)
    total = env.get("pageCount")
    if raw is None or total is None:
        return None
    if isinstance(raw, bool) or isinstance(total, bool):
        return None
    if isinstance(raw, int) and isinstance(total, int) and raw > total:
        return "COORD_OOB"
    return None

def check_dump_carets_pxDelta_lte_declared(env: Mapping[str, Any]) -> str | None:
    raw = env.get('pxDelta', None)
    declared = env.get("declaredCount")
    if raw is None or declared is None:
        return None
    if isinstance(raw, bool) or isinstance(declared, bool):
        return None
    if isinstance(raw, int) and isinstance(declared, int) and raw > declared:
        return "COUNT_DRIFT"
    return None

def check_dump_carets_pxDelta_under_cap(env: Mapping[str, Any]) -> str | None:
    raw = env.get('pxDelta', None)
    cap = env.get("capBytes")
    if raw is None or cap is None:
        return None
    if isinstance(raw, bool) or isinstance(cap, bool):
        return None
    if isinstance(raw, int) and isinstance(cap, int) and cap > 0 and raw > cap:
        return "OVER_ACCEPTED"
    return None

def check_dump_carets_pxDelta_year_range(env: Mapping[str, Any]) -> str | None:
    raw = env.get('pxDelta', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < 1990 or raw > 2100:
        return "USAGE"
    return None

def check_dump_carets_pxDelta_fail_vs_identical(env: Mapping[str, Any]) -> str | None:
    raw = env.get('pxDelta', None)
    identical = env.get("identical")
    if raw is None or identical is not True:
        return None
    if isinstance(raw, int) and not isinstance(raw, bool) and raw > 0:
        return "COUNT_DRIFT"
    return None

def check_dump_carets_pxDelta_ok_vs_total(env: Mapping[str, Any]) -> str | None:
    raw = env.get('pxDelta', None)
    total = env.get("total")
    if raw is None or total is None:
        return None
    if isinstance(raw, bool) or isinstance(total, bool):
        return None
    if isinstance(raw, int) and isinstance(total, int) and raw > total:
        return "RATE_IMPOSSIBLE"
    return None

def check_dump_carets_pxDelta_span_fits(env: Mapping[str, Any]) -> str | None:
    raw = env.get('pxDelta', None)
    rows = env.get("rows")
    cols = env.get("cols")
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if isinstance(rows, int) and not isinstance(rows, bool) and 'pxDelta' == "rowSpan" and raw > rows:
        return "SPAN_OOB"
    if isinstance(cols, int) and not isinstance(cols, bool) and 'pxDelta' == "colSpan" and raw > cols:
        return "SPAN_OOB"
    return None

def check_dump_carets_pxDelta_width_height(env: Mapping[str, Any]) -> str | None:
    raw = env.get('pxDelta', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if 'pxDelta' in ("width", "height", "bytes") and raw < 0:
        return "USAGE"
    return None

def check_dump_carets_pxDelta_batch_parts(env: Mapping[str, Any]) -> str | None:
    raw = env.get('pxDelta', None)
    n_in = env.get("inputN")
    n_ok = env.get("okN")
    n_fail = env.get("failN")
    if raw is None:
        return None
    if not all(isinstance(v, int) and not isinstance(v, bool) for v in (n_in, n_ok, n_fail) if v is not None):
        return None
    if isinstance(n_in, int) and isinstance(n_ok, int) and isinstance(n_fail, int):
        if n_in != n_ok + n_fail:
            return "COUNT_DRIFT"
    return None

def check_dump_carets_threshold_wrong_type_int(env: Mapping[str, Any]) -> str | None:
    raw = env.get('threshold', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return "USAGE"
    return None

def check_dump_carets_threshold_negative(env: Mapping[str, Any]) -> str | None:
    raw = env.get('threshold', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < 0:
        return "USAGE"
    return None

def check_dump_carets_threshold_nonzero_required(env: Mapping[str, Any]) -> str | None:
    raw = env.get('threshold', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw == 0:
        return "ZERO_BYTES"
    return None

def check_dump_carets_threshold_exit_closed(env: Mapping[str, Any]) -> str | None:
    raw = env.get('threshold', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw not in (0, 1, 2, 3, 4):
        return "EXIT_UNKNOWN"
    return None

def check_dump_carets_threshold_too_large(env: Mapping[str, Any]) -> str | None:
    raw = env.get('threshold', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw > 1_000_000_000:
        return "USAGE"
    return None

def check_dump_carets_threshold_fits_i32(env: Mapping[str, Any]) -> str | None:
    raw = env.get('threshold', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < -2147483648 or raw > 2147483647:
        return "USAGE"
    return None

def check_dump_carets_threshold_not_bool_int(env: Mapping[str, Any]) -> str | None:
    raw = env.get('threshold', None)
    if isinstance(raw, bool):
        return "USAGE"
    return None

def check_dump_carets_threshold_lte_page_count(env: Mapping[str, Any]) -> str | None:
    raw = env.get('threshold', None)
    total = env.get("pageCount")
    if raw is None or total is None:
        return None
    if isinstance(raw, bool) or isinstance(total, bool):
        return None
    if isinstance(raw, int) and isinstance(total, int) and raw > total:
        return "COORD_OOB"
    return None

def check_dump_carets_threshold_lte_declared(env: Mapping[str, Any]) -> str | None:
    raw = env.get('threshold', None)
    declared = env.get("declaredCount")
    if raw is None or declared is None:
        return None
    if isinstance(raw, bool) or isinstance(declared, bool):
        return None
    if isinstance(raw, int) and isinstance(declared, int) and raw > declared:
        return "COUNT_DRIFT"
    return None

def check_dump_carets_threshold_under_cap(env: Mapping[str, Any]) -> str | None:
    raw = env.get('threshold', None)
    cap = env.get("capBytes")
    if raw is None or cap is None:
        return None
    if isinstance(raw, bool) or isinstance(cap, bool):
        return None
    if isinstance(raw, int) and isinstance(cap, int) and cap > 0 and raw > cap:
        return "OVER_ACCEPTED"
    return None

def check_dump_carets_threshold_year_range(env: Mapping[str, Any]) -> str | None:
    raw = env.get('threshold', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < 1990 or raw > 2100:
        return "USAGE"
    return None

def check_dump_carets_threshold_fail_vs_identical(env: Mapping[str, Any]) -> str | None:
    raw = env.get('threshold', None)
    identical = env.get("identical")
    if raw is None or identical is not True:
        return None
    if isinstance(raw, int) and not isinstance(raw, bool) and raw > 0:
        return "COUNT_DRIFT"
    return None

def check_dump_carets_threshold_ok_vs_total(env: Mapping[str, Any]) -> str | None:
    raw = env.get('threshold', None)
    total = env.get("total")
    if raw is None or total is None:
        return None
    if isinstance(raw, bool) or isinstance(total, bool):
        return None
    if isinstance(raw, int) and isinstance(total, int) and raw > total:
        return "RATE_IMPOSSIBLE"
    return None

def check_dump_carets_threshold_span_fits(env: Mapping[str, Any]) -> str | None:
    raw = env.get('threshold', None)
    rows = env.get("rows")
    cols = env.get("cols")
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if isinstance(rows, int) and not isinstance(rows, bool) and 'threshold' == "rowSpan" and raw > rows:
        return "SPAN_OOB"
    if isinstance(cols, int) and not isinstance(cols, bool) and 'threshold' == "colSpan" and raw > cols:
        return "SPAN_OOB"
    return None

def check_dump_carets_threshold_width_height(env: Mapping[str, Any]) -> str | None:
    raw = env.get('threshold', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if 'threshold' in ("width", "height", "bytes") and raw < 0:
        return "USAGE"
    return None

def check_dump_carets_threshold_batch_parts(env: Mapping[str, Any]) -> str | None:
    raw = env.get('threshold', None)
    n_in = env.get("inputN")
    n_ok = env.get("okN")
    n_fail = env.get("failN")
    if raw is None:
        return None
    if not all(isinstance(v, int) and not isinstance(v, bool) for v in (n_in, n_ok, n_fail) if v is not None):
        return None
    if isinstance(n_in, int) and isinstance(n_ok, int) and isinstance(n_fail, int):
        if n_in != n_ok + n_fail:
            return "COUNT_DRIFT"
    return None

def check_dump_carets_written_wrong_type_int(env: Mapping[str, Any]) -> str | None:
    raw = env.get('written', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return "USAGE"
    return None

def check_dump_carets_written_negative(env: Mapping[str, Any]) -> str | None:
    raw = env.get('written', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < 0:
        return "USAGE"
    return None

def check_dump_carets_written_nonzero_required(env: Mapping[str, Any]) -> str | None:
    raw = env.get('written', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw == 0:
        return "ZERO_BYTES"
    return None

def check_dump_carets_written_exit_closed(env: Mapping[str, Any]) -> str | None:
    raw = env.get('written', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw not in (0, 1, 2, 3, 4):
        return "EXIT_UNKNOWN"
    return None

def check_dump_carets_written_too_large(env: Mapping[str, Any]) -> str | None:
    raw = env.get('written', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw > 1_000_000_000:
        return "USAGE"
    return None

def check_dump_carets_written_fits_i32(env: Mapping[str, Any]) -> str | None:
    raw = env.get('written', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < -2147483648 or raw > 2147483647:
        return "USAGE"
    return None

def check_dump_carets_written_not_bool_int(env: Mapping[str, Any]) -> str | None:
    raw = env.get('written', None)
    if isinstance(raw, bool):
        return "USAGE"
    return None

def check_dump_carets_written_lte_page_count(env: Mapping[str, Any]) -> str | None:
    raw = env.get('written', None)
    total = env.get("pageCount")
    if raw is None or total is None:
        return None
    if isinstance(raw, bool) or isinstance(total, bool):
        return None
    if isinstance(raw, int) and isinstance(total, int) and raw > total:
        return "COORD_OOB"
    return None

def check_dump_carets_written_lte_declared(env: Mapping[str, Any]) -> str | None:
    raw = env.get('written', None)
    declared = env.get("declaredCount")
    if raw is None or declared is None:
        return None
    if isinstance(raw, bool) or isinstance(declared, bool):
        return None
    if isinstance(raw, int) and isinstance(declared, int) and raw > declared:
        return "COUNT_DRIFT"
    return None

def check_dump_carets_written_under_cap(env: Mapping[str, Any]) -> str | None:
    raw = env.get('written', None)
    cap = env.get("capBytes")
    if raw is None or cap is None:
        return None
    if isinstance(raw, bool) or isinstance(cap, bool):
        return None
    if isinstance(raw, int) and isinstance(cap, int) and cap > 0 and raw > cap:
        return "OVER_ACCEPTED"
    return None

def check_dump_carets_written_year_range(env: Mapping[str, Any]) -> str | None:
    raw = env.get('written', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < 1990 or raw > 2100:
        return "USAGE"
    return None

def check_dump_carets_written_fail_vs_identical(env: Mapping[str, Any]) -> str | None:
    raw = env.get('written', None)
    identical = env.get("identical")
    if raw is None or identical is not True:
        return None
    if isinstance(raw, int) and not isinstance(raw, bool) and raw > 0:
        return "COUNT_DRIFT"
    return None

def check_dump_carets_written_ok_vs_total(env: Mapping[str, Any]) -> str | None:
    raw = env.get('written', None)
    total = env.get("total")
    if raw is None or total is None:
        return None
    if isinstance(raw, bool) or isinstance(total, bool):
        return None
    if isinstance(raw, int) and isinstance(total, int) and raw > total:
        return "RATE_IMPOSSIBLE"
    return None

def check_dump_carets_written_span_fits(env: Mapping[str, Any]) -> str | None:
    raw = env.get('written', None)
    rows = env.get("rows")
    cols = env.get("cols")
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if isinstance(rows, int) and not isinstance(rows, bool) and 'written' == "rowSpan" and raw > rows:
        return "SPAN_OOB"
    if isinstance(cols, int) and not isinstance(cols, bool) and 'written' == "colSpan" and raw > cols:
        return "SPAN_OOB"
    return None

def check_dump_carets_written_width_height(env: Mapping[str, Any]) -> str | None:
    raw = env.get('written', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if 'written' in ("width", "height", "bytes") and raw < 0:
        return "USAGE"
    return None

def check_dump_carets_written_batch_parts(env: Mapping[str, Any]) -> str | None:
    raw = env.get('written', None)
    n_in = env.get("inputN")
    n_ok = env.get("okN")
    n_fail = env.get("failN")
    if raw is None:
        return None
    if not all(isinstance(v, int) and not isinstance(v, bool) for v in (n_in, n_ok, n_fail) if v is not None):
        return None
    if isinstance(n_in, int) and isinstance(n_ok, int) and isinstance(n_fail, int):
        if n_in != n_ok + n_fail:
            return "COUNT_DRIFT"
    return None

def check_dump_carets_reread_wrong_type_int(env: Mapping[str, Any]) -> str | None:
    raw = env.get('reread', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return "USAGE"
    return None

def check_dump_carets_reread_negative(env: Mapping[str, Any]) -> str | None:
    raw = env.get('reread', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < 0:
        return "USAGE"
    return None

def check_dump_carets_reread_nonzero_required(env: Mapping[str, Any]) -> str | None:
    raw = env.get('reread', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw == 0:
        return "ZERO_BYTES"
    return None

def check_dump_carets_reread_exit_closed(env: Mapping[str, Any]) -> str | None:
    raw = env.get('reread', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw not in (0, 1, 2, 3, 4):
        return "EXIT_UNKNOWN"
    return None

def check_dump_carets_reread_too_large(env: Mapping[str, Any]) -> str | None:
    raw = env.get('reread', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw > 1_000_000_000:
        return "USAGE"
    return None

def check_dump_carets_reread_fits_i32(env: Mapping[str, Any]) -> str | None:
    raw = env.get('reread', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < -2147483648 or raw > 2147483647:
        return "USAGE"
    return None

def check_dump_carets_reread_not_bool_int(env: Mapping[str, Any]) -> str | None:
    raw = env.get('reread', None)
    if isinstance(raw, bool):
        return "USAGE"
    return None

def check_dump_carets_reread_lte_page_count(env: Mapping[str, Any]) -> str | None:
    raw = env.get('reread', None)
    total = env.get("pageCount")
    if raw is None or total is None:
        return None
    if isinstance(raw, bool) or isinstance(total, bool):
        return None
    if isinstance(raw, int) and isinstance(total, int) and raw > total:
        return "COORD_OOB"
    return None

def check_dump_carets_reread_lte_declared(env: Mapping[str, Any]) -> str | None:
    raw = env.get('reread', None)
    declared = env.get("declaredCount")
    if raw is None or declared is None:
        return None
    if isinstance(raw, bool) or isinstance(declared, bool):
        return None
    if isinstance(raw, int) and isinstance(declared, int) and raw > declared:
        return "COUNT_DRIFT"
    return None

def check_dump_carets_reread_under_cap(env: Mapping[str, Any]) -> str | None:
    raw = env.get('reread', None)
    cap = env.get("capBytes")
    if raw is None or cap is None:
        return None
    if isinstance(raw, bool) or isinstance(cap, bool):
        return None
    if isinstance(raw, int) and isinstance(cap, int) and cap > 0 and raw > cap:
        return "OVER_ACCEPTED"
    return None

def check_dump_carets_reread_year_range(env: Mapping[str, Any]) -> str | None:
    raw = env.get('reread', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < 1990 or raw > 2100:
        return "USAGE"
    return None

def check_dump_carets_reread_fail_vs_identical(env: Mapping[str, Any]) -> str | None:
    raw = env.get('reread', None)
    identical = env.get("identical")
    if raw is None or identical is not True:
        return None
    if isinstance(raw, int) and not isinstance(raw, bool) and raw > 0:
        return "COUNT_DRIFT"
    return None

def check_dump_carets_reread_ok_vs_total(env: Mapping[str, Any]) -> str | None:
    raw = env.get('reread', None)
    total = env.get("total")
    if raw is None or total is None:
        return None
    if isinstance(raw, bool) or isinstance(total, bool):
        return None
    if isinstance(raw, int) and isinstance(total, int) and raw > total:
        return "RATE_IMPOSSIBLE"
    return None

def check_dump_carets_reread_span_fits(env: Mapping[str, Any]) -> str | None:
    raw = env.get('reread', None)
    rows = env.get("rows")
    cols = env.get("cols")
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if isinstance(rows, int) and not isinstance(rows, bool) and 'reread' == "rowSpan" and raw > rows:
        return "SPAN_OOB"
    if isinstance(cols, int) and not isinstance(cols, bool) and 'reread' == "colSpan" and raw > cols:
        return "SPAN_OOB"
    return None

def check_dump_carets_reread_width_height(env: Mapping[str, Any]) -> str | None:
    raw = env.get('reread', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if 'reread' in ("width", "height", "bytes") and raw < 0:
        return "USAGE"
    return None

def check_dump_carets_reread_batch_parts(env: Mapping[str, Any]) -> str | None:
    raw = env.get('reread', None)
    n_in = env.get("inputN")
    n_ok = env.get("okN")
    n_fail = env.get("failN")
    if raw is None:
        return None
    if not all(isinstance(v, int) and not isinstance(v, bool) for v in (n_in, n_ok, n_fail) if v is not None):
        return None
    if isinstance(n_in, int) and isinstance(n_ok, int) and isinstance(n_fail, int):
        if n_in != n_ok + n_fail:
            return "COUNT_DRIFT"
    return None

def check_dump_carets_beforeCount_wrong_type_int(env: Mapping[str, Any]) -> str | None:
    raw = env.get('beforeCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return "USAGE"
    return None

def check_dump_carets_beforeCount_negative(env: Mapping[str, Any]) -> str | None:
    raw = env.get('beforeCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < 0:
        return "USAGE"
    return None

def check_dump_carets_beforeCount_nonzero_required(env: Mapping[str, Any]) -> str | None:
    raw = env.get('beforeCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw == 0:
        return "ZERO_BYTES"
    return None

def check_dump_carets_beforeCount_exit_closed(env: Mapping[str, Any]) -> str | None:
    raw = env.get('beforeCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw not in (0, 1, 2, 3, 4):
        return "EXIT_UNKNOWN"
    return None

def check_dump_carets_beforeCount_too_large(env: Mapping[str, Any]) -> str | None:
    raw = env.get('beforeCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw > 1_000_000_000:
        return "USAGE"
    return None

def check_dump_carets_beforeCount_fits_i32(env: Mapping[str, Any]) -> str | None:
    raw = env.get('beforeCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < -2147483648 or raw > 2147483647:
        return "USAGE"
    return None

def check_dump_carets_beforeCount_not_bool_int(env: Mapping[str, Any]) -> str | None:
    raw = env.get('beforeCount', None)
    if isinstance(raw, bool):
        return "USAGE"
    return None

def check_dump_carets_beforeCount_lte_page_count(env: Mapping[str, Any]) -> str | None:
    raw = env.get('beforeCount', None)
    total = env.get("pageCount")
    if raw is None or total is None:
        return None
    if isinstance(raw, bool) or isinstance(total, bool):
        return None
    if isinstance(raw, int) and isinstance(total, int) and raw > total:
        return "COORD_OOB"
    return None

def check_dump_carets_beforeCount_lte_declared(env: Mapping[str, Any]) -> str | None:
    raw = env.get('beforeCount', None)
    declared = env.get("declaredCount")
    if raw is None or declared is None:
        return None
    if isinstance(raw, bool) or isinstance(declared, bool):
        return None
    if isinstance(raw, int) and isinstance(declared, int) and raw > declared:
        return "COUNT_DRIFT"
    return None

def check_dump_carets_beforeCount_under_cap(env: Mapping[str, Any]) -> str | None:
    raw = env.get('beforeCount', None)
    cap = env.get("capBytes")
    if raw is None or cap is None:
        return None
    if isinstance(raw, bool) or isinstance(cap, bool):
        return None
    if isinstance(raw, int) and isinstance(cap, int) and cap > 0 and raw > cap:
        return "OVER_ACCEPTED"
    return None

def check_dump_carets_beforeCount_year_range(env: Mapping[str, Any]) -> str | None:
    raw = env.get('beforeCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < 1990 or raw > 2100:
        return "USAGE"
    return None

def check_dump_carets_beforeCount_fail_vs_identical(env: Mapping[str, Any]) -> str | None:
    raw = env.get('beforeCount', None)
    identical = env.get("identical")
    if raw is None or identical is not True:
        return None
    if isinstance(raw, int) and not isinstance(raw, bool) and raw > 0:
        return "COUNT_DRIFT"
    return None

def check_dump_carets_beforeCount_ok_vs_total(env: Mapping[str, Any]) -> str | None:
    raw = env.get('beforeCount', None)
    total = env.get("total")
    if raw is None or total is None:
        return None
    if isinstance(raw, bool) or isinstance(total, bool):
        return None
    if isinstance(raw, int) and isinstance(total, int) and raw > total:
        return "RATE_IMPOSSIBLE"
    return None

def check_dump_carets_beforeCount_span_fits(env: Mapping[str, Any]) -> str | None:
    raw = env.get('beforeCount', None)
    rows = env.get("rows")
    cols = env.get("cols")
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if isinstance(rows, int) and not isinstance(rows, bool) and 'beforeCount' == "rowSpan" and raw > rows:
        return "SPAN_OOB"
    if isinstance(cols, int) and not isinstance(cols, bool) and 'beforeCount' == "colSpan" and raw > cols:
        return "SPAN_OOB"
    return None

def check_dump_carets_beforeCount_width_height(env: Mapping[str, Any]) -> str | None:
    raw = env.get('beforeCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if 'beforeCount' in ("width", "height", "bytes") and raw < 0:
        return "USAGE"
    return None

def check_dump_carets_beforeCount_batch_parts(env: Mapping[str, Any]) -> str | None:
    raw = env.get('beforeCount', None)
    n_in = env.get("inputN")
    n_ok = env.get("okN")
    n_fail = env.get("failN")
    if raw is None:
        return None
    if not all(isinstance(v, int) and not isinstance(v, bool) for v in (n_in, n_ok, n_fail) if v is not None):
        return None
    if isinstance(n_in, int) and isinstance(n_ok, int) and isinstance(n_fail, int):
        if n_in != n_ok + n_fail:
            return "COUNT_DRIFT"
    return None

def check_dump_carets_afterCount_wrong_type_int(env: Mapping[str, Any]) -> str | None:
    raw = env.get('afterCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return "USAGE"
    return None

def check_dump_carets_afterCount_negative(env: Mapping[str, Any]) -> str | None:
    raw = env.get('afterCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < 0:
        return "USAGE"
    return None

def check_dump_carets_afterCount_nonzero_required(env: Mapping[str, Any]) -> str | None:
    raw = env.get('afterCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw == 0:
        return "ZERO_BYTES"
    return None

def check_dump_carets_afterCount_exit_closed(env: Mapping[str, Any]) -> str | None:
    raw = env.get('afterCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw not in (0, 1, 2, 3, 4):
        return "EXIT_UNKNOWN"
    return None

def check_dump_carets_afterCount_too_large(env: Mapping[str, Any]) -> str | None:
    raw = env.get('afterCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw > 1_000_000_000:
        return "USAGE"
    return None

def check_dump_carets_afterCount_fits_i32(env: Mapping[str, Any]) -> str | None:
    raw = env.get('afterCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < -2147483648 or raw > 2147483647:
        return "USAGE"
    return None

def check_dump_carets_afterCount_not_bool_int(env: Mapping[str, Any]) -> str | None:
    raw = env.get('afterCount', None)
    if isinstance(raw, bool):
        return "USAGE"
    return None

def check_dump_carets_afterCount_lte_page_count(env: Mapping[str, Any]) -> str | None:
    raw = env.get('afterCount', None)
    total = env.get("pageCount")
    if raw is None or total is None:
        return None
    if isinstance(raw, bool) or isinstance(total, bool):
        return None
    if isinstance(raw, int) and isinstance(total, int) and raw > total:
        return "COORD_OOB"
    return None

def check_dump_carets_afterCount_lte_declared(env: Mapping[str, Any]) -> str | None:
    raw = env.get('afterCount', None)
    declared = env.get("declaredCount")
    if raw is None or declared is None:
        return None
    if isinstance(raw, bool) or isinstance(declared, bool):
        return None
    if isinstance(raw, int) and isinstance(declared, int) and raw > declared:
        return "COUNT_DRIFT"
    return None

def check_dump_carets_afterCount_under_cap(env: Mapping[str, Any]) -> str | None:
    raw = env.get('afterCount', None)
    cap = env.get("capBytes")
    if raw is None or cap is None:
        return None
    if isinstance(raw, bool) or isinstance(cap, bool):
        return None
    if isinstance(raw, int) and isinstance(cap, int) and cap > 0 and raw > cap:
        return "OVER_ACCEPTED"
    return None

def check_dump_carets_afterCount_year_range(env: Mapping[str, Any]) -> str | None:
    raw = env.get('afterCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < 1990 or raw > 2100:
        return "USAGE"
    return None

def check_dump_carets_afterCount_fail_vs_identical(env: Mapping[str, Any]) -> str | None:
    raw = env.get('afterCount', None)
    identical = env.get("identical")
    if raw is None or identical is not True:
        return None
    if isinstance(raw, int) and not isinstance(raw, bool) and raw > 0:
        return "COUNT_DRIFT"
    return None

def check_dump_carets_afterCount_ok_vs_total(env: Mapping[str, Any]) -> str | None:
    raw = env.get('afterCount', None)
    total = env.get("total")
    if raw is None or total is None:
        return None
    if isinstance(raw, bool) or isinstance(total, bool):
        return None
    if isinstance(raw, int) and isinstance(total, int) and raw > total:
        return "RATE_IMPOSSIBLE"
    return None

def check_dump_carets_afterCount_span_fits(env: Mapping[str, Any]) -> str | None:
    raw = env.get('afterCount', None)
    rows = env.get("rows")
    cols = env.get("cols")
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if isinstance(rows, int) and not isinstance(rows, bool) and 'afterCount' == "rowSpan" and raw > rows:
        return "SPAN_OOB"
    if isinstance(cols, int) and not isinstance(cols, bool) and 'afterCount' == "colSpan" and raw > cols:
        return "SPAN_OOB"
    return None

def check_dump_carets_afterCount_width_height(env: Mapping[str, Any]) -> str | None:
    raw = env.get('afterCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if 'afterCount' in ("width", "height", "bytes") and raw < 0:
        return "USAGE"
    return None

def check_dump_carets_afterCount_batch_parts(env: Mapping[str, Any]) -> str | None:
    raw = env.get('afterCount', None)
    n_in = env.get("inputN")
    n_ok = env.get("okN")
    n_fail = env.get("failN")
    if raw is None:
        return None
    if not all(isinstance(v, int) and not isinstance(v, bool) for v in (n_in, n_ok, n_fail) if v is not None):
        return None
    if isinstance(n_in, int) and isinstance(n_ok, int) and isinstance(n_fail, int):
        if n_in != n_ok + n_fail:
            return "COUNT_DRIFT"
    return None

def check_dump_carets_ok_wrong_type_int(env: Mapping[str, Any]) -> str | None:
    raw = env.get('ok', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return "USAGE"
    return None

def check_dump_carets_ok_negative(env: Mapping[str, Any]) -> str | None:
    raw = env.get('ok', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < 0:
        return "USAGE"
    return None

def check_dump_carets_ok_nonzero_required(env: Mapping[str, Any]) -> str | None:
    raw = env.get('ok', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw == 0:
        return "ZERO_BYTES"
    return None

def check_dump_carets_ok_exit_closed(env: Mapping[str, Any]) -> str | None:
    raw = env.get('ok', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw not in (0, 1, 2, 3, 4):
        return "EXIT_UNKNOWN"
    return None

def check_dump_carets_ok_too_large(env: Mapping[str, Any]) -> str | None:
    raw = env.get('ok', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw > 1_000_000_000:
        return "USAGE"
    return None

def check_dump_carets_ok_fits_i32(env: Mapping[str, Any]) -> str | None:
    raw = env.get('ok', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < -2147483648 or raw > 2147483647:
        return "USAGE"
    return None

def check_dump_carets_ok_not_bool_int(env: Mapping[str, Any]) -> str | None:
    raw = env.get('ok', None)
    if isinstance(raw, bool):
        return "USAGE"
    return None

def check_dump_carets_ok_lte_page_count(env: Mapping[str, Any]) -> str | None:
    raw = env.get('ok', None)
    total = env.get("pageCount")
    if raw is None or total is None:
        return None
    if isinstance(raw, bool) or isinstance(total, bool):
        return None
    if isinstance(raw, int) and isinstance(total, int) and raw > total:
        return "COORD_OOB"
    return None

def check_dump_carets_ok_lte_declared(env: Mapping[str, Any]) -> str | None:
    raw = env.get('ok', None)
    declared = env.get("declaredCount")
    if raw is None or declared is None:
        return None
    if isinstance(raw, bool) or isinstance(declared, bool):
        return None
    if isinstance(raw, int) and isinstance(declared, int) and raw > declared:
        return "COUNT_DRIFT"
    return None

def check_dump_carets_ok_under_cap(env: Mapping[str, Any]) -> str | None:
    raw = env.get('ok', None)
    cap = env.get("capBytes")
    if raw is None or cap is None:
        return None
    if isinstance(raw, bool) or isinstance(cap, bool):
        return None
    if isinstance(raw, int) and isinstance(cap, int) and cap > 0 and raw > cap:
        return "OVER_ACCEPTED"
    return None

def check_dump_carets_ok_year_range(env: Mapping[str, Any]) -> str | None:
    raw = env.get('ok', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < 1990 or raw > 2100:
        return "USAGE"
    return None

def check_dump_carets_ok_fail_vs_identical(env: Mapping[str, Any]) -> str | None:
    raw = env.get('ok', None)
    identical = env.get("identical")
    if raw is None or identical is not True:
        return None
    if isinstance(raw, int) and not isinstance(raw, bool) and raw > 0:
        return "COUNT_DRIFT"
    return None

def check_dump_carets_ok_ok_vs_total(env: Mapping[str, Any]) -> str | None:
    raw = env.get('ok', None)
    total = env.get("total")
    if raw is None or total is None:
        return None
    if isinstance(raw, bool) or isinstance(total, bool):
        return None
    if isinstance(raw, int) and isinstance(total, int) and raw > total:
        return "RATE_IMPOSSIBLE"
    return None

def check_dump_carets_ok_span_fits(env: Mapping[str, Any]) -> str | None:
    raw = env.get('ok', None)
    rows = env.get("rows")
    cols = env.get("cols")
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if isinstance(rows, int) and not isinstance(rows, bool) and 'ok' == "rowSpan" and raw > rows:
        return "SPAN_OOB"
    if isinstance(cols, int) and not isinstance(cols, bool) and 'ok' == "colSpan" and raw > cols:
        return "SPAN_OOB"
    return None

def check_dump_carets_ok_width_height(env: Mapping[str, Any]) -> str | None:
    raw = env.get('ok', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if 'ok' in ("width", "height", "bytes") and raw < 0:
        return "USAGE"
    return None

def check_dump_carets_ok_batch_parts(env: Mapping[str, Any]) -> str | None:
    raw = env.get('ok', None)
    n_in = env.get("inputN")
    n_ok = env.get("okN")
    n_fail = env.get("failN")
    if raw is None:
        return None
    if not all(isinstance(v, int) and not isinstance(v, bool) for v in (n_in, n_ok, n_fail) if v is not None):
        return None
    if isinstance(n_in, int) and isinstance(n_ok, int) and isinstance(n_fail, int):
        if n_in != n_ok + n_fail:
            return "COUNT_DRIFT"
    return None

def check_dump_carets_total_wrong_type_int(env: Mapping[str, Any]) -> str | None:
    raw = env.get('total', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return "USAGE"
    return None

def check_dump_carets_total_negative(env: Mapping[str, Any]) -> str | None:
    raw = env.get('total', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < 0:
        return "USAGE"
    return None

def check_dump_carets_total_nonzero_required(env: Mapping[str, Any]) -> str | None:
    raw = env.get('total', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw == 0:
        return "ZERO_BYTES"
    return None

def check_dump_carets_total_exit_closed(env: Mapping[str, Any]) -> str | None:
    raw = env.get('total', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw not in (0, 1, 2, 3, 4):
        return "EXIT_UNKNOWN"
    return None

def check_dump_carets_total_too_large(env: Mapping[str, Any]) -> str | None:
    raw = env.get('total', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw > 1_000_000_000:
        return "USAGE"
    return None

def check_dump_carets_total_fits_i32(env: Mapping[str, Any]) -> str | None:
    raw = env.get('total', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < -2147483648 or raw > 2147483647:
        return "USAGE"
    return None

def check_dump_carets_total_not_bool_int(env: Mapping[str, Any]) -> str | None:
    raw = env.get('total', None)
    if isinstance(raw, bool):
        return "USAGE"
    return None

def check_dump_carets_total_lte_page_count(env: Mapping[str, Any]) -> str | None:
    raw = env.get('total', None)
    total = env.get("pageCount")
    if raw is None or total is None:
        return None
    if isinstance(raw, bool) or isinstance(total, bool):
        return None
    if isinstance(raw, int) and isinstance(total, int) and raw > total:
        return "COORD_OOB"
    return None

def check_dump_carets_total_lte_declared(env: Mapping[str, Any]) -> str | None:
    raw = env.get('total', None)
    declared = env.get("declaredCount")
    if raw is None or declared is None:
        return None
    if isinstance(raw, bool) or isinstance(declared, bool):
        return None
    if isinstance(raw, int) and isinstance(declared, int) and raw > declared:
        return "COUNT_DRIFT"
    return None

def check_dump_carets_total_under_cap(env: Mapping[str, Any]) -> str | None:
    raw = env.get('total', None)
    cap = env.get("capBytes")
    if raw is None or cap is None:
        return None
    if isinstance(raw, bool) or isinstance(cap, bool):
        return None
    if isinstance(raw, int) and isinstance(cap, int) and cap > 0 and raw > cap:
        return "OVER_ACCEPTED"
    return None

def check_dump_carets_total_year_range(env: Mapping[str, Any]) -> str | None:
    raw = env.get('total', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < 1990 or raw > 2100:
        return "USAGE"
    return None

def check_dump_carets_total_fail_vs_identical(env: Mapping[str, Any]) -> str | None:
    raw = env.get('total', None)
    identical = env.get("identical")
    if raw is None or identical is not True:
        return None
    if isinstance(raw, int) and not isinstance(raw, bool) and raw > 0:
        return "COUNT_DRIFT"
    return None

def check_dump_carets_total_ok_vs_total(env: Mapping[str, Any]) -> str | None:
    raw = env.get('total', None)
    total = env.get("total")
    if raw is None or total is None:
        return None
    if isinstance(raw, bool) or isinstance(total, bool):
        return None
    if isinstance(raw, int) and isinstance(total, int) and raw > total:
        return "RATE_IMPOSSIBLE"
    return None

def check_dump_carets_total_span_fits(env: Mapping[str, Any]) -> str | None:
    raw = env.get('total', None)
    rows = env.get("rows")
    cols = env.get("cols")
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if isinstance(rows, int) and not isinstance(rows, bool) and 'total' == "rowSpan" and raw > rows:
        return "SPAN_OOB"
    if isinstance(cols, int) and not isinstance(cols, bool) and 'total' == "colSpan" and raw > cols:
        return "SPAN_OOB"
    return None

def check_dump_carets_total_width_height(env: Mapping[str, Any]) -> str | None:
    raw = env.get('total', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if 'total' in ("width", "height", "bytes") and raw < 0:
        return "USAGE"
    return None

def check_dump_carets_total_batch_parts(env: Mapping[str, Any]) -> str | None:
    raw = env.get('total', None)
    n_in = env.get("inputN")
    n_ok = env.get("okN")
    n_fail = env.get("failN")
    if raw is None:
        return None
    if not all(isinstance(v, int) and not isinstance(v, bool) for v in (n_in, n_ok, n_fail) if v is not None):
        return None
    if isinstance(n_in, int) and isinstance(n_ok, int) and isinstance(n_fail, int):
        if n_in != n_ok + n_fail:
            return "COUNT_DRIFT"
    return None

def check_dump_carets_hangulYear_wrong_type_int(env: Mapping[str, Any]) -> str | None:
    raw = env.get('hangulYear', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return "USAGE"
    return None

def check_dump_carets_hangulYear_negative(env: Mapping[str, Any]) -> str | None:
    raw = env.get('hangulYear', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < 0:
        return "USAGE"
    return None

def check_dump_carets_hangulYear_nonzero_required(env: Mapping[str, Any]) -> str | None:
    raw = env.get('hangulYear', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw == 0:
        return "ZERO_BYTES"
    return None

def check_dump_carets_hangulYear_exit_closed(env: Mapping[str, Any]) -> str | None:
    raw = env.get('hangulYear', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw not in (0, 1, 2, 3, 4):
        return "EXIT_UNKNOWN"
    return None

def check_dump_carets_hangulYear_too_large(env: Mapping[str, Any]) -> str | None:
    raw = env.get('hangulYear', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw > 1_000_000_000:
        return "USAGE"
    return None

def check_dump_carets_hangulYear_fits_i32(env: Mapping[str, Any]) -> str | None:
    raw = env.get('hangulYear', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < -2147483648 or raw > 2147483647:
        return "USAGE"
    return None

def check_dump_carets_hangulYear_not_bool_int(env: Mapping[str, Any]) -> str | None:
    raw = env.get('hangulYear', None)
    if isinstance(raw, bool):
        return "USAGE"
    return None

def check_dump_carets_hangulYear_lte_page_count(env: Mapping[str, Any]) -> str | None:
    raw = env.get('hangulYear', None)
    total = env.get("pageCount")
    if raw is None or total is None:
        return None
    if isinstance(raw, bool) or isinstance(total, bool):
        return None
    if isinstance(raw, int) and isinstance(total, int) and raw > total:
        return "COORD_OOB"
    return None

def check_dump_carets_hangulYear_lte_declared(env: Mapping[str, Any]) -> str | None:
    raw = env.get('hangulYear', None)
    declared = env.get("declaredCount")
    if raw is None or declared is None:
        return None
    if isinstance(raw, bool) or isinstance(declared, bool):
        return None
    if isinstance(raw, int) and isinstance(declared, int) and raw > declared:
        return "COUNT_DRIFT"
    return None

def check_dump_carets_hangulYear_under_cap(env: Mapping[str, Any]) -> str | None:
    raw = env.get('hangulYear', None)
    cap = env.get("capBytes")
    if raw is None or cap is None:
        return None
    if isinstance(raw, bool) or isinstance(cap, bool):
        return None
    if isinstance(raw, int) and isinstance(cap, int) and cap > 0 and raw > cap:
        return "OVER_ACCEPTED"
    return None

def check_dump_carets_hangulYear_year_range(env: Mapping[str, Any]) -> str | None:
    raw = env.get('hangulYear', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < 1990 or raw > 2100:
        return "USAGE"
    return None

def check_dump_carets_hangulYear_fail_vs_identical(env: Mapping[str, Any]) -> str | None:
    raw = env.get('hangulYear', None)
    identical = env.get("identical")
    if raw is None or identical is not True:
        return None
    if isinstance(raw, int) and not isinstance(raw, bool) and raw > 0:
        return "COUNT_DRIFT"
    return None

def check_dump_carets_hangulYear_ok_vs_total(env: Mapping[str, Any]) -> str | None:
    raw = env.get('hangulYear', None)
    total = env.get("total")
    if raw is None or total is None:
        return None
    if isinstance(raw, bool) or isinstance(total, bool):
        return None
    if isinstance(raw, int) and isinstance(total, int) and raw > total:
        return "RATE_IMPOSSIBLE"
    return None

def check_dump_carets_hangulYear_span_fits(env: Mapping[str, Any]) -> str | None:
    raw = env.get('hangulYear', None)
    rows = env.get("rows")
    cols = env.get("cols")
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if isinstance(rows, int) and not isinstance(rows, bool) and 'hangulYear' == "rowSpan" and raw > rows:
        return "SPAN_OOB"
    if isinstance(cols, int) and not isinstance(cols, bool) and 'hangulYear' == "colSpan" and raw > cols:
        return "SPAN_OOB"
    return None

def check_dump_carets_hangulYear_width_height(env: Mapping[str, Any]) -> str | None:
    raw = env.get('hangulYear', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if 'hangulYear' in ("width", "height", "bytes") and raw < 0:
        return "USAGE"
    return None

def check_dump_carets_hangulYear_batch_parts(env: Mapping[str, Any]) -> str | None:
    raw = env.get('hangulYear', None)
    n_in = env.get("inputN")
    n_ok = env.get("okN")
    n_fail = env.get("failN")
    if raw is None:
        return None
    if not all(isinstance(v, int) and not isinstance(v, bool) for v in (n_in, n_ok, n_fail) if v is not None):
        return None
    if isinstance(n_in, int) and isinstance(n_ok, int) and isinstance(n_fail, int):
        if n_in != n_ok + n_fail:
            return "COUNT_DRIFT"
    return None

def check_dump_carets_sizeBytes_wrong_type_int(env: Mapping[str, Any]) -> str | None:
    raw = env.get('sizeBytes', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return "USAGE"
    return None

def check_dump_carets_sizeBytes_negative(env: Mapping[str, Any]) -> str | None:
    raw = env.get('sizeBytes', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < 0:
        return "USAGE"
    return None

def check_dump_carets_sizeBytes_nonzero_required(env: Mapping[str, Any]) -> str | None:
    raw = env.get('sizeBytes', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw == 0:
        return "ZERO_BYTES"
    return None

def check_dump_carets_sizeBytes_exit_closed(env: Mapping[str, Any]) -> str | None:
    raw = env.get('sizeBytes', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw not in (0, 1, 2, 3, 4):
        return "EXIT_UNKNOWN"
    return None

def check_dump_carets_sizeBytes_too_large(env: Mapping[str, Any]) -> str | None:
    raw = env.get('sizeBytes', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw > 1_000_000_000:
        return "USAGE"
    return None

def check_dump_carets_sizeBytes_fits_i32(env: Mapping[str, Any]) -> str | None:
    raw = env.get('sizeBytes', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < -2147483648 or raw > 2147483647:
        return "USAGE"
    return None

def check_dump_carets_sizeBytes_not_bool_int(env: Mapping[str, Any]) -> str | None:
    raw = env.get('sizeBytes', None)
    if isinstance(raw, bool):
        return "USAGE"
    return None

def check_dump_carets_sizeBytes_lte_page_count(env: Mapping[str, Any]) -> str | None:
    raw = env.get('sizeBytes', None)
    total = env.get("pageCount")
    if raw is None or total is None:
        return None
    if isinstance(raw, bool) or isinstance(total, bool):
        return None
    if isinstance(raw, int) and isinstance(total, int) and raw > total:
        return "COORD_OOB"
    return None

def check_dump_carets_sizeBytes_lte_declared(env: Mapping[str, Any]) -> str | None:
    raw = env.get('sizeBytes', None)
    declared = env.get("declaredCount")
    if raw is None or declared is None:
        return None
    if isinstance(raw, bool) or isinstance(declared, bool):
        return None
    if isinstance(raw, int) and isinstance(declared, int) and raw > declared:
        return "COUNT_DRIFT"
    return None

def check_dump_carets_sizeBytes_under_cap(env: Mapping[str, Any]) -> str | None:
    raw = env.get('sizeBytes', None)
    cap = env.get("capBytes")
    if raw is None or cap is None:
        return None
    if isinstance(raw, bool) or isinstance(cap, bool):
        return None
    if isinstance(raw, int) and isinstance(cap, int) and cap > 0 and raw > cap:
        return "OVER_ACCEPTED"
    return None

def check_dump_carets_sizeBytes_year_range(env: Mapping[str, Any]) -> str | None:
    raw = env.get('sizeBytes', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < 1990 or raw > 2100:
        return "USAGE"
    return None

def check_dump_carets_sizeBytes_fail_vs_identical(env: Mapping[str, Any]) -> str | None:
    raw = env.get('sizeBytes', None)
    identical = env.get("identical")
    if raw is None or identical is not True:
        return None
    if isinstance(raw, int) and not isinstance(raw, bool) and raw > 0:
        return "COUNT_DRIFT"
    return None

def check_dump_carets_sizeBytes_ok_vs_total(env: Mapping[str, Any]) -> str | None:
    raw = env.get('sizeBytes', None)
    total = env.get("total")
    if raw is None or total is None:
        return None
    if isinstance(raw, bool) or isinstance(total, bool):
        return None
    if isinstance(raw, int) and isinstance(total, int) and raw > total:
        return "RATE_IMPOSSIBLE"
    return None

def check_dump_carets_sizeBytes_span_fits(env: Mapping[str, Any]) -> str | None:
    raw = env.get('sizeBytes', None)
    rows = env.get("rows")
    cols = env.get("cols")
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if isinstance(rows, int) and not isinstance(rows, bool) and 'sizeBytes' == "rowSpan" and raw > rows:
        return "SPAN_OOB"
    if isinstance(cols, int) and not isinstance(cols, bool) and 'sizeBytes' == "colSpan" and raw > cols:
        return "SPAN_OOB"
    return None

def check_dump_carets_sizeBytes_width_height(env: Mapping[str, Any]) -> str | None:
    raw = env.get('sizeBytes', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if 'sizeBytes' in ("width", "height", "bytes") and raw < 0:
        return "USAGE"
    return None

def check_dump_carets_sizeBytes_batch_parts(env: Mapping[str, Any]) -> str | None:
    raw = env.get('sizeBytes', None)
    n_in = env.get("inputN")
    n_ok = env.get("okN")
    n_fail = env.get("failN")
    if raw is None:
        return None
    if not all(isinstance(v, int) and not isinstance(v, bool) for v in (n_in, n_ok, n_fail) if v is not None):
        return None
    if isinstance(n_in, int) and isinstance(n_ok, int) and isinstance(n_fail, int):
        if n_in != n_ok + n_fail:
            return "COUNT_DRIFT"
    return None

def check_dump_carets_capBytes_wrong_type_int(env: Mapping[str, Any]) -> str | None:
    raw = env.get('capBytes', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return "USAGE"
    return None

def check_dump_carets_capBytes_negative(env: Mapping[str, Any]) -> str | None:
    raw = env.get('capBytes', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < 0:
        return "USAGE"
    return None

def check_dump_carets_capBytes_nonzero_required(env: Mapping[str, Any]) -> str | None:
    raw = env.get('capBytes', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw == 0:
        return "ZERO_BYTES"
    return None

def check_dump_carets_capBytes_exit_closed(env: Mapping[str, Any]) -> str | None:
    raw = env.get('capBytes', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw not in (0, 1, 2, 3, 4):
        return "EXIT_UNKNOWN"
    return None

def check_dump_carets_capBytes_too_large(env: Mapping[str, Any]) -> str | None:
    raw = env.get('capBytes', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw > 1_000_000_000:
        return "USAGE"
    return None

def check_dump_carets_capBytes_fits_i32(env: Mapping[str, Any]) -> str | None:
    raw = env.get('capBytes', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < -2147483648 or raw > 2147483647:
        return "USAGE"
    return None

def check_dump_carets_capBytes_not_bool_int(env: Mapping[str, Any]) -> str | None:
    raw = env.get('capBytes', None)
    if isinstance(raw, bool):
        return "USAGE"
    return None

def check_dump_carets_capBytes_lte_page_count(env: Mapping[str, Any]) -> str | None:
    raw = env.get('capBytes', None)
    total = env.get("pageCount")
    if raw is None or total is None:
        return None
    if isinstance(raw, bool) or isinstance(total, bool):
        return None
    if isinstance(raw, int) and isinstance(total, int) and raw > total:
        return "COORD_OOB"
    return None

def check_dump_carets_capBytes_lte_declared(env: Mapping[str, Any]) -> str | None:
    raw = env.get('capBytes', None)
    declared = env.get("declaredCount")
    if raw is None or declared is None:
        return None
    if isinstance(raw, bool) or isinstance(declared, bool):
        return None
    if isinstance(raw, int) and isinstance(declared, int) and raw > declared:
        return "COUNT_DRIFT"
    return None

def check_dump_carets_capBytes_under_cap(env: Mapping[str, Any]) -> str | None:
    raw = env.get('capBytes', None)
    cap = env.get("capBytes")
    if raw is None or cap is None:
        return None
    if isinstance(raw, bool) or isinstance(cap, bool):
        return None
    if isinstance(raw, int) and isinstance(cap, int) and cap > 0 and raw > cap:
        return "OVER_ACCEPTED"
    return None

def check_dump_carets_capBytes_year_range(env: Mapping[str, Any]) -> str | None:
    raw = env.get('capBytes', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < 1990 or raw > 2100:
        return "USAGE"
    return None

def check_dump_carets_capBytes_fail_vs_identical(env: Mapping[str, Any]) -> str | None:
    raw = env.get('capBytes', None)
    identical = env.get("identical")
    if raw is None or identical is not True:
        return None
    if isinstance(raw, int) and not isinstance(raw, bool) and raw > 0:
        return "COUNT_DRIFT"
    return None

def check_dump_carets_capBytes_ok_vs_total(env: Mapping[str, Any]) -> str | None:
    raw = env.get('capBytes', None)
    total = env.get("total")
    if raw is None or total is None:
        return None
    if isinstance(raw, bool) or isinstance(total, bool):
        return None
    if isinstance(raw, int) and isinstance(total, int) and raw > total:
        return "RATE_IMPOSSIBLE"
    return None

def check_dump_carets_capBytes_span_fits(env: Mapping[str, Any]) -> str | None:
    raw = env.get('capBytes', None)
    rows = env.get("rows")
    cols = env.get("cols")
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if isinstance(rows, int) and not isinstance(rows, bool) and 'capBytes' == "rowSpan" and raw > rows:
        return "SPAN_OOB"
    if isinstance(cols, int) and not isinstance(cols, bool) and 'capBytes' == "colSpan" and raw > cols:
        return "SPAN_OOB"
    return None

def check_dump_carets_capBytes_width_height(env: Mapping[str, Any]) -> str | None:
    raw = env.get('capBytes', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if 'capBytes' in ("width", "height", "bytes") and raw < 0:
        return "USAGE"
    return None

def check_dump_carets_capBytes_batch_parts(env: Mapping[str, Any]) -> str | None:
    raw = env.get('capBytes', None)
    n_in = env.get("inputN")
    n_ok = env.get("okN")
    n_fail = env.get("failN")
    if raw is None:
        return None
    if not all(isinstance(v, int) and not isinstance(v, bool) for v in (n_in, n_ok, n_fail) if v is not None):
        return None
    if isinstance(n_in, int) and isinstance(n_ok, int) and isinstance(n_fail, int):
        if n_in != n_ok + n_fail:
            return "COUNT_DRIFT"
    return None

def check_dump_carets_rowsIn_wrong_type_int(env: Mapping[str, Any]) -> str | None:
    raw = env.get('rowsIn', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return "USAGE"
    return None

def check_dump_carets_rowsIn_negative(env: Mapping[str, Any]) -> str | None:
    raw = env.get('rowsIn', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < 0:
        return "USAGE"
    return None

def check_dump_carets_rowsIn_nonzero_required(env: Mapping[str, Any]) -> str | None:
    raw = env.get('rowsIn', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw == 0:
        return "ZERO_BYTES"
    return None

def check_dump_carets_rowsIn_exit_closed(env: Mapping[str, Any]) -> str | None:
    raw = env.get('rowsIn', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw not in (0, 1, 2, 3, 4):
        return "EXIT_UNKNOWN"
    return None

def check_dump_carets_rowsIn_too_large(env: Mapping[str, Any]) -> str | None:
    raw = env.get('rowsIn', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw > 1_000_000_000:
        return "USAGE"
    return None

def check_dump_carets_rowsIn_fits_i32(env: Mapping[str, Any]) -> str | None:
    raw = env.get('rowsIn', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < -2147483648 or raw > 2147483647:
        return "USAGE"
    return None

def check_dump_carets_rowsIn_not_bool_int(env: Mapping[str, Any]) -> str | None:
    raw = env.get('rowsIn', None)
    if isinstance(raw, bool):
        return "USAGE"
    return None

def check_dump_carets_rowsIn_lte_page_count(env: Mapping[str, Any]) -> str | None:
    raw = env.get('rowsIn', None)
    total = env.get("pageCount")
    if raw is None or total is None:
        return None
    if isinstance(raw, bool) or isinstance(total, bool):
        return None
    if isinstance(raw, int) and isinstance(total, int) and raw > total:
        return "COORD_OOB"
    return None

def check_dump_carets_rowsIn_lte_declared(env: Mapping[str, Any]) -> str | None:
    raw = env.get('rowsIn', None)
    declared = env.get("declaredCount")
    if raw is None or declared is None:
        return None
    if isinstance(raw, bool) or isinstance(declared, bool):
        return None
    if isinstance(raw, int) and isinstance(declared, int) and raw > declared:
        return "COUNT_DRIFT"
    return None

def check_dump_carets_rowsIn_under_cap(env: Mapping[str, Any]) -> str | None:
    raw = env.get('rowsIn', None)
    cap = env.get("capBytes")
    if raw is None or cap is None:
        return None
    if isinstance(raw, bool) or isinstance(cap, bool):
        return None
    if isinstance(raw, int) and isinstance(cap, int) and cap > 0 and raw > cap:
        return "OVER_ACCEPTED"
    return None

def check_dump_carets_rowsIn_year_range(env: Mapping[str, Any]) -> str | None:
    raw = env.get('rowsIn', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < 1990 or raw > 2100:
        return "USAGE"
    return None

def check_dump_carets_rowsIn_fail_vs_identical(env: Mapping[str, Any]) -> str | None:
    raw = env.get('rowsIn', None)
    identical = env.get("identical")
    if raw is None or identical is not True:
        return None
    if isinstance(raw, int) and not isinstance(raw, bool) and raw > 0:
        return "COUNT_DRIFT"
    return None

def check_dump_carets_rowsIn_ok_vs_total(env: Mapping[str, Any]) -> str | None:
    raw = env.get('rowsIn', None)
    total = env.get("total")
    if raw is None or total is None:
        return None
    if isinstance(raw, bool) or isinstance(total, bool):
        return None
    if isinstance(raw, int) and isinstance(total, int) and raw > total:
        return "RATE_IMPOSSIBLE"
    return None

def check_dump_carets_rowsIn_span_fits(env: Mapping[str, Any]) -> str | None:
    raw = env.get('rowsIn', None)
    rows = env.get("rows")
    cols = env.get("cols")
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if isinstance(rows, int) and not isinstance(rows, bool) and 'rowsIn' == "rowSpan" and raw > rows:
        return "SPAN_OOB"
    if isinstance(cols, int) and not isinstance(cols, bool) and 'rowsIn' == "colSpan" and raw > cols:
        return "SPAN_OOB"
    return None

def check_dump_carets_rowsIn_width_height(env: Mapping[str, Any]) -> str | None:
    raw = env.get('rowsIn', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if 'rowsIn' in ("width", "height", "bytes") and raw < 0:
        return "USAGE"
    return None

def check_dump_carets_rowsIn_batch_parts(env: Mapping[str, Any]) -> str | None:
    raw = env.get('rowsIn', None)
    n_in = env.get("inputN")
    n_ok = env.get("okN")
    n_fail = env.get("failN")
    if raw is None:
        return None
    if not all(isinstance(v, int) and not isinstance(v, bool) for v in (n_in, n_ok, n_fail) if v is not None):
        return None
    if isinstance(n_in, int) and isinstance(n_ok, int) and isinstance(n_fail, int):
        if n_in != n_ok + n_fail:
            return "COUNT_DRIFT"
    return None

def check_dump_carets_colsIn_wrong_type_int(env: Mapping[str, Any]) -> str | None:
    raw = env.get('colsIn', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return "USAGE"
    return None

def check_dump_carets_colsIn_negative(env: Mapping[str, Any]) -> str | None:
    raw = env.get('colsIn', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < 0:
        return "USAGE"
    return None

def check_dump_carets_colsIn_nonzero_required(env: Mapping[str, Any]) -> str | None:
    raw = env.get('colsIn', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw == 0:
        return "ZERO_BYTES"
    return None

def check_dump_carets_colsIn_exit_closed(env: Mapping[str, Any]) -> str | None:
    raw = env.get('colsIn', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw not in (0, 1, 2, 3, 4):
        return "EXIT_UNKNOWN"
    return None

def check_dump_carets_colsIn_too_large(env: Mapping[str, Any]) -> str | None:
    raw = env.get('colsIn', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw > 1_000_000_000:
        return "USAGE"
    return None

def check_dump_carets_colsIn_fits_i32(env: Mapping[str, Any]) -> str | None:
    raw = env.get('colsIn', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < -2147483648 or raw > 2147483647:
        return "USAGE"
    return None

def check_dump_carets_colsIn_not_bool_int(env: Mapping[str, Any]) -> str | None:
    raw = env.get('colsIn', None)
    if isinstance(raw, bool):
        return "USAGE"
    return None

def check_dump_carets_colsIn_lte_page_count(env: Mapping[str, Any]) -> str | None:
    raw = env.get('colsIn', None)
    total = env.get("pageCount")
    if raw is None or total is None:
        return None
    if isinstance(raw, bool) or isinstance(total, bool):
        return None
    if isinstance(raw, int) and isinstance(total, int) and raw > total:
        return "COORD_OOB"
    return None

def check_dump_carets_colsIn_lte_declared(env: Mapping[str, Any]) -> str | None:
    raw = env.get('colsIn', None)
    declared = env.get("declaredCount")
    if raw is None or declared is None:
        return None
    if isinstance(raw, bool) or isinstance(declared, bool):
        return None
    if isinstance(raw, int) and isinstance(declared, int) and raw > declared:
        return "COUNT_DRIFT"
    return None

def check_dump_carets_colsIn_under_cap(env: Mapping[str, Any]) -> str | None:
    raw = env.get('colsIn', None)
    cap = env.get("capBytes")
    if raw is None or cap is None:
        return None
    if isinstance(raw, bool) or isinstance(cap, bool):
        return None
    if isinstance(raw, int) and isinstance(cap, int) and cap > 0 and raw > cap:
        return "OVER_ACCEPTED"
    return None

def check_dump_carets_colsIn_year_range(env: Mapping[str, Any]) -> str | None:
    raw = env.get('colsIn', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < 1990 or raw > 2100:
        return "USAGE"
    return None

def check_dump_carets_colsIn_fail_vs_identical(env: Mapping[str, Any]) -> str | None:
    raw = env.get('colsIn', None)
    identical = env.get("identical")
    if raw is None or identical is not True:
        return None
    if isinstance(raw, int) and not isinstance(raw, bool) and raw > 0:
        return "COUNT_DRIFT"
    return None

def check_dump_carets_colsIn_ok_vs_total(env: Mapping[str, Any]) -> str | None:
    raw = env.get('colsIn', None)
    total = env.get("total")
    if raw is None or total is None:
        return None
    if isinstance(raw, bool) or isinstance(total, bool):
        return None
    if isinstance(raw, int) and isinstance(total, int) and raw > total:
        return "RATE_IMPOSSIBLE"
    return None

def check_dump_carets_colsIn_span_fits(env: Mapping[str, Any]) -> str | None:
    raw = env.get('colsIn', None)
    rows = env.get("rows")
    cols = env.get("cols")
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if isinstance(rows, int) and not isinstance(rows, bool) and 'colsIn' == "rowSpan" and raw > rows:
        return "SPAN_OOB"
    if isinstance(cols, int) and not isinstance(cols, bool) and 'colsIn' == "colSpan" and raw > cols:
        return "SPAN_OOB"
    return None

def check_dump_carets_colsIn_width_height(env: Mapping[str, Any]) -> str | None:
    raw = env.get('colsIn', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if 'colsIn' in ("width", "height", "bytes") and raw < 0:
        return "USAGE"
    return None

def check_dump_carets_colsIn_batch_parts(env: Mapping[str, Any]) -> str | None:
    raw = env.get('colsIn', None)
    n_in = env.get("inputN")
    n_ok = env.get("okN")
    n_fail = env.get("failN")
    if raw is None:
        return None
    if not all(isinstance(v, int) and not isinstance(v, bool) for v in (n_in, n_ok, n_fail) if v is not None):
        return None
    if isinstance(n_in, int) and isinstance(n_ok, int) and isinstance(n_fail, int):
        if n_in != n_ok + n_fail:
            return "COUNT_DRIFT"
    return None

def check_dump_carets_rowsOut_wrong_type_int(env: Mapping[str, Any]) -> str | None:
    raw = env.get('rowsOut', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return "USAGE"
    return None

def check_dump_carets_rowsOut_negative(env: Mapping[str, Any]) -> str | None:
    raw = env.get('rowsOut', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < 0:
        return "USAGE"
    return None

def check_dump_carets_rowsOut_nonzero_required(env: Mapping[str, Any]) -> str | None:
    raw = env.get('rowsOut', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw == 0:
        return "ZERO_BYTES"
    return None

def check_dump_carets_rowsOut_exit_closed(env: Mapping[str, Any]) -> str | None:
    raw = env.get('rowsOut', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw not in (0, 1, 2, 3, 4):
        return "EXIT_UNKNOWN"
    return None

def check_dump_carets_rowsOut_too_large(env: Mapping[str, Any]) -> str | None:
    raw = env.get('rowsOut', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw > 1_000_000_000:
        return "USAGE"
    return None

def check_dump_carets_rowsOut_fits_i32(env: Mapping[str, Any]) -> str | None:
    raw = env.get('rowsOut', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < -2147483648 or raw > 2147483647:
        return "USAGE"
    return None

def check_dump_carets_rowsOut_not_bool_int(env: Mapping[str, Any]) -> str | None:
    raw = env.get('rowsOut', None)
    if isinstance(raw, bool):
        return "USAGE"
    return None

def check_dump_carets_rowsOut_lte_page_count(env: Mapping[str, Any]) -> str | None:
    raw = env.get('rowsOut', None)
    total = env.get("pageCount")
    if raw is None or total is None:
        return None
    if isinstance(raw, bool) or isinstance(total, bool):
        return None
    if isinstance(raw, int) and isinstance(total, int) and raw > total:
        return "COORD_OOB"
    return None

def check_dump_carets_rowsOut_lte_declared(env: Mapping[str, Any]) -> str | None:
    raw = env.get('rowsOut', None)
    declared = env.get("declaredCount")
    if raw is None or declared is None:
        return None
    if isinstance(raw, bool) or isinstance(declared, bool):
        return None
    if isinstance(raw, int) and isinstance(declared, int) and raw > declared:
        return "COUNT_DRIFT"
    return None

def check_dump_carets_rowsOut_under_cap(env: Mapping[str, Any]) -> str | None:
    raw = env.get('rowsOut', None)
    cap = env.get("capBytes")
    if raw is None or cap is None:
        return None
    if isinstance(raw, bool) or isinstance(cap, bool):
        return None
    if isinstance(raw, int) and isinstance(cap, int) and cap > 0 and raw > cap:
        return "OVER_ACCEPTED"
    return None

def check_dump_carets_rowsOut_year_range(env: Mapping[str, Any]) -> str | None:
    raw = env.get('rowsOut', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < 1990 or raw > 2100:
        return "USAGE"
    return None

def check_dump_carets_rowsOut_fail_vs_identical(env: Mapping[str, Any]) -> str | None:
    raw = env.get('rowsOut', None)
    identical = env.get("identical")
    if raw is None or identical is not True:
        return None
    if isinstance(raw, int) and not isinstance(raw, bool) and raw > 0:
        return "COUNT_DRIFT"
    return None

def check_dump_carets_rowsOut_ok_vs_total(env: Mapping[str, Any]) -> str | None:
    raw = env.get('rowsOut', None)
    total = env.get("total")
    if raw is None or total is None:
        return None
    if isinstance(raw, bool) or isinstance(total, bool):
        return None
    if isinstance(raw, int) and isinstance(total, int) and raw > total:
        return "RATE_IMPOSSIBLE"
    return None

def check_dump_carets_rowsOut_span_fits(env: Mapping[str, Any]) -> str | None:
    raw = env.get('rowsOut', None)
    rows = env.get("rows")
    cols = env.get("cols")
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if isinstance(rows, int) and not isinstance(rows, bool) and 'rowsOut' == "rowSpan" and raw > rows:
        return "SPAN_OOB"
    if isinstance(cols, int) and not isinstance(cols, bool) and 'rowsOut' == "colSpan" and raw > cols:
        return "SPAN_OOB"
    return None

def check_dump_carets_rowsOut_width_height(env: Mapping[str, Any]) -> str | None:
    raw = env.get('rowsOut', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if 'rowsOut' in ("width", "height", "bytes") and raw < 0:
        return "USAGE"
    return None

def check_dump_carets_rowsOut_batch_parts(env: Mapping[str, Any]) -> str | None:
    raw = env.get('rowsOut', None)
    n_in = env.get("inputN")
    n_ok = env.get("okN")
    n_fail = env.get("failN")
    if raw is None:
        return None
    if not all(isinstance(v, int) and not isinstance(v, bool) for v in (n_in, n_ok, n_fail) if v is not None):
        return None
    if isinstance(n_in, int) and isinstance(n_ok, int) and isinstance(n_fail, int):
        if n_in != n_ok + n_fail:
            return "COUNT_DRIFT"
    return None

def check_dump_carets_colsOut_wrong_type_int(env: Mapping[str, Any]) -> str | None:
    raw = env.get('colsOut', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return "USAGE"
    return None

def check_dump_carets_colsOut_negative(env: Mapping[str, Any]) -> str | None:
    raw = env.get('colsOut', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < 0:
        return "USAGE"
    return None

def check_dump_carets_colsOut_nonzero_required(env: Mapping[str, Any]) -> str | None:
    raw = env.get('colsOut', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw == 0:
        return "ZERO_BYTES"
    return None

def check_dump_carets_colsOut_exit_closed(env: Mapping[str, Any]) -> str | None:
    raw = env.get('colsOut', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw not in (0, 1, 2, 3, 4):
        return "EXIT_UNKNOWN"
    return None

def check_dump_carets_colsOut_too_large(env: Mapping[str, Any]) -> str | None:
    raw = env.get('colsOut', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw > 1_000_000_000:
        return "USAGE"
    return None

def check_dump_carets_colsOut_fits_i32(env: Mapping[str, Any]) -> str | None:
    raw = env.get('colsOut', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < -2147483648 or raw > 2147483647:
        return "USAGE"
    return None

def check_dump_carets_colsOut_not_bool_int(env: Mapping[str, Any]) -> str | None:
    raw = env.get('colsOut', None)
    if isinstance(raw, bool):
        return "USAGE"
    return None

def check_dump_carets_colsOut_lte_page_count(env: Mapping[str, Any]) -> str | None:
    raw = env.get('colsOut', None)
    total = env.get("pageCount")
    if raw is None or total is None:
        return None
    if isinstance(raw, bool) or isinstance(total, bool):
        return None
    if isinstance(raw, int) and isinstance(total, int) and raw > total:
        return "COORD_OOB"
    return None

def check_dump_carets_colsOut_lte_declared(env: Mapping[str, Any]) -> str | None:
    raw = env.get('colsOut', None)
    declared = env.get("declaredCount")
    if raw is None or declared is None:
        return None
    if isinstance(raw, bool) or isinstance(declared, bool):
        return None
    if isinstance(raw, int) and isinstance(declared, int) and raw > declared:
        return "COUNT_DRIFT"
    return None

def check_dump_carets_colsOut_under_cap(env: Mapping[str, Any]) -> str | None:
    raw = env.get('colsOut', None)
    cap = env.get("capBytes")
    if raw is None or cap is None:
        return None
    if isinstance(raw, bool) or isinstance(cap, bool):
        return None
    if isinstance(raw, int) and isinstance(cap, int) and cap > 0 and raw > cap:
        return "OVER_ACCEPTED"
    return None

def check_dump_carets_colsOut_year_range(env: Mapping[str, Any]) -> str | None:
    raw = env.get('colsOut', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < 1990 or raw > 2100:
        return "USAGE"
    return None

def check_dump_carets_colsOut_fail_vs_identical(env: Mapping[str, Any]) -> str | None:
    raw = env.get('colsOut', None)
    identical = env.get("identical")
    if raw is None or identical is not True:
        return None
    if isinstance(raw, int) and not isinstance(raw, bool) and raw > 0:
        return "COUNT_DRIFT"
    return None

def check_dump_carets_colsOut_ok_vs_total(env: Mapping[str, Any]) -> str | None:
    raw = env.get('colsOut', None)
    total = env.get("total")
    if raw is None or total is None:
        return None
    if isinstance(raw, bool) or isinstance(total, bool):
        return None
    if isinstance(raw, int) and isinstance(total, int) and raw > total:
        return "RATE_IMPOSSIBLE"
    return None

def check_dump_carets_colsOut_span_fits(env: Mapping[str, Any]) -> str | None:
    raw = env.get('colsOut', None)
    rows = env.get("rows")
    cols = env.get("cols")
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if isinstance(rows, int) and not isinstance(rows, bool) and 'colsOut' == "rowSpan" and raw > rows:
        return "SPAN_OOB"
    if isinstance(cols, int) and not isinstance(cols, bool) and 'colsOut' == "colSpan" and raw > cols:
        return "SPAN_OOB"
    return None

def check_dump_carets_colsOut_width_height(env: Mapping[str, Any]) -> str | None:
    raw = env.get('colsOut', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if 'colsOut' in ("width", "height", "bytes") and raw < 0:
        return "USAGE"
    return None

def check_dump_carets_colsOut_batch_parts(env: Mapping[str, Any]) -> str | None:
    raw = env.get('colsOut', None)
    n_in = env.get("inputN")
    n_ok = env.get("okN")
    n_fail = env.get("failN")
    if raw is None:
        return None
    if not all(isinstance(v, int) and not isinstance(v, bool) for v in (n_in, n_ok, n_fail) if v is not None):
        return None
    if isinstance(n_in, int) and isinstance(n_ok, int) and isinstance(n_fail, int):
        if n_in != n_ok + n_fail:
            return "COUNT_DRIFT"
    return None

def check_dump_carets_fieldCount_wrong_type_int(env: Mapping[str, Any]) -> str | None:
    raw = env.get('fieldCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return "USAGE"
    return None

def check_dump_carets_fieldCount_negative(env: Mapping[str, Any]) -> str | None:
    raw = env.get('fieldCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < 0:
        return "USAGE"
    return None

def check_dump_carets_fieldCount_nonzero_required(env: Mapping[str, Any]) -> str | None:
    raw = env.get('fieldCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw == 0:
        return "ZERO_BYTES"
    return None

def check_dump_carets_fieldCount_exit_closed(env: Mapping[str, Any]) -> str | None:
    raw = env.get('fieldCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw not in (0, 1, 2, 3, 4):
        return "EXIT_UNKNOWN"
    return None

def check_dump_carets_fieldCount_too_large(env: Mapping[str, Any]) -> str | None:
    raw = env.get('fieldCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw > 1_000_000_000:
        return "USAGE"
    return None

def check_dump_carets_fieldCount_fits_i32(env: Mapping[str, Any]) -> str | None:
    raw = env.get('fieldCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < -2147483648 or raw > 2147483647:
        return "USAGE"
    return None

def check_dump_carets_fieldCount_not_bool_int(env: Mapping[str, Any]) -> str | None:
    raw = env.get('fieldCount', None)
    if isinstance(raw, bool):
        return "USAGE"
    return None

def check_dump_carets_fieldCount_lte_page_count(env: Mapping[str, Any]) -> str | None:
    raw = env.get('fieldCount', None)
    total = env.get("pageCount")
    if raw is None or total is None:
        return None
    if isinstance(raw, bool) or isinstance(total, bool):
        return None
    if isinstance(raw, int) and isinstance(total, int) and raw > total:
        return "COORD_OOB"
    return None

def check_dump_carets_fieldCount_lte_declared(env: Mapping[str, Any]) -> str | None:
    raw = env.get('fieldCount', None)
    declared = env.get("declaredCount")
    if raw is None or declared is None:
        return None
    if isinstance(raw, bool) or isinstance(declared, bool):
        return None
    if isinstance(raw, int) and isinstance(declared, int) and raw > declared:
        return "COUNT_DRIFT"
    return None

def check_dump_carets_fieldCount_under_cap(env: Mapping[str, Any]) -> str | None:
    raw = env.get('fieldCount', None)
    cap = env.get("capBytes")
    if raw is None or cap is None:
        return None
    if isinstance(raw, bool) or isinstance(cap, bool):
        return None
    if isinstance(raw, int) and isinstance(cap, int) and cap > 0 and raw > cap:
        return "OVER_ACCEPTED"
    return None

def check_dump_carets_fieldCount_year_range(env: Mapping[str, Any]) -> str | None:
    raw = env.get('fieldCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < 1990 or raw > 2100:
        return "USAGE"
    return None

def check_dump_carets_fieldCount_fail_vs_identical(env: Mapping[str, Any]) -> str | None:
    raw = env.get('fieldCount', None)
    identical = env.get("identical")
    if raw is None or identical is not True:
        return None
    if isinstance(raw, int) and not isinstance(raw, bool) and raw > 0:
        return "COUNT_DRIFT"
    return None

def check_dump_carets_fieldCount_ok_vs_total(env: Mapping[str, Any]) -> str | None:
    raw = env.get('fieldCount', None)
    total = env.get("total")
    if raw is None or total is None:
        return None
    if isinstance(raw, bool) or isinstance(total, bool):
        return None
    if isinstance(raw, int) and isinstance(total, int) and raw > total:
        return "RATE_IMPOSSIBLE"
    return None

def check_dump_carets_fieldCount_span_fits(env: Mapping[str, Any]) -> str | None:
    raw = env.get('fieldCount', None)
    rows = env.get("rows")
    cols = env.get("cols")
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if isinstance(rows, int) and not isinstance(rows, bool) and 'fieldCount' == "rowSpan" and raw > rows:
        return "SPAN_OOB"
    if isinstance(cols, int) and not isinstance(cols, bool) and 'fieldCount' == "colSpan" and raw > cols:
        return "SPAN_OOB"
    return None

def check_dump_carets_fieldCount_width_height(env: Mapping[str, Any]) -> str | None:
    raw = env.get('fieldCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if 'fieldCount' in ("width", "height", "bytes") and raw < 0:
        return "USAGE"
    return None

def check_dump_carets_fieldCount_batch_parts(env: Mapping[str, Any]) -> str | None:
    raw = env.get('fieldCount', None)
    n_in = env.get("inputN")
    n_ok = env.get("okN")
    n_fail = env.get("failN")
    if raw is None:
        return None
    if not all(isinstance(v, int) and not isinstance(v, bool) for v in (n_in, n_ok, n_fail) if v is not None):
        return None
    if isinstance(n_in, int) and isinstance(n_ok, int) and isinstance(n_fail, int):
        if n_in != n_ok + n_fail:
            return "COUNT_DRIFT"
    return None

def check_dump_carets_renderedCount_wrong_type_int(env: Mapping[str, Any]) -> str | None:
    raw = env.get('renderedCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return "USAGE"
    return None

def check_dump_carets_renderedCount_negative(env: Mapping[str, Any]) -> str | None:
    raw = env.get('renderedCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < 0:
        return "USAGE"
    return None

def check_dump_carets_renderedCount_nonzero_required(env: Mapping[str, Any]) -> str | None:
    raw = env.get('renderedCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw == 0:
        return "ZERO_BYTES"
    return None

def check_dump_carets_renderedCount_exit_closed(env: Mapping[str, Any]) -> str | None:
    raw = env.get('renderedCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw not in (0, 1, 2, 3, 4):
        return "EXIT_UNKNOWN"
    return None

def check_dump_carets_renderedCount_too_large(env: Mapping[str, Any]) -> str | None:
    raw = env.get('renderedCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw > 1_000_000_000:
        return "USAGE"
    return None

def check_dump_carets_renderedCount_fits_i32(env: Mapping[str, Any]) -> str | None:
    raw = env.get('renderedCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < -2147483648 or raw > 2147483647:
        return "USAGE"
    return None

def check_dump_carets_renderedCount_not_bool_int(env: Mapping[str, Any]) -> str | None:
    raw = env.get('renderedCount', None)
    if isinstance(raw, bool):
        return "USAGE"
    return None

def check_dump_carets_renderedCount_lte_page_count(env: Mapping[str, Any]) -> str | None:
    raw = env.get('renderedCount', None)
    total = env.get("pageCount")
    if raw is None or total is None:
        return None
    if isinstance(raw, bool) or isinstance(total, bool):
        return None
    if isinstance(raw, int) and isinstance(total, int) and raw > total:
        return "COORD_OOB"
    return None

def check_dump_carets_renderedCount_lte_declared(env: Mapping[str, Any]) -> str | None:
    raw = env.get('renderedCount', None)
    declared = env.get("declaredCount")
    if raw is None or declared is None:
        return None
    if isinstance(raw, bool) or isinstance(declared, bool):
        return None
    if isinstance(raw, int) and isinstance(declared, int) and raw > declared:
        return "COUNT_DRIFT"
    return None

def check_dump_carets_renderedCount_under_cap(env: Mapping[str, Any]) -> str | None:
    raw = env.get('renderedCount', None)
    cap = env.get("capBytes")
    if raw is None or cap is None:
        return None
    if isinstance(raw, bool) or isinstance(cap, bool):
        return None
    if isinstance(raw, int) and isinstance(cap, int) and cap > 0 and raw > cap:
        return "OVER_ACCEPTED"
    return None

def check_dump_carets_renderedCount_year_range(env: Mapping[str, Any]) -> str | None:
    raw = env.get('renderedCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < 1990 or raw > 2100:
        return "USAGE"
    return None

def check_dump_carets_renderedCount_fail_vs_identical(env: Mapping[str, Any]) -> str | None:
    raw = env.get('renderedCount', None)
    identical = env.get("identical")
    if raw is None or identical is not True:
        return None
    if isinstance(raw, int) and not isinstance(raw, bool) and raw > 0:
        return "COUNT_DRIFT"
    return None

def check_dump_carets_renderedCount_ok_vs_total(env: Mapping[str, Any]) -> str | None:
    raw = env.get('renderedCount', None)
    total = env.get("total")
    if raw is None or total is None:
        return None
    if isinstance(raw, bool) or isinstance(total, bool):
        return None
    if isinstance(raw, int) and isinstance(total, int) and raw > total:
        return "RATE_IMPOSSIBLE"
    return None

def check_dump_carets_renderedCount_span_fits(env: Mapping[str, Any]) -> str | None:
    raw = env.get('renderedCount', None)
    rows = env.get("rows")
    cols = env.get("cols")
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if isinstance(rows, int) and not isinstance(rows, bool) and 'renderedCount' == "rowSpan" and raw > rows:
        return "SPAN_OOB"
    if isinstance(cols, int) and not isinstance(cols, bool) and 'renderedCount' == "colSpan" and raw > cols:
        return "SPAN_OOB"
    return None

def check_dump_carets_renderedCount_width_height(env: Mapping[str, Any]) -> str | None:
    raw = env.get('renderedCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if 'renderedCount' in ("width", "height", "bytes") and raw < 0:
        return "USAGE"
    return None

def check_dump_carets_renderedCount_batch_parts(env: Mapping[str, Any]) -> str | None:
    raw = env.get('renderedCount', None)
    n_in = env.get("inputN")
    n_ok = env.get("okN")
    n_fail = env.get("failN")
    if raw is None:
        return None
    if not all(isinstance(v, int) and not isinstance(v, bool) for v in (n_in, n_ok, n_fail) if v is not None):
        return None
    if isinstance(n_in, int) and isinstance(n_ok, int) and isinstance(n_fail, int):
        if n_in != n_ok + n_fail:
            return "COUNT_DRIFT"
    return None

def check_dump_carets_imageCount_wrong_type_int(env: Mapping[str, Any]) -> str | None:
    raw = env.get('imageCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return "USAGE"
    return None

def check_dump_carets_imageCount_negative(env: Mapping[str, Any]) -> str | None:
    raw = env.get('imageCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < 0:
        return "USAGE"
    return None

def check_dump_carets_imageCount_nonzero_required(env: Mapping[str, Any]) -> str | None:
    raw = env.get('imageCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw == 0:
        return "ZERO_BYTES"
    return None

def check_dump_carets_imageCount_exit_closed(env: Mapping[str, Any]) -> str | None:
    raw = env.get('imageCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw not in (0, 1, 2, 3, 4):
        return "EXIT_UNKNOWN"
    return None

def check_dump_carets_imageCount_too_large(env: Mapping[str, Any]) -> str | None:
    raw = env.get('imageCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw > 1_000_000_000:
        return "USAGE"
    return None

def check_dump_carets_imageCount_fits_i32(env: Mapping[str, Any]) -> str | None:
    raw = env.get('imageCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < -2147483648 or raw > 2147483647:
        return "USAGE"
    return None

def check_dump_carets_imageCount_not_bool_int(env: Mapping[str, Any]) -> str | None:
    raw = env.get('imageCount', None)
    if isinstance(raw, bool):
        return "USAGE"
    return None

def check_dump_carets_imageCount_lte_page_count(env: Mapping[str, Any]) -> str | None:
    raw = env.get('imageCount', None)
    total = env.get("pageCount")
    if raw is None or total is None:
        return None
    if isinstance(raw, bool) or isinstance(total, bool):
        return None
    if isinstance(raw, int) and isinstance(total, int) and raw > total:
        return "COORD_OOB"
    return None

def check_dump_carets_imageCount_lte_declared(env: Mapping[str, Any]) -> str | None:
    raw = env.get('imageCount', None)
    declared = env.get("declaredCount")
    if raw is None or declared is None:
        return None
    if isinstance(raw, bool) or isinstance(declared, bool):
        return None
    if isinstance(raw, int) and isinstance(declared, int) and raw > declared:
        return "COUNT_DRIFT"
    return None

def check_dump_carets_imageCount_under_cap(env: Mapping[str, Any]) -> str | None:
    raw = env.get('imageCount', None)
    cap = env.get("capBytes")
    if raw is None or cap is None:
        return None
    if isinstance(raw, bool) or isinstance(cap, bool):
        return None
    if isinstance(raw, int) and isinstance(cap, int) and cap > 0 and raw > cap:
        return "OVER_ACCEPTED"
    return None

def check_dump_carets_imageCount_year_range(env: Mapping[str, Any]) -> str | None:
    raw = env.get('imageCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < 1990 or raw > 2100:
        return "USAGE"
    return None

def check_dump_carets_imageCount_fail_vs_identical(env: Mapping[str, Any]) -> str | None:
    raw = env.get('imageCount', None)
    identical = env.get("identical")
    if raw is None or identical is not True:
        return None
    if isinstance(raw, int) and not isinstance(raw, bool) and raw > 0:
        return "COUNT_DRIFT"
    return None

def check_dump_carets_imageCount_ok_vs_total(env: Mapping[str, Any]) -> str | None:
    raw = env.get('imageCount', None)
    total = env.get("total")
    if raw is None or total is None:
        return None
    if isinstance(raw, bool) or isinstance(total, bool):
        return None
    if isinstance(raw, int) and isinstance(total, int) and raw > total:
        return "RATE_IMPOSSIBLE"
    return None

def check_dump_carets_imageCount_span_fits(env: Mapping[str, Any]) -> str | None:
    raw = env.get('imageCount', None)
    rows = env.get("rows")
    cols = env.get("cols")
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if isinstance(rows, int) and not isinstance(rows, bool) and 'imageCount' == "rowSpan" and raw > rows:
        return "SPAN_OOB"
    if isinstance(cols, int) and not isinstance(cols, bool) and 'imageCount' == "colSpan" and raw > cols:
        return "SPAN_OOB"
    return None

def check_dump_carets_imageCount_width_height(env: Mapping[str, Any]) -> str | None:
    raw = env.get('imageCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if 'imageCount' in ("width", "height", "bytes") and raw < 0:
        return "USAGE"
    return None

def check_dump_carets_imageCount_batch_parts(env: Mapping[str, Any]) -> str | None:
    raw = env.get('imageCount', None)
    n_in = env.get("inputN")
    n_ok = env.get("okN")
    n_fail = env.get("failN")
    if raw is None:
        return None
    if not all(isinstance(v, int) and not isinstance(v, bool) for v in (n_in, n_ok, n_fail) if v is not None):
        return None
    if isinstance(n_in, int) and isinstance(n_ok, int) and isinstance(n_fail, int):
        if n_in != n_ok + n_fail:
            return "COUNT_DRIFT"
    return None

def check_dump_carets_nodeCount_wrong_type_int(env: Mapping[str, Any]) -> str | None:
    raw = env.get('nodeCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return "USAGE"
    return None

def check_dump_carets_nodeCount_negative(env: Mapping[str, Any]) -> str | None:
    raw = env.get('nodeCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < 0:
        return "USAGE"
    return None

def check_dump_carets_nodeCount_nonzero_required(env: Mapping[str, Any]) -> str | None:
    raw = env.get('nodeCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw == 0:
        return "ZERO_BYTES"
    return None

def check_dump_carets_nodeCount_exit_closed(env: Mapping[str, Any]) -> str | None:
    raw = env.get('nodeCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw not in (0, 1, 2, 3, 4):
        return "EXIT_UNKNOWN"
    return None

def check_dump_carets_nodeCount_too_large(env: Mapping[str, Any]) -> str | None:
    raw = env.get('nodeCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw > 1_000_000_000:
        return "USAGE"
    return None

def check_dump_carets_nodeCount_fits_i32(env: Mapping[str, Any]) -> str | None:
    raw = env.get('nodeCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < -2147483648 or raw > 2147483647:
        return "USAGE"
    return None

def check_dump_carets_nodeCount_not_bool_int(env: Mapping[str, Any]) -> str | None:
    raw = env.get('nodeCount', None)
    if isinstance(raw, bool):
        return "USAGE"
    return None

def check_dump_carets_nodeCount_lte_page_count(env: Mapping[str, Any]) -> str | None:
    raw = env.get('nodeCount', None)
    total = env.get("pageCount")
    if raw is None or total is None:
        return None
    if isinstance(raw, bool) or isinstance(total, bool):
        return None
    if isinstance(raw, int) and isinstance(total, int) and raw > total:
        return "COORD_OOB"
    return None

def check_dump_carets_nodeCount_lte_declared(env: Mapping[str, Any]) -> str | None:
    raw = env.get('nodeCount', None)
    declared = env.get("declaredCount")
    if raw is None or declared is None:
        return None
    if isinstance(raw, bool) or isinstance(declared, bool):
        return None
    if isinstance(raw, int) and isinstance(declared, int) and raw > declared:
        return "COUNT_DRIFT"
    return None

def check_dump_carets_nodeCount_under_cap(env: Mapping[str, Any]) -> str | None:
    raw = env.get('nodeCount', None)
    cap = env.get("capBytes")
    if raw is None or cap is None:
        return None
    if isinstance(raw, bool) or isinstance(cap, bool):
        return None
    if isinstance(raw, int) and isinstance(cap, int) and cap > 0 and raw > cap:
        return "OVER_ACCEPTED"
    return None

def check_dump_carets_nodeCount_year_range(env: Mapping[str, Any]) -> str | None:
    raw = env.get('nodeCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if raw < 1990 or raw > 2100:
        return "USAGE"
    return None

def check_dump_carets_nodeCount_fail_vs_identical(env: Mapping[str, Any]) -> str | None:
    raw = env.get('nodeCount', None)
    identical = env.get("identical")
    if raw is None or identical is not True:
        return None
    if isinstance(raw, int) and not isinstance(raw, bool) and raw > 0:
        return "COUNT_DRIFT"
    return None

def check_dump_carets_nodeCount_ok_vs_total(env: Mapping[str, Any]) -> str | None:
    raw = env.get('nodeCount', None)
    total = env.get("total")
    if raw is None or total is None:
        return None
    if isinstance(raw, bool) or isinstance(total, bool):
        return None
    if isinstance(raw, int) and isinstance(total, int) and raw > total:
        return "RATE_IMPOSSIBLE"
    return None

def check_dump_carets_nodeCount_span_fits(env: Mapping[str, Any]) -> str | None:
    raw = env.get('nodeCount', None)
    rows = env.get("rows")
    cols = env.get("cols")
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if isinstance(rows, int) and not isinstance(rows, bool) and 'nodeCount' == "rowSpan" and raw > rows:
        return "SPAN_OOB"
    if isinstance(cols, int) and not isinstance(cols, bool) and 'nodeCount' == "colSpan" and raw > cols:
        return "SPAN_OOB"
    return None

def check_dump_carets_nodeCount_width_height(env: Mapping[str, Any]) -> str | None:
    raw = env.get('nodeCount', None)
    if raw is None:
        return None
    if isinstance(raw, bool) or not isinstance(raw, int):
        return None
    if 'nodeCount' in ("width", "height", "bytes") and raw < 0:
        return "USAGE"
    return None

def check_dump_carets_nodeCount_batch_parts(env: Mapping[str, Any]) -> str | None:
    raw = env.get('nodeCount', None)
    n_in = env.get("inputN")
    n_ok = env.get("okN")
    n_fail = env.get("failN")
    if raw is None:
        return None
    if not all(isinstance(v, int) and not isinstance(v, bool) for v in (n_in, n_ok, n_fail) if v is not None):
        return None
    if isinstance(n_in, int) and isinstance(n_ok, int) and isinstance(n_fail, int):
        if n_in != n_ok + n_fail:
            return "COUNT_DRIFT"
    return None

def check_dump_carets_kind_wrong_type_str(env: Mapping[str, Any]) -> str | None:
    raw = env.get('kind', None)
    if raw is None:
        return None
    if not isinstance(raw, str):
        return "USAGE"
    return None

def check_dump_carets_kind_empty_string(env: Mapping[str, Any]) -> str | None:
    raw = env.get('kind', None)
    if raw is None:
        return None
    if isinstance(raw, str) and raw.strip() == "":
        return "USAGE"
    return None

def check_dump_carets_kind_hex64(env: Mapping[str, Any]) -> str | None:
    raw = env.get('kind', None)
    if raw is None:
        return None
    if not isinstance(raw, str):
        return "USAGE"
    if len(raw) != 64:
        return "USAGE"
    for ch in raw:
        if ch not in "0123456789abcdef":
            return "HASH_DEFECT"
    return None

def check_dump_carets_kind_level_closed(env: Mapping[str, Any]) -> str | None:
    raw = env.get('kind', None)
    if raw is None:
        return None
    if not isinstance(raw, str):
        return None
    if raw not in ("L1", "L2", "L3", "L4", "L5"):
        return "LEVEL_UNKNOWN"
    return None

def check_dump_carets_kind_kind_closed(env: Mapping[str, Any]) -> str | None:
    raw = env.get('kind', None)
    if raw is None:
        return None
    if not isinstance(raw, str):
        return None
    if raw not in ("date", "amount", "number", "all"):
        return "KIND_UNKNOWN"
    return None

def check_dump_carets_kind_schema_token(env: Mapping[str, Any]) -> str | None:
    raw = env.get('kind', None)
    if raw is None:
        return None
    if not isinstance(raw, str):
        return None
    if raw.count(".") != 1 and raw not in ("1.0", "1.1", "1.2"):
        if len(raw) == 0:
            return "USAGE"
    return None

def check_dump_carets_kind_next_call_shape(env: Mapping[str, Any]) -> str | None:
    raw = env.get('kind', None)
    if raw is None:
        return None
    if isinstance(raw, dict):
        if "name" in raw and not isinstance(raw.get("name"), str):
            return "USAGE"
    return None

def check_dump_carets_expectedSha_wrong_type_str(env: Mapping[str, Any]) -> str | None:
    raw = env.get('expectedSha', None)
    if raw is None:
        return None
    if not isinstance(raw, str):
        return "USAGE"
    return None

def check_dump_carets_expectedSha_empty_string(env: Mapping[str, Any]) -> str | None:
    raw = env.get('expectedSha', None)
    if raw is None:
        return None
    if isinstance(raw, str) and raw.strip() == "":
        return "USAGE"
    return None

def check_dump_carets_expectedSha_hex64(env: Mapping[str, Any]) -> str | None:
    raw = env.get('expectedSha', None)
    if raw is None:
        return None
    if not isinstance(raw, str):
        return "USAGE"
    if len(raw) != 64:
        return "USAGE"
    for ch in raw:
        if ch not in "0123456789abcdef":
            return "HASH_DEFECT"
    return None

def check_dump_carets_expectedSha_level_closed(env: Mapping[str, Any]) -> str | None:
    raw = env.get('expectedSha', None)
    if raw is None:
        return None
    if not isinstance(raw, str):
        return None
    if raw not in ("L1", "L2", "L3", "L4", "L5"):
        return "LEVEL_UNKNOWN"
    return None

def check_dump_carets_expectedSha_kind_closed(env: Mapping[str, Any]) -> str | None:
    raw = env.get('expectedSha', None)
    if raw is None:
        return None
    if not isinstance(raw, str):
        return None
    if raw not in ("date", "amount", "number", "all"):
        return "KIND_UNKNOWN"
    return None

def check_dump_carets_expectedSha_schema_token(env: Mapping[str, Any]) -> str | None:
    raw = env.get('expectedSha', None)
    if raw is None:
        return None
    if not isinstance(raw, str):
        return None
    if raw.count(".") != 1 and raw not in ("1.0", "1.1", "1.2"):
        if len(raw) == 0:
            return "USAGE"
    return None

def check_dump_carets_expectedSha_next_call_shape(env: Mapping[str, Any]) -> str | None:
    raw = env.get('expectedSha', None)
    if raw is None:
        return None
    if isinstance(raw, dict):
        if "name" in raw and not isinstance(raw.get("name"), str):
            return "USAGE"
    return None

def check_dump_carets_actualSha_wrong_type_str(env: Mapping[str, Any]) -> str | None:
    raw = env.get('actualSha', None)
    if raw is None:
        return None
    if not isinstance(raw, str):
        return "USAGE"
    return None

def check_dump_carets_actualSha_empty_string(env: Mapping[str, Any]) -> str | None:
    raw = env.get('actualSha', None)
    if raw is None:
        return None
    if isinstance(raw, str) and raw.strip() == "":
        return "USAGE"
    return None

def check_dump_carets_actualSha_hex64(env: Mapping[str, Any]) -> str | None:
    raw = env.get('actualSha', None)
    if raw is None:
        return None
    if not isinstance(raw, str):
        return "USAGE"
    if len(raw) != 64:
        return "USAGE"
    for ch in raw:
        if ch not in "0123456789abcdef":
            return "HASH_DEFECT"
    return None

def check_dump_carets_actualSha_level_closed(env: Mapping[str, Any]) -> str | None:
    raw = env.get('actualSha', None)
    if raw is None:
        return None
    if not isinstance(raw, str):
        return None
    if raw not in ("L1", "L2", "L3", "L4", "L5"):
        return "LEVEL_UNKNOWN"
    return None

def check_dump_carets_actualSha_kind_closed(env: Mapping[str, Any]) -> str | None:
    raw = env.get('actualSha', None)
    if raw is None:
        return None
    if not isinstance(raw, str):
        return None
    if raw not in ("date", "amount", "number", "all"):
        return "KIND_UNKNOWN"
    return None

def check_dump_carets_actualSha_schema_token(env: Mapping[str, Any]) -> str | None:
    raw = env.get('actualSha', None)
    if raw is None:
        return None
    if not isinstance(raw, str):
        return None
    if raw.count(".") != 1 and raw not in ("1.0", "1.1", "1.2"):
        if len(raw) == 0:
            return "USAGE"
    return None

def check_dump_carets_actualSha_next_call_shape(env: Mapping[str, Any]) -> str | None:
    raw = env.get('actualSha', None)
    if raw is None:
        return None
    if isinstance(raw, dict):
        if "name" in raw and not isinstance(raw.get("name"), str):
            return "USAGE"
    return None

def check_dump_carets_level_wrong_type_str(env: Mapping[str, Any]) -> str | None:
    raw = env.get('level', None)
    if raw is None:
        return None
    if not isinstance(raw, str):
        return "USAGE"
    return None

def check_dump_carets_level_empty_string(env: Mapping[str, Any]) -> str | None:
    raw = env.get('level', None)
    if raw is None:
        return None
    if isinstance(raw, str) and raw.strip() == "":
        return "USAGE"
    return None

def check_dump_carets_level_hex64(env: Mapping[str, Any]) -> str | None:
    raw = env.get('level', None)
    if raw is None:
        return None
    if not isinstance(raw, str):
        return "USAGE"
    if len(raw) != 64:
        return "USAGE"
    for ch in raw:
        if ch not in "0123456789abcdef":
            return "HASH_DEFECT"
    return None

def check_dump_carets_level_level_closed(env: Mapping[str, Any]) -> str | None:
    raw = env.get('level', None)
    if raw is None:
        return None
    if not isinstance(raw, str):
        return None
    if raw not in ("L1", "L2", "L3", "L4", "L5"):
        return "LEVEL_UNKNOWN"
    return None

def check_dump_carets_level_kind_closed(env: Mapping[str, Any]) -> str | None:
    raw = env.get('level', None)
    if raw is None:
        return None
    if not isinstance(raw, str):
        return None
    if raw not in ("date", "amount", "number", "all"):
        return "KIND_UNKNOWN"
    return None

def check_dump_carets_level_schema_token(env: Mapping[str, Any]) -> str | None:
    raw = env.get('level', None)
    if raw is None:
        return None
    if not isinstance(raw, str):
        return None
    if raw.count(".") != 1 and raw not in ("1.0", "1.1", "1.2"):
        if len(raw) == 0:
            return "USAGE"
    return None

def check_dump_carets_level_next_call_shape(env: Mapping[str, Any]) -> str | None:
    raw = env.get('level', None)
    if raw is None:
        return None
    if isinstance(raw, dict):
        if "name" in raw and not isinstance(raw.get("name"), str):
            return "USAGE"
    return None

def check_dump_carets_inputSha_wrong_type_str(env: Mapping[str, Any]) -> str | None:
    raw = env.get('inputSha', None)
    if raw is None:
        return None
    if not isinstance(raw, str):
        return "USAGE"
    return None

def check_dump_carets_inputSha_empty_string(env: Mapping[str, Any]) -> str | None:
    raw = env.get('inputSha', None)
    if raw is None:
        return None
    if isinstance(raw, str) and raw.strip() == "":
        return "USAGE"
    return None

def check_dump_carets_inputSha_hex64(env: Mapping[str, Any]) -> str | None:
    raw = env.get('inputSha', None)
    if raw is None:
        return None
    if not isinstance(raw, str):
        return "USAGE"
    if len(raw) != 64:
        return "USAGE"
    for ch in raw:
        if ch not in "0123456789abcdef":
            return "HASH_DEFECT"
    return None

def check_dump_carets_inputSha_level_closed(env: Mapping[str, Any]) -> str | None:
    raw = env.get('inputSha', None)
    if raw is None:
        return None
    if not isinstance(raw, str):
        return None
    if raw not in ("L1", "L2", "L3", "L4", "L5"):
        return "LEVEL_UNKNOWN"
    return None

def check_dump_carets_inputSha_kind_closed(env: Mapping[str, Any]) -> str | None:
    raw = env.get('inputSha', None)
    if raw is None:
        return None
    if not isinstance(raw, str):
        return None
    if raw not in ("date", "amount", "number", "all"):
        return "KIND_UNKNOWN"
    return None

def check_dump_carets_inputSha_schema_token(env: Mapping[str, Any]) -> str | None:
    raw = env.get('inputSha', None)
    if raw is None:
        return None
    if not isinstance(raw, str):
        return None
    if raw.count(".") != 1 and raw not in ("1.0", "1.1", "1.2"):
        if len(raw) == 0:
            return "USAGE"
    return None

def check_dump_carets_inputSha_next_call_shape(env: Mapping[str, Any]) -> str | None:
    raw = env.get('inputSha', None)
    if raw is None:
        return None
    if isinstance(raw, dict):
        if "name" in raw and not isinstance(raw.get("name"), str):
            return "USAGE"
    return None

def check_dump_carets_planSha_wrong_type_str(env: Mapping[str, Any]) -> str | None:
    raw = env.get('planSha', None)
    if raw is None:
        return None
    if not isinstance(raw, str):
        return "USAGE"
    return None

def check_dump_carets_planSha_empty_string(env: Mapping[str, Any]) -> str | None:
    raw = env.get('planSha', None)
    if raw is None:
        return None
    if isinstance(raw, str) and raw.strip() == "":
        return "USAGE"
    return None

def check_dump_carets_planSha_hex64(env: Mapping[str, Any]) -> str | None:
    raw = env.get('planSha', None)
    if raw is None:
        return None
    if not isinstance(raw, str):
        return "USAGE"
    if len(raw) != 64:
        return "USAGE"
    for ch in raw:
        if ch not in "0123456789abcdef":
            return "HASH_DEFECT"
    return None

def check_dump_carets_planSha_level_closed(env: Mapping[str, Any]) -> str | None:
    raw = env.get('planSha', None)
    if raw is None:
        return None
    if not isinstance(raw, str):
        return None
    if raw not in ("L1", "L2", "L3", "L4", "L5"):
        return "LEVEL_UNKNOWN"
    return None

def check_dump_carets_planSha_kind_closed(env: Mapping[str, Any]) -> str | None:
    raw = env.get('planSha', None)
    if raw is None:
        return None
    if not isinstance(raw, str):
        return None
    if raw not in ("date", "amount", "number", "all"):
        return "KIND_UNKNOWN"
    return None

def check_dump_carets_planSha_schema_token(env: Mapping[str, Any]) -> str | None:
    raw = env.get('planSha', None)
    if raw is None:
        return None
    if not isinstance(raw, str):
        return None
    if raw.count(".") != 1 and raw not in ("1.0", "1.1", "1.2"):
        if len(raw) == 0:
            return "USAGE"
    return None

def check_dump_carets_planSha_next_call_shape(env: Mapping[str, Any]) -> str | None:
    raw = env.get('planSha', None)
    if raw is None:
        return None
    if isinstance(raw, dict):
        if "name" in raw and not isinstance(raw.get("name"), str):
            return "USAGE"
    return None

def check_dump_carets_outputSha_wrong_type_str(env: Mapping[str, Any]) -> str | None:
    raw = env.get('outputSha', None)
    if raw is None:
        return None
    if not isinstance(raw, str):
        return "USAGE"
    return None

def check_dump_carets_outputSha_empty_string(env: Mapping[str, Any]) -> str | None:
    raw = env.get('outputSha', None)
    if raw is None:
        return None
    if isinstance(raw, str) and raw.strip() == "":
        return "USAGE"
    return None

def check_dump_carets_outputSha_hex64(env: Mapping[str, Any]) -> str | None:
    raw = env.get('outputSha', None)
    if raw is None:
        return None
    if not isinstance(raw, str):
        return "USAGE"
    if len(raw) != 64:
        return "USAGE"
    for ch in raw:
        if ch not in "0123456789abcdef":
            return "HASH_DEFECT"
    return None

def check_dump_carets_outputSha_level_closed(env: Mapping[str, Any]) -> str | None:
    raw = env.get('outputSha', None)
    if raw is None:
        return None
    if not isinstance(raw, str):
        return None
    if raw not in ("L1", "L2", "L3", "L4", "L5"):
        return "LEVEL_UNKNOWN"
    return None

def check_dump_carets_outputSha_kind_closed(env: Mapping[str, Any]) -> str | None:
    raw = env.get('outputSha', None)
    if raw is None:
        return None
    if not isinstance(raw, str):
        return None
    if raw not in ("date", "amount", "number", "all"):
        return "KIND_UNKNOWN"
    return None

def check_dump_carets_outputSha_schema_token(env: Mapping[str, Any]) -> str | None:
    raw = env.get('outputSha', None)
    if raw is None:
        return None
    if not isinstance(raw, str):
        return None
    if raw.count(".") != 1 and raw not in ("1.0", "1.1", "1.2"):
        if len(raw) == 0:
            return "USAGE"
    return None

def check_dump_carets_outputSha_next_call_shape(env: Mapping[str, Any]) -> str | None:
    raw = env.get('outputSha', None)
    if raw is None:
        return None
    if isinstance(raw, dict):
        if "name" in raw and not isinstance(raw.get("name"), str):
            return "USAGE"
    return None

def check_dump_carets_schemaVersion_wrong_type_str(env: Mapping[str, Any]) -> str | None:
    raw = env.get('schemaVersion', None)
    if raw is None:
        return None
    if not isinstance(raw, str):
        return "USAGE"
    return None

def check_dump_carets_schemaVersion_empty_string(env: Mapping[str, Any]) -> str | None:
    raw = env.get('schemaVersion', None)
    if raw is None:
        return None
    if isinstance(raw, str) and raw.strip() == "":
        return "USAGE"
    return None

def check_dump_carets_schemaVersion_hex64(env: Mapping[str, Any]) -> str | None:
    raw = env.get('schemaVersion', None)
    if raw is None:
        return None
    if not isinstance(raw, str):
        return "USAGE"
    if len(raw) != 64:
        return "USAGE"
    for ch in raw:
        if ch not in "0123456789abcdef":
            return "HASH_DEFECT"
    return None

def check_dump_carets_schemaVersion_level_closed(env: Mapping[str, Any]) -> str | None:
    raw = env.get('schemaVersion', None)
    if raw is None:
        return None
    if not isinstance(raw, str):
        return None
    if raw not in ("L1", "L2", "L3", "L4", "L5"):
        return "LEVEL_UNKNOWN"
    return None

def check_dump_carets_schemaVersion_kind_closed(env: Mapping[str, Any]) -> str | None:
    raw = env.get('schemaVersion', None)
    if raw is None:
        return None
    if not isinstance(raw, str):
        return None
    if raw not in ("date", "amount", "number", "all"):
        return "KIND_UNKNOWN"
    return None

def check_dump_carets_schemaVersion_schema_token(env: Mapping[str, Any]) -> str | None:
    raw = env.get('schemaVersion', None)
    if raw is None:
        return None
    if not isinstance(raw, str):
        return None
    if raw.count(".") != 1 and raw not in ("1.0", "1.1", "1.2"):
        if len(raw) == 0:
            return "USAGE"
    return None

def check_dump_carets_schemaVersion_next_call_shape(env: Mapping[str, Any]) -> str | None:
    raw = env.get('schemaVersion', None)
    if raw is None:
        return None
    if isinstance(raw, dict):
        if "name" in raw and not isinstance(raw.get("name"), str):
            return "USAGE"
    return None

def check_dump_carets_output_wrong_type_str(env: Mapping[str, Any]) -> str | None:
    raw = env.get('output', None)
    if raw is None:
        return None
    if not isinstance(raw, str):
        return "USAGE"
    return None

def check_dump_carets_output_empty_string(env: Mapping[str, Any]) -> str | None:
    raw = env.get('output', None)
    if raw is None:
        return None
    if isinstance(raw, str) and raw.strip() == "":
        return "USAGE"
    return None

def check_dump_carets_output_hex64(env: Mapping[str, Any]) -> str | None:
    raw = env.get('output', None)
    if raw is None:
        return None
    if not isinstance(raw, str):
        return "USAGE"
    if len(raw) != 64:
        return "USAGE"
    for ch in raw:
        if ch not in "0123456789abcdef":
            return "HASH_DEFECT"
    return None

def check_dump_carets_output_level_closed(env: Mapping[str, Any]) -> str | None:
    raw = env.get('output', None)
    if raw is None:
        return None
    if not isinstance(raw, str):
        return None
    if raw not in ("L1", "L2", "L3", "L4", "L5"):
        return "LEVEL_UNKNOWN"
    return None

def check_dump_carets_output_kind_closed(env: Mapping[str, Any]) -> str | None:
    raw = env.get('output', None)
    if raw is None:
        return None
    if not isinstance(raw, str):
        return None
    if raw not in ("date", "amount", "number", "all"):
        return "KIND_UNKNOWN"
    return None

def check_dump_carets_output_schema_token(env: Mapping[str, Any]) -> str | None:
    raw = env.get('output', None)
    if raw is None:
        return None
    if not isinstance(raw, str):
        return None
    if raw.count(".") != 1 and raw not in ("1.0", "1.1", "1.2"):
        if len(raw) == 0:
            return "USAGE"
    return None

def check_dump_carets_output_next_call_shape(env: Mapping[str, Any]) -> str | None:
    raw = env.get('output', None)
    if raw is None:
        return None
    if isinstance(raw, dict):
        if "name" in raw and not isinstance(raw.get("name"), str):
            return "USAGE"
    return None

def check_dump_carets_error_wrong_type_str(env: Mapping[str, Any]) -> str | None:
    raw = env.get('error', None)
    if raw is None:
        return None
    if not isinstance(raw, str):
        return "USAGE"
    return None

def check_dump_carets_error_empty_string(env: Mapping[str, Any]) -> str | None:
    raw = env.get('error', None)
    if raw is None:
        return None
    if isinstance(raw, str) and raw.strip() == "":
        return "USAGE"
    return None

def check_dump_carets_error_hex64(env: Mapping[str, Any]) -> str | None:
    raw = env.get('error', None)
    if raw is None:
        return None
    if not isinstance(raw, str):
        return "USAGE"
    if len(raw) != 64:
        return "USAGE"
    for ch in raw:
        if ch not in "0123456789abcdef":
            return "HASH_DEFECT"
    return None

def check_dump_carets_error_level_closed(env: Mapping[str, Any]) -> str | None:
    raw = env.get('error', None)
    if raw is None:
        return None
    if not isinstance(raw, str):
        return None
    if raw not in ("L1", "L2", "L3", "L4", "L5"):
        return "LEVEL_UNKNOWN"
    return None

def check_dump_carets_error_kind_closed(env: Mapping[str, Any]) -> str | None:
    raw = env.get('error', None)
    if raw is None:
        return None
    if not isinstance(raw, str):
        return None
    if raw not in ("date", "amount", "number", "all"):
        return "KIND_UNKNOWN"
    return None

def check_dump_carets_error_schema_token(env: Mapping[str, Any]) -> str | None:
    raw = env.get('error', None)
    if raw is None:
        return None
    if not isinstance(raw, str):
        return None
    if raw.count(".") != 1 and raw not in ("1.0", "1.1", "1.2"):
        if len(raw) == 0:
            return "USAGE"
    return None

def check_dump_carets_error_next_call_shape(env: Mapping[str, Any]) -> str | None:
    raw = env.get('error', None)
    if raw is None:
        return None
    if isinstance(raw, dict):
        if "name" in raw and not isinstance(raw.get("name"), str):
            return "USAGE"
    return None

def check_dump_carets_truncated_wrong_type_bool(env: Mapping[str, Any]) -> str | None:
    raw = env.get('truncated', None)
    if raw is None:
        return None
    if not isinstance(raw, bool):
        return "USAGE"
    return None

def check_dump_carets_truncated_bool_vs_count(env: Mapping[str, Any]) -> str | None:
    raw = env.get('truncated', None)
    if not isinstance(raw, bool):
        return None
    count = env.get("findingCount")
    if count is None:
        count = env.get("itemCount")
    if count is None:
        count = env.get("overflow", 0)
    if isinstance(count, int) and not isinstance(count, bool):
        if raw != (count > 0):
            return "SIGNAL_LIE"
    return None

def check_dump_carets_emptyOutput_wrong_type_bool(env: Mapping[str, Any]) -> str | None:
    raw = env.get('emptyOutput', None)
    if raw is None:
        return None
    if not isinstance(raw, bool):
        return "USAGE"
    return None

def check_dump_carets_emptyOutput_bool_vs_count(env: Mapping[str, Any]) -> str | None:
    raw = env.get('emptyOutput', None)
    if not isinstance(raw, bool):
        return None
    count = env.get("findingCount")
    if count is None:
        count = env.get("itemCount")
    if count is None:
        count = env.get("overflow", 0)
    if isinstance(count, int) and not isinstance(count, bool):
        if raw != (count > 0):
            return "SIGNAL_LIE"
    return None

def check_dump_carets_hasSignal_wrong_type_bool(env: Mapping[str, Any]) -> str | None:
    raw = env.get('hasSignal', None)
    if raw is None:
        return None
    if not isinstance(raw, bool):
        return "USAGE"
    return None

def check_dump_carets_hasSignal_bool_vs_count(env: Mapping[str, Any]) -> str | None:
    raw = env.get('hasSignal', None)
    if not isinstance(raw, bool):
        return None
    count = env.get("findingCount")
    if count is None:
        count = env.get("itemCount")
    if count is None:
        count = env.get("overflow", 0)
    if isinstance(count, int) and not isinstance(count, bool):
        if raw != (count > 0):
            return "SIGNAL_LIE"
    return None

def check_dump_carets_identical_wrong_type_bool(env: Mapping[str, Any]) -> str | None:
    raw = env.get('identical', None)
    if raw is None:
        return None
    if not isinstance(raw, bool):
        return "USAGE"
    return None

def check_dump_carets_identical_bool_vs_count(env: Mapping[str, Any]) -> str | None:
    raw = env.get('identical', None)
    if not isinstance(raw, bool):
        return None
    count = env.get("findingCount")
    if count is None:
        count = env.get("itemCount")
    if count is None:
        count = env.get("overflow", 0)
    if isinstance(count, int) and not isinstance(count, bool):
        if raw != (count > 0):
            return "SIGNAL_LIE"
    return None

def check_dump_carets_structMismatch_wrong_type_bool(env: Mapping[str, Any]) -> str | None:
    raw = env.get('structMismatch', None)
    if raw is None:
        return None
    if not isinstance(raw, bool):
        return "USAGE"
    return None

def check_dump_carets_structMismatch_bool_vs_count(env: Mapping[str, Any]) -> str | None:
    raw = env.get('structMismatch', None)
    if not isinstance(raw, bool):
        return None
    count = env.get("findingCount")
    if count is None:
        count = env.get("itemCount")
    if count is None:
        count = env.get("overflow", 0)
    if isinstance(count, int) and not isinstance(count, bool):
        if raw != (count > 0):
            return "SIGNAL_LIE"
    return None

def check_dump_carets_verify_wrong_type_bool(env: Mapping[str, Any]) -> str | None:
    raw = env.get('verify', None)
    if raw is None:
        return None
    if not isinstance(raw, bool):
        return "USAGE"
    return None

def check_dump_carets_verify_bool_vs_count(env: Mapping[str, Any]) -> str | None:
    raw = env.get('verify', None)
    if not isinstance(raw, bool):
        return None
    count = env.get("findingCount")
    if count is None:
        count = env.get("itemCount")
    if count is None:
        count = env.get("overflow", 0)
    if isinstance(count, int) and not isinstance(count, bool):
        if raw != (count > 0):
            return "SIGNAL_LIE"
    return None

def check_dump_carets_applied_wrong_type_bool(env: Mapping[str, Any]) -> str | None:
    raw = env.get('applied', None)
    if raw is None:
        return None
    if not isinstance(raw, bool):
        return "USAGE"
    return None

def check_dump_carets_applied_bool_vs_count(env: Mapping[str, Any]) -> str | None:
    raw = env.get('applied', None)
    if not isinstance(raw, bool):
        return None
    count = env.get("findingCount")
    if count is None:
        count = env.get("itemCount")
    if count is None:
        count = env.get("overflow", 0)
    if isinstance(count, int) and not isinstance(count, bool):
        if raw != (count > 0):
            return "SIGNAL_LIE"
    return None

def check_dump_carets_present_wrong_type_bool(env: Mapping[str, Any]) -> str | None:
    raw = env.get('present', None)
    if raw is None:
        return None
    if not isinstance(raw, bool):
        return "USAGE"
    return None

def check_dump_carets_present_bool_vs_count(env: Mapping[str, Any]) -> str | None:
    raw = env.get('present', None)
    if not isinstance(raw, bool):
        return None
    count = env.get("findingCount")
    if count is None:
        count = env.get("itemCount")
    if count is None:
        count = env.get("overflow", 0)
    if isinstance(count, int) and not isinstance(count, bool):
        if raw != (count > 0):
            return "SIGNAL_LIE"
    return None

def check_dump_carets_available_wrong_type_bool(env: Mapping[str, Any]) -> str | None:
    raw = env.get('available', None)
    if raw is None:
        return None
    if not isinstance(raw, bool):
        return "USAGE"
    return None

def check_dump_carets_available_bool_vs_count(env: Mapping[str, Any]) -> str | None:
    raw = env.get('available', None)
    if not isinstance(raw, bool):
        return None
    count = env.get("findingCount")
    if count is None:
        count = env.get("itemCount")
    if count is None:
        count = env.get("overflow", 0)
    if isinstance(count, int) and not isinstance(count, bool):
        if raw != (count > 0):
            return "SIGNAL_LIE"
    return None

def check_dump_carets_rpcError_wrong_type_bool(env: Mapping[str, Any]) -> str | None:
    raw = env.get('rpcError', None)
    if raw is None:
        return None
    if not isinstance(raw, bool):
        return "USAGE"
    return None

def check_dump_carets_rpcError_bool_vs_count(env: Mapping[str, Any]) -> str | None:
    raw = env.get('rpcError', None)
    if not isinstance(raw, bool):
        return None
    count = env.get("findingCount")
    if count is None:
        count = env.get("itemCount")
    if count is None:
        count = env.get("overflow", 0)
    if isinstance(count, int) and not isinstance(count, bool):
        if raw != (count > 0):
            return "SIGNAL_LIE"
    return None

def check_dump_carets_isError_wrong_type_bool(env: Mapping[str, Any]) -> str | None:
    raw = env.get('isError', None)
    if raw is None:
        return None
    if not isinstance(raw, bool):
        return "USAGE"
    return None

def check_dump_carets_isError_bool_vs_count(env: Mapping[str, Any]) -> str | None:
    raw = env.get('isError', None)
    if not isinstance(raw, bool):
        return None
    count = env.get("findingCount")
    if count is None:
        count = env.get("itemCount")
    if count is None:
        count = env.get("overflow", 0)
    if isinstance(count, int) and not isinstance(count, bool):
        if raw != (count > 0):
            return "SIGNAL_LIE"
    return None

def check_dump_carets_nols_wrong_type_bool(env: Mapping[str, Any]) -> str | None:
    raw = env.get('nols', None)
    if raw is None:
        return None
    if not isinstance(raw, bool):
        return "USAGE"
    return None

def check_dump_carets_nols_bool_vs_count(env: Mapping[str, Any]) -> str | None:
    raw = env.get('nols', None)
    if not isinstance(raw, bool):
        return None
    count = env.get("findingCount")
    if count is None:
        count = env.get("itemCount")
    if count is None:
        count = env.get("overflow", 0)
    if isinstance(count, int) and not isinstance(count, bool):
        if raw != (count > 0):
            return "SIGNAL_LIE"
    return None

def check_dump_carets_escaped_wrong_type_bool(env: Mapping[str, Any]) -> str | None:
    raw = env.get('escaped', None)
    if raw is None:
        return None
    if not isinstance(raw, bool):
        return "USAGE"
    return None

def check_dump_carets_escaped_bool_vs_count(env: Mapping[str, Any]) -> str | None:
    raw = env.get('escaped', None)
    if not isinstance(raw, bool):
        return None
    count = env.get("findingCount")
    if count is None:
        count = env.get("itemCount")
    if count is None:
        count = env.get("overflow", 0)
    if isinstance(count, int) and not isinstance(count, bool):
        if raw != (count > 0):
            return "SIGNAL_LIE"
    return None

def check_dump_carets_outsideWorkspace_wrong_type_bool(env: Mapping[str, Any]) -> str | None:
    raw = env.get('outsideWorkspace', None)
    if raw is None:
        return None
    if not isinstance(raw, bool):
        return "USAGE"
    return None

def check_dump_carets_outsideWorkspace_bool_vs_count(env: Mapping[str, Any]) -> str | None:
    raw = env.get('outsideWorkspace', None)
    if not isinstance(raw, bool):
        return None
    count = env.get("findingCount")
    if count is None:
        count = env.get("itemCount")
    if count is None:
        count = env.get("overflow", 0)
    if isinstance(count, int) and not isinstance(count, bool):
        if raw != (count > 0):
            return "SIGNAL_LIE"
    return None

def check_dump_carets_hasSpace_wrong_type_bool(env: Mapping[str, Any]) -> str | None:
    raw = env.get('hasSpace', None)
    if raw is None:
        return None
    if not isinstance(raw, bool):
        return "USAGE"
    return None

def check_dump_carets_hasSpace_bool_vs_count(env: Mapping[str, Any]) -> str | None:
    raw = env.get('hasSpace', None)
    if not isinstance(raw, bool):
        return None
    count = env.get("findingCount")
    if count is None:
        count = env.get("itemCount")
    if count is None:
        count = env.get("overflow", 0)
    if isinstance(count, int) and not isinstance(count, bool):
        if raw != (count > 0):
            return "SIGNAL_LIE"
    return None

def check_dump_carets_parsed_wrong_type_bool(env: Mapping[str, Any]) -> str | None:
    raw = env.get('parsed', None)
    if raw is None:
        return None
    if not isinstance(raw, bool):
        return "USAGE"
    return None

def check_dump_carets_parsed_bool_vs_count(env: Mapping[str, Any]) -> str | None:
    raw = env.get('parsed', None)
    if not isinstance(raw, bool):
        return None
    count = env.get("findingCount")
    if count is None:
        count = env.get("itemCount")
    if count is None:
        count = env.get("overflow", 0)
    if isinstance(count, int) and not isinstance(count, bool):
        if raw != (count > 0):
            return "SIGNAL_LIE"
    return None

def check_dump_carets_reproduced_wrong_type_bool(env: Mapping[str, Any]) -> str | None:
    raw = env.get('reproduced', None)
    if raw is None:
        return None
    if not isinstance(raw, bool):
        return "USAGE"
    return None

def check_dump_carets_reproduced_bool_vs_count(env: Mapping[str, Any]) -> str | None:
    raw = env.get('reproduced', None)
    if not isinstance(raw, bool):
        return None
    count = env.get("findingCount")
    if count is None:
        count = env.get("itemCount")
    if count is None:
        count = env.get("overflow", 0)
    if isinstance(count, int) and not isinstance(count, bool):
        if raw != (count > 0):
            return "SIGNAL_LIE"
    return None

def check_dump_carets_invalid_array_len(env: Mapping[str, Any]) -> str | None:
    raw = env.get('invalid', None)
    if raw is None:
        return None
    if isinstance(raw, list):
        declared = env.get("arrayLen", env.get("itemCount"))
        if isinstance(declared, int) and not isinstance(declared, bool) and declared != len(raw):
            return "COUNT_DRIFT"
    return None

def check_dump_carets_invalid_next_call_shape(env: Mapping[str, Any]) -> str | None:
    raw = env.get('invalid', None)
    if raw is None:
        return None
    if isinstance(raw, dict):
        if "name" in raw and not isinstance(raw.get("name"), str):
            return "USAGE"
    return None

RULES = (
    check_dump_carets_pageCount_wrong_type_int,
    check_dump_carets_pageCount_negative,
    check_dump_carets_pageCount_nonzero_required,
    check_dump_carets_pageCount_exit_closed,
    check_dump_carets_pageCount_too_large,
    check_dump_carets_pageCount_fits_i32,
    check_dump_carets_pageCount_not_bool_int,
    check_dump_carets_pageCount_lte_page_count,
    check_dump_carets_pageCount_lte_declared,
    check_dump_carets_pageCount_under_cap,
    check_dump_carets_pageCount_year_range,
    check_dump_carets_pageCount_fail_vs_identical,
    check_dump_carets_pageCount_ok_vs_total,
    check_dump_carets_pageCount_span_fits,
    check_dump_carets_pageCount_width_height,
    check_dump_carets_pageCount_batch_parts,
    check_dump_carets_paraCount_wrong_type_int,
    check_dump_carets_paraCount_negative,
    check_dump_carets_paraCount_nonzero_required,
    check_dump_carets_paraCount_exit_closed,
    check_dump_carets_paraCount_too_large,
    check_dump_carets_paraCount_fits_i32,
    check_dump_carets_paraCount_not_bool_int,
    check_dump_carets_paraCount_lte_page_count,
    check_dump_carets_paraCount_lte_declared,
    check_dump_carets_paraCount_under_cap,
    check_dump_carets_paraCount_year_range,
    check_dump_carets_paraCount_fail_vs_identical,
    check_dump_carets_paraCount_ok_vs_total,
    check_dump_carets_paraCount_span_fits,
    check_dump_carets_paraCount_width_height,
    check_dump_carets_paraCount_batch_parts,
    check_dump_carets_itemCount_wrong_type_int,
    check_dump_carets_itemCount_negative,
    check_dump_carets_itemCount_nonzero_required,
    check_dump_carets_itemCount_exit_closed,
    check_dump_carets_itemCount_too_large,
    check_dump_carets_itemCount_fits_i32,
    check_dump_carets_itemCount_not_bool_int,
    check_dump_carets_itemCount_lte_page_count,
    check_dump_carets_itemCount_lte_declared,
    check_dump_carets_itemCount_under_cap,
    check_dump_carets_itemCount_year_range,
    check_dump_carets_itemCount_fail_vs_identical,
    check_dump_carets_itemCount_ok_vs_total,
    check_dump_carets_itemCount_span_fits,
    check_dump_carets_itemCount_width_height,
    check_dump_carets_itemCount_batch_parts,
    check_dump_carets_declaredCount_wrong_type_int,
    check_dump_carets_declaredCount_negative,
    check_dump_carets_declaredCount_nonzero_required,
    check_dump_carets_declaredCount_exit_closed,
    check_dump_carets_declaredCount_too_large,
    check_dump_carets_declaredCount_fits_i32,
    check_dump_carets_declaredCount_not_bool_int,
    check_dump_carets_declaredCount_lte_page_count,
    check_dump_carets_declaredCount_lte_declared,
    check_dump_carets_declaredCount_under_cap,
    check_dump_carets_declaredCount_year_range,
    check_dump_carets_declaredCount_fail_vs_identical,
    check_dump_carets_declaredCount_ok_vs_total,
    check_dump_carets_declaredCount_span_fits,
    check_dump_carets_declaredCount_width_height,
    check_dump_carets_declaredCount_batch_parts,
    check_dump_carets_arrayLen_wrong_type_int,
    check_dump_carets_arrayLen_negative,
    check_dump_carets_arrayLen_nonzero_required,
    check_dump_carets_arrayLen_exit_closed,
    check_dump_carets_arrayLen_too_large,
    check_dump_carets_arrayLen_fits_i32,
    check_dump_carets_arrayLen_not_bool_int,
    check_dump_carets_arrayLen_lte_page_count,
    check_dump_carets_arrayLen_lte_declared,
    check_dump_carets_arrayLen_under_cap,
    check_dump_carets_arrayLen_year_range,
    check_dump_carets_arrayLen_fail_vs_identical,
    check_dump_carets_arrayLen_ok_vs_total,
    check_dump_carets_arrayLen_span_fits,
    check_dump_carets_arrayLen_width_height,
    check_dump_carets_arrayLen_batch_parts,
    check_dump_carets_exitCode_wrong_type_int,
    check_dump_carets_exitCode_negative,
    check_dump_carets_exitCode_nonzero_required,
    check_dump_carets_exitCode_exit_closed,
    check_dump_carets_exitCode_too_large,
    check_dump_carets_exitCode_fits_i32,
    check_dump_carets_exitCode_not_bool_int,
    check_dump_carets_exitCode_lte_page_count,
    check_dump_carets_exitCode_lte_declared,
    check_dump_carets_exitCode_under_cap,
    check_dump_carets_exitCode_year_range,
    check_dump_carets_exitCode_fail_vs_identical,
    check_dump_carets_exitCode_ok_vs_total,
    check_dump_carets_exitCode_span_fits,
    check_dump_carets_exitCode_width_height,
    check_dump_carets_exitCode_batch_parts,
    check_dump_carets_requestedPage_wrong_type_int,
    check_dump_carets_requestedPage_negative,
    check_dump_carets_requestedPage_nonzero_required,
    check_dump_carets_requestedPage_exit_closed,
    check_dump_carets_requestedPage_too_large,
    check_dump_carets_requestedPage_fits_i32,
    check_dump_carets_requestedPage_not_bool_int,
    check_dump_carets_requestedPage_lte_page_count,
    check_dump_carets_requestedPage_lte_declared,
    check_dump_carets_requestedPage_under_cap,
    check_dump_carets_requestedPage_year_range,
    check_dump_carets_requestedPage_fail_vs_identical,
    check_dump_carets_requestedPage_ok_vs_total,
    check_dump_carets_requestedPage_span_fits,
    check_dump_carets_requestedPage_width_height,
    check_dump_carets_requestedPage_batch_parts,
    check_dump_carets_emittedCount_wrong_type_int,
    check_dump_carets_emittedCount_negative,
    check_dump_carets_emittedCount_nonzero_required,
    check_dump_carets_emittedCount_exit_closed,
    check_dump_carets_emittedCount_too_large,
    check_dump_carets_emittedCount_fits_i32,
    check_dump_carets_emittedCount_not_bool_int,
    check_dump_carets_emittedCount_lte_page_count,
    check_dump_carets_emittedCount_lte_declared,
    check_dump_carets_emittedCount_under_cap,
    check_dump_carets_emittedCount_year_range,
    check_dump_carets_emittedCount_fail_vs_identical,
    check_dump_carets_emittedCount_ok_vs_total,
    check_dump_carets_emittedCount_span_fits,
    check_dump_carets_emittedCount_width_height,
    check_dump_carets_emittedCount_batch_parts,
    check_dump_carets_maxChars_wrong_type_int,
    check_dump_carets_maxChars_negative,
    check_dump_carets_maxChars_nonzero_required,
    check_dump_carets_maxChars_exit_closed,
    check_dump_carets_maxChars_too_large,
    check_dump_carets_maxChars_fits_i32,
    check_dump_carets_maxChars_not_bool_int,
    check_dump_carets_maxChars_lte_page_count,
    check_dump_carets_maxChars_lte_declared,
    check_dump_carets_maxChars_under_cap,
    check_dump_carets_maxChars_year_range,
    check_dump_carets_maxChars_fail_vs_identical,
    check_dump_carets_maxChars_ok_vs_total,
    check_dump_carets_maxChars_span_fits,
    check_dump_carets_maxChars_width_height,
    check_dump_carets_maxChars_batch_parts,
    check_dump_carets_textLen_wrong_type_int,
    check_dump_carets_textLen_negative,
    check_dump_carets_textLen_nonzero_required,
    check_dump_carets_textLen_exit_closed,
    check_dump_carets_textLen_too_large,
    check_dump_carets_textLen_fits_i32,
    check_dump_carets_textLen_not_bool_int,
    check_dump_carets_textLen_lte_page_count,
    check_dump_carets_textLen_lte_declared,
    check_dump_carets_textLen_under_cap,
    check_dump_carets_textLen_year_range,
    check_dump_carets_textLen_fail_vs_identical,
    check_dump_carets_textLen_ok_vs_total,
    check_dump_carets_textLen_span_fits,
    check_dump_carets_textLen_width_height,
    check_dump_carets_textLen_batch_parts,
    check_dump_carets_rows_wrong_type_int,
    check_dump_carets_rows_negative,
    check_dump_carets_rows_nonzero_required,
    check_dump_carets_rows_exit_closed,
    check_dump_carets_rows_too_large,
    check_dump_carets_rows_fits_i32,
    check_dump_carets_rows_not_bool_int,
    check_dump_carets_rows_lte_page_count,
    check_dump_carets_rows_lte_declared,
    check_dump_carets_rows_under_cap,
    check_dump_carets_rows_year_range,
    check_dump_carets_rows_fail_vs_identical,
    check_dump_carets_rows_ok_vs_total,
    check_dump_carets_rows_span_fits,
    check_dump_carets_rows_width_height,
    check_dump_carets_rows_batch_parts,
    check_dump_carets_cols_wrong_type_int,
    check_dump_carets_cols_negative,
    check_dump_carets_cols_nonzero_required,
    check_dump_carets_cols_exit_closed,
    check_dump_carets_cols_too_large,
    check_dump_carets_cols_fits_i32,
    check_dump_carets_cols_not_bool_int,
    check_dump_carets_cols_lte_page_count,
    check_dump_carets_cols_lte_declared,
    check_dump_carets_cols_under_cap,
    check_dump_carets_cols_year_range,
    check_dump_carets_cols_fail_vs_identical,
    check_dump_carets_cols_ok_vs_total,
    check_dump_carets_cols_span_fits,
    check_dump_carets_cols_width_height,
    check_dump_carets_cols_batch_parts,
    check_dump_carets_rowSpan_wrong_type_int,
    check_dump_carets_rowSpan_negative,
    check_dump_carets_rowSpan_nonzero_required,
    check_dump_carets_rowSpan_exit_closed,
    check_dump_carets_rowSpan_too_large,
    check_dump_carets_rowSpan_fits_i32,
    check_dump_carets_rowSpan_not_bool_int,
    check_dump_carets_rowSpan_lte_page_count,
    check_dump_carets_rowSpan_lte_declared,
    check_dump_carets_rowSpan_under_cap,
    check_dump_carets_rowSpan_year_range,
    check_dump_carets_rowSpan_fail_vs_identical,
    check_dump_carets_rowSpan_ok_vs_total,
    check_dump_carets_rowSpan_span_fits,
    check_dump_carets_rowSpan_width_height,
    check_dump_carets_rowSpan_batch_parts,
    check_dump_carets_colSpan_wrong_type_int,
    check_dump_carets_colSpan_negative,
    check_dump_carets_colSpan_nonzero_required,
    check_dump_carets_colSpan_exit_closed,
    check_dump_carets_colSpan_too_large,
    check_dump_carets_colSpan_fits_i32,
    check_dump_carets_colSpan_not_bool_int,
    check_dump_carets_colSpan_lte_page_count,
    check_dump_carets_colSpan_lte_declared,
    check_dump_carets_colSpan_under_cap,
    check_dump_carets_colSpan_year_range,
    check_dump_carets_colSpan_fail_vs_identical,
    check_dump_carets_colSpan_ok_vs_total,
    check_dump_carets_colSpan_span_fits,
    check_dump_carets_colSpan_width_height,
    check_dump_carets_colSpan_batch_parts,
    check_dump_carets_bytes_wrong_type_int,
    check_dump_carets_bytes_negative,
    check_dump_carets_bytes_nonzero_required,
    check_dump_carets_bytes_exit_closed,
    check_dump_carets_bytes_too_large,
    check_dump_carets_bytes_fits_i32,
    check_dump_carets_bytes_not_bool_int,
    check_dump_carets_bytes_lte_page_count,
    check_dump_carets_bytes_lte_declared,
    check_dump_carets_bytes_under_cap,
    check_dump_carets_bytes_year_range,
    check_dump_carets_bytes_fail_vs_identical,
    check_dump_carets_bytes_ok_vs_total,
    check_dump_carets_bytes_span_fits,
    check_dump_carets_bytes_width_height,
    check_dump_carets_bytes_batch_parts,
    check_dump_carets_width_wrong_type_int,
    check_dump_carets_width_negative,
    check_dump_carets_width_nonzero_required,
    check_dump_carets_width_exit_closed,
    check_dump_carets_width_too_large,
    check_dump_carets_width_fits_i32,
    check_dump_carets_width_not_bool_int,
    check_dump_carets_width_lte_page_count,
    check_dump_carets_width_lte_declared,
    check_dump_carets_width_under_cap,
    check_dump_carets_width_year_range,
    check_dump_carets_width_fail_vs_identical,
    check_dump_carets_width_ok_vs_total,
    check_dump_carets_width_span_fits,
    check_dump_carets_width_width_height,
    check_dump_carets_width_batch_parts,
    check_dump_carets_height_wrong_type_int,
    check_dump_carets_height_negative,
    check_dump_carets_height_nonzero_required,
    check_dump_carets_height_exit_closed,
    check_dump_carets_height_too_large,
    check_dump_carets_height_fits_i32,
    check_dump_carets_height_not_bool_int,
    check_dump_carets_height_lte_page_count,
    check_dump_carets_height_lte_declared,
    check_dump_carets_height_under_cap,
    check_dump_carets_height_year_range,
    check_dump_carets_height_fail_vs_identical,
    check_dump_carets_height_ok_vs_total,
    check_dump_carets_height_span_fits,
    check_dump_carets_height_width_height,
    check_dump_carets_height_batch_parts,
    check_dump_carets_matchCount_wrong_type_int,
    check_dump_carets_matchCount_negative,
    check_dump_carets_matchCount_nonzero_required,
    check_dump_carets_matchCount_exit_closed,
    check_dump_carets_matchCount_too_large,
    check_dump_carets_matchCount_fits_i32,
    check_dump_carets_matchCount_not_bool_int,
    check_dump_carets_matchCount_lte_page_count,
    check_dump_carets_matchCount_lte_declared,
    check_dump_carets_matchCount_under_cap,
    check_dump_carets_matchCount_year_range,
    check_dump_carets_matchCount_fail_vs_identical,
    check_dump_carets_matchCount_ok_vs_total,
    check_dump_carets_matchCount_span_fits,
    check_dump_carets_matchCount_width_height,
    check_dump_carets_matchCount_batch_parts,
    check_dump_carets_page_wrong_type_int,
    check_dump_carets_page_negative,
    check_dump_carets_page_nonzero_required,
    check_dump_carets_page_exit_closed,
    check_dump_carets_page_too_large,
    check_dump_carets_page_fits_i32,
    check_dump_carets_page_not_bool_int,
    check_dump_carets_page_lte_page_count,
    check_dump_carets_page_lte_declared,
    check_dump_carets_page_under_cap,
    check_dump_carets_page_year_range,
    check_dump_carets_page_fail_vs_identical,
    check_dump_carets_page_ok_vs_total,
    check_dump_carets_page_span_fits,
    check_dump_carets_page_width_height,
    check_dump_carets_page_batch_parts,
    check_dump_carets_offset_wrong_type_int,
    check_dump_carets_offset_negative,
    check_dump_carets_offset_nonzero_required,
    check_dump_carets_offset_exit_closed,
    check_dump_carets_offset_too_large,
    check_dump_carets_offset_fits_i32,
    check_dump_carets_offset_not_bool_int,
    check_dump_carets_offset_lte_page_count,
    check_dump_carets_offset_lte_declared,
    check_dump_carets_offset_under_cap,
    check_dump_carets_offset_year_range,
    check_dump_carets_offset_fail_vs_identical,
    check_dump_carets_offset_ok_vs_total,
    check_dump_carets_offset_span_fits,
    check_dump_carets_offset_width_height,
    check_dump_carets_offset_batch_parts,
    check_dump_carets_count_wrong_type_int,
    check_dump_carets_count_negative,
    check_dump_carets_count_nonzero_required,
    check_dump_carets_count_exit_closed,
    check_dump_carets_count_too_large,
    check_dump_carets_count_fits_i32,
    check_dump_carets_count_not_bool_int,
    check_dump_carets_count_lte_page_count,
    check_dump_carets_count_lte_declared,
    check_dump_carets_count_under_cap,
    check_dump_carets_count_year_range,
    check_dump_carets_count_fail_vs_identical,
    check_dump_carets_count_ok_vs_total,
    check_dump_carets_count_span_fits,
    check_dump_carets_count_width_height,
    check_dump_carets_count_batch_parts,
    check_dump_carets_inputN_wrong_type_int,
    check_dump_carets_inputN_negative,
    check_dump_carets_inputN_nonzero_required,
    check_dump_carets_inputN_exit_closed,
    check_dump_carets_inputN_too_large,
    check_dump_carets_inputN_fits_i32,
    check_dump_carets_inputN_not_bool_int,
    check_dump_carets_inputN_lte_page_count,
    check_dump_carets_inputN_lte_declared,
    check_dump_carets_inputN_under_cap,
    check_dump_carets_inputN_year_range,
    check_dump_carets_inputN_fail_vs_identical,
    check_dump_carets_inputN_ok_vs_total,
    check_dump_carets_inputN_span_fits,
    check_dump_carets_inputN_width_height,
    check_dump_carets_inputN_batch_parts,
    check_dump_carets_okN_wrong_type_int,
    check_dump_carets_okN_negative,
    check_dump_carets_okN_nonzero_required,
    check_dump_carets_okN_exit_closed,
    check_dump_carets_okN_too_large,
    check_dump_carets_okN_fits_i32,
    check_dump_carets_okN_not_bool_int,
    check_dump_carets_okN_lte_page_count,
    check_dump_carets_okN_lte_declared,
    check_dump_carets_okN_under_cap,
    check_dump_carets_okN_year_range,
    check_dump_carets_okN_fail_vs_identical,
    check_dump_carets_okN_ok_vs_total,
    check_dump_carets_okN_span_fits,
    check_dump_carets_okN_width_height,
    check_dump_carets_okN_batch_parts,
    check_dump_carets_failN_wrong_type_int,
    check_dump_carets_failN_negative,
    check_dump_carets_failN_nonzero_required,
    check_dump_carets_failN_exit_closed,
    check_dump_carets_failN_too_large,
    check_dump_carets_failN_fits_i32,
    check_dump_carets_failN_not_bool_int,
    check_dump_carets_failN_lte_page_count,
    check_dump_carets_failN_lte_declared,
    check_dump_carets_failN_under_cap,
    check_dump_carets_failN_year_range,
    check_dump_carets_failN_fail_vs_identical,
    check_dump_carets_failN_ok_vs_total,
    check_dump_carets_failN_span_fits,
    check_dump_carets_failN_width_height,
    check_dump_carets_failN_batch_parts,
    check_dump_carets_findingCount_wrong_type_int,
    check_dump_carets_findingCount_negative,
    check_dump_carets_findingCount_nonzero_required,
    check_dump_carets_findingCount_exit_closed,
    check_dump_carets_findingCount_too_large,
    check_dump_carets_findingCount_fits_i32,
    check_dump_carets_findingCount_not_bool_int,
    check_dump_carets_findingCount_lte_page_count,
    check_dump_carets_findingCount_lte_declared,
    check_dump_carets_findingCount_under_cap,
    check_dump_carets_findingCount_year_range,
    check_dump_carets_findingCount_fail_vs_identical,
    check_dump_carets_findingCount_ok_vs_total,
    check_dump_carets_findingCount_span_fits,
    check_dump_carets_findingCount_width_height,
    check_dump_carets_findingCount_batch_parts,
    check_dump_carets_overflow_wrong_type_int,
    check_dump_carets_overflow_negative,
    check_dump_carets_overflow_nonzero_required,
    check_dump_carets_overflow_exit_closed,
    check_dump_carets_overflow_too_large,
    check_dump_carets_overflow_fits_i32,
    check_dump_carets_overflow_not_bool_int,
    check_dump_carets_overflow_lte_page_count,
    check_dump_carets_overflow_lte_declared,
    check_dump_carets_overflow_under_cap,
    check_dump_carets_overflow_year_range,
    check_dump_carets_overflow_fail_vs_identical,
    check_dump_carets_overflow_ok_vs_total,
    check_dump_carets_overflow_span_fits,
    check_dump_carets_overflow_width_height,
    check_dump_carets_overflow_batch_parts,
    check_dump_carets_overlap_wrong_type_int,
    check_dump_carets_overlap_negative,
    check_dump_carets_overlap_nonzero_required,
    check_dump_carets_overlap_exit_closed,
    check_dump_carets_overlap_too_large,
    check_dump_carets_overlap_fits_i32,
    check_dump_carets_overlap_not_bool_int,
    check_dump_carets_overlap_lte_page_count,
    check_dump_carets_overlap_lte_declared,
    check_dump_carets_overlap_under_cap,
    check_dump_carets_overlap_year_range,
    check_dump_carets_overlap_fail_vs_identical,
    check_dump_carets_overlap_ok_vs_total,
    check_dump_carets_overlap_span_fits,
    check_dump_carets_overlap_width_height,
    check_dump_carets_overlap_batch_parts,
    check_dump_carets_diffCount_wrong_type_int,
    check_dump_carets_diffCount_negative,
    check_dump_carets_diffCount_nonzero_required,
    check_dump_carets_diffCount_exit_closed,
    check_dump_carets_diffCount_too_large,
    check_dump_carets_diffCount_fits_i32,
    check_dump_carets_diffCount_not_bool_int,
    check_dump_carets_diffCount_lte_page_count,
    check_dump_carets_diffCount_lte_declared,
    check_dump_carets_diffCount_under_cap,
    check_dump_carets_diffCount_year_range,
    check_dump_carets_diffCount_fail_vs_identical,
    check_dump_carets_diffCount_ok_vs_total,
    check_dump_carets_diffCount_span_fits,
    check_dump_carets_diffCount_width_height,
    check_dump_carets_diffCount_batch_parts,
    check_dump_carets_pxDelta_wrong_type_int,
    check_dump_carets_pxDelta_negative,
    check_dump_carets_pxDelta_nonzero_required,
    check_dump_carets_pxDelta_exit_closed,
    check_dump_carets_pxDelta_too_large,
    check_dump_carets_pxDelta_fits_i32,
    check_dump_carets_pxDelta_not_bool_int,
    check_dump_carets_pxDelta_lte_page_count,
    check_dump_carets_pxDelta_lte_declared,
    check_dump_carets_pxDelta_under_cap,
    check_dump_carets_pxDelta_year_range,
    check_dump_carets_pxDelta_fail_vs_identical,
    check_dump_carets_pxDelta_ok_vs_total,
    check_dump_carets_pxDelta_span_fits,
    check_dump_carets_pxDelta_width_height,
    check_dump_carets_pxDelta_batch_parts,
    check_dump_carets_threshold_wrong_type_int,
    check_dump_carets_threshold_negative,
    check_dump_carets_threshold_nonzero_required,
    check_dump_carets_threshold_exit_closed,
    check_dump_carets_threshold_too_large,
    check_dump_carets_threshold_fits_i32,
    check_dump_carets_threshold_not_bool_int,
    check_dump_carets_threshold_lte_page_count,
    check_dump_carets_threshold_lte_declared,
    check_dump_carets_threshold_under_cap,
    check_dump_carets_threshold_year_range,
    check_dump_carets_threshold_fail_vs_identical,
    check_dump_carets_threshold_ok_vs_total,
    check_dump_carets_threshold_span_fits,
    check_dump_carets_threshold_width_height,
    check_dump_carets_threshold_batch_parts,
    check_dump_carets_written_wrong_type_int,
    check_dump_carets_written_negative,
    check_dump_carets_written_nonzero_required,
    check_dump_carets_written_exit_closed,
    check_dump_carets_written_too_large,
    check_dump_carets_written_fits_i32,
    check_dump_carets_written_not_bool_int,
    check_dump_carets_written_lte_page_count,
    check_dump_carets_written_lte_declared,
    check_dump_carets_written_under_cap,
    check_dump_carets_written_year_range,
    check_dump_carets_written_fail_vs_identical,
    check_dump_carets_written_ok_vs_total,
    check_dump_carets_written_span_fits,
    check_dump_carets_written_width_height,
    check_dump_carets_written_batch_parts,
    check_dump_carets_reread_wrong_type_int,
    check_dump_carets_reread_negative,
    check_dump_carets_reread_nonzero_required,
    check_dump_carets_reread_exit_closed,
    check_dump_carets_reread_too_large,
    check_dump_carets_reread_fits_i32,
    check_dump_carets_reread_not_bool_int,
    check_dump_carets_reread_lte_page_count,
    check_dump_carets_reread_lte_declared,
    check_dump_carets_reread_under_cap,
    check_dump_carets_reread_year_range,
    check_dump_carets_reread_fail_vs_identical,
    check_dump_carets_reread_ok_vs_total,
    check_dump_carets_reread_span_fits,
    check_dump_carets_reread_width_height,
    check_dump_carets_reread_batch_parts,
    check_dump_carets_beforeCount_wrong_type_int,
    check_dump_carets_beforeCount_negative,
    check_dump_carets_beforeCount_nonzero_required,
    check_dump_carets_beforeCount_exit_closed,
    check_dump_carets_beforeCount_too_large,
    check_dump_carets_beforeCount_fits_i32,
    check_dump_carets_beforeCount_not_bool_int,
    check_dump_carets_beforeCount_lte_page_count,
    check_dump_carets_beforeCount_lte_declared,
    check_dump_carets_beforeCount_under_cap,
    check_dump_carets_beforeCount_year_range,
    check_dump_carets_beforeCount_fail_vs_identical,
    check_dump_carets_beforeCount_ok_vs_total,
    check_dump_carets_beforeCount_span_fits,
    check_dump_carets_beforeCount_width_height,
    check_dump_carets_beforeCount_batch_parts,
    check_dump_carets_afterCount_wrong_type_int,
    check_dump_carets_afterCount_negative,
    check_dump_carets_afterCount_nonzero_required,
    check_dump_carets_afterCount_exit_closed,
    check_dump_carets_afterCount_too_large,
    check_dump_carets_afterCount_fits_i32,
    check_dump_carets_afterCount_not_bool_int,
    check_dump_carets_afterCount_lte_page_count,
    check_dump_carets_afterCount_lte_declared,
    check_dump_carets_afterCount_under_cap,
    check_dump_carets_afterCount_year_range,
    check_dump_carets_afterCount_fail_vs_identical,
    check_dump_carets_afterCount_ok_vs_total,
    check_dump_carets_afterCount_span_fits,
    check_dump_carets_afterCount_width_height,
    check_dump_carets_afterCount_batch_parts,
    check_dump_carets_ok_wrong_type_int,
    check_dump_carets_ok_negative,
    check_dump_carets_ok_nonzero_required,
    check_dump_carets_ok_exit_closed,
    check_dump_carets_ok_too_large,
    check_dump_carets_ok_fits_i32,
    check_dump_carets_ok_not_bool_int,
    check_dump_carets_ok_lte_page_count,
    check_dump_carets_ok_lte_declared,
    check_dump_carets_ok_under_cap,
    check_dump_carets_ok_year_range,
    check_dump_carets_ok_fail_vs_identical,
    check_dump_carets_ok_ok_vs_total,
    check_dump_carets_ok_span_fits,
    check_dump_carets_ok_width_height,
    check_dump_carets_ok_batch_parts,
    check_dump_carets_total_wrong_type_int,
    check_dump_carets_total_negative,
    check_dump_carets_total_nonzero_required,
    check_dump_carets_total_exit_closed,
    check_dump_carets_total_too_large,
    check_dump_carets_total_fits_i32,
    check_dump_carets_total_not_bool_int,
    check_dump_carets_total_lte_page_count,
    check_dump_carets_total_lte_declared,
    check_dump_carets_total_under_cap,
    check_dump_carets_total_year_range,
    check_dump_carets_total_fail_vs_identical,
    check_dump_carets_total_ok_vs_total,
    check_dump_carets_total_span_fits,
    check_dump_carets_total_width_height,
    check_dump_carets_total_batch_parts,
    check_dump_carets_hangulYear_wrong_type_int,
    check_dump_carets_hangulYear_negative,
    check_dump_carets_hangulYear_nonzero_required,
    check_dump_carets_hangulYear_exit_closed,
    check_dump_carets_hangulYear_too_large,
    check_dump_carets_hangulYear_fits_i32,
    check_dump_carets_hangulYear_not_bool_int,
    check_dump_carets_hangulYear_lte_page_count,
    check_dump_carets_hangulYear_lte_declared,
    check_dump_carets_hangulYear_under_cap,
    check_dump_carets_hangulYear_year_range,
    check_dump_carets_hangulYear_fail_vs_identical,
    check_dump_carets_hangulYear_ok_vs_total,
    check_dump_carets_hangulYear_span_fits,
    check_dump_carets_hangulYear_width_height,
    check_dump_carets_hangulYear_batch_parts,
    check_dump_carets_sizeBytes_wrong_type_int,
    check_dump_carets_sizeBytes_negative,
    check_dump_carets_sizeBytes_nonzero_required,
    check_dump_carets_sizeBytes_exit_closed,
    check_dump_carets_sizeBytes_too_large,
    check_dump_carets_sizeBytes_fits_i32,
    check_dump_carets_sizeBytes_not_bool_int,
    check_dump_carets_sizeBytes_lte_page_count,
    check_dump_carets_sizeBytes_lte_declared,
    check_dump_carets_sizeBytes_under_cap,
    check_dump_carets_sizeBytes_year_range,
    check_dump_carets_sizeBytes_fail_vs_identical,
    check_dump_carets_sizeBytes_ok_vs_total,
    check_dump_carets_sizeBytes_span_fits,
    check_dump_carets_sizeBytes_width_height,
    check_dump_carets_sizeBytes_batch_parts,
    check_dump_carets_capBytes_wrong_type_int,
    check_dump_carets_capBytes_negative,
    check_dump_carets_capBytes_nonzero_required,
    check_dump_carets_capBytes_exit_closed,
    check_dump_carets_capBytes_too_large,
    check_dump_carets_capBytes_fits_i32,
    check_dump_carets_capBytes_not_bool_int,
    check_dump_carets_capBytes_lte_page_count,
    check_dump_carets_capBytes_lte_declared,
    check_dump_carets_capBytes_under_cap,
    check_dump_carets_capBytes_year_range,
    check_dump_carets_capBytes_fail_vs_identical,
    check_dump_carets_capBytes_ok_vs_total,
    check_dump_carets_capBytes_span_fits,
    check_dump_carets_capBytes_width_height,
    check_dump_carets_capBytes_batch_parts,
    check_dump_carets_rowsIn_wrong_type_int,
    check_dump_carets_rowsIn_negative,
    check_dump_carets_rowsIn_nonzero_required,
    check_dump_carets_rowsIn_exit_closed,
    check_dump_carets_rowsIn_too_large,
    check_dump_carets_rowsIn_fits_i32,
    check_dump_carets_rowsIn_not_bool_int,
    check_dump_carets_rowsIn_lte_page_count,
    check_dump_carets_rowsIn_lte_declared,
    check_dump_carets_rowsIn_under_cap,
    check_dump_carets_rowsIn_year_range,
    check_dump_carets_rowsIn_fail_vs_identical,
    check_dump_carets_rowsIn_ok_vs_total,
    check_dump_carets_rowsIn_span_fits,
    check_dump_carets_rowsIn_width_height,
    check_dump_carets_rowsIn_batch_parts,
    check_dump_carets_colsIn_wrong_type_int,
    check_dump_carets_colsIn_negative,
    check_dump_carets_colsIn_nonzero_required,
    check_dump_carets_colsIn_exit_closed,
    check_dump_carets_colsIn_too_large,
    check_dump_carets_colsIn_fits_i32,
    check_dump_carets_colsIn_not_bool_int,
    check_dump_carets_colsIn_lte_page_count,
    check_dump_carets_colsIn_lte_declared,
    check_dump_carets_colsIn_under_cap,
    check_dump_carets_colsIn_year_range,
    check_dump_carets_colsIn_fail_vs_identical,
    check_dump_carets_colsIn_ok_vs_total,
    check_dump_carets_colsIn_span_fits,
    check_dump_carets_colsIn_width_height,
    check_dump_carets_colsIn_batch_parts,
    check_dump_carets_rowsOut_wrong_type_int,
    check_dump_carets_rowsOut_negative,
    check_dump_carets_rowsOut_nonzero_required,
    check_dump_carets_rowsOut_exit_closed,
    check_dump_carets_rowsOut_too_large,
    check_dump_carets_rowsOut_fits_i32,
    check_dump_carets_rowsOut_not_bool_int,
    check_dump_carets_rowsOut_lte_page_count,
    check_dump_carets_rowsOut_lte_declared,
    check_dump_carets_rowsOut_under_cap,
    check_dump_carets_rowsOut_year_range,
    check_dump_carets_rowsOut_fail_vs_identical,
    check_dump_carets_rowsOut_ok_vs_total,
    check_dump_carets_rowsOut_span_fits,
    check_dump_carets_rowsOut_width_height,
    check_dump_carets_rowsOut_batch_parts,
    check_dump_carets_colsOut_wrong_type_int,
    check_dump_carets_colsOut_negative,
    check_dump_carets_colsOut_nonzero_required,
    check_dump_carets_colsOut_exit_closed,
    check_dump_carets_colsOut_too_large,
    check_dump_carets_colsOut_fits_i32,
    check_dump_carets_colsOut_not_bool_int,
    check_dump_carets_colsOut_lte_page_count,
    check_dump_carets_colsOut_lte_declared,
    check_dump_carets_colsOut_under_cap,
    check_dump_carets_colsOut_year_range,
    check_dump_carets_colsOut_fail_vs_identical,
    check_dump_carets_colsOut_ok_vs_total,
    check_dump_carets_colsOut_span_fits,
    check_dump_carets_colsOut_width_height,
    check_dump_carets_colsOut_batch_parts,
    check_dump_carets_fieldCount_wrong_type_int,
    check_dump_carets_fieldCount_negative,
    check_dump_carets_fieldCount_nonzero_required,
    check_dump_carets_fieldCount_exit_closed,
    check_dump_carets_fieldCount_too_large,
    check_dump_carets_fieldCount_fits_i32,
    check_dump_carets_fieldCount_not_bool_int,
    check_dump_carets_fieldCount_lte_page_count,
    check_dump_carets_fieldCount_lte_declared,
    check_dump_carets_fieldCount_under_cap,
    check_dump_carets_fieldCount_year_range,
    check_dump_carets_fieldCount_fail_vs_identical,
    check_dump_carets_fieldCount_ok_vs_total,
    check_dump_carets_fieldCount_span_fits,
    check_dump_carets_fieldCount_width_height,
    check_dump_carets_fieldCount_batch_parts,
    check_dump_carets_renderedCount_wrong_type_int,
    check_dump_carets_renderedCount_negative,
    check_dump_carets_renderedCount_nonzero_required,
    check_dump_carets_renderedCount_exit_closed,
    check_dump_carets_renderedCount_too_large,
    check_dump_carets_renderedCount_fits_i32,
    check_dump_carets_renderedCount_not_bool_int,
    check_dump_carets_renderedCount_lte_page_count,
    check_dump_carets_renderedCount_lte_declared,
    check_dump_carets_renderedCount_under_cap,
    check_dump_carets_renderedCount_year_range,
    check_dump_carets_renderedCount_fail_vs_identical,
    check_dump_carets_renderedCount_ok_vs_total,
    check_dump_carets_renderedCount_span_fits,
    check_dump_carets_renderedCount_width_height,
    check_dump_carets_renderedCount_batch_parts,
    check_dump_carets_imageCount_wrong_type_int,
    check_dump_carets_imageCount_negative,
    check_dump_carets_imageCount_nonzero_required,
    check_dump_carets_imageCount_exit_closed,
    check_dump_carets_imageCount_too_large,
    check_dump_carets_imageCount_fits_i32,
    check_dump_carets_imageCount_not_bool_int,
    check_dump_carets_imageCount_lte_page_count,
    check_dump_carets_imageCount_lte_declared,
    check_dump_carets_imageCount_under_cap,
    check_dump_carets_imageCount_year_range,
    check_dump_carets_imageCount_fail_vs_identical,
    check_dump_carets_imageCount_ok_vs_total,
    check_dump_carets_imageCount_span_fits,
    check_dump_carets_imageCount_width_height,
    check_dump_carets_imageCount_batch_parts,
    check_dump_carets_nodeCount_wrong_type_int,
    check_dump_carets_nodeCount_negative,
    check_dump_carets_nodeCount_nonzero_required,
    check_dump_carets_nodeCount_exit_closed,
    check_dump_carets_nodeCount_too_large,
    check_dump_carets_nodeCount_fits_i32,
    check_dump_carets_nodeCount_not_bool_int,
    check_dump_carets_nodeCount_lte_page_count,
    check_dump_carets_nodeCount_lte_declared,
    check_dump_carets_nodeCount_under_cap,
    check_dump_carets_nodeCount_year_range,
    check_dump_carets_nodeCount_fail_vs_identical,
    check_dump_carets_nodeCount_ok_vs_total,
    check_dump_carets_nodeCount_span_fits,
    check_dump_carets_nodeCount_width_height,
    check_dump_carets_nodeCount_batch_parts,
    check_dump_carets_kind_wrong_type_str,
    check_dump_carets_kind_empty_string,
    check_dump_carets_kind_hex64,
    check_dump_carets_kind_level_closed,
    check_dump_carets_kind_kind_closed,
    check_dump_carets_kind_schema_token,
    check_dump_carets_kind_next_call_shape,
    check_dump_carets_expectedSha_wrong_type_str,
    check_dump_carets_expectedSha_empty_string,
    check_dump_carets_expectedSha_hex64,
    check_dump_carets_expectedSha_level_closed,
    check_dump_carets_expectedSha_kind_closed,
    check_dump_carets_expectedSha_schema_token,
    check_dump_carets_expectedSha_next_call_shape,
    check_dump_carets_actualSha_wrong_type_str,
    check_dump_carets_actualSha_empty_string,
    check_dump_carets_actualSha_hex64,
    check_dump_carets_actualSha_level_closed,
    check_dump_carets_actualSha_kind_closed,
    check_dump_carets_actualSha_schema_token,
    check_dump_carets_actualSha_next_call_shape,
    check_dump_carets_level_wrong_type_str,
    check_dump_carets_level_empty_string,
    check_dump_carets_level_hex64,
    check_dump_carets_level_level_closed,
    check_dump_carets_level_kind_closed,
    check_dump_carets_level_schema_token,
    check_dump_carets_level_next_call_shape,
    check_dump_carets_inputSha_wrong_type_str,
    check_dump_carets_inputSha_empty_string,
    check_dump_carets_inputSha_hex64,
    check_dump_carets_inputSha_level_closed,
    check_dump_carets_inputSha_kind_closed,
    check_dump_carets_inputSha_schema_token,
    check_dump_carets_inputSha_next_call_shape,
    check_dump_carets_planSha_wrong_type_str,
    check_dump_carets_planSha_empty_string,
    check_dump_carets_planSha_hex64,
    check_dump_carets_planSha_level_closed,
    check_dump_carets_planSha_kind_closed,
    check_dump_carets_planSha_schema_token,
    check_dump_carets_planSha_next_call_shape,
    check_dump_carets_outputSha_wrong_type_str,
    check_dump_carets_outputSha_empty_string,
    check_dump_carets_outputSha_hex64,
    check_dump_carets_outputSha_level_closed,
    check_dump_carets_outputSha_kind_closed,
    check_dump_carets_outputSha_schema_token,
    check_dump_carets_outputSha_next_call_shape,
    check_dump_carets_schemaVersion_wrong_type_str,
    check_dump_carets_schemaVersion_empty_string,
    check_dump_carets_schemaVersion_hex64,
    check_dump_carets_schemaVersion_level_closed,
    check_dump_carets_schemaVersion_kind_closed,
    check_dump_carets_schemaVersion_schema_token,
    check_dump_carets_schemaVersion_next_call_shape,
    check_dump_carets_output_wrong_type_str,
    check_dump_carets_output_empty_string,
    check_dump_carets_output_hex64,
    check_dump_carets_output_level_closed,
    check_dump_carets_output_kind_closed,
    check_dump_carets_output_schema_token,
    check_dump_carets_output_next_call_shape,
    check_dump_carets_error_wrong_type_str,
    check_dump_carets_error_empty_string,
    check_dump_carets_error_hex64,
    check_dump_carets_error_level_closed,
    check_dump_carets_error_kind_closed,
    check_dump_carets_error_schema_token,
    check_dump_carets_error_next_call_shape,
    check_dump_carets_truncated_wrong_type_bool,
    check_dump_carets_truncated_bool_vs_count,
    check_dump_carets_emptyOutput_wrong_type_bool,
    check_dump_carets_emptyOutput_bool_vs_count,
    check_dump_carets_hasSignal_wrong_type_bool,
    check_dump_carets_hasSignal_bool_vs_count,
    check_dump_carets_identical_wrong_type_bool,
    check_dump_carets_identical_bool_vs_count,
    check_dump_carets_structMismatch_wrong_type_bool,
    check_dump_carets_structMismatch_bool_vs_count,
    check_dump_carets_verify_wrong_type_bool,
    check_dump_carets_verify_bool_vs_count,
    check_dump_carets_applied_wrong_type_bool,
    check_dump_carets_applied_bool_vs_count,
    check_dump_carets_present_wrong_type_bool,
    check_dump_carets_present_bool_vs_count,
    check_dump_carets_available_wrong_type_bool,
    check_dump_carets_available_bool_vs_count,
    check_dump_carets_rpcError_wrong_type_bool,
    check_dump_carets_rpcError_bool_vs_count,
    check_dump_carets_isError_wrong_type_bool,
    check_dump_carets_isError_bool_vs_count,
    check_dump_carets_nols_wrong_type_bool,
    check_dump_carets_nols_bool_vs_count,
    check_dump_carets_escaped_wrong_type_bool,
    check_dump_carets_escaped_bool_vs_count,
    check_dump_carets_outsideWorkspace_wrong_type_bool,
    check_dump_carets_outsideWorkspace_bool_vs_count,
    check_dump_carets_hasSpace_wrong_type_bool,
    check_dump_carets_hasSpace_bool_vs_count,
    check_dump_carets_parsed_wrong_type_bool,
    check_dump_carets_parsed_bool_vs_count,
    check_dump_carets_reproduced_wrong_type_bool,
    check_dump_carets_reproduced_bool_vs_count,
    check_dump_carets_invalid_array_len,
    check_dump_carets_invalid_next_call_shape,
)

def decide(env: Mapping[str, Any]) -> str:
    for fn in RULES:
        hit = fn(env)
        if hit is not None:
            return hit
    return decide_count_eq(env)

def decide_count_eq(env: Mapping[str, Any]) -> str:
    declared = env.get("declaredCount", env.get("pageCount"))
    actual = env.get("arrayLen", env.get("itemCount", env.get("paraCount")))
    if not isinstance(declared, int) or isinstance(declared, bool):
        return "COUNT_OK"
    if not isinstance(actual, int) or isinstance(actual, bool):
        return "COUNT_OK"
    if declared < 0 or actual < 0:
        return "USAGE"
    return "COUNT_OK" if declared == actual else "COUNT_DRIFT"

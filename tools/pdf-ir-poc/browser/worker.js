import init, { compile_pdf_page, compile_pdf_page_painted_rules, compile_pdf_page_tables, HwpDocument } from "./pkg/rhwp_pdf_oxide_poc.js";

const ready = init();
ready.then(()=>self.postMessage({kind:"ready"})).catch(error=>self.postMessage({kind:"fatal",error:String(error)}));
self.onmessage = async ({ data }) => {
  try {
    await ready;
    const started = performance.now();
    const compile=data.tables ? compile_pdf_page_tables : data.paintedRules ? compile_pdf_page_painted_rules : compile_pdf_page;
    const compiled = compile(new Uint8Array(data.bytes), 0);
    const hwpx = compiled.hwpx();
    const report = JSON.parse(compiled.report());
    compiled.free();
    const document = new HwpDocument(hwpx);
    const pageCount = document.pageCount();
    const svg = document.renderPageSvg(0);
    document.free();
    self.postMessage({ id: data.id, report, svg, pageCount, elapsedMs: performance.now() - started, hwpx }, [hwpx.buffer]);
  } catch (error) {
    self.postMessage({ id: data.id, error: String(error) });
  }
};

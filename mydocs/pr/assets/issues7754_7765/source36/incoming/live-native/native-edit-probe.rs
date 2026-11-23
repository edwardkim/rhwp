// Diagnostic preservation probe; public edit APIs and official Native PNG exporter.
use rhwp::wasm_api::HwpDocument;
use rhwp::renderer::font_environment::FontEnvironment;
use rhwp::document_core::queries::{png_export::NativePngExporter,rendering::PngExportOptions};
use rhwp::paint::RenderProfile;
use std::{fs,path::{Path,PathBuf}};
fn path(i:usize)->String {format!(r#"[{{"controlIndex":2,"cellIndex":13,"cellParaIndex":4}},{{"controlIndex":0,"cellIndex":2,"cellParaIndex":{i}}}]"#)}
fn prepare(doc:&mut HwpDocument)->NativePngExporter {
 let mut paths=Vec::<PathBuf>::new();
 let supply=rhwp::renderer::native_webfonts::prepare(&doc.document().doc_info,&paths).unwrap();
 paths.extend(supply.paths.iter().cloned());
 let native=FontEnvironment::from_native_document_substitutions(&doc.document().doc_info,&paths).unwrap();
 doc.set_font_environment(supply.merge_into_environment(native).unwrap()).unwrap();
 let exporter=NativePngExporter::new(RenderProfile::Print,PngExportOptions{scale:Some(1.0),dpi:Some(96.0),font_paths:paths,..Default::default()});
 exporter.prepare_document(doc).unwrap();exporter
}
fn render(doc:&HwpDocument,exporter:&NativePngExporter,out:&Path,lines:usize){
 fs::create_dir_all(out.join("rhwp_png")).unwrap();fs::create_dir_all(out.join("render_tree")).unwrap();
 let mut carets=vec![];
 for i in 1..=lines {carets.push(serde_json::from_str::<serde_json::Value>(&doc.get_cursor_rect_by_path(0,0,&path(i),0).unwrap()).unwrap());}
 for p in 0..doc.page_count(){
  fs::write(out.join("rhwp_png").join(format!("page-{:04}.png",p+1)),exporter.render_page(doc,p).unwrap()).unwrap();
  fs::write(out.join("render_tree").join(format!("page-{:04}.json",p+1)),doc.get_page_render_tree(p).unwrap()).unwrap();
 }
 fs::write(out.join("document.json"),serde_json::to_string_pretty(&serde_json::json!({"page_count":doc.page_count(),"font_environment":doc.font_environment(),"carets":carets})).unwrap()).unwrap();
}
fn main(){
 let a:Vec<String>=std::env::args().collect();let source=Path::new(&a[1]);let lines:usize=a[2].parse().unwrap();let out=Path::new(&a[3]);
 let mut doc=HwpDocument::from_bytes(&fs::read(source).unwrap()).unwrap();doc.set_hangul2024_compat(false);let exporter=prepare(&mut doc);
 for i in 0..lines {let n=doc.get_cell_paragraph_length_by_path(0,0,&path(i)).unwrap();doc.split_paragraph_in_cell_by_path_api(0,0,&path(i),n,None).unwrap();doc.insert_text_in_cell_by_path_api(0,0,&path(i+1),0,&format!("줄 {}",i+1)).unwrap();}
 fs::create_dir_all(out).unwrap();let hwp=doc.export_hwp().unwrap();let hwpx=doc.export_hwpx().unwrap();fs::write(out.join("edited.hwp"),&hwp).unwrap();fs::write(out.join("edited.hwpx"),&hwpx).unwrap();
 render(&doc,&exporter,&out.join("live"),lines);
 for (name,bytes) in [("reopen-hwp",hwp),("reopen-hwpx",hwpx)] {let mut d=HwpDocument::from_bytes(&bytes).unwrap();d.set_hangul2024_compat(false);let ex=prepare(&mut d);render(&d,&ex,&out.join(name),lines);}
}

use rhwp::renderer::{TextStyle,layout::{EmbeddedTextMeasurer,TextMeasurer},native_pua_fonts::NativePuaFonts};
use std::sync::Arc;
fn font(character:char)->Vec<u8>{
 let mut bytes=include_bytes!("/home/tsjang/rhwp/tests/fixtures/fonts/RHWPHftOutline.otf").to_vec();
 let count=u16::from_be_bytes([bytes[4],bytes[5]]) as usize;
 let cmap=(0..count).find_map(|i|{let p=12+i*16;(bytes.get(p..p+4)==Some(b"cmap")).then(||u32::from_be_bytes(bytes[p+8..p+12].try_into().unwrap()) as usize)}).unwrap();
 let sub=cmap+u32::from_be_bytes(bytes[cmap+8..cmap+12].try_into().unwrap()) as usize;
 for offset in [sub+14,sub+20]{bytes[offset..offset+2].copy_from_slice(&(character as u16).to_be_bytes());}
 bytes[sub+24..sub+26].copy_from_slice(&1u16.wrapping_sub(character as u16).to_be_bytes());bytes
}
fn width(style:&TextStyle)->f64{let p=EmbeddedTextMeasurer.compute_char_positions("ᄒᆞᆫ",style);*p.last().unwrap()}
fn main(){
 let mut s=TextStyle{font_family:"HY Sinmyeongjo".into(),font_size:20.0,source_font_is_hft:true,hft_font_face:true,hft_hangul_face:true,original_glyph_metric_family:Some(Arc::new("한양신명조".into())),..Default::default()};
 let before=width(&s);let mut f=NativePuaFonts::default();f.register_otf("display",false,false,&font('\u{f53a}'),0).unwrap();s.native_otf_metrics=f.get_otf("display",false,false);let display=width(&s);s.natural_program_metrics=true;let natural=width(&s);
 println!("{{\"original_hft_width\":{before},\"display_only_program_width\":{display},\"explicit_natural_program_width\":{natural},\"source_primary_identity\":false}}");
}

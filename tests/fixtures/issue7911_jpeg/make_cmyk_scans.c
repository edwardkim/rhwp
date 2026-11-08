#include <stdio.h>
#include <stdlib.h>
#include <jpeglib.h>
int main(int argc, char **argv) {
 if(argc!=3) return 1;
 int mode=atoi(argv[2]); struct jpeg_compress_struct c; struct jpeg_error_mgr e;
 c.err=jpeg_std_error(&e);jpeg_create_compress(&c);FILE *out=fopen(argv[1],"wb");if(!out)return 2;jpeg_stdio_dest(&c,out);
 c.image_width=(mode>=6?257:19);c.image_height=(mode>=6?129:17);c.input_components=4;c.in_color_space=JCS_CMYK;jpeg_set_defaults(&c);jpeg_set_quality(&c,95,TRUE);
 if(mode!=0 && mode!=7) jpeg_set_colorspace(&c,JCS_YCCK);
 jpeg_scan_info scans[4]={0};
 if(mode==0){for(int i=0;i<4;i++){scans[i].comps_in_scan=1;scans[i].component_index[0]=i;scans[i].Ss=0;scans[i].Se=63;}c.scan_info=scans;c.num_scans=4;}
 if(mode>=1 && mode<=5){for(int i=0;i<4;i++){c.comp_info[i].h_samp_factor=1;c.comp_info[i].v_samp_factor=1;}c.comp_info[0].h_samp_factor=(mode==5?3:(mode==1?1:2));c.comp_info[0].v_samp_factor=(mode==2 || mode==5?1:2);}
 if(mode==4 || mode==7)jpeg_simple_progression(&c);
 jpeg_start_compress(&c,TRUE);unsigned char row[257*4];while(c.next_scanline<c.image_height){for(int x=0;x<c.image_width;x++)for(int k=0;k<4;k++)row[4*x+k]=((c.next_scanline*c.image_width+x)*17+k*67)%256;JSAMPROW r=row;jpeg_write_scanlines(&c,&r,1);}jpeg_finish_compress(&c);jpeg_destroy_compress(&c);fclose(out);return 0;
}

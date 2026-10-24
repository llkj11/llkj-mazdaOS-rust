"""Parser checks with synthetic data; no owner firmware is needed in CI."""
import importlib.util
import struct
import tempfile
import unittest
import zlib
from pathlib import Path

spec=importlib.util.spec_from_file_location('importer',Path(__file__).with_name('import-firmware-ui.py'))
importer=importlib.util.module_from_spec(spec)
spec.loader.exec_module(importer)

def chunk(kind,data):
    return struct.pack('>I',len(data))+kind+data+struct.pack('>I',zlib.crc32(kind+data)&0xffffffff)
def png(width,height,color,raw):
    return b'\x89PNG\r\n\x1a\n'+chunk(b'IHDR',struct.pack('>IIBBBBB',width,height,8,color,0,0,0))+chunk(b'IDAT',zlib.compress(raw))+chunk(b'IEND',b'')

class ImportChecks(unittest.TestCase):
    def test_rgb_adds_opaque_alpha_and_rgba_preserves_transparency(self):
        self.assertEqual(importer.png_rgba(png(2,1,2,b'\0\xff\0\0\0\xff\0'))[16:],bytes([255,0,0,255,0,255,0,255]))
        self.assertEqual(importer.png_rgba(png(1,1,6,b'\0\x10\x20\x30\x40'))[16:],bytes([16,32,48,64]))
    def test_all_png_filter_modes_recover_the_same_two_rows(self):
        previous=bytes([8,16,24,32,40,48,56,64]);current=bytes([9,18,27,36,45,54,63,72])
        for mode in range(5):
            encoded=bytearray()
            for i,value in enumerate(current):
                left=current[i-4] if i>=4 else 0
                up=previous[i];corner=previous[i-4] if i>=4 else 0
                p=left+up-corner;dist=[abs(p-left),abs(p-up),abs(p-corner)]
                predictor=[0,left,up,(left+up)//2,[left,up,corner][dist.index(min(dist))]][mode]
                encoded.append((value-predictor)&255)
            self.assertEqual(importer.png_rgba(png(2,2,6,b'\0'+previous+bytes([mode])+encoded))[16:],previous+current)
    def test_crc_corruption_and_truncation_fail(self):
        data=bytearray(png(1,1,6,b'\0\1\2\3\4'));data[20]^=1
        for invalid in [bytes(data),b'not png',png(1,1,6,b'\0\1\2\3\4')[:-5],png(1,1,6,b'\0\1\2\3\4')[:-12]]:
            with self.assertRaises((ValueError,struct.error)): importer.png_rgba(invalid)
    def test_expansion_limits_and_wrong_row_sizes_fail(self):
        for data in [png(1601,1,6,b'\0'),png(1,961,6,b'\0'),png(1,1,6,b'\0'*1_000_000),png(1,1,6,b'\0')]:
            with self.assertRaises(ValueError): importer.png_rgba(data)
    def test_unknown_filter_and_indexed_color_fail(self):
        for data in [png(1,1,6,b'\5\1\2\3\4'),png(1,1,3,b'\0\1')]:
            with self.assertRaises(ValueError): importer.png_rgba(data)
    def test_wrong_package_is_rejected_before_any_extraction(self):
        with tempfile.TemporaryDirectory() as directory:
            path=Path(directory);(path/'wrong.up').write_bytes(b'not reviewed firmware')
            with self.assertRaisesRegex(ValueError,'hash differs'):
                importer.extract(path/'wrong.up',path/'output','unused')
            self.assertFalse((path/'output').exists())

if __name__=='__main__': unittest.main()

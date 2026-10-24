"""Parser checks with synthetic data; no owner firmware is needed in CI."""
import importlib.util
import json
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
    def language_reference(self,directory):
        gui=Path(directory)/'jci/gui';app=gui/'apps/syssettings/js/syssettingsApp.js';app.parent.mkdir(parents=True)
        ids=['LANGS_US_ENGLISH','LANGS_NA_SPANISH','LANGS_CN_FRENCH'];codes=['en_US','es_MX','fr_CN']
        app.write_text('this._cachedLanguagesList='+json.dumps([{'languageID':i,'vrSupport':True,'ttsSupport':False} for i in ids])+'; ignored();')
        framework=gui/'framework/js/Localization.js';framework.parent.mkdir(parents=True);framework.write_text('this.LANG_CODES='+json.dumps(dict(zip(ids,codes)))+'; ignored();')
        for name in ['common','system','syssettings','audiosettings','emnavi','sysupdate','btpairing']:
            parent=gui/'resources/js'/name;parent.mkdir(parents=True)
            for code in codes:
                (parent/(name+'AppDict_'+code+'.js')).write_text('var Localization_fixture='+json.dumps({'Name':'Source' if code=='en_US' else 'témoignage'})+'; ignored();')
        return gui
    def test_language_asset_preserves_source_ids_codes_flags_and_literal_unicode(self):
        with tempfile.TemporaryDirectory() as directory:
            self.language_reference(directory);data=importer.language_assets(Path(directory))
            self.assertEqual(data[:8],b'MZLANG01');self.assertEqual(struct.unpack('<I',data[8:12])[0],3)
            self.assertIn(b'LANGS_NA_SPANISH',data);self.assertIn('témoignage'.encode(),data)
    def test_language_asset_rejects_executable_values_and_unsafe_locale_mapping(self):
        with tempfile.TemporaryDirectory() as directory:
            gui=self.language_reference(directory)
            dictionary=gui/'resources/js/syssettings/syssettingsAppDict_fr_CN.js'
            dictionary.write_text('var Localization_fixture={Name:dangerous()};')
            with self.assertRaises(ValueError):importer.language_assets(Path(directory))
            dictionary.write_text('var Localization_fixture={Name:"safe"};')
            (gui/'framework/js/Localization.js').write_text('this.LANG_CODES={LANGS_US_ENGLISH:"../outside",LANGS_NA_SPANISH:"es_MX",LANGS_CN_FRENCH:"fr_CN"};')
            with self.assertRaises(ValueError):importer.language_assets(Path(directory))
    def test_dictionary_crlf_becomes_one_line_break_but_lone_controls_are_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            gui=self.language_reference(directory)
            dictionary=gui/'resources/js/btpairing/btpairingAppDict_es_MX.js'
            dictionary.write_text('var Localization_fixture='+json.dumps({'Name':'line\r\nnext'})+';')
            data=importer.language_assets(Path(directory))
            self.assertIn(b'line\nnext',data);self.assertNotIn(b'line\r\nnext',data)
            dictionary.write_text('var Localization_fixture='+json.dumps({'Name':'line\rnext'})+';')
            with self.assertRaises(ValueError):importer.language_assets(Path(directory))
    def gif(self,width=1,payload=b'\x44\x01',transparent=True):
        header=b'GIF89a'+struct.pack('<HHBBB',width,1,128,0,0)+b'\0\0\0\xff\xff\xff'
        control=b'!\xf9\x04\x01\0\0\0\0' if transparent else b''
        return header+control+b','+struct.pack('<HHHHB',0,0,width,1,0)+b'\x02'+bytes([len(payload)])+payload+b'\0;'
    def test_static_gif_preserves_palette_transparency_and_lzw_repeat_codes(self):
        self.assertEqual(importer.gif_rgba(self.gif())[16:],b'\0\0\0\0')
        self.assertEqual(importer.gif_rgba(self.gif(3,b'\x8c\x0b',False))[16:],b'\xff'*12)
    def test_gif_rejects_truncation_oversized_frames_and_expansion_or_invalid_codes(self):
        data=self.gif()
        for end in range(len(data)):
            with self.assertRaises(ValueError):importer.gif_rgba(data[:end])
        for invalid in [data+b'extra',self.gif(1,b'\x8c\x0b'),self.gif(1,b'\xfc\x01'),self.gif(1601),self.gif(1,b'\x04')]:
            with self.assertRaises(ValueError):importer.gif_rgba(invalid)
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
    def test_disclosure_accepts_only_literal_concatenation_and_preserves_newlines(self):
        with tempfile.TemporaryDirectory() as directory:
            path=Path(directory)/'jci/gui/apps/syssettings/js/legalDisclosure.js';path.parent.mkdir(parents=True)
            path.write_text('var ScrollDetailCtrlProperties={scrollDetailBody:"First\\n" + /* inert */ "é"}; dangerous();')
            self.assertEqual(importer.legal_text(Path(directory)), 'MZLEGAL1\nFirst\né'.encode())
    def test_disclosure_rejects_calls_properties_malformed_and_control_text(self):
        for text in ['"a"+dangerous()','"a".replace("a","b")','"a"+','"unsafe\\ttext"','"'+('x'*32769)+'"']:
            with tempfile.TemporaryDirectory() as directory:
                path=Path(directory)/'jci/gui/apps/syssettings/js/legalDisclosure.js';path.parent.mkdir(parents=True)
                path.write_text('var ScrollDetailCtrlProperties={scrollDetailBody:'+text+'};')
                with self.assertRaises((ValueError,IndexError)):importer.legal_text(Path(directory))
    def test_clock_labels_preserve_source_order_and_unicode_without_evaluation(self):
        with tempfile.TemporaryDirectory() as directory:
            gui=Path(directory)/'jci/gui';app=gui/'apps/syssettings/js/syssettingsApp.js';app.parent.mkdir(parents=True)
            dictionary=gui/'resources/js/syssettings/syssettingsAppDict_en_US.js';dictionary.parent.mkdir(parents=True)
            app.write_text("this._SelectTimeZoneCtxtDataList={items:[{text1Id:'second'},{text1Id:'first'}]}; dangerous();")
            dictionary.write_text('var Localization_syssettingsAppDict_en_US='+json.dumps({'first':'UTC+01:00 été','second':'UTC-05:00 Eastern'})+'; dangerous();')
            self.assertEqual(importer.clock_zone_text(Path(directory)).decode(),'MZTIMEZONES1\nUTC-05:00 Eastern\nUTC+01:00 été\n')
    def test_clock_labels_reject_expressions_controls_and_oversized_rows(self):
        for value in ["dangerous()",json.dumps('unsafe\nlabel'),json.dumps('x'*257)]:
            with tempfile.TemporaryDirectory() as directory:
                gui=Path(directory)/'jci/gui';app=gui/'apps/syssettings/js/syssettingsApp.js';app.parent.mkdir(parents=True)
                dictionary=gui/'resources/js/syssettings/syssettingsAppDict_en_US.js';dictionary.parent.mkdir(parents=True)
                app.write_text("this._SelectTimeZoneCtxtDataList={items:[{text1Id:'zone'}]};")
                dictionary.write_text('var Localization_syssettingsAppDict_en_US={zone:'+value+'};')
                with self.assertRaises(ValueError):importer.clock_zone_text(Path(directory))

if __name__=='__main__': unittest.main()

import importlib.util
from pathlib import Path
import unittest
spec=importlib.util.spec_from_file_location('catalog',Path(__file__).with_name('catalog-settings.py'))
catalog=importlib.util.module_from_spec(spec);spec.loader.exec_module(catalog)

class LiteralChecks(unittest.TestCase):
    def test_reads_nested_controls_comments_and_exact_strings(self):
        source=r'''{rows:[{text1Id:'Brightness',min:-5,max:5},/* comment */ {text1:'It\'s fine',enabled:true}],nothing:null}'''
        data=catalog.LiteralReader(source).value()
        self.assertEqual(data['rows'][0],{'text1Id':'Brightness','min':-5,'max':5})
        self.assertEqual(data['rows'][1]['text1'],"It's fine")
        self.assertIsNone(data['nothing'])
    def test_calls_and_function_bodies_are_inert_not_evaluated(self):
        data=catalog.LiteralReader("{fn:danger('a,b',{x:[1,2]}), callback:function(){return malicious();}, next:42}").value()
        self.assertEqual(data['fn'],{'$expr':"danger('a,b',{x:[1,2]})"})
        self.assertIn('malicious()',data['callback']['$expr'])
        self.assertEqual(data['next'],42)
    def test_decodes_unicode_and_keeps_member_references_explicit(self):
        data=catalog.LiteralReader(r"{text:'\u00c9t\xe9', dataList:this._Rows}").value()
        self.assertEqual(data['text'],'Été')
        self.assertEqual(catalog.resolve(data['dataList'],{'_Rows':{'items':[]}}),{'items':[]})
        self.assertEqual(catalog.resolve(data['dataList'],{}),{'$expr':'this._Rows'})
    def test_malformed_declarations_fail_instead_of_guessing(self):
        for source in ["{x:'unterminated}",'{x 5}', '{/* unclosed', '[1,2']:
            with self.assertRaises((ValueError,IndexError)):catalog.LiteralReader(source).value()
    def test_reference_cycles_are_bounded(self):
        value={'$expr':'this._A'}
        self.assertEqual(catalog.resolve(value,{'_A':value}),value)

if __name__=='__main__':unittest.main()

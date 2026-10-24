"""Owner-local NA language dictionaries; parse literal objects without evaluating JS."""
from pathlib import Path
import re
import struct
from firmware_literals import LiteralReader

LANGUAGES=('LANGS_US_ENGLISH','LANGS_NA_SPANISH','LANGS_CN_FRENCH')
CODES=('en_US','es_MX','fr_CN')
APPS=('common','system','syssettings','audiosettings','emnavi','sysupdate','btpairing')

def declaration(source,pattern):
    match=re.search(pattern,source)
    if not match:raise ValueError('missing language declaration')
    return LiteralReader(source,match.end()-1).value()

def dictionary(gui,app,locale):
    path=gui/'resources/js'/app/(app+'AppDict_'+locale+'.js')
    return declaration(path.read_text(),r'var\s+Localization_\w+\s*=\s*\{')

def language_assets(reference):
    gui=reference/'jci/gui'
    source=(gui/'apps/syssettings/js/syssettingsApp.js').read_text()
    languages=declaration(source,r'this\._cachedLanguagesList\s*=\s*\[')
    codes=declaration((gui/'framework/js/Localization.js').read_text(),r'this\.LANG_CODES\s*=\s*\{')
    rows=[row for row in languages if row['languageID'] in LANGUAGES]
    if tuple(row['languageID'] for row in rows)!=LANGUAGES:raise ValueError('unexpected NA language fixture')
    english={app:dictionary(gui,app,'en_US') for app in APPS}
    output=bytearray(b'MZLANG01'+struct.pack('<I',len(rows)))
    def string(value):
        if not isinstance(value,str) or len(value)>4096 or any(ord(c)<32 and c!='\n' or 127<=ord(c)<=159 for c in value):raise ValueError('invalid localized string')
        data=value.encode('utf-8');output.extend(struct.pack('<I',len(data)));output.extend(data)
    for row,expected in zip(rows,CODES):
        code=codes[row['languageID']]
        if code!=expected or not isinstance(row['vrSupport'],bool) or not isinstance(row['ttsSupport'],bool):raise ValueError('invalid NA language metadata')
        string(row['languageID']);string(code)
        output.extend(bytes([row['vrSupport'],row['ttsSupport']]))
        pairs={}
        for app in APPS:
            target=dictionary(gui,app,code)
            # Keep app and string id. The native lookup does not guess ambiguous aliases.
            for key,value in english[app].items():
                translated=target.get(key)
                if isinstance(value,str) and key in target and not isinstance(translated,str):raise ValueError('nonliteral translation')
                if isinstance(value,str) and isinstance(translated,str) and len(value)<=4096 and len(translated)<=4096:
                    pairs[(app,key)]=(value.replace("\r\n","\n"),translated.replace("\r\n","\n"))
        output.extend(struct.pack('<I',len(pairs)))
        for (app,key),(original,translated) in sorted(pairs.items()):
            for value in (app,key,original,translated):string(value)
    if len(output)>4_000_000:raise ValueError('oversized language asset')
    return bytes(output)

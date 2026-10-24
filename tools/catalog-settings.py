#!/usr/bin/env python3
"""Read firmware declarations as data; never evaluate JavaScript or invoke firmware callbacks."""
import argparse
import hashlib
import json
from pathlib import Path
import re

APPS=('syssettings','audiosettings','vehsettings','btpairing','netmgmt','sysupdate')

class LiteralReader:
    def __init__(self,source,position=0): self.source,self.position=source,position
    def skip(self):
        while self.position<len(self.source):
            if self.source[self.position].isspace(): self.position+=1
            elif self.source.startswith('//',self.position):
                end=self.source.find('\n',self.position);self.position=len(self.source) if end<0 else end+1
            elif self.source.startswith('/*',self.position):
                end=self.source.find('*/',self.position+2)
                if end<0: raise ValueError('unterminated comment')
                self.position=end+2
            else: break
    def string(self):
        quote=self.source[self.position];self.position+=1;out=[]
        escapes={'n':'\n','r':'\r','t':'\t','b':'\b','f':'\f','v':'\v','0':'\0'}
        while self.position<len(self.source):
            ch=self.source[self.position];self.position+=1
            if ch==quote:return ''.join(out)
            if ch=='\\':
                ch=self.source[self.position];self.position+=1
                if ch in ('u','x'):
                    width=4 if ch=='u' else 2
                    out.append(chr(int(self.source[self.position:self.position+width],16)));self.position+=width
                elif ch=='\n':continue
                elif ch=='\r':
                    if self.source[self.position:self.position+1]=='\n':self.position+=1
                else:out.append(escapes.get(ch,ch))
            else:out.append(ch)
        raise ValueError('unterminated string')
    def key(self):
        self.skip()
        if self.source[self.position] in ('"',"'"):return self.string()
        match=re.match(r'[A-Za-z_$][\w$]*|\d+',self.source[self.position:])
        if not match:raise ValueError('invalid object key near '+self.source[self.position:self.position+50])
        self.position+=len(match[0]);return match[0]
    def value(self):
        self.skip();start=self.position
        if self.source[start]=='{':return self.object()
        if self.source[start]=='[':return self.array()
        # Expressions stay inert strings, including calls, callbacks and member references.
        stack=[]
        while self.position<len(self.source):
            self.skip()
            if self.position>=len(self.source):break
            ch=self.source[self.position]
            if not stack and ch in ',}])':break
            if ch in ('"',"'"):self.string();continue
            if ch in '([{':stack.append(ch)
            elif ch in ')]}':
                if not stack:break
                stack.pop()
            self.position+=1
        raw=self.source[start:self.position].strip()
        if raw in ('true','false','null'):return {'true':True,'false':False,'null':None}[raw]
        if re.fullmatch(r'-?\d+(\.\d+)?',raw):return float(raw) if '.' in raw else int(raw)
        if raw.startswith(('"',"'")):
            reader=LiteralReader(raw)
            decoded=reader.string();reader.skip()
            if reader.position==len(raw):return decoded
        return {'$expr':raw}
    def object(self):
        self.position+=1;out={}
        while True:
            self.skip()
            if self.source[self.position]=='}':self.position+=1;return out
            key=self.key();self.skip()
            if self.source[self.position]!=':':raise ValueError('missing colon')
            self.position+=1;out[key]=self.value();self.skip()
            if self.source[self.position]==',':self.position+=1
            elif self.source[self.position]!='}':raise ValueError('missing object separator')
    def array(self):
        self.position+=1;out=[]
        while True:
            self.skip()
            if self.source[self.position]==']':self.position+=1;return out
            out.append(self.value());self.skip()
            if self.source[self.position]==',':self.position+=1
            elif self.source[self.position]!=']':raise ValueError('missing array separator')

def resolve(value,variables,depth=0):
    if depth>12:return value
    if isinstance(value,dict) and set(value)=={'$expr'}:
        match=re.fullmatch(r'this\.(\w+)',value['$expr'])
        if match and match[1] in variables:return resolve(variables[match[1]],variables,depth+1)
    return value

def dictionary(gui,app):
    paths=[gui/'resources/js'/app/f'{app}AppDict_en_US.js',gui/'apps'/app/'js'/f'{app}AppDict_en_US.js']
    for path in paths:
        if not path.exists():continue
        source=path.read_text(errors='replace');match=re.search(r'var\s+Localization_\w+\s*=\s*\{',source)
        if match:return LiteralReader(source,match.end()-1).value(),path
    return {},None

def build(reference):
    gui=reference/'jci/gui';common,_=dictionary(gui,'common');apps=[];sources=[]
    for app in APPS:
        path=gui/'apps'/app/'js'/f'{app}App.js';raw=path.read_bytes();source=raw.decode('utf-8',errors='replace')
        variables={};failures=[]
        for match in re.finditer(r'this\.(\w+)\s*=\s*([\[{])',source):
            if match[1] in variables:continue
            try:variables[match[1]]=LiteralReader(source,match.end()-1).value()
            except (ValueError,IndexError) as error:failures.append({'name':match[1],'line':source[:match.start()].count('\n')+1,'error':str(error)})
        contexts=variables.get('_contextTable')
        if not isinstance(contexts,dict):raise ValueError(f'{app}: context table was not parsed')
        translations,dictpath=dictionary(gui,app)
        def localize(key):
            if not isinstance(key,str):return None
            if key.startswith('common.'):return common.get(key[7:])
            return translations.get(key)
        screens=[]
        for name,context in contexts.items():
            if not isinstance(context,dict) or 'template' not in context:continue
            properties=context.get('controlProperties',{})
            control=next(iter(properties),None)
            config=properties.get(control,{})
            if not isinstance(config,dict):config={}
            data=resolve(config.get('dataList'),variables)
            rows=data.get('items',[]) if isinstance(data,dict) else []
            if not isinstance(rows,list):rows=[]
            localized=[]
            for row in rows:
                if not isinstance(row,dict):continue
                item=dict(row)
                for field in ['text1Id','text2Id','label1Id','button1Id','button2Id','button3Id']:
                    label=localize(row.get(field))
                    if label is not None:item[field.removesuffix('Id')]=label
                localized.append(item)
            title=config.get('title',{})
            screen={'id':f'{app}.{name}','app':app,'context':name,'template':context['template'],'control':control,
                    'status_title':localize(context.get('sbNameId')),'config':config,'rows':localized,
                    'title':title,'text1':localize(config.get('text1Id')),'text2':localize(config.get('text2Id'))}
            screens.append(screen)
        list_definitions={name:value for name,value in variables.items() if isinstance(value,dict) and ('items' in value or name.endswith('HelperArray'))}
        apps.append({'app':app,'screens':screens,'lists':list_definitions,'unresolved_declarations':failures,'translations':translations})
        for sourcepath in [path,dictpath]:
            if sourcepath:sources.append({'path':str(sourcepath.relative_to(reference)),'sha256':hashlib.sha256(sourcepath.read_bytes()).hexdigest()})
    return {'schema':1,'package':'NA 70.00.100A','apps':apps,'common':common,'sources':sources}

if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('reference',type=Path);parser.add_argument('output',type=Path);args=parser.parse_args()
    catalog=build(args.reference);args.output.parent.mkdir(parents=True,exist_ok=True);args.output.write_text(json.dumps(catalog,indent=2,ensure_ascii=False)+'\n')
    print(json.dumps({app['app']:{'screens':len(app['screens']),'rows':sum(len(screen['rows']) for screen in app['screens']),'lists':len(app['lists']),'parse_failures':app['unresolved_declarations']} for app in catalog['apps']},indent=2))

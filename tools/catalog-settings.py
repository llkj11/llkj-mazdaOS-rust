#!/usr/bin/env python3
"""Read firmware declarations as data; never evaluate JavaScript or invoke firmware callbacks."""
import argparse
import hashlib
import json
from pathlib import Path
import re

APPS=('syssettings','audiosettings','vehsettings','btpairing','netmgmt','sysupdate')

from firmware_literals import LiteralReader

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

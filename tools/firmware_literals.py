"""Inert literal reader shared by offline firmware tools. Never evaluates JavaScript."""
import re

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

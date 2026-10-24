"""Bounded single-frame GIF data decoder for the original unpair icon. No execution."""
import struct

def gif_rgba(data):
    if len(data)>4_000_000 or data[:6] not in (b'GIF87a',b'GIF89a') or len(data)<13:
        raise ValueError('invalid GIF header')
    width,height,packed,background,_=struct.unpack_from('<HHBBB',data,6)
    if not (0<width<=1600 and 0<height<=960):raise ValueError('unbounded GIF dimensions')
    position=13
    def take(count):
        nonlocal position
        if count<0 or position+count>len(data):raise ValueError('truncated GIF')
        block=data[position:position+count];position+=count;return block
    def palette(flags):
        return take(3*(2**((flags&7)+1))) if flags&128 else b''
    def blocks():
        output=bytearray()
        while True:
            length=take(1)[0]
            if length==0:return bytes(output)
            output.extend(take(length))
            if len(output)>4_000_000:raise ValueError('unbounded GIF blocks')
    colors=palette(packed)
    transparent=None
    pixels=None
    while True:
        marker=take(1)[0]
        if marker==0x3b:
            if pixels is None or position!=len(data):raise ValueError('invalid GIF end')
            return b'MZRGBA01'+struct.pack('<II',width,height)+pixels
        if marker==0x21:
            kind=take(1)[0]
            if kind==0xf9:
                if take(1)!=b'\x04':raise ValueError('invalid graphic control')
                flags,_,index=struct.unpack('<BHB',take(4))
                if take(1)!=b'\0':raise ValueError('invalid graphic terminator')
                transparent=index if flags&1 else None
            elif kind in (0xfe,0xff):blocks()  # comments/application data are inert
            else:raise ValueError('unsupported GIF extension')
            continue
        if marker!=0x2c or pixels is not None:raise ValueError('only one GIF frame is supported')
        left,top,frame_width,frame_height,flags=struct.unpack('<HHHHB',take(9))
        if (left,top,frame_width,frame_height)!=(0,0,width,height) or flags&64:
            raise ValueError('only full noninterlaced GIF frames are supported')
        local=palette(flags)
        if local:colors=local
        if not colors or background>=len(colors)//3:raise ValueError('missing GIF palette')
        minimum=take(1)[0]
        if not 2<=minimum<=8:raise ValueError('invalid GIF code size')
        indices=_lzw(blocks(),minimum,width*height)
        pixels=bytearray()
        for index in indices:
            if index>=len(colors)//3:raise ValueError('GIF palette index out of range')
            pixels.extend(colors[index*3:index*3+3]);pixels.append(0 if index==transparent else 255)

def _lzw(data,minimum,count):
    clear=1<<minimum;end=clear+1
    table=[];previous=None;bits=0;size=minimum+1;output=bytearray()
    while bits+size<=len(data)*8:
        word=int.from_bytes(data[bits//8:bits//8+3],'little')
        code=(word>>(bits%8))&((1<<size)-1);bits+=size
        if code==clear:
            table=[bytes([n]) for n in range(clear)]+[b'',b''];previous=None;size=minimum+1
            continue
        if not table:raise ValueError('GIF LZW requires a clear code')
        if code==end:
            if len(output)!=count:raise ValueError('GIF pixel count mismatch')
            return bytes(output)
        if code<len(table) and table[code]:entry=table[code]
        elif code==len(table) and previous is not None:entry=previous+previous[:1]
        else:raise ValueError('invalid GIF LZW code')
        if len(output)+len(entry)>count:raise ValueError('GIF expansion exceeds pixel count')
        output.extend(entry)
        if previous is not None and len(table)<4096:
            table.append(previous+entry[:1])
            if len(table)==1<<size and size<12:size+=1
        previous=entry
    raise ValueError('missing GIF end code')

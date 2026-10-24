#!/usr/bin/env python3
"""Read owner firmware as data. Never executes an extracted file or installer instruction."""
import argparse
import getpass
import gzip
import hashlib
import json
import os
import re
from pathlib import Path, PurePosixPath
import shutil
import struct
import subprocess
import tarfile
import tempfile
import zlib
from firmware_literals import LiteralReader

PACKAGE_SHA256 = '5938019df7e333447c7bf60aa8f6a7e7f9cd50d6eeb7b6e33605800b85bd4ad0'
GUI_PREFIX = 'jci/gui/'
ASSETS = {
    'left-back': 'common/controls/LeftBtn/images/BtnLeft_En.png',
    'left-back-disabled': 'common/controls/LeftBtn/images/BtnLeft_Ds.png',
    'system-dialog-bg': 'common/controls/Dialog3/images/DialogBG_Style01.png',
    'system-dialog-ok': 'common/controls/Dialog3/images/Dialog_1Btn_Focus.png',
    'system-meter': 'common/images/IndeterminateMeter.png',
    'legal-background': 'common/controls/ScrollDetail/images/ScrollDetailBackground.png',
    'legal-focus': 'common/controls/ScrollDetail/images/ScrollDetailBackground_Focus.png',
    'legal-header': 'common/controls/ScrollDetail/images/ScrollDetailHeadBack_Bg.png',
    'legal-scroll-bg': 'common/controls/ScrollDetail/images/ScrollbarBg.png',
    'legal-scroll': 'common/controls/ScrollDetail/images/Scrollbar.png',
    'legal-shadow': 'common/controls/ScrollDetail/images/ScrollDetailBottomShadow.png',
    'clock-background': 'common/controls/ClockSettings2/images/ClockSettingBG.png',
    'clock-colon': 'common/controls/ClockSettings2/images/ClockSetting_Colon_En.png',
    'clock-panel': 'common/controls/Ump3/images/UMP_Bg.png',
    'clock-arch': 'common/controls/Ump3/images/UMP_Bg_Arch.png',
    'clock-separator': 'common/controls/Ump3/images/UMP_Btn_Separator.png',
    'clock-focus': 'common/controls/Ump3/images/UMP_Focus_6.png',
    'clock-back': 'common/images/icons/IcnUmpBack_En.png',
    'clock-minus': 'common/images/icons/IcnUmpMinus_En.png',
    'clock-minus-disabled': 'common/images/icons/IcnUmpMinus_Ds.png',
    'clock-plus': 'common/images/icons/IcnUmpPlus_En.png',
    'clock-plus-disabled': 'common/images/icons/IcnUmpPlus_Ds.png',
    'clock-ampm': 'common/images/icons/IcnUmpAmPm_En.png',
    'clock-ampm-disabled': 'common/images/icons/IcnUmpAmPm_Ds.png',
    'settings-radio': 'common/images/icons/IcnListRadio_Checked_En.png',
    'settings-radio-empty': 'common/images/icons/IcnListRadio_Unchecked_En.png',
    'settings-radio-disabled': 'common/images/icons/IcnListRadio_Checked_Ds.png',
    'settings-radio-empty-disabled': 'common/images/icons/IcnListRadio_Unchecked_Ds.png',
    'settings-title': 'common/controls/List2/images/TitleStyle01_HeaderBg.png',
    'settings-check': 'common/controls/List2/images/IcnListCheck_Checked_En.png',
    'settings-uncheck': 'common/controls/List2/images/IcnListCheck_Unchecked_En.png',
    'settings-check-disabled': 'common/controls/List2/images/IcnListCheck_Checked_Ds.png',
    'settings-uncheck-disabled': 'common/controls/List2/images/IcnListCheck_Unchecked_Ds.png',
    'settings-tick': 'common/controls/List2/images/List_SliderTick.png',
    'settings-tick-center': 'common/controls/List2/images/List_SliderTickCtr.png',
    'settings-test': 'apps/syssettings/images/SettingsTestImage.png',
    'settings-inline': 'common/controls/List2/images/InLineButton.png',
    'settings-inline-selected': 'common/controls/List2/images/InLineButton_Selected.png',
    'settings-slider-bg': 'common/controls/List2/images/ListItem_SliderBG-1.png',
    'settings-slider-fill': 'common/controls/List2/images/ListItem_Slider-1.png',
    'settings-slider-focus': 'common/controls/List2/images/ListItem_Slider_Focus-1.png',
    'settings-knob': 'common/controls/List2/images/ListItem_Slider_Knob.png',
    'settings-knob-focus': 'common/controls/List2/images/ListItem_Slider_KnobFocus-1.png',
    'settings-dialog-bg': 'common/controls/Dialog3/images/DialogBG_Style02.png',
    'settings-dialog-button': 'common/controls/Dialog3/images/Dialog_2Btn.png',
    'settings-dialog-button-focus': 'common/controls/Dialog3/images/Dialog_2Btn_Focus.png',
    'settings-off': 'apps/system/controls/OffScreen/images/OffScreenBackground.png',
    'compass-bg': 'apps/emnavi/controls/Compass/images/CompassBg.png',
    'compass-foreground': 'apps/emnavi/controls/Compass/images/CompassForeground.png',
    'compass-rail': 'apps/emnavi/controls/Compass/images/GreyRail.png',
    'compass-step': 'apps/emnavi/controls/Compass/images/Step.png',
    'compass-needle': 'apps/emnavi/controls/Compass/images/Needle.png',
    'compass-line': 'apps/emnavi/controls/Compass/images/WhiteLine.png',
    'tab-prev': 'common/controls/Tabs/images/Tab_ArrowLeft.png',
    'tab-next': 'common/controls/Tabs/images/Tab_ArrowRight.png',
    'tab': 'common/controls/Tabs/images/Tab_Style04.png',
    'tab-focus': 'common/controls/Tabs/images/TabHighlight_Style04.png',
    'background': 'common/images/background.png',
    'status': 'common/controls/StatusBar/images/StatusBarBg.png',
    'ellipse': 'apps/system/controls/MainMenu/images/Ellipse.png',
    'glow': 'apps/system/controls/MainMenu/images/highlights/Glow.png',
    'home': 'common/images/icons/IcnSbHome.png',
    'list-background': 'common/controls/List2/images/ListBackground.png',
    'list-focus': 'common/controls/List2/images/ListFocus.png',
    'list-separator': 'common/controls/List2/images/ListItemSeparator.png',
    'back': 'common/controls/LeftBtn/images/ChromeArcRight.png',
}
for coin in ['Apps', 'Audio', 'Com', 'Nav', 'Set']:
    for suffix in ['', '_Focus']:
        ASSETS[f'{coin}{suffix}'] = f'apps/system/controls/MainMenu/images/coins/Home{coin}{suffix}.png'


def png_rgba(data):
    """Decode only the bounded, non-interlaced RGB/RGBA8 PNGs used by this firmware."""
    if data[:8] != b'\x89PNG\r\n\x1a\n':
        raise ValueError('not PNG')
    pos, compressed, dimensions = 8, bytearray(), None
    seen_end = False
    while pos < len(data):
        length = struct.unpack_from('>I', data, pos)[0]
        kind = data[pos+4:pos+8]
        block = data[pos+8:pos+8+length]
        if len(block) != length or pos+12+length > len(data):
            raise ValueError('truncated PNG')
        crc = struct.unpack_from('>I', data, pos+8+length)[0]
        if zlib.crc32(kind+block) & 0xffffffff != crc:
            raise ValueError('PNG CRC mismatch')
        if kind == b'IHDR':
            w,h,depth,color,method,filtering,interlace = struct.unpack('>IIBBBBB', block)
            if not (0 < w <= 1600 and 0 < h <= 960 and depth == 8 and color in (2,6) and method == filtering == interlace == 0):
                raise ValueError('unsupported PNG format')
            dimensions = w,h,3 if color == 2 else 4
        elif kind == b'IDAT':
            compressed.extend(block)
        elif kind == b'IEND':
            if length != 0: raise ValueError('invalid PNG end chunk')
            seen_end = True
            break
        pos += length+12
    if dimensions is None or not seen_end:
        raise ValueError('missing PNG dimensions or end chunk')
    w,h,bpp = dimensions
    stride = w*bpp
    decoder = zlib.decompressobj()
    raw = decoder.decompress(compressed, (stride+1)*h+1)
    if len(raw) != (stride+1)*h or not decoder.eof:
        raise ValueError('PNG expansion size mismatch')
    pixels, previous = bytearray(), bytearray(stride)
    for row in range(h):
        offset = row*(stride+1)
        mode = raw[offset]
        line = bytearray(raw[offset+1:offset+1+stride])
        if mode > 4:
            raise ValueError('unknown PNG filter')
        for i in range(stride):
            left = line[i-bpp] if i >= bpp else 0
            up = previous[i]
            corner = previous[i-bpp] if i >= bpp else 0
            predictor = 0
            if mode == 1: predictor = left
            elif mode == 2: predictor = up
            elif mode == 3: predictor = (left+up)//2
            elif mode == 4:
                p = left+up-corner
                distances = [abs(p-left), abs(p-up), abs(p-corner)]
                predictor = [left, up, corner][distances.index(min(distances))]
            line[i] = (line[i]+predictor) & 255
        if bpp == 4: pixels.extend(line)
        else:
            for i in range(0,stride,3): pixels.extend(line[i:i+3]+b'\xff')
        previous = line
    return b'MZRGBA01'+struct.pack('<II',w,h)+pixels


def clock_zone_text(reference):
    """Keep the exact owner's zone order and English labels as inert local text."""
    gui=reference/'jci/gui'
    source=(gui/'apps/syssettings/js/syssettingsApp.js').read_text()
    match=re.search(r'this\._SelectTimeZoneCtxtDataList\s*=\s*\{',source)
    if not match: raise ValueError('missing original time-zone declaration')
    rows=LiteralReader(source,match.end()-1).value()['items']
    dictionary=(gui/'resources/js/syssettings/syssettingsAppDict_en_US.js').read_text()
    match=re.search(r'var\s+Localization_\w+\s*=\s*\{',dictionary)
    if not match: raise ValueError('missing original English dictionary')
    labels=LiteralReader(dictionary,match.end()-1).value()
    result=[]
    for row in rows:
        text=labels[row['text1Id']]
        if not isinstance(text,str) or not text or len(text)>256 or any(ord(c)<32 or ord(c)==127 for c in text):
            raise ValueError('invalid time-zone label')
        result.append(text)
    if not 1<=len(result)<=128: raise ValueError('unexpected time-zone count')
    return ('MZTIMEZONES1\n'+'\n'.join(result)+'\n').encode('utf-8')

def legal_text(reference):
    """Decode only a sum of quoted string literals; reject every executable expression."""
    source=(reference/'jci/gui/apps/syssettings/js/legalDisclosure.js').read_text()
    match=re.search(r'var\s+ScrollDetailCtrlProperties\s*=\s*\{',source)
    if not match: raise ValueError('missing original disclosure declaration')
    value=LiteralReader(source,match.end()-1).value()['scrollDetailBody']
    if isinstance(value,str): text=value
    else:
        reader=LiteralReader(value['$expr']);parts=[]
        while True:
            reader.skip()
            if reader.position>=len(reader.source) or reader.source[reader.position] not in ('"',"'"):
                raise ValueError('disclosure is not literal text')
            parts.append(reader.string());reader.skip()
            if reader.position==len(reader.source):break
            if reader.source[reader.position]!='+':raise ValueError('disclosure expression rejected')
            reader.position+=1
        text=''.join(parts)
    if not text or len(text)>32768 or any(ord(c)<32 and c!='\n' or 127<=ord(c)<=159 for c in text):
        raise ValueError('invalid disclosure text')
    return ('MZLEGAL1\n'+text).encode('utf-8')

def extract(firmware, destination, password):
    hasher = hashlib.sha256()
    with firmware.open('rb') as file:
        for block in iter(lambda: file.read(1_048_576), b''): hasher.update(block)
    digest = hasher.hexdigest()
    if digest != PACKAGE_SHA256:
        raise ValueError('Expected the reviewed NA 70.00.100A reinstall package; hash differs')
    destination.mkdir(parents=True, exist_ok=True)
    if any(destination.iterdir()):
        raise ValueError('Use an empty output directory to preserve earlier extractions')
    unzip = shutil.which('unzip')
    if unzip is None: raise ValueError('system unzip is required')
    inventory = []
    with tempfile.TemporaryFile() as payload:
        subprocess.run([unzip,'-p','-P',password,str(firmware),'rootfs1upd/e0000000001.dat.gz'],stdout=payload,check=True)
        payload.seek(0)
        # This package contains a gzip wrapper around a second gzip tar archive.
        with gzip.GzipFile(fileobj=payload) as wrapper, tarfile.open(fileobj=wrapper, mode='r|gz') as archive:
            for member in archive:
                name = member.name.removeprefix('./')
                if not (name.startswith(GUI_PREFIX) or name.startswith('jci/fonts/')): continue
                if not member.isfile(): continue  # no symlinks, devices, ownership or executable modes
                path = PurePosixPath(name)
                if path.is_absolute() or '..' in path.parts or member.size > 32_000_000:
                    raise ValueError('unsafe archive entry')
                data = archive.extractfile(member).read(member.size+1)
                if len(data) != member.size: raise ValueError('truncated tar entry')
                target = destination/'reference'/path
                target.parent.mkdir(parents=True,exist_ok=True)
                target.write_bytes(data)
                target.chmod(0o600)
                inventory.append({'path':name,'bytes':len(data),'sha256':hashlib.sha256(data).hexdigest()})
    rendered = destination/'render-assets'
    rendered.mkdir()
    for key, relative in ASSETS.items():
        data=(destination/'reference/jci/gui'/relative).read_bytes()
        rgba=bytearray(png_rgba(data))
        if key in ('compass-rail','compass-step','compass-needle'):
            for index in range(19,len(rgba),4): rgba[index]=round(rgba[index]*0.29)
        (rendered/f'{key}.rgba').write_bytes(rgba)
    (rendered/'clock-zones.txt').write_bytes(clock_zone_text(destination/'reference'))
    (rendered/'legal.txt').write_bytes(legal_text(destination/'reference'))
    manifest={'package_sha256':digest,'files':inventory,'render_assets':ASSETS}
    (destination/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
    shutil.copyfile(destination/'manifest.json',rendered/'manifest.json')
    print(f'Extracted {len(inventory)} GUI/font files and converted {len(ASSETS)} original images. No firmware code was executed.')

if __name__ == '__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('firmware',type=Path)
    parser.add_argument('destination',type=Path)
    args=parser.parse_args()
    extract(args.firmware,args.destination,os.environ.get('MZD_FIRMWARE_PASSWORD') or getpass.getpass('Firmware archive password: '))

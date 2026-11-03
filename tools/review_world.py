#!/usr/bin/env python3
"""Prepare a NEW engineering gallery through the official launcher/control API.

Requires an installed native Create instance and its locally supplied asset JAR.
Never opens or replaces an existing world. Runtime gameplay remains Rust.
"""
import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import time
from common import ROOT
from catalog import BLOCKS, ITEMS

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--instance',required=True)
    parser.add_argument('--world',default='CreateEngineering04')
    parser.add_argument('--launcher',type=Path,default=Path('/Applications/Horizon.app/Contents/MacOS/horizon-launcher'))
    parser.add_argument('--ctl',type=Path,required=True)
    args=parser.parse_args()
    instances=json.loads(subprocess.check_output([str(args.launcher),'--json','instance','list'],text=True))
    instance=next(i for i in instances if args.instance in [i['id'],i['name']])
    home=Path(instance['home'])
    if (home/'saves'/args.world).exists():parser.error('World already exists; choose a new --world. It will not be replaced.')
    version=instance.get('version')
    if not version or not version.startswith('native-'):parser.error('Build and pin the native Create version for this instance first.')
    if not list((home/'horizon-mods/create-resources').glob('*.jar')):parser.error('Place the supported Create JAR in the instance first.')
    ROOT.joinpath('.local').mkdir(exist_ok=True)
    work=Path(tempfile.mkdtemp(prefix='review04-',dir=ROOT/'.local'))
    socket=work/'ctl.sock';transcript=[]
    env=os.environ.copy();env.update(HORIZON_NO_AUDIO='1',HORIZON_LAUNCHER_NO_SELF_UPDATE='1')
    def call(*words):
        p=subprocess.run([str(args.ctl),'--socket',str(socket),*map(str,words)],text=True,capture_output=True,timeout=180)
        if p.returncode:raise RuntimeError(str(words)+p.stdout+p.stderr)
        value=json.loads(p.stdout);transcript.append({'command':list(map(str,words)),'reply':value});return value
    def put(x,y,z,name):return call('setblock',x,y,z,name if ':' in name else 'create:'+name)
    def data(x,y,z,nbt):return call('command',f'data merge block {x} {y} {z} '+nbt)
    def sign(x,z,lines):
        put(x,64,z,'minecraft:oak_sign[rotation=0]')
        data(x,64,z,'{front_text:{messages:'+json.dumps(lines,ensure_ascii=False)+',color:"black"},is_waxed:1b}')
    def speed(x,y,z):
        import re
        text=' '.join(call('command',f'data get block {x} {y} {z}')['chat'])
        match=re.search(r'Speed:\s*(-?[\d.]+)',text);assert match,text;return float(match[1])
    def channel(x,y,z,r):
        # A contained, reversible vanilla bubble current. Replace magma with soul sand.
        for yy in range(y-1,y+4):
            for xx,zz in [(x-1,z+r),(x+1,z+r),(x,z+r+1),(x,z+r-1)]:
                if call('block',xx,yy,zz)['block']=='minecraft:air':put(xx,yy,zz,'minecraft:glass')
        put(x,y-2,z+r,'minecraft:magma_block')
        for yy in range(y-1,y+3):put(x,yy,z+r,'minecraft:water')
    with (work/'game.log').open('w') as log:
        command=[str(args.launcher),'play','--instance',instance['id'],'--version',version,'--world',args.world,'--creative','--arg=--seed','--arg=5','--arg=--render-distance','--arg=4','--arg=--control-socket','--arg='+str(socket),'--arg=--headless']
        process=subprocess.Popen(command,env=env,stdout=log,stderr=subprocess.STDOUT)
        try:
            deadline=time.monotonic()+180
            while True:
                if process.poll() is not None:raise RuntimeError('Launcher stopped; inspect '+str(work/'game.log'))
                try:
                    if call('state').get('world')==args.world:break
                except RuntimeError:pass
                if time.monotonic()>deadline:raise TimeoutError('Review world startup')
                time.sleep(.25)
            call('wait-ready',120);call('command','tp @s 12 80 12');call('wait-ready',120)
            call('fill',-4,64,-4,31,81,22,'minecraft:air');call('fill',-4,63,-4,31,63,22,'minecraft:smooth_stone')
            for x,z in [(0,2),(0,8),(13,2),(24,2)]:
                put(x,64,z,'andesite_casing');put(x,65,z,'andesite_casing')
            for x,name in [(0,'creative_motor[facing=east]'),(1,'shaft[axis=x]'),(2,'mechanical_press[facing=east]')]:put(x,66,2,name)
            put(2,64,2,'depot');data(0,66,2,'{ScrollValue:256}');call('container',2,64,2,'create:brass_ingot',8);call('wait',150);data(0,66,2,'{ScrollValue:16}')
            assert sum(s['count'] for s in call('container',2,64,2)['slots'] if s and s['item']=='create:brass_sheet')==8
            for x,name in [(0,'creative_motor[facing=east]'),(1,'shaft[axis=x]'),(2,'speedometer[facing=south,axis_along_first=true]'),(3,'stressometer[facing=south,axis_along_first=true]'),(4,'gearbox[axis=y]'),(5,'shaft[axis=x]'),(6,'mechanical_press[facing=east]')]:put(x,66,8,name)
            put(6,64,8,'depot');put(2,65,9,'andesite_casing');put(2,66,9,'minecraft:comparator[facing=north]');put(2,65,10,'andesite_casing');put(2,66,10,'minecraft:redstone_wire')
            for z,kind in [(14,'clutch'),(18,'gearshift')]:
                for x,name in [(0,'creative_motor[facing=east]'),(1,'shaft[axis=x]'),(2,kind+'[axis=x]'),(3,'shaft[axis=x]'),(4,'cogwheel[axis=x]')]:put(x,64,z,name)
                put(2,65,z,'minecraft:lever[face=floor,facing=north]')
            # A live water-driven factory: 8 RPM -> large/small gears -> 16 RPM press.
            for x,y,z,name in [(13,67,2,'water_wheel[facing=east]'),(14,67,2,'shaft[axis=x]'),(15,67,2,'large_cogwheel[axis=x]'),(15,66,3,'cogwheel[axis=x]'),(16,66,3,'speedometer[facing=south,axis_along_first=true]'),(17,66,3,'mechanical_press[facing=east]'),(17,64,3,'depot')]:put(x,y,z,name)
            channel(13,67,2,1);call('wait',40)
            assert abs(speed(14,67,2))==8 and abs(speed(16,66,3))==16
            # Large wheel's structural 3x3 ring is built by the Rust block itself.
            put(24,67,2,'large_water_wheel[axis=x]');call('wait',8)
            for x,name in [(25,'shaft[axis=x]'),(26,'speedometer[facing=south,axis_along_first=true]'),(27,'stressometer[facing=south,axis_along_first=true]')]:put(x,67,2,name)
            channel(24,67,2,2);data(24,67,2,'{Material:{Name:"minecraft:cherry_planks"}}');call('wait',40)
            assert abs(speed(25,67,2))==4
            for x,name in [(18,'hand_crank[facing=west]'),(19,'shaft[axis=x]'),(20,'mechanical_press[facing=east]')]:put(x,66,12,name)
            put(20,64,12,'depot');call('container',20,64,12,'minecraft:copper_ingot',2)
            for x,z,name in [(9,12,'gearbox[axis=x]'),(9,13,'shaft[axis=z]'),(9,11,'shaft[axis=z]'),(18,17,'andesite_encased_shaft[axis=x]'),(20,17,'brass_encased_shaft[axis=z]'),(22,17,'andesite_casing'),(24,17,'brass_casing'),(26,17,'minecraft:stripped_oak_log'),(28,17,'minecraft:crafting_table')]:put(x,64,z,name)
            put(9,65,12,'shaft[axis=y]');put(9,66,12,'cogwheel[axis=y]')
            # Vanilla barrel deliberately supplies materials; Create depots never open this menu.
            put(27,64,10,'minecraft:barrel');put(29,64,10,'minecraft:barrel')
            for x,z,lines in [(2,5,['ПРЕСС / 8 ЛИСТОВ','Пустая рука: взять','Положите 8 слитков','Латунь тоже есть']), (3,12,['ПРИБОРЫ И ОЧКИ','Наведите на сеть','ПКМ мотор: RPM','Компаратор сзади']), (2,16,['СЦЕПЛЕНИЕ','Рычаг: разрыв','Вход вращается','Ключ: поворот']), (3,20,['РЕВЕРС','Рычаг меняет знак','Сравните валы','']), (9,15,['ВЕРТ. РЕДУКТОР','Тот же gearbox','Ключ: повернуть','Shift+ключ: снять']), (15,6,['ВОДЯНОЙ ПРИВОД','8 RPM / 256 SU','Передача 2:1','Далее пресс 16 RPM']), (25,7,['БОЛЬШОЕ КОЛЕСО','4 RPM / 512 SU','Доски меняют вид','Магма: течение вниз']), (20,14,['РУЧНОЙ ПРЕСС','ПКМ: вращение','Shift: реверс','В депо 2 меди']), (26,12,['МАТЕРИАЛЫ','Два обычных бочонка','Доски, вода, слитки','Верстак справа'])]:sign(x,z,lines)
            materials=['minecraft:oak_planks','minecraft:cherry_planks','minecraft:spruce_planks','minecraft:water_bucket','minecraft:soul_sand','minecraft:magma_block','minecraft:iron_ingot','minecraft:copper_ingot','minecraft:gold_ingot','create:brass_ingot','minecraft:andesite','minecraft:iron_nugget','minecraft:redstone','minecraft:glass','minecraft:string']
            for slot,item in enumerate(materials):
                barrel=27 if slot<9 else 29
                call('command',f'item replace block {barrel} 64 10 container.{slot%9} with {item} '+('1' if item.endswith('bucket') else '32'))
            call('command','kill @e[type=minecraft:item]');call('command','clear @s')
            order=['wrench','goggles','shaft','cogwheel','large_cogwheel','vertical_gearbox','water_wheel','large_water_wheel','brass_ingot']
            order += [name for name in BLOCKS+ITEMS if name not in order]
            for name in order:call('command','give @s create:'+name+(' 1' if name in ['wrench','goggles'] else ' 16'))
            call('command','item replace entity @s armor.head with create:goggles')
            call('command','time set day');call('command','weather clear');call('options-set','lang','ru_ru');call('options-set','chatVisibility','2')
            call('command','tp @s 11.5 64 18.5');call('look-at',15,66,3);call('select','create:wrench');call('wait',10);state=call('state');call('save');call('quit');process.wait(timeout=30)
        finally:
            (work/'transcript.json').write_text(json.dumps(transcript,indent=2,ensure_ascii=False)+'\n')
            if process.poll() is None:
                try:call('quit');process.wait(timeout=30)
                except Exception:process.terminate();process.wait(timeout=10)
    (work/'result.json').write_text(json.dumps({'passed':True,'instance':instance['id'],'world':args.world,'version':version,'state':state},indent=2)+'\n')
    print(work/'result.json',flush=True)

if __name__=='__main__':main()

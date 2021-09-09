import os
import sys
import configparser


sys.path.append(os.path.dirname(__file__))
pkgname = 'ais'
cfgfile = os.path.join(os.path.expanduser('~'), '.config', f'{pkgname}.cfg')


dbpath = os.path.join(os.path.expanduser('~'), f'{pkgname}.db')
data_dir = os.path.join(os.path.expanduser('~'), f'{pkgname}') + os.path.sep
tmp_dir = os.path.join(data_dir, 'tmp_parsing') + os.path.sep 
zones_dir = os.path.join(data_dir, 'zones') + os.path.sep 
rawdata_dir = os.path.join(data_dir, 'rawdata') + os.path.sep

printdefault = lambda names, vals, quote='': '\n'.join([f'{n} = {quote}{v}{quote}' for n, v in zip(names, vals)])

if os.path.isfile(cfgfile):
    cfg = configparser.ConfigParser()
    with open(cfgfile, 'r') as f:
        cfg.read_string('[DEFAULT]\n' + f.read())

    settings = dict(cfg['DEFAULT'])

    dbpath = settings['dbpath']             if 'dbpath'      in settings.keys() else dbpath
    data_dir = settings['data_dir']         if 'data_dir'    in settings.keys() else data_dir
    tmp_dir = settings['tmp_dir']           if 'tmp_dir'     in settings.keys() else tmp_dir
    zones_dir = settings['zones_dir']       if 'zones_dir'   in settings.keys() else zones_dir
    rawdata_dir = settings['rawdata_dir']   if 'rawdata_dir' in settings.keys() else zones_dir

else:
    print(f'''no config file found, applying default configs:\n\n{
    printdefault(names=['dbpath', 'data_dir', 'tmp_dir', 'zones_dir',  'rawdata_dir'], 
                 vals=[dbpath, data_dir, tmp_dir, zones_dir,  rawdata_dir])
    }\n\nto remove this warning, copy and paste the above text to {cfgfile} ''')


# common imports that should be shared with module subdirectories
common = printdefault(names=['dbpath', 'data_dir', 'tmp_dir', 'zones_dir',  'rawdata_dir'], 
                      vals=[dbpath, data_dir, tmp_dir, zones_dir,  rawdata_dir],
                      quote="'",    
                      )
commonpaths = [os.path.join(os.path.dirname(__file__), dirname, 'common.py') for dirname in ['.', 'database', 'webdata']]

for fpath in commonpaths:
    with open(fpath, 'w') as f:
        f.write(common)



from .database.create_tables import (
        sqlite_create_table_msg18,
        sqlite_create_table_msg123,
        sqlite_create_table_polygons,
        create_table_msg5,
        create_table_msg24,
        create_table_msg27,
        aggregate_static_msg5_msg24,
    )

from .database.dbconn import (
        dbconn
    )

from .database.decoder import (
        decode_msgs,
        dt_2_epoch,
        epoch_2_dt,
    )

from .database import lambdas

from .database import qryfcn

from .database.qrygen import qrygen

from .gebco import Gebco

from .gis import (
        haversine,
        delta_meters,
        delta_seconds,
        delta_knots,
        delta_reported_knots,
        dms2dd,
        strdms2dd,
        Domain,
        ZoneGeom,
        ZoneGeomFromTxt,
    )

from .index import index

from .interp import (
        interp_time,
    )

from .merge_data import merge_layers

from .network_graph import graph as network_graph

from .proc_util import (
        fast_unzip,
    )

from .shore_dist import shore_dist_gfw

from .track_gen import (
        trackgen,
        segment,
        filtermask,
        writecsv,
    )

from .wsa import wsa

for fpath in commonpaths:
    os.remove(fpath)

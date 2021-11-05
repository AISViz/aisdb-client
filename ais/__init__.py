import os
import sys
import configparser
import logging


LOGLEVEL = os.environ.get('LOGLEVEL', 'INFO')
logging.basicConfig(format='%(message)s', level=LOGLEVEL, datefmt='%Y-%m-%d %I:%M:%S')

sys.path.append(os.path.dirname(__file__))
pkgname = 'ais'
cfgfile = os.path.join(os.path.expanduser('~'), '.config', f'{pkgname}.cfg')


# default config values
data_dir = os.path.join(os.path.expanduser('~'), f'{pkgname}') + os.path.sep
dbpath = os.path.join(data_dir, f'{pkgname}.db')
tmp_dir = os.path.join(data_dir, 'tmp_parsing') + os.path.sep 
zones_dir = os.path.join(data_dir, 'zones') + os.path.sep 
rawdata_dir = os.path.join(data_dir, 'rawdata') + os.path.sep
host_addr = 'localhost'
host_port = 9999

# common imports that should be shared with module subdirectories
commondirs = ['.', 'database', 'webdata']
cfgnames = ['data_dir', 'dbpath', 'tmp_dir', 'zones_dir', 'rawdata_dir', 'host_addr', 'host_port']

# legacy support
table_prefix = 'ais_'
legacy_cfg = ['table_prefix']

printdefault = lambda cfgnames, quote='': '\n'.join([f'{c} = {quote}{eval(c)}{quote}' for c in cfgnames])


# read config file
if os.path.isfile(cfgfile):

    cfg = configparser.ConfigParser()
    try:
        with open(cfgfile, 'r') as f:
            cfg.read_string('[DEFAULT]\n' + f.read())
        if len(list(cfg.keys())) > 1: 
            raise KeyError('Error in config file: wrong number of sections')
    except configparser.Error as err:
        print(f'could not read the configuration file!\n')
        raise err.with_traceback(None)
    settings = dict(cfg['DEFAULT'])

    # initialize config settings as variables
    for setting in cfgnames:
        exec(f'''{setting} = settings['{setting}'] if '{setting}' in settings.keys() else {setting}''')

    # convert port string to integer
    if isinstance(host_port, str):
        assert host_port.isnumeric() and float(host_port) % 1 == 0, 'host_port must be an integer value'
        host_port = int(host_port)

else:
    print(f'''no config file found, applying default configs:\n\n{
            printdefault(cfgnames)
            }\n\nto remove this warning, copy and paste the above text to {cfgfile} ''')

    # create default dirs if they dont exist
    os.path.isdir(data_dir) or os.mkdir(data_dir)
    os.path.isdir(tmp_dir) or os.mkdir(tmp_dir)
    os.path.isdir(zones_dir) or os.mkdir(zones_dir)
    os.path.isdir(rawdata_dir) or os.mkdir(rawdata_dir)



class import_handler():
    
    def __init__(self):
        self.commonpaths = [os.path.join(os.path.dirname(__file__), dirname, 'common.py') for dirname in commondirs]

    def __enter__(self):
        common = printdefault(cfgnames + legacy_cfg, quote="'")
        for fpath in self.commonpaths:
            with open(fpath, 'w') as f:
                f.write(common)

    def __exit__(self, exc_type, exc_value, tb):
        for fpath in self.commonpaths:
            os.remove(fpath)


with import_handler() as importconfigs:

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

    from .network_graph import graph

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



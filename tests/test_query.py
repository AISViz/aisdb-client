from datetime import datetime, timedelta
import numpy as np
#np.set_printoptions(precision=5, linewidth=80, formatter=dict(datetime=datetime, timedelta=timedelta), floatmode='maxprec', suppress=True)
import shapely.wkt

from database import *
from shapely.geometry import Polygon, LineString, MultiPoint
from gis import *
from track_viz import *
from track_gen import *



#canvaspoly = viz.poly_from_coords()  # select map coordinates with the cursor
canvaspoly = shapely.wkt.loads( 'POLYGON ((-61.51747881355931 46.25069648888631, -62.00013241525424 46.13520233725761, -62.19676906779659 45.77895246569407, -61.8452065677966 45.27803122330256, -61.56514830508475 45.10586058602501, -60.99907309322032 45.05537064981205, -60.71305614406779 45.20670660550304, -60.46875 45.56660601402942, -60.85010593220338 45.86615507310925, -61.13016419491525 45.92006919377324, -61.51747881355931 46.25069648888631))')
poly_xy = canvaspoly.boundary.coords.xy


dbpath = '/run/media/matt/My Passport/june2018-06-01_test.db'
dbpath = '/run/media/matt/My Passport/june2018-06-0_test2.db'


def test_query_smallboundary_statictables():

    # static: msg 5 union 24
    dt = datetime.now()
    rows = qrygen(
            start   = datetime(2018,6,1),
            end     = datetime(2018,6,2),
        ).run_qry(dbpath=dbpath, callback=rtree_in_bbox_time_mmsi, qryfcn=static) 
    delta =datetime.now() - dt
    print(f'query time: {delta.total_seconds():.2f}s')



def test_query_smallboundary_dynamictables():

    # dynamic: msg 123 union 18
    dt = datetime.now()
    rows = qrygen(
            xy = merge(canvaspoly.boundary.coords.xy),
            start   = datetime(2018,6,1),
            end     = datetime(2018,6,2),
            xmin    = min(poly_xy[0]), 
            xmax    = max(poly_xy[0]), 
            ymin    = min(poly_xy[1]), 
            ymax    = max(poly_xy[1]),
        ).run_qry(dbpath=dbpath, callback=rtree_in_bbox_time_mmsi, qryfcn=rtree_dynamic) 
    delta =datetime.now() - dt
    print(f'query time: {delta.total_seconds():.2f}s')


def test_query_smallboundary_join_static_dynamic():

    # join rtree tables with aggregate position reports 
    dt = datetime.now()
    rows = qrygen(
            xy = merge(canvaspoly.boundary.coords.xy),
            start   = datetime(2018,6,1),
            end     = datetime(2018,6,2),
            xmin    = min(poly_xy[0]), 
            xmax    = max(poly_xy[0]), 
            ymin    = min(poly_xy[1]), 
            ymax    = max(poly_xy[1]),
        ).run_qry(dbpath, callback=rtree_in_bbox_time_mmsi, qryfcn=leftjoin_dynamic_static)
    delta =datetime.now() - dt
    print(f'query time: {delta.total_seconds():.2f}s')


def test_plot_smallboundary():

    viz = TrackViz()

    canvaspoly = viz.poly_from_coords()
    poly_xy = canvaspoly.boundary.coords.xy

    rows = qrygen(
            xy = merge(canvaspoly.boundary.coords.xy),
            start   = datetime(2018,6,1),
            end     = datetime(2018,6,2),
            xmin    = min(poly_xy[0]), 
            xmax    = max(poly_xy[0]), 
            ymin    = min(poly_xy[1]), 
            ymax    = max(poly_xy[1]),
        ).run_qry(dbpath, callback=rtree_in_bbox_time_mmsi, qryfcn=leftjoin_dynamic_static) 

    filters = [
            lambda track, rng: [True for _ in rng][:-1],
        ]

    # generate track lines
    identifiers = []
    trackfeatures = []
    ptfeatures = []
    for track in trackgen(rows, ):#colnames=['mmsi', 'time', 'lon', 'lat', 'cog', 'sog']):
        rng = range(0, len(track['lon']))
        mask = filtermask(track, rng, filters)
        if track['lon'][rng][0] <= -180: mask[0] = False
        print(f'{track["mmsi"]} {rng=}:\tfiltered ', len(rng) - sum(mask),'/', len(rng))
        if sum(mask) < 2: continue
        linegeom = LineString(zip(track['lon'][rng][mask], track['lat'][rng][mask]))
        trackfeatures.append(linegeom)
        pts = MultiPoint(list(zip(track['lon'][rng][mask], track['lat'][rng][mask])))
        ptfeatures.append(pts)
        identifiers.append(track['type'][0] or track['mmsi'])

    for ft, ident in zip(trackfeatures, identifiers): 
        viz.add_feature_polyline(ft, ident)

    viz.clear_lines()
    
    '''
    i = 0

    i += 1
    ft = trackfeatures[i]
    ident=identifiers[i]
    viz.add_feature_polyline(ft, ident)

    rows[rows[:,0] == 316002048]
    '''


def test_sdd_hdd():

    # load in some data
    fpath   = '/run/media/matt/Seagate Backup Plus Drive1/CCG_Terrestrial_AIS_Network/Raw_data/2018/CCG_AIS_Log_2018-06-01.csv'
    dbpath1 = 'output/dbtest1.db'
    dbpath2 = '/run/media/matt/My Passport/dbtest2.db'
    
    # test parsing time
    t0 = datetime.now()
    decode_raw_pyais(fpath, dbpath1)
    t1 = datetime.now()
    print(f'dbpath1: {dbpath1}\t{(t1-t0).total_seconds()}s')

    t2 = datetime.now()
    decode_raw_pyais(fpath, dbpath2)
    t3 = datetime.now()
    print(f'dbpath2: {dbpath2}\t{(t3-t2).total_seconds()}s')

    import shapely.wkt
    canvaspoly = shapely.wkt.loads( 'POLYGON ((-61.51747881355931 46.25069648888631, -62.00013241525424 46.13520233725761, -62.19676906779659 45.77895246569407, -61.8452065677966 45.27803122330256, -61.56514830508475 45.10586058602501, -60.99907309322032 45.05537064981205, -60.71305614406779 45.20670660550304, -60.46875 45.56660601402942, -60.85010593220338 45.86615507310925, -61.13016419491525 45.92006919377324, -61.51747881355931 46.25069648888631))')
    poly_xy = canvaspoly.boundary.coords.xy
    
    # test query time
    aisdb = dbconn(dbpath=dbpath1)
    conn, cur = aisdb.conn, aisdb.cur
    qry = qrygen(
            xy = merge(canvaspoly.boundary.coords.xy),
            start   = datetime(2018,6,1),
            end     = datetime(2018,6,2),
            xmin    = min(poly_xy[0]), 
            xmax    = max(poly_xy[0]), 
            ymin    = min(poly_xy[1]), 
            ymax    = max(poly_xy[1]),
        ).crawl(callback=rtree_in_bbox_time_mmsi, qryfcn=rtree_minified) 
    dt = datetime.now()
    cur.execute(qry)
    delta =datetime.now()
    rows = np.array(cur.fetchall())
    print(f'query time {dbpath1}: {(delta - dt).microseconds}s')
    conn.close()

    aisdb = dbconn(dbpath=dbpath2)
    conn, cur = aisdb.conn, aisdb.cur
    qry = qrygen(
            xy = merge(canvaspoly.boundary.coords.xy),
            start   = datetime(2018,6,1),
            end     = datetime(2018,6,2),
            xmin    = min(poly_xy[0]), 
            xmax    = max(poly_xy[0]), 
            ymin    = min(poly_xy[1]), 
            ymax    = max(poly_xy[1]),
        ).crawl(callback=rtree_in_bbox_time_mmsi, qryfcn=rtree_minified) 
    dt = datetime.now()
    cur.execute(qry)
    delta =datetime.now()
    rows = np.array(cur.fetchall())
    print(f'query time {dbpath1}: {(delta - dt).microseconds}s')
    conn.close()

    #os.remove(dbpath1)
    #os.remove(dbpath2)

os.path.listdir(dbpath1)



def test_query_legacy():
    pass




exit()




viz = TrackViz()

canvaspoly = viz.poly_from_coords()
#viz.add_feature_polyline(canvaspoly, ident='canvas_markers', opacity=0.35, color=(180, 230, 180))
canvaspoly = shapely.wkt.loads( 'POLYGON ((-61.51747881355931 46.25069648888631, -62.00013241525424 46.13520233725761, -62.19676906779659 45.77895246569407, -61.8452065677966 45.27803122330256, -61.56514830508475 45.10586058602501, -60.99907309322032 45.05537064981205, -60.71305614406779 45.20670660550304, -60.46875 45.56660601402942, -60.85010593220338 45.86615507310925, -61.13016419491525 45.92006919377324, -61.51747881355931 46.25069648888631))')
poly_xy = canvaspoly.boundary.coords.xy

qry = qrygen(
        xy = merge(canvaspoly.boundary.coords.xy),
        start   = datetime(2018,6,1),
        end     = datetime(2018,6,2),
        xmin    = min(poly_xy[0]), 
        xmax    = max(poly_xy[0]), 
        ymin    = min(poly_xy[1]), 
        ymax    = max(poly_xy[1]),
    ).crawl(callback=rtree_in_bbox_time_mmsi, qryfcn=rtree_minified) 

print(qry)
'''
qry=qry.replace('ais_', 'ais_s_')
cur.execute( 'EXPLAIN QUERY PLAN \n'  + qry)
'''

dt = datetime.now()
cur.execute(qry)
print(f'query time: {(datetime.now() - dt).seconds}s')

rows = np.array(cur.fetchall())



filters = [
        lambda track, rng: compute_knots(track, rng) < 40,
    ]

filters = [
        lambda track, rng: compute_knots(track, rng) < 1,
    ]

filters = [
        lambda track, rng: [True for _ in rng][:-1],
    ]

# generate track lines
identifiers = []
trackfeatures = []
ptfeatures = []
for track in trackgen(rows, ):#colnames=['mmsi', 'time', 'lon', 'lat', 'cog', 'sog']):
    rng = range(0, len(track['lon']))
    mask = filtermask(track, rng, filters)
    if track['lon'][rng][0] <= -180: mask[0] = False
    print(f'{track["mmsi"]} {rng=}:\tfiltered ', len(rng) - sum(mask),'/', len(rng))
    if sum(mask) < 2: continue
    linegeom = LineString(zip(track['lon'][rng][mask], track['lat'][rng][mask]))
    trackfeatures.append(linegeom)
    pts = MultiPoint(list(zip(track['lon'][rng][mask], track['lat'][rng][mask])))
    ptfeatures.append(pts)
    identifiers.append(track['type'][0] or track['mmsi'])


# pass geometry to application window
for ft, ident in zip(trackfeatures, identifiers): 
    viz.add_feature_polyline(ft, ident)

    '''
    i = 0

    i += 1
    ft = trackfeatures[i]
    ident=identifiers[i]
    viz.add_feature_polyline(ft, ident)

for track in trackgen(rows):#, colnames=colnames):
    #if track['mmsi'] == 316001312:
    if track['mmsi'] == 316002048:
        break

rows[rows[:,0] == 316002048]
    
    '''
    
viz.clear_lines()

import os
from datetime import datetime

from shapely.geometry import Polygon

from aisdb import data_dir
from aisdb.database.dbqry import DBQuery
from aisdb.track_gen import TrackGen
from aisdb.webdata.merge_data import (
    merge_layers,
    merge_tracks_bathymetry,
    merge_tracks_shoredist,
    # merge_tracks_hullgeom,
)
from aisdb.database import sqlfcn_callbacks
from aisdb.gis import Domain
from aisdb.tests.create_testing_data import (
    sample_dynamictable_insertdata,
    sample_gulfstlawrence_bbox,
)


def prepare_qry():
    testdbpath = os.path.join(data_dir, 'testdb', 'test.db')
    sample_dynamictable_insertdata(testdbpath)

    z1 = Polygon(zip(*sample_gulfstlawrence_bbox()))
    domain = Domain('gulf domain', zones=[{'name': 'z1', 'geometry': z1}])

    start = datetime(2000, 1, 1)
    end = datetime(2000, 2, 1)

    rowgen = DBQuery(
        start=start,
        end=end,
        xmin=domain.minX,
        xmax=domain.maxX,
        ymin=domain.minY,
        ymax=domain.maxY,
        callback=sqlfcn_callbacks.in_timerange,
    ).gen_qry(dbpath=testdbpath)

    return rowgen


def test_merge_shoredist():
    merged = merge_tracks_shoredist(TrackGen(prepare_qry()))
    test = next(merged)
    print(test)


def test_merge_bathymetry():
    merged = merge_tracks_bathymetry(TrackGen(prepare_qry()))
    test = next(merged)
    print(test)


def test_merge_hullgeom():
    assert False, 'need to rewrite this'
    #merged = merge_tracks_hullgeom(TrackGen(prepare_qry()))
    #test = next(merged)
    #print(test)


def test_merge_layers_all():
    merged = merge_layers(TrackGen(prepare_qry()))
    test = next(merged)
    print(test)
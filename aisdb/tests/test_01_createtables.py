import os
import warnings
import sqlite3

from aisdb.database.dbconn import DBConn, PostgresDBConn
from aisdb.database.decoder import decode_msgs
from aisdb.database.create_tables import (
    aggregate_static_msgs_sqlite,
    sqlite_createtable_dynamicreport,
    sqlite_createtable_staticreport,
)
from aisdb.tests.create_testing_data import postgres_test_conn


def test_create_static_table(tmpdir):
    dbpath = os.path.join(tmpdir, 'test_create_static_table.db')
    with DBConn() as dbconn:
        sqlite_createtable_staticreport(dbconn, month="202009", dbpath=dbpath)


def test_create_dynamic_table(tmpdir):
    dbpath = os.path.join(tmpdir, 'test_create_dynamic_table.db')
    with DBConn() as dbconn:
        sqlite_createtable_dynamicreport(dbconn, month="202009", dbpath=dbpath)


def test_create_static_aggregate_table(tmpdir):
    warnings.filterwarnings('error')
    dbpath = os.path.join(tmpdir, 'test_create_static_aggregate_table.db')
    testingdata_csv = os.path.join(os.path.dirname(__file__), 'testdata',
                                   'test_data_20210701.csv')
    with DBConn() as dbconn:
        decode_msgs([testingdata_csv],
                    dbconn=dbconn,
                    dbpath=dbpath,
                    source='TESTING')
        aggregate_static_msgs_sqlite(dbconn, ["202107"])


def test_create_from_CSV(tmpdir):
    testingdata_csv = os.path.join(os.path.dirname(__file__), 'testdata',
                                   'test_data_20210701.csv')
    dbpath = os.path.join(tmpdir, 'test_create_from_CSV.db')
    with DBConn() as dbconn:
        decode_msgs(
            dbconn=dbconn,
            filepaths=[testingdata_csv],
            dbpath=dbpath,
            source='TESTING',
            vacuum=False,
        )

        # new tables will be created by the decode_msgs function
        # sqlite_createtable_dynamicreport(dbconn, month="202107", dbpath=dbpath)
        # aggregate_static_msgs(dbconn, ["202107"])

        # executes SQL statement: ATTACH database AS dbname
        dbconn._attach(dbpath)

        curr = dbconn.cursor()
        curr.execute(
            # need to specify datbabase name in SQL statement
            "SELECT name FROM test_create_from_CSV.sqlite_schema "
            "WHERE type='table' ORDER BY name;")
        rows = curr.fetchall()
        temp = [row['name'] for row in rows]
        print(temp)
        assert len(temp) == 4

    # alternatively, open a new connection to the database:
    new_dbconn = sqlite3.Connection(dbpath)
    new_dbconn.row_factory = sqlite3.Row
    new_cursor = new_dbconn.cursor()
    new_cursor.execute("SELECT name FROM sqlite_schema "
                       "WHERE type='table' ORDER BY name;")
    rows = new_cursor.fetchall()
    temp = [row['name'] for row in rows]
    print(temp)
    assert len(temp) == 4


def test_create_from_CSV_postgres(tmpdir):
    testingdata_csv = os.path.join(os.path.dirname(__file__), 'testdata',
                                   'test_data_20210701.csv')
    with PostgresDBConn(**postgres_test_conn) as dbconn:
        #dbconn.execute('DROP TABLE IF EXISTS ais_202107_dynamic')
        #dbconn.execute('DROP TABLE IF EXISTS hashmap')
        decode_msgs(
            dbconn=dbconn,
            filepaths=[testingdata_csv],
            source='TESTING',
            vacuum=False,
        )
        cur = dbconn.cursor()
        cur.execute(
            # need to specify datbabase name in SQL statement
            "SELECT table_name FROM information_schema.tables "
            "WHERE table_schema = 'public' ORDER BY table_name;")
        tables = [row[0] for row in cur.fetchall()]
        assert 'ais_202107_dynamic' in tables

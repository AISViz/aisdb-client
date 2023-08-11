import os
from collections import Counter

import numpy as np
import warnings

from aisdb.database.dbconn import DBConn, SQLiteDBConn, PostgresDBConn
from aisdb import sqlpath

with open(os.path.join(sqlpath, 'createtable_dynamic_clustered.sql'),
          'r') as f:
    sql_createtable_dynamic = f.read()

with open(os.path.join(sqlpath, 'createtable_static.sql'), 'r') as f:
    sql_createtable_static = f.read()

with open(os.path.join(sqlpath, 'createtable_static_aggregate.sql'), 'r') as f:
    sql_aggregate = f.read()


def sqlite_createtable_dynamicreport(dbconn, month):
    assert isinstance(dbconn,
                      (DBConn)), f'not an SQLiteDBConn object: {dbconn}'
    dbconn.execute(sql_createtable_dynamic.format(month))


def sqlite_createtable_staticreport(dbconn, month):
    assert isinstance(dbconn, (DBConn)), f'not an SQLiteDBConn object {dbconn}'
    sql = sql_createtable_static.format(month)
    dbconn.execute(sql)


def aggregate_static_msgs_sqlite(dbconn: SQLiteDBConn,
                                 months_str: list,
                                 verbose: bool = True):
    ''' collect an aggregate of static vessel reports for each unique MMSI
        identifier. The most frequently repeated values for each MMSI will
        be kept when multiple different reports appear for the same MMSI

        this function should be called every time data is added to the database

        args:
            dbconn (:class:`aisdb.database.dbconn.SQLiteDBConn`)
                database connection object
            months_str (list)
                list of strings with format: YYYYmm
            verbose (bool)
                logs messages to stdout
    '''

    if not isinstance(dbconn, SQLiteDBConn):
        raise ValueError(
            'db argument must be an SQLiteDBConn database connection')

    assert hasattr(dbconn, 'dbpath')
    assert not hasattr(dbconn, 'dbpaths')

    cur = dbconn.cursor()

    for month in months_str:
        # check for monthly tables in dbfiles containing static reports
        cur.execute(
            'SELECT name FROM sqlite_master '
            'WHERE type="table" AND name=?', [f'ais_{month}_static'])
        if cur.fetchall() == []:
            continue

        sqlite_createtable_staticreport(dbconn=dbconn, month=month)
        if verbose:
            print('aggregating static reports into '
                  f'static_{month}_aggregate...')
        cur.execute('SELECT DISTINCT s.mmsi FROM '
                    f'ais_{month}_static AS s')
        mmsis = np.array(cur.fetchall(), dtype=int).flatten()

        cur.execute('DROP TABLE IF EXISTS '
                    f'static_{month}_aggregate')

        sql_select = '''
          SELECT
            s.mmsi, s.imo, TRIM(vessel_name) as vessel_name, s.ship_type,
            s.call_sign, s.dim_bow, s.dim_stern, s.dim_port, s.dim_star,
            s.draught
          FROM ais_{}_static AS s WHERE s.mmsi = ?
        '''.format(month)

        agg_rows = []
        for mmsi in mmsis:
            _ = cur.execute(sql_select, (str(mmsi), ))
            cols = np.array(cur.fetchall(), dtype=object).T
            assert len(cols) > 0

            filtercols = np.array(
                [
                    np.array(list(filter(None, col)), dtype=object)
                    for col in cols
                ],
                dtype=object,
            )

            paddedcols = np.array(
                [col if len(col) > 0 else [None] for col in filtercols],
                dtype=object,
            )

            aggregated = [
                Counter(col).most_common(1)[0][0] for col in paddedcols
            ]

            agg_rows.append(aggregated)

        cur.execute(
            sql_aggregate.format(month).replace(f'static_{month}_aggregate',
                                                f'static_{month}_aggregate'))

        if len(agg_rows) == 0:  # pragma: no cover
            warnings.warn('no rows to aggregate! '
                          f'table: static_{month}_aggregate')
            continue

        skip_nommsi = np.array(agg_rows, dtype=object)
        assert len(skip_nommsi.shape) == 2
        skip_nommsi = skip_nommsi[skip_nommsi[:, 0] != None]
        assert len(skip_nommsi) > 1
        cur.executemany((
            f'INSERT INTO static_{month}_aggregate '
            f"VALUES ({','.join(['?' for _ in range(skip_nommsi.shape[1])])}) "
        ), skip_nommsi)

        dbconn.commit()


def aggregate_static_msgs_postgres(dbconn: PostgresDBConn,
                                   months_str: list,
                                   verbose: bool = True):
    ''' collect an aggregate of static vessel reports for each unique MMSI
        identifier. The most frequently repeated values for each MMSI will
        be kept when multiple different reports appear for the same MMSI

        this function should be called every time data is added to the database

        args:
            dbconn (:class:`aisdb.database.dbconn.DBConn`)
                database connection object
            months_str (list)
                list of strings with format: YYYYmm
            verbose (bool)
                logs messages to stdout
    '''

    if not isinstance(dbconn, PostgresDBConn):
        raise ValueError(
            'db argument must be a PostgresDBConn database connection')

    cur = dbconn.cursor()

    for month in months_str:
        # check for monthly tables in dbfiles containing static reports
        cur.execute('SELECT table_name FROM information_schema.tables '
                    f'WHERE table_name = \'ais_{month}_static\'')
        static_tables = cur.fetchall()
        if static_tables == []:
            continue

        #sqlite_createtable_staticreport(dbconn, month, dbpath)
        if verbose:
            print('aggregating static reports into '
                  f'static_{month}_aggregate...')
        cur.execute(f'SELECT DISTINCT s.mmsi FROM ais_{month}_static AS s')
        mmsis = np.array(cur.fetchall(), dtype=int).flatten()

        cur.execute(f'DROP TABLE IF EXISTS static_{month}_aggregate')

        formatter = '?' if isinstance(dbconn, SQLiteDBConn) else '%s'
        sql_select = f'''
          SELECT
            s.mmsi, s.imo, TRIM(vessel_name) as vessel_name, s.ship_type,
            s.call_sign, s.dim_bow, s.dim_stern, s.dim_port, s.dim_star,
            s.draught
          FROM ais_{month}_static AS s WHERE s.mmsi = {formatter}
        '''

        agg_rows = []
        for mmsi in mmsis:
            _ = cur.execute(sql_select, (str(mmsi), ))
            cols = np.array(cur.fetchall(), dtype=object).T
            assert len(cols) > 0

            filtercols = np.array(
                [
                    np.array(list(filter(None, col)), dtype=object)
                    for col in cols
                ],
                dtype=object,
            )

            paddedcols = np.array(
                [col if len(col) > 0 else [None] for col in filtercols],
                dtype=object,
            )

            aggregated = [
                Counter(col).most_common(1)[0][0] for col in paddedcols
            ]

            agg_rows.append(aggregated)

        cur.execute(sql_aggregate.format(month))

        if len(agg_rows) == 0:  # pragma: no cover
            warnings.warn('no rows to aggregate! '
                          f'table: static_{month}_aggregate')
            return

        skip_nommsi = np.array(agg_rows, dtype=object)
        assert len(skip_nommsi.shape) == 2
        skip_nommsi = skip_nommsi[skip_nommsi[:, 0] != None]
        assert len(skip_nommsi) > 1
        insert_stmt = (
            f'INSERT INTO static_{month}_aggregate '
            f"VALUES ({','.join([formatter for _ in range(skip_nommsi.shape[1])])}) "
        )
        cur.executemany(insert_stmt, map(tuple, skip_nommsi))

        dbconn.commit()

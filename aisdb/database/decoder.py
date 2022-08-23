''' Parsing NMEA messages to create an SQL database.
    See function decode_msgs() for usage
'''

import os
from hashlib import md5
import pickle
import orjson
import sqlite3

from aisdb.database.dbconn import DBConn
from aisdb.aisdb import decoder
from aisdb.track_gen import serialize_tracks, deserialize_tracks


class FileChecksums():

    md5 = md5

    def __init__(self, dbpath):
        self.dbconn = sqlite3.connect(dbpath)
        self._checksums_table()

    def _bitshift_hex32_int32(self, hex32):
        ''' convert 32-char hexadecimal string to 32-bit signed integer '''
        assert isinstance(hex32, str)
        assert len(hex32) == 32
        return (int(hex32, base=16) >> 64) - (2**63)

    def _checksums_table(self):
        cur = self.dbconn.cursor()
        cur.execute('''
            CREATE TABLE IF NOT EXISTS
            hashmap(
                hash INTEGER PRIMARY KEY,
                bytes BLOB
            )
            WITHOUT ROWID;''')
        #cur.execute('CREATE UNIQUE INDEX IF NOT EXISTS ' 'idx_map on hashmap(hash)')
        #zeros = ''.join(['0' for _ in range(32)])
        #ones = ''.join(['f' for _ in range(32)])
        #minval = (int(zeros, base=16) >> 64) - (2**63)
        #maxval = (int(ones, base=16) >> 64) - (2**63)
        #cur.execute('INSERT OR IGNORE INTO hashmap VALUES (?,?)', (minval, pickle.dumps(None)))
        #cur.execute('INSERT OR IGNORE INTO hashmap VALUES (?,?)', (maxval, pickle.dumps(None)))
        #dbconn.close()
        self.dbconn.commit()

    def _insert_checksum(self, checksum, obj=True):
        '''
            args:
                checksum:
                    object hash digest i.e. hexadecimal string with length 32
                obj:
                    arbitrary binary object or other serializeable data
        '''
        if obj is not None and len(obj) > 0:
            for track in obj:
                track['dynamic'] = tuple(track['dynamic'])
                track['static'] = tuple(track['static'])
        hashint = self._bitshift_hex32_int32(checksum)
        print(f'INSERT HASH {hashint}')
        cur = self.dbconn.cursor()
        cur.execute(
            'INSERT INTO hashmap VALUES (?,?)',
            [
                hashint,
                orjson.dumps(obj, option=orjson.OPT_SERIALIZE_NUMPY)
                #serialize_tracks(obj)
            ])
        self.dbconn.commit()
        #dbconn.close()

    def checksum_exists(self, checksum):
        '''
            args:
                checksum:
                    object hash - 32-length hex digest
        '''
        cur = self.dbconn.cursor()
        cur.execute('SELECT bytes FROM hashmap WHERE hash == ?',
                    [self._bitshift_hex32_int32(checksum)])
        res = cur.fetchall()
        #dbconn.close()

        if res == []:
            return False
        return orjson.loads(res[0][0])


def decode_msgs(filepaths,
                dbconn,
                dbpath,
                source,
                vacuum=False,
                skip_checksum=False,
                verbose=False):
    ''' Decode NMEA format AIS messages and store in an SQLite database.
        To speed up decoding, create the database on a different hard drive
        from where the raw data is stored.
        A checksum of the first kilobyte of every file will be stored to
        prevent loading the same file twice.

        args:
            filepaths (list)
                absolute filepath locations for AIS message files to be
                ingested into the database
            dbconn (:class:`aisdb.database.dbconn.DBConn`)
                database connection object
            dbpath (string)
                database filepath to store results in
            source (string)
                data source name or description. will be used as a primary key
                column, so duplicate messages from different sources will not be
                ignored as duplicates upon insert
            vacuum (boolean, str)
                if True, the database will be vacuumed after completion.
                if string, the database will be vacuumed into the filepath
                given. Consider vacuuming to second hard disk to speed this up

        returns:
            None

        example:

        >>> import os
        >>> from aisdb import decode_msgs, DBConn

        >>> dbpath = os.path.join('testdata', 'doctest.db')
        >>> filepaths = ['aisdb/tests/test_data_20210701.csv',
        ...              'aisdb/tests/test_data_20211101.nm4']
        >>> with DBConn() as dbconn:
        ...     decode_msgs(filepaths=filepaths, dbconn=dbconn, dbpath=dbpath,
        ...     source='TESTING')
        >>> os.remove(dbpath)
    '''
    if not isinstance(dbconn, DBConn):  # pragma: no cover
        if isinstance(dbconn):
            raise ValueError('Files must be decoded synchronously!')
        raise ValueError('db argument must be a DBConn database connection. '
                         f'got {DBConn}')

    if len(filepaths) == 0:  # pragma: no cover
        raise ValueError('must supply atleast one filepath.')

    dbindex = FileChecksums(dbpath)
    for file in filepaths:
        if not skip_checksum:
            with open(os.path.abspath(file), 'rb') as f:
                signature = md5(f.read(1000)).hexdigest()
                if file[-4:] == '.csv':  # skip header row (~1.6kb)
                    _ = f.read(600)
                    signature = md5(f.read(1000)).hexdigest()
            if dbindex.checksum_exists(signature):
                if verbose:  # pragma: no cover
                    print(f'found matching checksum, skipping {file}')
                continue
        decoder(dbpath=dbpath, files=[file], source=source, verbose=verbose)
        if not skip_checksum:
            dbindex._insert_checksum(signature)
    dbindex.dbconn.close()

    if vacuum is not False:
        print("finished parsing data\nvacuuming...")
        if vacuum is True:
            dbconn.execute('VACUUM')
        elif isinstance(vacuum, str):
            assert not os.path.isfile(vacuum)
            dbconn.execute(f"VACUUM INTO '{vacuum}'")
        else:
            raise ValueError('vacuum arg must be boolean or filepath string')
        dbconn.commit()

    return

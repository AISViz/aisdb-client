'''
Run the server in a development environment:
    python -m flask --app aisdb/rest_api.py run

Deploying flask to production with IIS:
    <https://learn.microsoft.com/en-us/visualstudio/python/configure-web-apps-for-iis-windows>
'''
from datetime import datetime, timedelta
from tempfile import SpooledTemporaryFile
import secrets

import aisdb
from aisdb import PostgresDBConn, DBQuery

from flask import (
    Flask,
    Markup,
    flash,
    request,
    send_file,
)

# Maximum bytes for client CSV files stored in memory before spilling to disk.
# Set the directory for files exceeding this value with $TMPDIR
MAX_CLIENT_MEMORY = 1024 * 1e6  # 1GB

# TODO: auth
app = Flask("aisdb-rest-api")
app.config.from_mapping(SECRET_KEY=secrets.token_bytes())

test_db_args = dict(
    host='fc00::17',
    port=5431,
    user='postgres',
    password='devel',
)


@app.route('/', methods=['GET', 'POST'])
def download():
    http_qry = dict(request.args)
    default_query = {
        #'start': int((datetime.utcnow() - timedelta(hours=24)).timestamp()),
        #'end': int(datetime.utcnow().timestamp()),
        'start': int(datetime(2021, 7, 1).timestamp()),
        'end': int(datetime(2021, 11, 30).timestamp()),
        'xmin': -65,
        'xmax': -62,
        'ymin': 43,
        'ymax': 45,
    }

    need_keys = set(default_query.keys())
    recv_keys = set(http_qry.keys())
    missing = need_keys - recv_keys

    if len(missing) > 0:
        example_qry = '?' + '&'.join(f'{k}={v}'
                                     for k, v in default_query.items())
        return Markup(f'Error: missing keys from request: {missing}<br>'
                      f'example:<br><code>{example_qry}<code>')

    http_qry['start'] = datetime.utcfromtimestamp(int(http_qry['start']))
    http_qry['end'] = datetime.utcfromtimestamp(int(http_qry['end']))

    for arg in ['xmin', 'xmax', 'ymin', 'ymax']:
        http_qry[arg] = float(http_qry[arg])

    with PostgresDBConn(**test_db_args) as dbconn:

        buf = SpooledTemporaryFile(max_size=MAX_CLIENT_MEMORY)

        dbqry = DBQuery(dbconn=dbconn,
                        callback=aisdb.sqlfcn_callbacks.in_bbox_time_validmmsi,
                        **http_qry).gen_qry(verbose=False)
        tracks = aisdb.TrackGen(dbqry, decimate=0.0001)

        try:
            aisdb.write_csv(tracks, buf)
            buf.flush()
            buf.seek(0)
            send_file(buf,
                      mimetype='csv',
                      as_attachment=True,
                      download_name='ais_test.csv',
                      max_age=int(timedelta(days=7).total_seconds()))
            buf.seek(0)
            count = sum(1 for csvrow in buf)
            return Markup(f"sent {count} rows to client")
        except StopIteration:
            return Markup("No results found for query")
        except Exception as err:
            raise err


if __name__ == '__main__':
    app.run()

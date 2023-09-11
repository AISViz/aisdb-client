'''
deploying to production in IIS:
<https://learn.microsoft.com/en-us/visualstudio/python/configure-web-apps-for-iis-windows>
'''
from datetime import datetime, timedelta
import secrets
import io

from database.dbconn import PostgresDBConn

import orjson
from flask import Flask, send_file

app = Flask("aisdb-rest-api")
app.config.from_mapping(SECRET_KEY=secrets.token_bytes())

dbconn = PostgresDBConn()


@app.route('/', methods=['GET'])
def index():
    response = {"status": 0}
    return orjson.dumps(response)


def download(qry=None):
    default_query = {
        'start': int((datetime.utcnow() - timedelta(hours=24)).timestamp()),
        'end': int((datetime.utcnow()).timestamp()),
        'xmin': -65,
        'xmax': -62,
        'ymin': 43,
        'ymax': 45,
    }
    with dbconn:
        aisdb.DBQuery(dbconn=dbconn, **default_query)
    buf = io.BytesIo()
    return send_file(buf,
                     mimetype='csv',
                     as_attachment=True,
                     download_name='ais_test.csv',
                     max_age=int(timedelta(days=7).total_seconds()))


app.run()

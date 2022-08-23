import asyncio
from os import environ

from aisdb.websocket_server import SocketServ
from aisdb import DomainFromTxts

from dotenv import load_dotenv

load_dotenv()

dbpath = environ.get('AISDBPATH', '/home/ais_env/ais/ais.db')
zones_dir = environ.get('AISDBZONES', '/home/ais_env/ais/zones/')
trafficDBpath = environ.get('AISDBMARINETRAFFIC',
                            '/home/ais_env/ais/marinetraffic.db')

print(f'starting websocket\n{dbpath = }\n{zones_dir = }\n{trafficDBpath = }\n')

domain = DomainFromTxts(
    domainName='example',
    folder=zones_dir,
    correct_coordinate_range=False,
)

serv = SocketServ(
    dbpath=dbpath,
    domain=domain,
    trafficDBpath=trafficDBpath,
    enable_ssl=False,
)

try:
    asyncio.run(serv.main())
finally:
    asyncio.run(serv.dbconn.close())
    serv.request_cache.cache_db.dbconn.close()

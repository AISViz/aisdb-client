import os

aisdb_sql = os.path.abspath(os.path.join(__file__, '..', '..', 'aisdb_sql/'))

# query position reports using rtree indexes
dynamic = lambda month, callback, kwargs: (f'''
    SELECT d.mmsi, d.time, d.longitude, d.latitude
      FROM ais_{month}_dynamic AS d
      WHERE {callback(month=month, alias='d', **kwargs)}''')

# query static vessel data from monthly aggregate tables
static = lambda month, **_: (f'''
    SELECT mmsi, vessel_name, ship_type,
            dim_bow, dim_stern, dim_port, dim_star, imo
        FROM static_{month}_aggregate''')

# common table expression SELECT statements for concatenation with UNION
leftjoin = lambda month: (f'''
SELECT dynamic_{month}.mmsi, dynamic_{month}.time,
            dynamic_{month}.longitude, dynamic_{month}.latitude,
            static_{month}.imo, static_{month}.vessel_name,
            static_{month}.dim_bow, static_{month}.dim_stern,
            static_{month}.dim_port, static_{month}.dim_star,
            static_{month}.ship_type, ref.coarse_type_txt
        FROM dynamic_{month}
    LEFT JOIN static_{month}
        ON dynamic_{month}.mmsi = static_{month}.mmsi
    LEFT JOIN ref
        ON static_{month}.ship_type = ref.coarse_type''')

# declare common table expressions for use in SQL 'WITH' statements
aliases = lambda month, callback, kwargs: (f'''
dynamic_{month} AS ( {dynamic(month, callback, kwargs)}
),
static_{month} AS ( {static(month)}
),
ref AS MATERIALIZED ( SELECT coarse_type, coarse_type_txt
        FROM coarsetype_ref
)
''')

# iterate over month tables to create SQL query spanning desired time range
crawl = lambda months, callback, **kwargs: ('WITH' + ','.join([
    aliases(month=month, callback=callback, kwargs=kwargs) for month in months
]) + '\nUNION'.join([leftjoin(month=month)
                     for month in months]) + '\nORDER BY 1, 2')

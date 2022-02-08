        CREATE TABLE IF NOT EXISTS ais_{}_static (
            mmsi INTEGER,
            time INTEGER,
            vessel_name TEXT,
            ship_type INT,
            call_sign TEXT,
            imo INTEGER,
            dim_bow INTEGER,
            dim_stern INTEGER,
            dim_port INTEGER,
            dim_star INTEGER,
            draught INTEGER,
            destination TEXT,
            ais_version TEXT,
            fixing_device STRING,
            eta_month INTEGER,
            eta_day INTEGER,
            eta_hour INTEGER,
            eta_minute INTEGER,
            PRIMARY KEY (mmsi, time, imo)
        ) WITHOUT ROWID;

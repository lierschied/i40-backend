DEFINE NAMESPACE main;
USE NS main;

DEFINE DATABASE main;
USE DB main;

DEFINE TABLE station SCHEMAFULL;

DEFINE FIELD name ON station TYPE string;

-- 
DEFINE TABLE sensor SCHEMAFULL;

DEFINE FIELD node ON sensor TYPE string;
DEFINE FIELD station ON sensor TYPE record(station);
DEFINE FIELD display_name ON sensor TYPE string;

--
DEFINE TABLE sensor_value SCHEMAFULL;

DEFINE FIELD sensor ON sensor_value TYPE record(sensor);
DEFINE FIELD value ON sensor_value TYPE string;
DEFINE FIELD source_timestamp ON sensor_value TYPE datetime;
DEFINE FIELD server_timestamp ON sensor_value TYPE datetime;

--
DEFINE TABLE user SCHEMAFULL
    PERMISSIONS
        FOR select, update WHERE id = $auth.id, 
        FOR create, delete NONE;
DEFINE FIELD name ON user TYPE string;
DEFINE FIELD email ON user TYPE string;
DEFINE FIELD password ON user TYPE string;
DEFINE INDEX idx_user ON user COLUMNS email UNIQUE;


--
-- Insert dummy data
--
LET $station = INSERT INTO station (name) VALUES ('Palettenlager');
LET $sensor = INSERT INTO sensor (display_name, station) VALUES ('Füllstand', $station.id);

INSERT INTO sensor_value (sensor, value) VALUES ($sensor.id, 12344);
INSERT INTO sensor_value (sensor, value) VALUES ($sensor.id, 12345);
INSERT INTO sensor_value (sensor, value) VALUES ($sensor.id, 12399);
INSERT INTO sensor_value (sensor, value) VALUES ($sensor.id, 12300);

CREATE user SET email = 'test@mail.de', name = 'Hugo', password = crypto::argon2::generate('1234')

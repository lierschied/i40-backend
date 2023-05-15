DEFINE NAMESPACE main;
USE NS main;

DEFINE DATABASE main;
USE DB main;


--
-- Station
--
DEFINE TABLE station SCHEMAFULL;
-- Station fields
DEFINE FIELD name ON station TYPE string ASSERT $value != NONE;


--
-- Sensor
--
DEFINE TABLE sensor SCHEMAFULL;
--Sensor fields
DEFINE FIELD station ON sensor TYPE record(station) ASSERT $value != NONE;
DEFINE FIELD display_name ON sensor TYPE string ASSERT $value != NONE;


--
-- Sensor_value
--
DEFINE TABLE sensor_value SCHEMAFULL;
-- Sensor_value fields
DEFINE FIELD sensor ON sensor_value TYPE record(sensor) ASSERT $value != NONE;
DEFINE FIELD value ON sensor_value TYPE string ASSERT $value != NONE;
-- DEFINE FIELD source_timestamp ON sensor_value TYPE datetime;
DEFINE FIELD server_timestamp ON sensor_value TYPE datetime ASSERT $value != NONE;


--
-- hasValue RELATION
--
DEFINE TABLE hasValue SCHEMAFULL;
DEFINE FIELD in ON hasValue TYPE record(sensor);
DEFINE FIELD out ON hasValue TYPE record(sensor_value);


--
-- user
--
DEFINE TABLE user SCHEMAFULL
-- only for 
    PERMISSIONS
        FOR select, update WHERE id = $auth.id, 
        FOR create, delete NONE;
DEFINE FIELD name ON user TYPE string;
DEFINE FIELD email ON user TYPE string ASSERT $value != NONE;
DEFINE FIELD password ON user TYPE string ASSERT $value != NONE;
DEFINE FIELD icon ON user TYPE string;
DEFINE INDEX idx_user ON user COLUMNS email UNIQUE;


--
-- Functions
--
DEFINE FUNCTION fn::slice_array($arr: array, $start: int, $end: int) {
    RETURN SELECT * FROM $arr LIMIT BY $end START AT $start;
};


--
-- Insert dummy data
--
INSERT INTO station (name) VALUES ('palettenlager');
INSERT INTO station (name) VALUES ('presswerk');

CREATE user SET email = 'lucy@cyber.night', name = 'Lucy', password = crypto::argon2::generate('1234'), icon = 'https://www.egames.news/__export/1677000358920/sites/debate/img/2023/02/21/lucy_cyberpunk_edgerunners.jpg_172596871.jpg';
